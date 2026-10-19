use lintra::{BomKind, BomPolicy, InputError, NewlinePolicy, RecordReader, preflight_bom};
use std::{
    cell::Cell,
    error::Error,
    io::{self, BufReader, Cursor, Read},
    rc::Rc,
};

const SIGNATURES: &[(BomKind, &[u8])] = &[
    (BomKind::Utf8, b"\xef\xbb\xbf"),
    (BomKind::Utf16Le, b"\xff\xfe"),
    (BomKind::Utf16Be, b"\xfe\xff"),
    (BomKind::Utf32Le, b"\xff\xfe\0\0"),
    (BomKind::Utf32Be, b"\0\0\xfe\xff"),
];

#[test]
fn default_rejects_each_known_marker_as_a_technical_error_with_a_remedy() {
    assert_eq!(BomPolicy::default(), BomPolicy::Reject);
    for &(kind, signature) in SIGNATURES {
        let bytes = [signature, b"ABC\r\n"].concat();
        let error = preflight_bom(Cursor::new(bytes), BomPolicy::default()).unwrap_err();
        assert!(matches!(error,InputError::BomRejected(found) if found==kind));
        assert!(error.to_string().contains(&kind.to_string()));
        assert!(error.to_string().contains("BOM"));
        assert!(error.to_string().contains("StripKnownBom"));
        assert!(error.source().is_none());
    }
}

#[test]
fn explicit_strip_removes_only_the_initial_marker_and_does_not_transcode() {
    for &(kind, signature) in SIGNATURES {
        let data = [b"A  \r\n".as_slice(), signature, b"B\0\xff"].concat();
        let bytes = [signature, data.as_slice()].concat();
        let mut checked = preflight_bom(Cursor::new(bytes), BomPolicy::StripKnownBom).unwrap();
        assert_eq!(checked.detected_bom(), Some(kind));
        let mut output = Vec::new();
        checked.read_to_end(&mut output).unwrap();
        assert_eq!(output, data);
    }
}

#[test]
fn plain_empty_short_and_incomplete_signatures_are_preserved_byte_for_byte() {
    for data in [
        b"".as_slice(),
        b"A",
        b"AB",
        b"ABC",
        b"ABCD",
        b"ABCDE  \n",
        b"\xef",
        b"\xef\xbb",
        b"\xef\xbb\0",
        b"\xfe",
        b"\0\0\xfe",
        b"\xffA",
    ] {
        for policy in [BomPolicy::Reject, BomPolicy::StripKnownBom] {
            let mut checked = preflight_bom(Cursor::new(data), policy).unwrap();
            assert_eq!(checked.detected_bom(), None);
            assert_eq!(checked.read(&mut []).unwrap(), 0);
            let mut output = Vec::new();
            checked.read_to_end(&mut output).unwrap();
            assert_eq!(output, data);
        }
    }
    for &(_, signature) in SIGNATURES {
        let mut checked = preflight_bom(Cursor::new(signature), BomPolicy::StripKnownBom).unwrap();
        let mut output = Vec::new();
        checked.read_to_end(&mut output).unwrap();
        assert!(output.is_empty());
    }
}

#[test]
fn notepad_utf8_bom_and_crlf_integrate_with_the_record_reader() {
    let checked = preflight_bom(
        Cursor::new(b"\xef\xbb\xbfABC  \r\nDEF\r\n"),
        BomPolicy::StripKnownBom,
    )
    .unwrap();
    let mut reader = RecordReader::new(
        BufReader::with_capacity(1, checked),
        NewlinePolicy::AutoLfOrCrlf,
    );
    let mut buffer = Vec::new();
    let first = reader.read_next(&mut buffer).unwrap().unwrap();
    assert_eq!((first.line_number, first.bytes), (1, b"ABC  ".as_slice()));
    let second = reader.read_next(&mut buffer).unwrap().unwrap();
    assert_eq!((second.line_number, second.bytes), (2, b"DEF".as_slice()));
    assert_eq!(reader.read_next(&mut buffer).unwrap(), None);
}

struct Fragmented<R> {
    inner: R,
    interrupted: bool,
    bytes_read: Rc<Cell<usize>>,
}
impl<R: Read> Read for Fragmented<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let length = out.len().min(1);
        let count = self.inner.read(&mut out[..length])?;
        self.bytes_read.set(self.bytes_read.get() + count);
        Ok(count)
    }
}
#[test]
fn short_reads_and_interruption_do_not_break_detection_or_prefix_replay() {
    for &(kind, signature) in SIGNATURES {
        let data = [signature, b"ABCDE"].concat();
        let bytes_read = Rc::new(Cell::new(0));
        let fragmented = Fragmented {
            inner: Cursor::new(data),
            interrupted: false,
            bytes_read: bytes_read.clone(),
        };
        let mut checked = preflight_bom(fragmented, BomPolicy::StripKnownBom).unwrap();
        assert_eq!(checked.detected_bom(), Some(kind));
        assert_eq!(bytes_read.get(), 4);
        let mut output = Vec::new();
        checked.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"ABCDE");
        assert_eq!(bytes_read.get(), signature.len() + 5);
    }
}

struct Failed;
impl Read for Failed {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "synthetic preflight failure",
        ))
    }
}
#[test]
fn io_failure_is_preserved_without_returning_a_partially_checked_stream() {
    let error = match preflight_bom(Failed, BomPolicy::StripKnownBom) {
        Err(error) => error,
        Ok(_) => panic!("expected I/O failure"),
    };
    assert!(matches!(error, InputError::Read(_)));
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        io::ErrorKind::PermissionDenied
    );
}
