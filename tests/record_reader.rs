use lintra::{InputError, NewlinePolicy, RecordReader};
use std::{
    error::Error,
    io::{self, BufRead, BufReader, Cursor, Read},
};

fn records(bytes: &[u8], policy: NewlinePolicy) -> Vec<Vec<u8>> {
    let mut reader = RecordReader::new(BufReader::with_capacity(1, Cursor::new(bytes)), policy);
    let mut buffer = Vec::new();
    let mut result = Vec::new();
    while let Some(record) = reader.read_next(&mut buffer).unwrap() {
        result.push(record.bytes.to_vec());
        assert_eq!(record.line_number, result.len() as u64);
    }
    assert_eq!(reader.line_number(), result.len() as u64);
    assert!(buffer.is_empty());
    result
}

#[test]
fn auto_handles_mixed_terminators_without_trimming_or_decoding() {
    assert_eq!(
        records(b"A  \nB \r\n\r\n\nC\rD\r", NewlinePolicy::AutoLfOrCrlf),
        [
            b"A  ".to_vec(),
            b"B ".to_vec(),
            Vec::<u8>::new(),
            Vec::<u8>::new(),
            b"C\rD\r".to_vec()
        ]
    );
    assert_eq!(
        records(
            &[0xff, 0, 0x80, b' ', b'\r', b'\n'],
            NewlinePolicy::AutoLfOrCrlf
        ),
        [vec![0xff, 0, 0x80, b' ']]
    );
}

#[test]
fn lf_only_removes_only_lf_and_crlf_only_frames_only_crlf() {
    assert_eq!(
        records(b"A\r\nB\nC\r", NewlinePolicy::LfOnly),
        [b"A\r".to_vec(), b"B".to_vec(), b"C\r".to_vec()]
    );
    assert_eq!(
        records(b"A\nB\r\nC\rD\nE\r\nF\n", NewlinePolicy::CrlfOnly),
        [b"A\nB".to_vec(), b"C\rD\nE".to_vec(), b"F\n".to_vec()]
    );
}

#[test]
fn empty_input_and_final_delimiter_never_create_a_phantom_record() {
    for policy in [
        NewlinePolicy::AutoLfOrCrlf,
        NewlinePolicy::LfOnly,
        NewlinePolicy::CrlfOnly,
    ] {
        assert!(records(b"", policy).is_empty());
        assert_eq!(records(b"A", policy), [b"A".to_vec()]);
    }
    assert_eq!(
        records(b"A\n", NewlinePolicy::AutoLfOrCrlf),
        [b"A".to_vec()]
    );
    assert_eq!(records(b"A\r\n", NewlinePolicy::CrlfOnly), [b"A".to_vec()]);
    assert_eq!(
        records(b"\n\n", NewlinePolicy::AutoLfOrCrlf),
        [Vec::<u8>::new(), Vec::<u8>::new()]
    );
    assert_eq!(
        records(b"\r\n", NewlinePolicy::CrlfOnly),
        [Vec::<u8>::new()]
    );
    assert_eq!(
        records(b"\r", NewlinePolicy::AutoLfOrCrlf),
        [b"\r".to_vec()]
    );
}

#[test]
fn buffer_is_cleared_reused_and_borrowed_without_being_retained_by_reader() {
    let mut reader = RecordReader::new(Cursor::new(b"A  \nBC\n"), NewlinePolicy::AutoLfOrCrlf);
    let mut buffer = Vec::with_capacity(64);
    buffer.extend_from_slice(b"stale data");
    let pointer = buffer.as_ptr();
    let first = reader.read_next(&mut buffer).unwrap().unwrap();
    assert_eq!(first.bytes, b"A  ");
    assert_eq!(first.bytes.as_ptr(), pointer);
    assert_eq!(reader.line_number(), 1);
    let second = reader.read_next(&mut buffer).unwrap().unwrap();
    assert_eq!(second.bytes, b"BC");
    assert_eq!(second.bytes.as_ptr(), pointer);
    assert_eq!(reader.line_number(), 2);
    assert_eq!(reader.read_next(&mut buffer).unwrap(), None);
    assert_eq!(reader.line_number(), 2);
    assert!(buffer.is_empty());
    assert_eq!(buffer.as_ptr(), pointer);
    assert_eq!(reader.read_next(&mut buffer).unwrap(), None);
    assert_eq!(reader.line_number(), 2);
}

#[test]
fn records_borrow_only_their_caller_buffer_not_the_reader() {
    let mut reader = RecordReader::new(Cursor::new(b"A\nB\n"), NewlinePolicy::AutoLfOrCrlf);
    let mut one = Vec::new();
    let mut two = Vec::new();
    let first = reader.read_next(&mut one).unwrap().unwrap();
    let second = reader.read_next(&mut two).unwrap().unwrap();
    assert_eq!(
        (first.bytes, second.bytes),
        (b"A".as_slice(), b"B".as_slice())
    );
}

#[test]
fn large_record_and_bom_are_preserved_as_raw_bytes() {
    let mut bytes = vec![b'X'; 65536];
    bytes.extend_from_slice(b"  \r\nY");
    let output = records(&bytes, NewlinePolicy::AutoLfOrCrlf);
    assert_eq!(output[0], bytes[..65538]);
    assert_eq!(output[1], b"Y");
    assert_eq!(
        records(b"\xef\xbb\xbfA\n", NewlinePolicy::AutoLfOrCrlf),
        [b"\xef\xbb\xbfA".to_vec()]
    );
}

struct FailingInput {
    prefix: Cursor<Vec<u8>>,
}
impl Read for FailingInput {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let bytes = self.fill_buf()?;
        let count = out.len().min(bytes.len());
        out[..count].copy_from_slice(&bytes[..count]);
        self.consume(count);
        Ok(count)
    }
}
impl BufRead for FailingInput {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.prefix.position() as usize == self.prefix.get_ref().len() {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic failure",
            ))
        } else {
            self.prefix.fill_buf()
        }
    }
    fn consume(&mut self, count: usize) {
        self.prefix.consume(count);
    }
}
#[test]
fn partial_io_failure_is_not_a_record_and_cannot_resume_as_a_truncated_record() {
    let mut reader = RecordReader::new(
        FailingInput {
            prefix: Cursor::new(b"A\npartial".to_vec()),
        },
        NewlinePolicy::AutoLfOrCrlf,
    );
    let mut buffer = Vec::new();
    assert_eq!(
        reader.read_next(&mut buffer).unwrap().unwrap().line_number,
        1
    );
    let error = reader.read_next(&mut buffer).unwrap_err();
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
    assert_eq!(buffer, b"partial");
    assert_eq!(reader.line_number(), 1);
    assert!(matches!(
        reader.read_next(&mut buffer),
        Err(InputError::PreviousFailure)
    ));
    assert_eq!(reader.line_number(), 1);
    assert!(buffer.is_empty());
}
