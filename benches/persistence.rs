//! Real SQLite synthetic qualification. Each volume runs in a fresh process;
//! OS peak RSS/working set includes SQLite C, Rust, fixtures and allocator costs.
//! No DEC-14 pass threshold is invented. This harness is never shipped to users.
use chrono::{TimeZone, Utc};
use lintra::*;
use rusqlite::{Connection, params};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Instant,
};
struct Provider(u8);
impl PersistenceKeyProvider for Provider {
    fn resolve(&self, id: &str) -> Result<Arc<PersistenceKey>, ProtectionError> {
        if id != "synthetic-key" {
            return Err(ProtectionError::KeyUnavailable);
        }
        Ok(Arc::new(PersistenceKey::new(vec![self.0; 64])?))
    }
}
fn namespace(unit: &str) -> PersistenceNamespace {
    PersistenceNamespace::new(vec![("unit".into(), NamespaceValue::String(unit.into()))]).unwrap()
}
fn schema() -> CompiledFlowSchema {
    // Extend only the synthetic fixture's key width; never a CAD/SIOC schema.
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../bundles/embedded/schemas/SYNTHETIC/v1/flow.json"
    ))
    .unwrap();

    raw["traces"][0]["record_length"] = serde_json::json!(11);
    raw["traces"][0]["fields"][0]["physical"]["length"] = serde_json::json!(8);
    raw["traces"][0]["fields"][1]["physical"]["start"] = serde_json::json!(9);
    SchemaCompiler
        .compile({
            let mut raw: RawFlowSchema = serde_json::from_value(raw).unwrap();
            raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
            raw
        })
        .unwrap()
}
fn make_row(
    p: &IdentityProtector,
    schema: &CompiledFlowSchema,
    unit: &str,
    key: usize,
    body: &str,
) -> StoredIdentity {
    let bytes = format!("{key:08}{body:3}").into_bytes();
    let view = RecordView::new(&bytes);
    let trace = &schema.traces()[0];
    let pk = build_join_key(&view, &trace.record_key().field_indexes, trace).unwrap();
    let projection = build_join_key(&view, &[1], trace).unwrap();
    let scope = IdentityProtectionScope::new(
        namespace(unit),
        FlowId("SYNTHETIC".into()),
        PersistenceSchemaScope::SchemaVersion {
            version: SchemaVersion("v1".into()),
        },
        TraceId("T1".into()),
        "record_key".into(),
    )
    .unwrap();
    p.protect_projection(&scope, &pk, &Sha256Digest([17; 32]), &projection)
        .unwrap()
}
fn source(unit: &str, id: &str) -> SourceSubmission {
    SourceSubmission::new(SourceSubmissionParts {
        namespace: namespace(unit),
        submission_id: id.into(),
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_hash: Sha256Digest([0; 32]),
        period: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        validator_version: "synthetic-test".into(),
        validation_mode: ValidationMode::LocalOnly,
        validation_status: RunStatus::CompletedValid,
        reference_data_versions: vec![],
        traces: vec![SubmissionTrace {
            trace_id: TraceId("T1".into()),
            presence: TracePresence::Absent,
            input_sha256: None,
        }],
        fingerprint: Fingerprint::new(
            "synthetic-unprotected".into(),
            "fixture".into(),
            b"SYNTHETIC-RAW-SUBMISSION-MARKER".to_vec(),
        )
        .unwrap(),
        state: SessionState::Validated,
    })
    .unwrap()
}
fn snapshot(
    repo: &SQLitePersistenceAdapter,
    unit: &str,
    rev: u64,
    id: &str,
    rows: Vec<StoredIdentity>,
) -> HistoricalSnapshot {
    HistoricalSnapshot::new(
        SnapshotRevision::new(rev),
        repo.protector()
            .protect_submission(&source(unit, id))
            .unwrap(),
        rows,
    )
    .unwrap()
}
fn publish(
    repo: &SQLitePersistenceAdapter,
    snapshot: &HistoricalSnapshot,
) -> Result<CommitReceipt, PersistenceError> {
    let expected = if snapshot.revision().value() == 1 {
        None
    } else {
        Some(SnapshotRevision::new(snapshot.revision().value() - 1))
    };
    let mut tx = repo.begin_transaction(snapshot.namespace(), expected)?;
    tx.stage_snapshot(snapshot)?;
    tx.commit()
}

fn create(path: &Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::create(path, Some("synthetic-key"), &Provider(0xaa)).unwrap()
}
fn open(path: &Path) -> SQLitePersistenceAdapter {
    SQLitePersistenceAdapter::open(path, Some("synthetic-key"), &Provider(0xaa)).unwrap()
}
#[cfg(target_os = "linux")]
fn peak_bytes() -> u64 {
    let status = fs::read_to_string("/proc/self/status").unwrap();
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap()
        * 1024
}
#[cfg(windows)]
fn peak_bytes() -> u64 {
    use windows_sys::Win32::System::{ProcessStatus::*, Threading::*};
    let mut counters = std::mem::MaybeUninit::<PROCESS_MEMORY_COUNTERS>::zeroed();
    // SAFETY: output is correctly sized/aligned; current process handle is valid.
    assert_ne!(
        unsafe {
            K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                counters.as_mut_ptr(),
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        },
        0
    );
    // SAFETY: the successful API call initialized counters.
    unsafe { counters.assume_init() }.PeakWorkingSetSize as u64
}
#[cfg(not(any(target_os = "linux", windows)))]
fn peak_bytes() -> u64 {
    panic!("qualification requires a supported native OS")
}
fn restart(root: &Path, output: &Path, count: usize) {
    let start = Instant::now();
    let repo = open(root);
    let reopen = start.elapsed().as_secs_f64();
    let schema = schema();
    for key in [0, count / 2, count - 1] {
        let expected = make_row(repo.protector(), &schema, "A", key, "A");
        assert_eq!(
            repo.lookup_identity(&namespace("A"), expected.identity())
                .unwrap(),
            Some(expected)
        );
    }
    fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"reopen_seconds":reopen,"peak_bytes":peak_bytes(),"recovered_samples":3}),
        )
        .unwrap(),
    )
    .unwrap();
}
fn measure(count: usize, output: &Path) {
    let base = std::env::current_dir()
        .unwrap()
        .join("target/sqlite-benchmark");
    fs::create_dir_all(&base).unwrap();
    let parent = tempfile::tempdir_in(&base).unwrap();
    let root = parent.path().join("archive");
    let repo = create(&root);
    let schema = schema();
    let started = Instant::now();
    let rows = (0..count)
        .map(|key| make_row(repo.protector(), &schema, "A", key, "A"))
        .collect();
    let candidate = snapshot(&repo, "A", 1, "synthetic-benchmark", rows);
    let prepare_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    publish(&repo, &candidate).unwrap();
    let commit_seconds = started.elapsed().as_secs_f64();
    let mut times = Vec::new();
    for iteration in 0..1000 {
        let expected = &candidate.identities()[(iteration * 7919) % count];
        let start = Instant::now();
        assert_eq!(
            repo.lookup_identity(&namespace("A"), expected.identity())
                .unwrap()
                .as_ref(),
            Some(expected)
        );
        times.push(start.elapsed().as_nanos() as u64);
    }
    times.sort_unstable();
    let bytes = fs::metadata(root.join("lintra.db")).unwrap().len();
    let before_reopen_peak = peak_bytes();
    drop(candidate);
    drop(repo);
    let restart_output = parent.path().join("restart.json");
    assert!(
        Command::new(std::env::current_exe().unwrap())
            .arg("--restart")
            .arg(&root)
            .arg(&restart_output)
            .arg(count.to_string())
            .status()
            .unwrap()
            .success()
    );
    let restart: serde_json::Value =
        serde_json::from_slice(&fs::read(restart_output).unwrap()).unwrap();
    // On-disk test-only table deliberately has no index. Compare identical
    // point predicates; never use this path in the adapter or per-record loop.
    let c = Connection::open(root.join("lintra.db")).unwrap();
    let table_start = Instant::now();
    c.execute_batch("CREATE TABLE qualification_unindexed AS SELECT ns,identity FROM latest")
        .unwrap();
    let index_fixture_seconds = table_start.elapsed().as_secs_f64();
    let ns = serde_json::to_vec(&namespace("A")).unwrap();
    let key = make_row(open(&root).protector(), &schema, "A", count - 1, "A");
    let id = serde_json::to_vec(key.identity()).unwrap();
    let mut indexed = 0.;
    let mut unindexed = 0.;
    for _ in 0..10 {
        let start = Instant::now();
        let n: i64 = c
            .query_row(
                "SELECT count(*) FROM latest WHERE ns=?1 AND identity=?2",
                params![ns, id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        indexed += start.elapsed().as_secs_f64();
        let start = Instant::now();
        let n: i64 = c
            .query_row(
                "SELECT count(*) FROM qualification_unindexed WHERE ns=?1 AND identity=?2",
                params![ns, id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        unindexed += start.elapsed().as_secs_f64();
    }
    let plan: Vec<String> = c
        .prepare("EXPLAIN QUERY PLAN SELECT ordinal FROM latest WHERE ns=?1 AND identity=?2")
        .unwrap()
        .query_map(params![ns, id], |r| r.get(3))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(
        plan.iter()
            .all(|p| p.contains("SEARCH") && !p.contains("SCAN"))
    );
    c.execute_batch("DROP TABLE qualification_unindexed")
        .unwrap();
    drop(c);
    let repo = open(&root);
    let settings = repo.settings().unwrap();
    let next = snapshot(&repo, "A", 2, "synthetic-contention", vec![]);
    let mut blocker = Connection::open(root.join("lintra.db")).unwrap();
    let held = blocker
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    let start = Instant::now();
    assert_eq!(
        publish(&repo, &next).unwrap_err(),
        PersistenceError::StorageBusy
    );
    let contention_reject_seconds = start.elapsed().as_secs_f64();
    drop(held);
    let start = Instant::now();
    publish(&repo, &next).unwrap();
    let contention_retry_commit_seconds = start.elapsed().as_secs_f64();

    let report = json!({"identities":count,"contention_reject_seconds":contention_reject_seconds,"contention_retry_commit_seconds":contention_retry_commit_seconds,"prepare_seconds":prepare_seconds,"commit_seconds":commit_seconds,"lookup_samples":1000,"lookup_p50_ns":times[500],"lookup_p95_ns":times[950],"lookup_max_ns":times[999],"peak_bytes_pre_restart":before_reopen_peak,"peak_bytes_total":peak_bytes(),"restart":restart,"database_bytes":bytes,"index_comparison_samples":10,"indexed_sql_mean_seconds":indexed/10.,"unindexed_sql_mean_seconds":unindexed/10.,"index_fixture_seconds":index_fixture_seconds,"lookup_plan":plan,"sqlite":settings.sqlite_version,"journal_mode":settings.journal_mode,"synchronous":settings.synchronous,"cache_kib":settings.cache_kib,"mmap_bytes":settings.mmap_bytes,"profile":if cfg!(debug_assertions){"debug-smoke"}else{"release"},"os":std::env::consts::OS,"memory_scope":"process peak includes C/Rust/fixtures; restart measured in fresh child"});
    fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!(
        "identities={count} commit_seconds={commit_seconds:.3} lookup_p95_ns={} peak_bytes={}",
        times[950],
        peak_bytes()
    );
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|s| s == "--case") {
        measure(
            args[1].to_str().unwrap().parse().unwrap(),
            Path::new(&args[2]),
        );
        return;
    }
    if args.first().is_some_and(|s| s == "--restart") {
        restart(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args[3].to_str().unwrap().parse().unwrap(),
        );
        return;
    }
    let output = args
        .windows(2)
        .find(|a| a[0] == "--output")
        .map(|a| PathBuf::from(&a[1]))
        .unwrap_or_else(|| PathBuf::from("target/persistence-benchmark.json"));
    let smoke = cfg!(debug_assertions) || args.iter().any(|s| s == "--smoke");
    let counts = if smoke {
        vec![1000]
    } else {
        vec![10_000, 100_000, 300_000]
    };
    let mut reports = Vec::new();
    for count in counts {
        let part = output.with_extension(format!("{count}.json"));
        assert!(
            Command::new(std::env::current_exe().unwrap())
                .arg("--case")
                .arg(count.to_string())
                .arg(&part)
                .status()
                .unwrap()
                .success()
        );
        reports
            .push(serde_json::from_slice::<serde_json::Value>(&fs::read(&part).unwrap()).unwrap());
        fs::remove_file(part).unwrap();
    }
    fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"task":"T-PERSIST-05","synthetic_only":true,"measurements":reports}),
        )
        .unwrap(),
    )
    .unwrap();
}
