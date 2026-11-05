//! DEC-17 A1 only. Compiled solely in library tests on the experiment branch.
//! Independent LocalOnly executions share a compiled schema, never a sink,
//! reader, audit session or mutable summary. Protected report staging is charged
//! to both sequential and parallel variants, then merged in trace order.
use crate::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::BTreeMap,
    error::Error,
    fs::{self},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};
type Failure = Box<dyn Error + Send + Sync>;

struct Meter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static METERING: AtomicBool = AtomicBool::new(false);
fn acquired(n: usize) {
    let live = LIVE.fetch_add(n, Ordering::Relaxed) + n;
    if METERING.load(Ordering::Relaxed) {
        PEAK.fetch_max(live, Ordering::Relaxed);
    }
}
// SAFETY: unchanged allocation arguments/pointers are delegated to System;
// counters use atomics and never allocate or inspect the allocation contents.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            acquired(l.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            acquired(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        LIVE.fetch_sub(l.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            if n >= l.size() {
                acquired(n - l.size());
            } else {
                LIVE.fetch_sub(l.size() - n, Ordering::Relaxed);
            }
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Meter = Meter;
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(target_os = "linux")]
fn rss() -> u64 {
    fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap()
        * 1024
}
#[cfg(windows)]
fn rss() -> u64 {
    use windows_sys::Win32::System::{ProcessStatus::*, Threading::*};
    let mut c = std::mem::MaybeUninit::<PROCESS_MEMORY_COUNTERS>::zeroed();
    // SAFETY: current-process pseudohandle, writable correctly sized buffer;
    // success initializes PROCESS_MEMORY_COUNTERS before assume_init.
    assert_ne!(
        unsafe {
            K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                c.as_mut_ptr(),
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        },
        0
    );
    unsafe { c.assume_init() }.WorkingSetSize as u64
}
#[cfg(not(any(target_os = "linux", windows)))]
fn rss() -> u64 {
    panic!("unqualified OS")
}

fn model() -> (Arc<CompiledFlowSchema>, String) {
    let mut v: Value = serde_json::from_slice(include_bytes!(
        "../../tests/golden/synthetic/v1/schema.json"
    ))
    .unwrap();
    v["flow_id"] = json!("DEC17-SYNTHETIC");
    v["schema_version"] = json!("local-a1-v1");
    let template = v["traces"][0]["fields"][0].clone();
    let mut fields = vec![];
    for (id, start, width, kind, pattern) in [
        ("ID", 1, 8, "numeric", r"\A[0-9]{8}\z"),
        ("CODE", 9, 2, "numeric", r"\A0[12]\z"),
    ] {
        let mut f = template.clone();
        f["id"] = json!(id);
        f["name"] = json!(id);
        f["physical"] = json!({"type":kind,"start":start,"length":width,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}});
        f["constraints"] =
            json!([{"kind":"regex","pattern":pattern,"match":"required","target":"raw"}]);
        fields.push(f);
    }
    for i in 0..8 {
        let mut f = template.clone();
        f["id"] = json!(format!("PAYLOAD{i}"));
        f["name"] = json!(format!("Synthetic payload {i}"));
        f["physical"] = json!({"type":"alphanumeric","start":11+i*16,"length":16,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}});
        f["constraints"] = json!([{"kind":"regex","pattern":r"\A[A-Z0-9]{16}\z","match":"required","target":"raw"}]);
        fields.push(f);
    }
    let mut trace = v["traces"][0].clone();
    trace["record_length"] = json!(138);
    trace["fields"] = json!(fields);
    trace["record_key"]["fields"] = json!(["ID"]);
    trace["identity"] = json!([]);
    v["traces"] = json!(
        (0..4)
            .map(|i| {
                let mut t = trace.clone();
                t["id"] = json!(format!("T{i}"));
                t
            })
            .collect::<Vec<_>>()
    );
    let mut raw: RawFlowSchema = serde_json::from_value(v).unwrap();
    let hash = schema_fingerprint(&raw).unwrap().to_string();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    (Arc::new(SchemaCompiler.compile(raw).unwrap()), hash)
}
fn fixture(root: &Path, total: usize, skewed: bool, heavy: bool) -> Vec<PathBuf> {
    let weights = if skewed { [80, 10, 5, 5] } else { [25; 4] };
    let mut paths = vec![];
    for (index, weight) in weights.into_iter().enumerate() {
        let path = root.join(format!("input-{index}.txt"));
        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut out = BufWriter::new(file);
        for line in 1..=total * weight / 100 {
            let bad = if heavy {
                line % 4 == 0
            } else {
                line % 1000 == 0
            };
            let mut row = format!("{line:08}").into_bytes();
            row.extend_from_slice(if bad { b"99" } else { b"01" });
            for _ in 0..8 {
                row.extend_from_slice(if bad && heavy {
                    b"!!!!!!!!!!!!!!!!"
                } else {
                    b"ABCDEF0123456789"
                });
            }
            row.push(b'\n');
            out.write_all(&row).unwrap();
        }
        out.flush().unwrap();
        paths.push(path);
    }
    paths
}
struct Artifact {
    report: tempfile::NamedTempFile,
    summary_file: tempfile::NamedTempFile,
    summary: Value,
    summary_original_utf8: String,
    progress: String,
}
fn run_trace(
    schema: &CompiledFlowSchema,
    hash: &str,
    index: usize,
    path: &Path,
    root: &Path,
) -> Result<Artifact, Failure> {
    let trace = &schema.traces()[index];
    let audits = InputAuditSession::new();
    let mut report = tempfile::NamedTempFile::new_in(root)?;
    crate::persistence::dec17_private_file(report.path(), report.as_file())?;
    let mut sink = JsonlSink::new(BufWriter::new(report.as_file_mut()));
    let mut summary = ValidationSummaryBuilder::new(
        schema.key().clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let mut progress = Sha256::new();
    let mut on_event = |event: TraceEvent| progress.update(format!("{event:?}").as_bytes());
    let mut control = TraceControl {
        is_cancelled: &|| false,
        on_event: &mut on_event,
    };
    let completion = TraceValidator::new(schema, trace.id(), NewlinePolicy::AutoLfOrCrlf)?
        .validate_path(path, &mut sink, &mut summary, &mut control, &audits)?;
    assert!(matches!(completion, TraceCompletion::Completed { .. }));
    let metadata = audits.finish(RunMetadata {
        validation_mode: ValidationMode::LocalOnly,
        validator_version: format!(
            "{}+dec17-spike.{}",
            env!("CARGO_PKG_VERSION"),
            env!("LINTRA_BUILD_COMMIT")
        ),
        schema_hash: hash.into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    })?;
    let summary = summary.finish(metadata)?;
    sink.on_summary(&summary)?;
    let mut writer = sink.finish()?;
    writer.flush()?;
    drop(writer);
    let mut summary_file = tempfile::NamedTempFile::new_in(root)?;
    crate::persistence::dec17_private_file(summary_file.path(), summary_file.as_file())?;
    let mut writer = write_run_summary(BufWriter::new(summary_file.as_file_mut()), &summary)?;
    writer.flush()?;
    drop(writer);
    let mut bytes = vec![];
    summary_file.reopen()?.read_to_end(&mut bytes)?;
    Ok(Artifact {
        report,
        summary_file,
        summary: serde_json::from_slice(&bytes)?,
        summary_original_utf8: String::from_utf8(bytes)?,
        progress: progress
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    })
}
struct Sample {
    artifacts: Vec<Artifact>,
    merged: tempfile::NamedTempFile,
    metrics: Value,
}
struct MeterGuard;
impl Drop for MeterGuard {
    fn drop(&mut self) {
        METERING.store(false, Ordering::Relaxed);
    }
}

struct StopSampler<'a>(&'a AtomicBool);
impl Drop for StopSampler<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
fn sample(
    schema: &CompiledFlowSchema,
    hash: &str,
    paths: &[PathBuf],
    root: &Path,
    workers: usize,
) -> Sample {
    let starting_rss = rss();
    let baseline = LIVE.load(Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    METERING.store(true, Ordering::Relaxed);
    let _meter_guard = MeterGuard;
    let stop = AtomicBool::new(false);
    let start = Instant::now();
    let (artifacts, merged, pass_seconds, aggregation_seconds, peak_rss) = thread::scope(|scope| {
        let sampler = scope.spawn(|| {
            let mut peak = rss();
            while !stop.load(Ordering::Relaxed) {
                peak = peak.max(rss());
                thread::sleep(Duration::from_millis(2));
            }
            peak.max(rss())
        });
        // This guard stops the sampler during panic unwinding too, before scope
        // joins its owned threads. A worker fault cannot strand a sampler.
        let stop_guard = StopSampler(&stop);
        let artifacts = if workers == 0 {
            (0..paths.len())
                .map(|i| run_trace(schema, hash, i, &paths[i], root).unwrap())
                .collect::<Vec<_>>()
        } else {
            let handles = (0..workers)
                .map(|w| {
                    scope.spawn(move || {
                        (w..paths.len())
                            .step_by(workers)
                            .map(|i| (i, run_trace(schema, hash, i, &paths[i], root).unwrap()))
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            let mut results = handles
                .into_iter()
                .flat_map(|h| h.join().expect("owned validation worker panicked"))
                .collect::<Vec<_>>();
            results.sort_by_key(|(i, _)| *i);
            results.into_iter().map(|(_, a)| a).collect()
        };
        let pass_seconds = start.elapsed().as_secs_f64();
        let aggregate = Instant::now();
        let mut merged = tempfile::NamedTempFile::new_in(root).unwrap();
        crate::persistence::dec17_private_file(merged.path(), merged.as_file()).unwrap();
        let mut writer = BufWriter::new(merged.as_file_mut());
        for artifact in &artifacts {
            std::io::copy(&mut artifact.report.reopen().unwrap(), &mut writer).unwrap();
        }
        writer.flush().unwrap();
        drop(writer);
        let aggregation_seconds = aggregate.elapsed().as_secs_f64();
        drop(stop_guard);
        (
            artifacts,
            merged,
            pass_seconds,
            aggregation_seconds,
            sampler.join().unwrap(),
        )
    });
    let total_seconds = start.elapsed().as_secs_f64();
    METERING.store(false, Ordering::Relaxed);
    let heap = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    let merged_bytes = merged.as_file().metadata().unwrap().len();
    let staged_bytes = artifacts
        .iter()
        .map(|a| {
            a.report.as_file().metadata().unwrap().len()
                + a.summary_file.as_file().metadata().unwrap().len()
        })
        .sum::<u64>();
    Sample {
        artifacts,
        merged,
        metrics: json!({"workers":workers,"variant":if workers==0{"sequential"}else{"std_threads"},"trace_seconds":pass_seconds,"aggregation_seconds":aggregation_seconds,"total_seconds":total_seconds,"additional_peak_requested_heap_bytes":heap,"peak_rss_sampled_bytes":peak_rss,"starting_rss_bytes":starting_rss,"additional_peak_rss_sampled_bytes":peak_rss.saturating_sub(starting_rss),"rss_sampling_ms":2,"merged_bytes":merged_bytes,"staging_plus_merged_bytes":staged_bytes+merged_bytes,"scope":"A1 independent LocalOnly; not a Full measurement"}),
    }
}
fn report_digest(file: &tempfile::NamedTempFile) -> String {
    reader_digest(file.reopen().unwrap())
}
fn reader_digest(mut reader: impl Read) -> String {
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = reader.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
fn same_bytes(a: &tempfile::NamedTempFile, b: &tempfile::NamedTempFile) -> bool {
    let mut a = BufReader::new(a.reopen().unwrap());
    let mut b = BufReader::new(b.reopen().unwrap());
    let mut left = [0; 65536];
    let mut right = [0; 65536];
    loop {
        let mut n = 0;
        while n < left.len() {
            let k = a.read(&mut left[n..]).unwrap();
            if k == 0 {
                break;
            }
            n += k;
        }
        b.read_exact(&mut right[..n]).unwrap();
        if left[..n] != right[..n] {
            return false;
        }
        if n == 0 {
            return b.read(&mut right[..1]).unwrap() == 0;
        }
    }
}
fn compare_summary(reference: &Value, actual: &Value, path: &str, diffs: &mut Vec<Value>) {
    if reference == actual {
        return;
    }
    if path == "/elapsed" {
        diffs.push(json!({"path":path,"reference":reference,"actual":actual,"reason":"real monotonic elapsed retained unchanged; implementation audit before experiment"}));
        return;
    }
    match (reference, actual) {
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            for (k, v) in a {
                compare_summary(v, &b[k], &format!("{path}/{k}"), diffs);
            }
        }
        _ => panic!("deterministic summary differs at {path}: {reference} vs {actual}"),
    }
}

#[test]
#[ignore = "opt-in foreground DEC-17 experiment; no product feature"]
fn a1_local_only() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<CompiledFlowSchema>();
    send_sync::<PreparedReferenceData>();
    send_sync::<PreparedReferenceEvidence>();
    let output = PathBuf::from(
        std::env::var_os("LINTRA_DEC17_A1_OUTPUT").expect("explicit experiment output required"),
    );
    assert!(!output.exists());
    let parent = output.parent().unwrap();
    fs::create_dir_all(parent).unwrap();
    let owner = tempfile::Builder::new()
        .prefix("dec17-owned-")
        .tempdir_in(parent)
        .unwrap();
    let root = owner.path().join("private");
    crate::persistence::dec17_private_root(&root).unwrap();
    let smoke = std::env::var_os("LINTRA_DEC17_SMOKE").is_some();
    let (schema, hash) = model();
    let parallelism = thread::available_parallelism().unwrap().get();
    let variants = if parallelism >= 4 {
        vec![0, 1, 2, 4]
    } else if parallelism >= 2 {
        vec![0, 1, 2]
    } else {
        vec![0, 1]
    };
    let cases = if smoke {
        vec![(100, false, false), (100, true, true)]
    } else {
        vec![
            (10_000, false, false),
            (100_000, false, false),
            (500_000, false, false),
            (100_000, true, false),
            (500_000, true, false),
            (100_000, false, true),
            (500_000, false, true),
        ]
    };
    let mut measurements = vec![];
    for (total, skewed, heavy) in cases {
        let fixture_dir = tempfile::Builder::new()
            .prefix("case-")
            .tempdir_in(&root)
            .unwrap();
        let paths = fixture(fixture_dir.path(), total, skewed, heavy);
        let reports = fixture_dir.path().join("reports");
        crate::persistence::dec17_private_root(&reports).unwrap();
        let reference = sample(&schema, &hash, &paths, &reports, 0);
        let report_hash = report_digest(&reference.merged);
        let input_hashes = paths
            .iter()
            .map(|p| reader_digest(fs::File::open(p).unwrap()))
            .collect::<Vec<_>>();
        let weights = if skewed { [80, 10, 5, 5] } else { [25; 4] };
        for (index, artifact) in reference.artifacts.iter().enumerate() {
            let records = total * weights[index] / 100;
            let bad = records / if heavy { 4 } else { 1000 };
            assert_eq!(artifact.summary["records_processed"], records);
            assert_eq!(artifact.summary["invalid_records"], bad);
            assert_eq!(artifact.summary["valid_records"], records - bad);
            assert_eq!(
                artifact.summary["total_errors"],
                bad * if heavy { 9 } else { 1 }
            );
            assert_eq!(
                artifact.summary["inputs"][0]["completion"]["kind"],
                "complete"
            );
            assert_eq!(
                artifact.summary["inputs"][0]["completion"]["sha256"],
                input_hashes[index]
            );
        }
        for repetition in 0..if smoke { 1 } else { 3 } {
            let mut order = variants.clone();
            if repetition % 2 == 1 {
                order.reverse();
            }
            if repetition == 2 {
                order.rotate_left(2);
            }
            for workers in order {
                let result = sample(&schema, &hash, &paths, &reports, workers);
                assert!(same_bytes(&reference.merged, &result.merged));
                let mut time_diffs = vec![];
                for (a, b) in reference.artifacts.iter().zip(&result.artifacts) {
                    compare_summary(&a.summary, &b.summary, "", &mut time_diffs);
                    assert_eq!(a.progress, b.progress);
                }
                let mut record = result.metrics;
                record["total_records"] = json!(total);
                record["skewed"] = json!(skewed);
                record["error_heavy"] = json!(heavy);
                record["repetition"] = json!(repetition);
                record["jsonl_sha256"] = json!(report_hash);
                record["input_sha256"] = json!(input_hashes);
                record["byte_equal"] = json!(true);
                record["summary_elapsed_differences"] = json!(time_diffs);
                record["summary_original_utf8"] = json!(
                    result
                        .artifacts
                        .iter()
                        .map(|a| &a.summary_original_utf8)
                        .collect::<Vec<_>>()
                );
                record["summary_values"] = json!(
                    result
                        .artifacts
                        .iter()
                        .map(|a| &a.summary)
                        .collect::<Vec<_>>()
                );
                println!(
                    "A1 total={total} skewed={skewed} heavy={heavy} workers={workers} seconds={}",
                    record["total_seconds"]
                );
                measurements.push(record);
            }
        }
    }
    owner
        .close()
        .expect("owned experiment files must be removed");
    let result = json!({"task":"T-P17-05","activity":"A1","synthetic":true,"experimental_only":true,"source_commit":env!("LINTRA_BUILD_COMMIT"),"package_version":env!("CARGO_PKG_VERSION"),"available_parallelism":parallelism,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"sampler_threads":1,"worker_counts_include_validation_workers_only":true,"compiler_excluded":true,"schema_hash":hash,"harness_sha256":digest(include_bytes!("a1.rs")),"measurements":measurements});
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&result).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}
