//! OS I/O probes only: these do not implement input normalization or domain policies.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new() -> io::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock should be after the Unix epoch")
            .as_nanos();

        loop {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "lintra platform paths {} {timestamp} {sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn byte_roundtrip_preserves_data_under_paths_with_spaces() -> io::Result<()> {
    let directory = TempDirectory::new()?;
    let input_directory = directory.0.join("synthetic input files");
    fs::create_dir(&input_directory)?;

    let cases: [(&str, &[u8]); 4] = [
        ("lf data.bin", b"synthetic\nsecond line\n"),
        ("crlf data.bin", b"synthetic\r\nsecond line\r\n"),
        ("bom data.bin", b"\xef\xbb\xbfsynthetic\r\n"),
        ("trailing spaces.bin", b"synthetic  \nsecond line   "),
    ];

    for (name, expected) in cases {
        let path = input_directory.join(name);
        fs::write(&path, expected)?;
        assert_eq!(fs::read(&path)?, expected, "byte mismatch for {name}");
    }

    Ok(())
}

#[test]
fn missing_file_under_path_with_spaces_returns_not_found() -> io::Result<()> {
    let directory = TempDirectory::new()?;
    let path = directory.0.join("missing input file.bin");

    let error = fs::read(path).expect_err("a missing file must not be read successfully");
    assert_eq!(error.kind(), io::ErrorKind::NotFound);

    Ok(())
}
