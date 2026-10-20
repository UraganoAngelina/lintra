//! Input-layer integration, with synthetic bytes in memory and native runner files.
use lintra::{
    BomKind, BomPolicy, EncodingSpec, InputError, NewlinePolicy, RecordReader, RecordView,
    preflight_bom, render_diagnostic_value,
};
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
struct TempFile(PathBuf);
impl TempFile {
    fn new(bytes: &[u8]) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        loop {
            let index = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "lintra input sintetico à {} {now} {index}.txt",
                std::process::id()
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let temporary = Self(path);
                    file.write_all(bytes).unwrap();
                    return temporary;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create synthetic input: {error}"),
            }
        }
    }
    fn open(&self) -> File {
        File::open(&self.0).unwrap()
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn assert_records<R: BufRead>(input: R, policy: NewlinePolicy, expected: &[&[u8]]) {
    let mut reader = RecordReader::new(input, policy);
    let mut buffer = b"stale buffer".to_vec();
    for (index, expected) in expected.iter().enumerate() {
        let record = reader.read_next(&mut buffer).unwrap().unwrap();
        assert_eq!(record.line_number, index as u64 + 1);
        assert_eq!(record.bytes, *expected);
        let view = RecordView::new(record.bytes);
        assert_eq!(view.raw().as_ptr(), record.bytes.as_ptr());
        assert_eq!(view.raw(), *expected);
    }
    for _ in 0..2 {
        assert_eq!(reader.read_next(&mut buffer).unwrap(), None);
        assert!(buffer.is_empty());
        assert_eq!(reader.line_number(), expected.len() as u64);
    }
}

#[test]
fn memory_and_native_files_share_framing_even_at_buffer_boundaries() {
    type Case<'a> = (&'a [u8], NewlinePolicy, &'a [&'a [u8]]);
    let cases: &[Case<'_>] = &[
        (b"", NewlinePolicy::AutoLfOrCrlf, &[]),
        (
            b"A  \nB \r\n\nC\rD",
            NewlinePolicy::AutoLfOrCrlf,
            &[b"A  ", b"B ", b"", b"C\rD"],
        ),
        (b"\r\n\r\n", NewlinePolicy::AutoLfOrCrlf, &[b"", b""]),
        (b"\xff\0 \r\n", NewlinePolicy::AutoLfOrCrlf, &[b"\xff\0 "]),
        (
            b"A\r\nB\nC\r",
            NewlinePolicy::LfOnly,
            &[b"A\r", b"B", b"C\r"],
        ),
        (
            b"A\nB\r\nC\rD\n",
            NewlinePolicy::CrlfOnly,
            &[b"A\nB", b"C\rD\n"],
        ),
        (b"\r\n", NewlinePolicy::CrlfOnly, &[b""]),
        (b"A\r", NewlinePolicy::CrlfOnly, &[b"A\r"]),
    ];
    for &(bytes, policy, expected) in cases {
        let file = TempFile::new(bytes);
        for capacity in [1, 2, 7, 8192] {
            assert_records(
                BufReader::with_capacity(capacity, Cursor::new(bytes)),
                policy,
                expected,
            );
            assert_records(
                BufReader::with_capacity(capacity, file.open()),
                policy,
                expected,
            );
        }
    }
}

#[test]
fn bom_file_preflight_reader_view_and_diagnostics_preserve_exact_payload() {
    let file = TempFile::new(b"\xef\xbb\xbfA \xff\0 \r\nB  \n");
    assert!(matches!(
        preflight_bom(file.open(), BomPolicy::Reject),
        Err(InputError::BomRejected(BomKind::Utf8))
    ));
    for capacity in [1, 2, 8192] {
        let checked = preflight_bom(file.open(), BomPolicy::StripKnownBom).unwrap();
        let mut reader = RecordReader::new(
            BufReader::with_capacity(capacity, checked),
            NewlinePolicy::AutoLfOrCrlf,
        );
        let mut buffer = Vec::new();
        let record = reader.read_next(&mut buffer).unwrap().unwrap();
        assert_eq!(record.line_number, 1);
        let view = RecordView::new(record.bytes);
        let diagnostic = render_diagnostic_value(view.raw(), &EncodingSpec::Utf8);
        assert_eq!(diagnostic.display, "A \u{fffd}\\0 ");
        assert_eq!(diagnostic.raw_hex.as_deref(), Some("41 20 FF 00 20"));
        assert_eq!(view.raw(), b"A \xff\0 ");
        assert_eq!(
            reader.read_next(&mut buffer).unwrap().unwrap().bytes,
            b"B  "
        );
        assert!(reader.read_next(&mut buffer).unwrap().is_none());
    }
}

#[test]
fn megabyte_record_and_empty_files_have_no_size_or_phantom_line_assumptions() {
    let mut payload = vec![b'X'; 1024 * 1024];
    payload.extend_from_slice(b"  \r");
    let input = [payload.as_slice(), b"\r\nZ"].concat();
    let file = TempFile::new(&input);
    for capacity in [3, 8192] {
        assert_records(
            BufReader::with_capacity(capacity, file.open()),
            NewlinePolicy::AutoLfOrCrlf,
            &[&payload, b"Z"],
        );
        assert_records(
            BufReader::with_capacity(capacity, Cursor::new(&input)),
            NewlinePolicy::AutoLfOrCrlf,
            &[&payload, b"Z"],
        );
    }
    for bytes in [b"".as_slice(), b"\xef\xbb\xbf"] {
        let empty = TempFile::new(bytes);
        let checked = preflight_bom(empty.open(), BomPolicy::StripKnownBom).unwrap();
        assert_records(BufReader::new(checked), NewlinePolicy::AutoLfOrCrlf, &[]);
    }
}

#[derive(Debug)]
struct FailAfterPrefix(Cursor<Vec<u8>>);
impl Read for FailAfterPrefix {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let source = self.fill_buf()?;
        let size = source.len().min(out.len());
        out[..size].copy_from_slice(&source[..size]);
        self.consume(size);
        Ok(size)
    }
}
impl BufRead for FailAfterPrefix {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.0.position() == self.0.get_ref().len() as u64 {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic row failure",
            ))
        } else {
            self.0.fill_buf()
        }
    }
    fn consume(&mut self, amount: usize) {
        self.0.consume(amount);
    }
}

#[test]
fn io_failure_at_each_row_keeps_context_cause_and_terminal_state() {
    for (policy, delimiter) in [
        (NewlinePolicy::AutoLfOrCrlf, b"\n".as_slice()),
        (NewlinePolicy::LfOnly, b"\n"),
        (NewlinePolicy::CrlfOnly, b"\r\n"),
    ] {
        for line in 1..=4 {
            for partial in [b"".as_slice(), b"partial"] {
                let mut input = Vec::new();
                for _ in 1..line {
                    input.extend_from_slice(b"OK");
                    input.extend_from_slice(delimiter);
                }
                input.extend_from_slice(partial);
                let mut reader = RecordReader::new(FailAfterPrefix(Cursor::new(input)), policy);
                let mut buffer = Vec::new();
                for number in 1..line {
                    assert_eq!(
                        reader.read_next(&mut buffer).unwrap().unwrap().line_number,
                        number
                    );
                }
                let error = reader.read_next(&mut buffer).unwrap_err();
                assert_eq!(error.line_number(), Some(line));
                assert!(error.to_string().contains(&format!("line {line}")));
                assert_eq!(
                    error
                        .source()
                        .unwrap()
                        .downcast_ref::<io::Error>()
                        .unwrap()
                        .kind(),
                    io::ErrorKind::PermissionDenied
                );
                assert_eq!(reader.line_number(), line - 1);
                assert_eq!(buffer, partial);
                assert!(matches!(
                    reader.read_next(&mut buffer),
                    Err(InputError::PreviousFailure)
                ));
                assert!(buffer.is_empty());
            }
        }
    }
}

#[test]
fn preflight_has_no_line_and_counter_overflow_never_hides_the_io_cause() {
    let error = preflight_bom(FailAfterPrefix(Cursor::new(vec![])), BomPolicy::Reject).unwrap_err();
    assert_eq!(error.line_number(), None);
    let error = InputError::RecordRead {
        last_completed_line: u64::MAX,
        source: io::Error::other("synthetic"),
    };
    assert_eq!(error.line_number(), None);
    assert!(error.to_string().contains(&u64::MAX.to_string()));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
}
