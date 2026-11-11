//! Test-only barriers/faults. Release binaries contain no environment hook.
use super::*;
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fault {
    Flush,
    FileSync,
    Rename,
    DirectorySync,
}
pub(super) fn inject(selected: Option<Fault>, current: Fault) -> io::Result<()> {
    if selected == Some(current) {
        Err(io::Error::new(
            io::ErrorKind::StorageFull,
            "synthetic writeback fault",
        ))
    } else {
        Ok(())
    }
}
pub(super) fn checkpoint(phase: &str, destination: &Path) {
    if std::env::var("LINTRA_REPORT_TEST_PHASE").ok().as_deref() != Some(phase)
        || std::env::var_os("LINTRA_REPORT_TEST_DESTINATION")
            .is_none_or(|p| Path::new(&p) != destination)
    {
        return;
    }
    let root = destination.parent().unwrap();
    fs::write(root.join("barrier.ready"), phase).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while !root.join("release.barrier").exists() {
        assert!(
            Instant::now() < deadline,
            "owned test child barrier timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn completion() -> ReportCompletion {
    ReportCompletion {
        run_status: RunStatus::CompletedInvalid,
        truncated: false,
    }
}

fn fixture() -> tempfile::TempDir {
    // Use the checkout's real local filesystem, not an ephemeral /tmp tmpfs.
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/report-durability-tests");
    fs::create_dir_all(&parent).unwrap();
    tempfile::tempdir_in(parent).unwrap()
}

#[test]
fn injected_flush_sync_and_rename_failures_preserve_the_previous_report() {
    for (fault, expected) in [
        (Fault::Flush, OutputOperation::Flush),
        (Fault::FileSync, OutputOperation::Sync),
        (Fault::Rename, OutputOperation::Publish),
    ] {
        let root = fixture();
        let path = root.path().join("synthetic.report");
        fs::write(&path, b"old complete output").unwrap();
        let mut pending =
            AtomicReportFile::create_replacing(&path, PartialOutputPolicy::Delete).unwrap();
        pending.writer().write_all(b"new complete output").unwrap();
        pending.fault = Some(fault);
        let error = pending.complete(completion()).unwrap_err();
        assert!(!error.output_was_published());
        assert!(
            matches!(error, ReportError::Output { operation, ref source }
            if operation == expected && source.kind() == io::ErrorKind::StorageFull)
        );
        assert_eq!(fs::read(&path).unwrap(), b"old complete output");
        assert!(!path.with_file_name("synthetic.report.part").exists());
    }
}

#[test]
fn directory_sync_failure_is_technical_but_does_not_claim_publication_was_undone() {
    let root = fixture();
    let path = root.path().join("synthetic.report");
    fs::write(&path, b"old complete output").unwrap();
    let mut pending =
        AtomicReportFile::create_replacing(&path, PartialOutputPolicy::Delete).unwrap();
    pending.writer().write_all(b"new complete output").unwrap();
    pending.fault = Some(Fault::DirectorySync);
    let error = pending.complete(completion()).unwrap_err();
    assert!(error.output_was_published());
    assert!(
        matches!(error, ReportError::PublishedDurabilityUnconfirmed { ref source }
        if source.kind() == io::ErrorKind::StorageFull)
    );
    assert_eq!(fs::read(&path).unwrap(), b"new complete output");
    assert!(!path.with_file_name("synthetic.report.part").exists());
    assert!(error.to_string().contains("published"));
}

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
const OLD: &[u8] = b"synthetic old complete report\n";
const NEW: &[u8] = b"synthetic new complete report\n";

#[test]
#[ignore = "Owned subprocess helper; parent invokes it with a private synthetic destination"]
fn process_worker() {
    let path = PathBuf::from(std::env::var_os("LINTRA_REPORT_TEST_DESTINATION").unwrap());
    let mut pending =
        AtomicReportFile::create_replacing(&path, PartialOutputPolicy::Delete).unwrap();
    checkpoint("created", &path);
    pending.writer().write_all(NEW).unwrap();
    checkpoint("written", &path);
    assert_eq!(
        pending.complete(completion()).unwrap(),
        AtomicOutputOutcome::Published
    );
    checkpoint("receipt_lost", &path);
}

#[test]
fn actual_kill_before_and_after_rename_preserves_complete_outputs_and_blocks_stale_part_reuse() {
    for phase in [
        "created",
        "written",
        "file_synced",
        "renamed",
        "publication_acknowledged",
        "receipt_lost",
    ] {
        let root = fixture();
        let path = root.path().join("synthetic.report");
        let part = root.path().join("synthetic.report.part");
        fs::write(&path, OLD).unwrap();
        let mut child = OwnedChild(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "report::atomic::durability_tests::process_worker",
                    "--ignored",
                    "--nocapture",
                ])
                .env("LINTRA_REPORT_TEST_DESTINATION", &path)
                .env("LINTRA_REPORT_TEST_PHASE", phase)
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(20);
        while !root.path().join("barrier.ready").exists() {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "child exited before {phase}"
            );
            assert!(Instant::now() < deadline, "child did not reach {phase}");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            fs::read(root.path().join("barrier.ready")).unwrap(),
            phase.as_bytes()
        );
        child.0.kill().unwrap();
        assert!(!child.0.wait().unwrap().success());
        let published = matches!(
            phase,
            "renamed" | "publication_acknowledged" | "receipt_lost"
        );
        assert_eq!(fs::read(&path).unwrap(), if published { NEW } else { OLD });
        assert_eq!(part.exists(), !published);
        if !published {
            let stale_bytes = fs::read(&part).unwrap();
            assert!(
                matches!(AtomicReportFile::create_replacing(&path, PartialOutputPolicy::Delete),
                Err(ReportError::Output { operation: OutputOperation::Create, ref source })
                if source.kind() == io::ErrorKind::AlreadyExists)
            );
            assert_eq!(fs::read(&part).unwrap(), stale_bytes);
            // Explicit isolated recovery of this test-owned fixture, not an
            // automatic product purge or permission to remove unknown parts.
            fs::rename(&part, root.path().join("isolated-stale.part")).unwrap();
            let mut retry =
                AtomicReportFile::create_replacing(&path, PartialOutputPolicy::Delete).unwrap();
            retry.writer().write_all(NEW).unwrap();
            assert_eq!(
                retry.complete(completion()).unwrap(),
                AtomicOutputOutcome::Published
            );
        }
        assert_eq!(fs::read(&path).unwrap(), NEW);
    }
}
