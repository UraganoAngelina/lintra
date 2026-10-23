//! Native Windows console-event test, without extra test crates or CLI hooks.
#![cfg(windows)]
use std::{
    ffi::c_void,
    fs,
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

// Exact Win32 ABI declarations, used only in this Windows test process.
// Kernel32 is a system DLL, not a new runtime dependency of the engine.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleCP() -> u32;
    fn AllocConsole() -> i32;
    fn FreeConsole() -> i32;
    fn GetStdHandle(kind: u32) -> *mut c_void;
    fn SetStdHandle(kind: u32, handle: *mut c_void) -> i32;
    fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
}
struct Console(bool);
impl Console {
    /// A child group must share a real console with the event sender. If the CI
    /// test has none, allocate one and restore Cargo's inherited standard pipes.
    fn attach() -> Self {
        // SAFETY: GetConsoleCP has no pointer arguments or ownership effects.
        if unsafe { GetConsoleCP() } != 0 {
            return Self(false);
        }
        let kinds = [(-10_i32) as u32, (-11_i32) as u32, (-12_i32) as u32];
        // SAFETY: Win32 standard handle selectors; these borrowed handles remain
        // owned by the process and are never closed or dereferenced by this test.
        let handles = kinds.map(|kind| unsafe { GetStdHandle(kind) });
        // SAFETY: AllocConsole has no pointer arguments; this process has none.
        assert_ne!(
            unsafe { AllocConsole() },
            0,
            "{}",
            std::io::Error::last_os_error()
        );
        let console = Self(true);
        for (kind, handle) in kinds.into_iter().zip(handles) {
            // SAFETY: restore the exact inherited handle for the same selector.
            assert_ne!(
                unsafe { SetStdHandle(kind, handle) },
                0,
                "{}",
                std::io::Error::last_os_error()
            );
        }
        console
    }
}
impl Drop for Console {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: detach only the console allocated by this guard, after
            // the child has exited; no handle is closed through this call.
            unsafe {
                FreeConsole();
            }
        }
    }
}
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
struct ChildGuard(Option<Child>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
#[test]
fn console_break_returns_130_and_never_publishes_cancelled_reports() {
    let _console = Console::attach();
    let root = std::env::temp_dir().join(format!(
        "lintra console à {} {}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("schemas/SYNTHETIC/v1")).unwrap();
    let fixture = Fixture(root);
    let root = &fixture.0;
    fs::create_dir(root.join("reference-data")).unwrap();
    fs::create_dir(root.join("cwd")).unwrap();
    fs::write(
        root.join("schemas/SYNTHETIC/v1/flow.json"),
        include_bytes!("fixtures/application/schema.json"),
    )
    .unwrap();
    let mut input = fs::File::create(root.join("input data.txt")).unwrap();
    let block = b"001A  \n".repeat(8192);
    for _ in 0..128 {
        input.write_all(&block).unwrap();
    }
    drop(input);
    fs::write(root.join("errors.jsonl"), b"old errors").unwrap();
    fs::write(root.join("summary.json"), b"old summary").unwrap();
    fs::write(root.join("report.xlsx"), b"old workbook").unwrap();
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CTRL_BREAK_EVENT: u32 = 1;
    let child = Command::new(env!("CARGO_BIN_EXE_lintra"))
        .creation_flags(CREATE_NEW_PROCESS_GROUP)
        .current_dir(root.join("cwd"))
        .env_remove("LINTRA_SCHEMA_DIR")
        .env_remove("LINTRA_REFERENCE_DATA_DIR")
        .args([
            "validate",
            "--flow",
            "SYNTHETIC",
            "--schema-version",
            "v1",
            "--trace",
            "T1",
            "--encoding",
            "utf8",
        ])
        .arg("--schema-dir")
        .arg(root.join("schemas"))
        .arg("--reference-data-dir")
        .arg(root.join("reference-data"))
        .arg("--input")
        .arg(root.join("input data.txt"))
        .arg("--errors")
        .arg(root.join("errors.jsonl"))
        .arg("--summary")
        .arg(root.join("summary.json"))
        .arg("--xlsx")
        .arg(root.join("report.xlsx"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = ChildGuard(Some(child));
    let process = child.0.as_mut().unwrap();
    let mut stderr = BufReader::new(process.stderr.take().unwrap());
    let mut first = String::new();
    stderr.read_line(&mut first).unwrap();
    assert_eq!(first, "{\"event\":\"run_started\"}\n");
    // SAFETY: signal only the explicitly created child process group sharing
    // this console. Nonzero PID prevents a broadcast to Cargo or other tests.
    assert_ne!(
        unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, process.id()) },
        0,
        "{}",
        std::io::Error::last_os_error()
    );
    let out = child.0.take().unwrap().wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(130));
    let human = std::str::from_utf8(&out.stdout).unwrap();
    assert!(human.starts_with("Result: CANCELLED\n"));
    assert!(human.contains("results are incomplete"));
    assert!(!human.contains("No validation errors found"));
    assert_eq!(fs::read(root.join("errors.jsonl")).unwrap(), b"old errors");
    assert_eq!(fs::read(root.join("summary.json")).unwrap(), b"old summary");
    assert!(!root.join("errors.jsonl.part").exists());
    assert!(!root.join("summary.json.part").exists());
    assert_eq!(fs::read(root.join("report.xlsx")).unwrap(), b"old workbook");
    assert!(!root.join("report.xlsx.part").exists());
}
