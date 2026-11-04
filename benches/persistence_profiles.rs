//! Opt-in synthetic storage qualification, not a normative SIOC/CAD validator.
//! Each backend/shape/volume runs in a fresh owned process. Memory is OS peak
//! RSS/working set including fixtures, Rust and SQLite C; never compare to the
//! requested-heap metric of benches/validation.rs as if they were equivalent.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
#[path = "../tests/support/persistence_profiles.rs"]
mod profiles;
use profiles::*;

/// Reap children on failure/unwind as well as success. A deadline fails the
/// measurement; it is never interpreted as an empty archive or a timing result.
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn child(args: &[String]) {
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(args)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(600);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "synthetic measurement child failed");
            return;
        }
        assert!(
            Instant::now() < deadline,
            "synthetic measurement deadline exceeded"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(target_os = "linux")]
fn peak_bytes() -> u64 {
    fs::read_to_string("/proc/self/status")
        .unwrap()
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
    // SAFETY: valid output buffer and size, current process pseudohandle. The
    // successful call initializes the entire structure before assume_init.
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
    unsafe { counters.assume_init() }.PeakWorkingSetSize as u64
}
#[cfg(not(any(target_os = "linux", windows)))]
fn peak_bytes() -> u64 {
    panic!("native qualification requires Linux or Windows")
}

fn shape(label: &str) -> Shape {
    match label {
        "sioc-shaped-synthetic" => Shape::Sioc,
        "cad-shaped-synthetic" => Shape::Cad,
        _ => panic!("unknown synthetic shape"),
    }
}
fn path_string(path: &Path) -> String {
    path.to_str()
        .expect("benchmark output path must be UTF-8")
        .to_owned()
}
fn save(path: &Path, value: &Value) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(file, value).unwrap();
}
fn restart(root: &Path, output: &Path, shape: Shape, count: usize) {
    let start = Instant::now();
    let repo = SQLitePersistenceAdapter::open(root, Some(KEY_ID), &Provider(0xab)).unwrap();
    let reopen_seconds = start.elapsed().as_secs_f64();
    let fixture = Fixture::new(shape, count);
    assert_eq!(
        repo.current_generation(&namespace("synthetic-a")).unwrap(),
        Some(SnapshotRevision::new(2))
    );
    assert!(
        repo.load_submission(&namespace("synthetic-a"), "synthetic-session-1")
            .unwrap()
            .is_some()
    );
    for i in 0..5 {
        let expected = fixture.row(repo.protector(), "synthetic-a", i, true);
        assert_eq!(
            repo.lookup_identity(&namespace("synthetic-a"), expected.identity())
                .unwrap(),
            Some(expected)
        );
    }
    assert_eq!(
        repo.current_generation(&namespace("synthetic-b")).unwrap(),
        None
    );
    save(
        output,
        &json!({"reopen_seconds":reopen_seconds,"peak_bytes":peak_bytes(),"verified_trace_samples":5,"generation":2}),
    );
}
fn sample(backend: &str, shape: Shape, count: usize, output: &Path) {
    let parent = PathBuf::from("target/persistence-profile-runs");
    fs::create_dir_all(&parent).unwrap();
    let temporary = tempfile::tempdir_in(parent).unwrap();
    let root = std::env::current_dir()
        .unwrap()
        .join(temporary.path())
        .join("archive");
    let p = IdentityProtector::preflight(Some(KEY_ID), &Provider(0xab), None).unwrap();
    let repo: Box<dyn PersistentContextRepository> = match backend {
        "memory-fake" => Box::new(InMemoryPersistenceRepository::with_protection_audit(
            p.audit().unwrap(),
        )),
        "sqlite" => Box::new(
            SQLitePersistenceAdapter::create(&root, Some(KEY_ID), &Provider(0xab)).unwrap(),
        ),
        _ => panic!("unknown measurement backend"),
    };
    let fixture = Fixture::new(shape, count);
    let mut timings = Vec::new();
    for revision in 1..=2 {
        let start = Instant::now();
        let snapshot = fixture.snapshot(&p, "synthetic-a", revision, revision == 2);
        let preparation = start.elapsed().as_secs_f64();
        let start = Instant::now();
        assert_eq!(
            publish(repo.as_ref(), &snapshot).unwrap().revision(),
            snapshot.revision()
        );
        timings.push(json!({"revision":revision,"prepare_seconds":preparation,"commit_seconds":start.elapsed().as_secs_f64()}));
    }
    let mut lookup = Vec::new();
    for i in 0..100 {
        let expected = fixture.row(&p, "synthetic-a", (i * 7919) % count, true);
        let start = Instant::now();
        assert_eq!(
            repo.lookup_identity(&namespace("synthetic-a"), expected.identity())
                .unwrap(),
            Some(expected)
        );
        lookup.push(start.elapsed().as_nanos() as u64);
    }
    lookup.sort_unstable();
    let start = Instant::now();
    let first = repo
        .load_snapshot(&namespace("synthetic-a"), SnapshotRevision::new(1))
        .unwrap()
        .unwrap();
    let load_snapshot_seconds = start.elapsed().as_secs_f64();
    assert_eq!(first.identities().len(), count);
    assert_eq!(
        first.identities()[0],
        fixture.row(&p, "synthetic-a", 0, false)
    );
    assert_eq!(
        repo.current_generation(&namespace("synthetic-b")).unwrap(),
        None
    );
    drop(first);
    let peak = peak_bytes();
    drop(repo);
    let (database_bytes, reopened) = if backend == "sqlite" {
        let bytes = fs::metadata(root.join("lintra.db")).unwrap().len();
        let result = temporary.path().join("restart.json");
        child(&[
            "--restart".into(),
            path_string(&root),
            path_string(&result),
            shape.label().into(),
            count.to_string(),
        ]);
        (
            Some(bytes),
            Some(serde_json::from_slice::<Value>(&fs::read(result).unwrap()).unwrap()),
        )
    } else {
        (None, None)
    };
    save(
        output,
        &json!({"backend":backend,"shape":shape.label(),"pk_component_counts":shape.arities(),
        "records_per_snapshot":count,"snapshots":2,"timings":timings,"lookup_samples":100,
        "lookup_p50_ns":lookup[50],"lookup_p95_ns":lookup[95],"load_snapshot_seconds":load_snapshot_seconds,
        "peak_process_bytes":peak,"database_bytes":database_bytes,"restart":reopened,
        "five_traces_verified":true,"namespace_isolation_verified":true,
        "memory_scope":"OS peak RSS/working set includes fixtures, Rust and SQLite C; memory backend is a transaction fake"}),
    );
}
fn main() {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|s| s != "--bench")
        .collect();
    if args.first().is_some_and(|s| s == "--case") {
        assert_eq!(args.len(), 5);
        sample(
            &args[1],
            shape(&args[2]),
            args[3].parse().unwrap(),
            Path::new(&args[4]),
        );
        return;
    }
    if args.first().is_some_and(|s| s == "--restart") {
        assert_eq!(args.len(), 5);
        restart(
            Path::new(&args[1]),
            Path::new(&args[2]),
            shape(&args[3]),
            args[4].parse().unwrap(),
        );
        return;
    }
    if args.is_empty() || args == ["--test"] {
        println!(
            "Opt-in only: cargo bench --bench persistence_profiles -- --output PATH [--smoke]"
        );
        return;
    }
    assert!(args.len() == 2 || (args.len() == 3 && args[2] == "--smoke"));
    assert_eq!(args[0], "--output");
    // Reserve the final output before expensive work, refusing existing evidence.
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])
        .unwrap();
    let counts: &[usize] = if args.len() == 3 {
        &[100]
    } else {
        &[10_000, 100_000, 300_000]
    };
    let temporary = tempfile::tempdir().unwrap();
    let mut results = Vec::new();
    for &count in counts {
        for shape in [Shape::Sioc, Shape::Cad] {
            for backend in ["memory-fake", "sqlite"] {
                let part = temporary
                    .path()
                    .join(format!("{backend}-{}-{count}.json", shape.label()));
                child(&[
                    "--case".into(),
                    backend.into(),
                    shape.label().into(),
                    count.to_string(),
                    path_string(&part),
                ]);
                results.push(serde_json::from_slice::<Value>(&fs::read(part).unwrap()).unwrap());
                println!("qualified {backend} {} records={count}", shape.label());
            }
        }
    }
    let hex = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    let rust = Command::new("rustc").arg("--version").output().unwrap();
    assert!(rust.status.success());
    serde_json::to_writer_pretty(file,&json!({"task":"T-PERSIST-12","format_version":1,
        "synthetic_only":true,"normative_validation":false,"optimized":!cfg!(debug_assertions),
        "os":std::env::consts::OS,"arch":std::env::consts::ARCH,"compiler_commit":env!("LINTRA_BUILD_COMMIT"),
        "harness_sha256":hex(include_bytes!("persistence_profiles.rs")),
        "fixture_sha256":hex(include_bytes!("../tests/support/persistence_profiles.rs")),
        "sqlite_adapter_sha256":hex(include_bytes!("../src/persistence/sqlite/mod.rs")),
        "memory_adapter_sha256":hex(include_bytes!("../src/persistence/memory.rs")),
        "rustc":String::from_utf8(rust.stdout).unwrap().trim(),
        "thresholds":null,"results":results})).unwrap();
}
