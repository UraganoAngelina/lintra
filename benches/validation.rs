//! Opt-in synthetic benchmark, not a performance assertion or clinical fixture.
//! Run `cargo bench --bench validation -- --output target/benchmark.json`.
//! Requested heap peaks exclude allocator overhead/RSS; the instrumented allocator
//! affects throughput, so compare results only with the same harness/toolchain.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs::{self, File, OpenOptions},
    hint::black_box,
    io::{BufWriter, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

#[path = "../tests/support/full_schema.rs"]
mod full_schema;

struct Meter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED: AtomicUsize = AtomicUsize::new(0);
static MEASURING: AtomicBool = AtomicBool::new(false);

fn acquired(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    if MEASURING.load(Ordering::Relaxed) {
        PEAK.fetch_max(live, Ordering::Relaxed);
    }
}

/// Count successful calls only. Reallocation counts its complete requested target
/// size, independently of whether System extends in place; this is allocation
/// traffic, not live memory or bytes actually copied by the allocator.
fn requested(bytes: usize, reallocation: bool) {
    if MEASURING.load(Ordering::Relaxed) {
        let counter = if reallocation {
            &REALLOCATIONS
        } else {
            &ALLOCATIONS
        };
        counter.fetch_add(1, Ordering::Relaxed);
        REQUESTED.fetch_add(bytes, Ordering::Relaxed);
    }
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// SAFETY: all operations forward unchanged pointer/layout arguments to System.
// Accounting uses atomics only, never allocates or inspects allocation contents.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwards the caller's valid layout to the system allocator.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            acquired(layout.size());
            requested(layout.size(), false);
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same layout and zero-initialization contract as System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            acquired(layout.size());
            requested(layout.size(), false);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the owning allocator receives the original pointer/layout.
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: System receives the original allocation and requested size.
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            requested(size, true);
            if size >= layout.size() {
                acquired(size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Meter = Meter;

/// Successful allocation traffic during one synchronous sample.
#[derive(serde::Serialize)]
struct HeapTraffic {
    successful_allocations: usize,
    successful_reallocations: usize,
    cumulative_requested_bytes: usize,
}

/// Fixture IO and JSON serialization happen outside the measurement. Baseline
/// subtraction excludes already-live fixtures from additional peak memory.
fn measure<T>(work: impl FnOnce() -> T) -> (T, f64, usize, HeapTraffic) {
    let baseline = LIVE.load(Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    ALLOCATIONS.store(0, Ordering::Relaxed);
    REALLOCATIONS.store(0, Ordering::Relaxed);
    REQUESTED.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let start = Instant::now();
    let result = work();
    let seconds = start.elapsed().as_secs_f64();
    MEASURING.store(false, Ordering::Relaxed);
    let bytes = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    (
        result,
        seconds,
        bytes,
        HeapTraffic {
            successful_allocations: ALLOCATIONS.load(Ordering::Relaxed),
            successful_reallocations: REALLOCATIONS.load(Ordering::Relaxed),
            cumulative_requested_bytes: REQUESTED.load(Ordering::Relaxed),
        },
    )
}

#[derive(Default)]
struct CountSink(u64);
impl ValidationSink for CountSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.0 += 1;
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        Ok(())
    }
}

/// Reuse the qualified synthetic cross-trace rule, widening PK/join keys so a
/// million records remain unique. DATA differs only in error-heavy source rows.
fn schema() -> Value {
    let mut raw = full_schema::raw();
    for trace in raw["traces"].as_array_mut().unwrap() {
        trace["record_length"] = json!(18);
        for (field, start, length) in [(0, 1, 8), (1, 9, 8), (2, 17, 2)] {
            trace["fields"][field]["physical"]["start"] = json!(start);
            trace["fields"][field]["physical"]["length"] = json!(length);
        }
    }
    raw
}

fn records(path: &Path, count: usize, invalid: bool) {
    let mut writer = BufWriter::new(File::create(path).unwrap());
    for index in 0..count {
        writeln!(
            writer,
            "{index:08}{index:08}{}",
            if invalid { "NO" } else { "OK" }
        )
        .unwrap();
    }
    writer.flush().unwrap();
}

fn sample(count: usize, invalid: bool) -> Value {
    let root = tempfile::tempdir().unwrap();
    let schemas = root.path().join("schemas");
    let references = root.path().join("reference-data");
    fs::create_dir_all(schemas.join("SYNTHETIC/v1")).unwrap();
    fs::create_dir(&references).unwrap();
    fs::write(
        schemas.join("SYNTHETIC/v1/flow.json"),
        serde_json::to_vec(&schema()).unwrap(),
    )
    .unwrap();
    let source = root.path().join("source.dat");
    let target = root.path().join("target.dat");
    records(&source, count / 2, invalid);
    records(&target, count / 2, false);
    let service = build_local_service_with_reference_data(
        schemas,
        references,
        RawResolvedEncoding::Utf8,
        DatasetPreparationPolicy::Strict,
    )
    .unwrap();
    let request = ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope: ValidationScope::Flow(FlowValidationInput {
            traces: vec![
                TraceValidationInput {
                    trace_id: TraceId("S".into()),
                    source: InputSource::Path(source),
                },
                TraceValidationInput {
                    trace_id: TraceId("T".into()),
                    source: InputSource::Path(target),
                },
            ],
        }),
        mode: ValidationMode::Full,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    };
    let mut sink = CountSink::default();
    let (outcome, seconds, bytes, traffic) = measure(|| {
        service.validate(
            request,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )
    });
    let outcome = outcome.unwrap();
    let expected = if invalid { count as u64 / 2 } else { 0 };
    assert_eq!(outcome.summary.records_processed, count as u64);
    assert_eq!(outcome.summary.total_errors, expected);
    assert_eq!(sink.0, expected);
    assert_eq!(
        outcome.summary.completeness(),
        ValidationCompleteness::Complete
    );
    assert_eq!(
        outcome.status,
        if invalid {
            RunStatus::CompletedInvalid
        } else {
            RunStatus::CompletedValid
        }
    );
    json!({"scenario":if invalid {"error-heavy"} else {"valid-heavy"}, "records":count,
        "errors":expected, "seconds":seconds, "records_per_second":count as f64 / seconds,
        "heap_traffic":traffic, "peak_additional_requested_heap_bytes":bytes, "record_width_bytes":18,
        "scope":"Full service including compilation, two passes and count-only sink; fixture IO excluded"})
}

fn projection(count: usize) -> Value {
    let mut raw: RawFlowSchema = serde_json::from_value(schema()).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let compiled = SchemaCompiler.compile(raw).unwrap();
    let record = RecordView::new(b"0000000100000001OK");
    let ((), seconds, bytes, traffic) = measure(|| {
        for line in 1..=count / 2 {
            for trace in compiled.traces() {
                let value =
                    project_record(&record, trace, RecordIdentity::default(), line as u64).unwrap();
                assert_eq!(value.values.len(), 3);
                black_box(value);
            }
        }
    });
    json!({"scenario":"projection", "records":count, "seconds":seconds,
        "records_per_second":count as f64 / seconds, "heap_traffic":traffic, "peak_additional_requested_heap_bytes":bytes,
        "record_width_bytes":18, "scope":"project_record, three fields and join keys, drop each projection; no ContextStore or IO"})
}

/// Isolate the owned single-component encoding from projection and ContextStore.
/// An exact byte oracle precedes measurement. The loop allocates only each key;
/// its counters also qualify the meter (one 16-byte allocation, no realloc).
fn join_key(count: usize) -> Value {
    let mut raw: RawFlowSchema = serde_json::from_value(schema()).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let compiled = SchemaCompiler.compile(raw).unwrap();
    let trace = &compiled.traces()[0];
    let record = RecordView::new(b"0000000100000001OK");
    let expected = b"\0\0\0\0\0\0\0\x0800000001";
    assert_eq!(
        build_join_key(&record, &[0], trace).unwrap().as_bytes(),
        expected
    );
    let ((), seconds, bytes, traffic) = measure(|| {
        for _ in 0..count {
            black_box(
                build_join_key(black_box(&record), black_box(&[0]), black_box(trace)).unwrap(),
            );
        }
    });
    assert_eq!(traffic.successful_allocations, count);
    assert_eq!(traffic.successful_reallocations, 0);
    assert_eq!(traffic.cumulative_requested_bytes, count * 16);
    assert_eq!(bytes, 16);
    json!({"scenario":"join-key", "records":count, "seconds":seconds,
        "records_per_second":count as f64 / seconds, "heap_traffic":traffic,
        "peak_additional_requested_heap_bytes":bytes,
        "scope":"one logical component, exact length-prefixed bytes; drop each key; no IO or ContextStore"})
}

fn main() {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|s| s != "--bench")
        .collect();
    if args.is_empty() || args == ["--test"] {
        println!("Opt-in only: cargo bench --bench validation -- --output PATH [--smoke]");
        return;
    }
    assert!(
        args.len() == 2 || (args.len() == 3 && args[2] == "--smoke"),
        "expected --output PATH [--smoke]"
    );
    assert_eq!(args[0], "--output");
    // Reserve before running expensive work; never replace previous evidence.
    let output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])
        .unwrap();
    let counts: &[usize] = if args.len() == 3 {
        &[100]
    } else {
        &[10_000, 100_000, 1_000_000]
    };
    let mut results = Vec::new();
    for &count in counts {
        results.push(sample(count, false));
        results.push(sample(count, true));
        results.push(projection(count));
        results.push(join_key(count));
    }
    let rust = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .unwrap();
    assert!(rust.status.success());
    serde_json::to_writer_pretty(output, &json!({
        "benchmark_format_version":2, "synthetic":true, "compiler_commit":env!("LINTRA_BUILD_COMMIT"),
        "validator_version":env!("CARGO_PKG_VERSION"), "rustc":String::from_utf8(rust.stdout).unwrap().trim(),
        "harness_source_sha256":digest(include_bytes!("validation.rs")),
        "synthetic_schema_source_sha256":digest(include_bytes!("../tests/support/full_schema.rs")),
        "join_key_source_sha256":digest(include_bytes!("../src/context/join_key.rs")),
        "projection_source_sha256":digest(include_bytes!("../src/context/projected_record.rs")),
        "os":std::env::consts::OS, "arch":std::env::consts::ARCH, "optimized":!cfg!(debug_assertions),
        "memory_metric":"peak additional requested Rust heap bytes, not RSS; allocator instrumentation affects throughput",
        "thresholds":null, "results":results
    })).unwrap();
    println!("benchmark results written to {}", args[1]);
}
