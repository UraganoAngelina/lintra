//! T-P17-06: reproducible generated inputs and real boundary faults.
//! No random dependency, patient fixture, performance threshold or panic recovery.
use lintra::*;
use serde_json::json;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Cursor, Read, Write},
    path::Path,
    sync::Arc,
};

/// Fixed-state generator makes failures reproducible on both supported x64 OSes.
fn next(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed ^ (*seed >> 29)
}

#[test]
fn generated_ranges_agree_with_wide_integer_bounds_and_borrow_original_bytes() {
    let bytes: Vec<u8> = (0..=255).collect();
    let mut seed = 0x4c49_4e54_5241u64;
    for iteration in 0usize..20_000 {
        let start = if iteration.is_multiple_of(3) {
            next(&mut seed) as usize
        } else {
            next(&mut seed) as usize % 300
        };
        let len = if iteration.is_multiple_of(5) {
            next(&mut seed) as usize
        } else {
            next(&mut seed) as usize % 300
        };
        let end = start as u128 + len as u128;
        let admissible = len != 0 && end <= usize::MAX as u128;
        let range = ByteRange::new(start, len);
        assert_eq!(range.is_ok(), admissible);
        if let Ok(range) = range {
            assert_eq!(range.end_exclusive() as u128, end);
            match range.slice(&bytes) {
                Some(view) => {
                    assert!(end <= bytes.len() as u128);
                    assert_eq!(view, &bytes[start..end as usize]);
                    assert_eq!(view.as_ptr(), bytes[start..].as_ptr());
                }
                None => assert!(end > bytes.len() as u128),
            }
        }
    }
    for (start, len) in [
        (0, 0),
        (usize::MAX, 1),
        (1, usize::MAX),
        (usize::MAX - 1, 1),
    ] {
        assert_eq!(
            ByteRange::new(start, len).is_ok(),
            len != 0 && start as u128 + len as u128 <= usize::MAX as u128
        );
    }
}

#[test]
fn generated_payloads_survive_mixed_terminators_and_fragmented_reads() {
    let mut seed = 0x1706_2026;
    for case in 0usize..512 {
        let mut expected = Vec::new();
        for row in 0..8 {
            let len = next(&mut seed) as usize % 32;
            let mut data = Vec::new();
            for _ in 0..len {
                let byte = next(&mut seed) as u8;
                data.push(if byte == b'\n' || byte == b'\r' {
                    b' '
                } else {
                    byte
                });
            }
            if row == 7 {
                data.push(b'X');
            } // No ambiguous final empty row.
            expected.push(data);
        }
        let mut input = Vec::new();
        for (index, payload) in expected.iter().enumerate() {
            input.extend_from_slice(payload);
            if index < 7 || case.is_multiple_of(2) {
                input.extend_from_slice(if next(&mut seed) & 1 == 0 {
                    b"\n"
                } else {
                    b"\r\n"
                });
            }
        }
        let mut reader = RecordReader::new(
            BufReader::with_capacity(1 + case % 7, Cursor::new(input)),
            NewlinePolicy::AutoLfOrCrlf,
        );
        let mut buffer = Vec::with_capacity(64);
        for (index, payload) in expected.iter().enumerate() {
            let row = reader.read_next(&mut buffer).unwrap().unwrap();
            assert_eq!(row.bytes, payload);
            assert_eq!(row.line_number, index as u64 + 1);
        }
        assert!(reader.read_next(&mut buffer).unwrap().is_none());
        assert!(reader.read_next(&mut buffer).unwrap().is_none());
    }
}

/// Reads real bytes up to an exact offset, then fails instead of reporting EOF.
/// Buffer fragmentation must not transform a technical fault into a short row.
struct FaultRead {
    input: Cursor<Vec<u8>>,
    fail_at: usize,
    kind: io::ErrorKind,
}
impl Read for FaultRead {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        let position = self.input.position() as usize;
        if position >= self.fail_at {
            return Err(io::Error::new(self.kind, "synthetic injected read fault"));
        }
        let count = output.len().min(self.fail_at - position);
        self.input.read(&mut output[..count])
    }
}

#[test]
fn read_fault_at_every_offset_never_publishes_partial_record_or_resumes() {
    let input = b"A\nBB\r\nCCC\nTAIL";
    for fail_at in 0..=input.len() {
        for capacity in [1, 2, 7] {
            for kind in [
                io::ErrorKind::Other,
                io::ErrorKind::PermissionDenied,
                io::ErrorKind::ConnectionReset,
                io::ErrorKind::UnexpectedEof,
            ] {
                let source = FaultRead {
                    input: Cursor::new(input.to_vec()),
                    fail_at,
                    kind,
                };
                let mut reader = RecordReader::new(
                    BufReader::with_capacity(capacity, source),
                    NewlinePolicy::AutoLfOrCrlf,
                );
                let mut buffer = Vec::new();
                let mut complete = 0;
                let error = loop {
                    match reader.read_next(&mut buffer) {
                        Ok(Some(_)) => complete += 1,
                        Ok(None) => panic!("injected fault became successful EOF"),
                        Err(error) => break error,
                    }
                };
                let expected = input[..fail_at].iter().filter(|&&b| b == b'\n').count() as u64;
                assert_eq!(complete, expected);
                assert!(
                    matches!(error, InputError::RecordRead { last_completed_line, source } if last_completed_line == expected && source.kind() == kind)
                );
                assert!(matches!(
                    reader.read_next(&mut buffer),
                    Err(InputError::PreviousFailure)
                ));
            }
        }
    }
}

fn raw() -> RawFlowSchema {
    serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap()
}

#[test]
fn tuple_boundaries_and_arbitrary_bytes_cannot_collapse_into_the_same_join_key() {
    let mut unique = HashSet::new();
    for split in 1..6 {
        let mut model = serde_json::to_value(raw()).unwrap();
        let field = model["traces"][0]["fields"][0].clone();
        let fields: Vec<_> = [("A", 1, split), ("B", split + 1, 6 - split)]
            .into_iter()
            .map(|(id, start, length)| {
                let mut field = field.clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(start);
                field["physical"]["length"] = json!(length);
                field["physical"]["alignment"] = json!({"kind":"exact"});
                field["missing_value"] = json!({"kind":"never_missing"});
                field
            })
            .collect();
        model["traces"][0]["fields"] = json!(fields);
        model["traces"][0]["record_key"]["fields"] = json!(["A"]);
        let mut raw: RawFlowSchema = serde_json::from_value(model).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        let compiled = SchemaCompiler.compile(raw).unwrap();
        let trace = &compiled.traces()[0];
        for byte in 0..=255u8 {
            let bytes = [byte, b':', 0, b'|', 255, b' '];
            let key = build_join_key(&RecordView::new(&bytes), &[0, 1], trace).unwrap();
            assert!(unique.insert(key));
            for len in 0..6 {
                assert!(build_join_key(&RecordView::new(&bytes[..len]), &[0, 1], trace).is_err());
            }
            assert!(build_join_key(&RecordView::new(&bytes), &[usize::MAX], trace).is_err());
        }
    }
    assert_eq!(unique.len(), 5 * 256);
}

#[test]
fn canonical_roundtrip_preserves_generated_unicode_metadata_and_exact_numbers() {
    let profiles: [&[u8]; 4] = [
        include_bytes!("golden/synthetic/v1/schema.json"),
        include_bytes!("golden/semantic/v1/schema.json"),
        include_bytes!("golden/dsl/v1/schema.json"),
        include_bytes!("golden/derived/v1/schema.json"),
    ];
    for profile in profiles {
        for seed in 0..128 {
            let mut raw: RawFlowSchema = serde_json::from_slice(profile).unwrap();
            let text = format!("synthetic {seed}: à É \" \\ \n \0 🦀");
            raw.source.insert(
                "generated".into(),
                json!({"z":[text, null, false],"a":{"value":"00123","seed":seed}}),
            );
            raw.source.insert(
                "exact_number".into(),
                serde_json::from_str("-123456789012345678901234567890.0001").unwrap(),
            );
            let bytes = canonical_schema_bytes(&raw).unwrap();
            let restored: RawFlowSchema = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(restored, raw);
            assert_eq!(canonical_schema_bytes(&restored).unwrap(), bytes);
            assert_eq!(
                schema_fingerprint(&restored).unwrap(),
                schema_fingerprint(&raw).unwrap()
            );
        }
    }
}

#[test]
fn malformed_external_schema_bytes_fail_without_a_panic_or_partial_model() {
    let bytes = canonical_schema_bytes(&raw()).unwrap();
    for end in 0..bytes.len() {
        assert!(serde_json::from_slice::<RawFlowSchema>(&bytes[..end]).is_err());
    }
    for index in (0..bytes.len()).step_by(7) {
        for poison in [0u8, 255] {
            let mut corrupted = bytes.clone();
            corrupted[index] = poison;
            assert!(serde_json::from_slice::<RawFlowSchema>(&corrupted).is_err());
        }
    }
    let plain = String::from_utf8(bytes).unwrap();
    assert!(plain.contains("\"source\":{}"));
    let deep = plain.replace(
        "\"source\":{}",
        &format!(
            "\"source\":{{\"nested\":{}0{}}}",
            "[".repeat(512),
            "]".repeat(512)
        ),
    );
    assert!(serde_json::from_str::<RawFlowSchema>(&deep).is_err());
}

#[test]
fn large_real_file_reuses_one_record_buffer_without_accumulating_rows() {
    let file = tempfile::NamedTempFile::new().unwrap();
    {
        let mut writer = BufWriter::new(file.reopen().unwrap());
        for _ in 0..250_000 {
            writer.write_all(b"001A  \r\n").unwrap();
        }
        writer.flush().unwrap();
    }
    let mut reader = RecordReader::new(
        BufReader::new(File::open(file.path()).unwrap()),
        NewlinePolicy::AutoLfOrCrlf,
    );
    let mut buffer = Vec::with_capacity(64);
    let pointer = buffer.as_ptr();
    let mut count = 0;
    while let Some(row) = reader.read_next(&mut buffer).unwrap() {
        assert_eq!(row.bytes, b"001A  ");
        assert_eq!(row.bytes.as_ptr(), pointer);
        count += 1;
        assert_eq!(row.line_number, count);
    }
    assert_eq!(count, 250_000);
    assert!(buffer.capacity() <= 64);
}

struct FaultSink {
    fail_error_at: Option<u64>,
    fail_summary: bool,
    errors: u64,
    summaries: u64,
}
impl ValidationSink for FaultSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.errors += 1;
        if self.fail_error_at == Some(self.errors) {
            return Err(ReportError::Write(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "synthetic sink fault",
            )));
        }
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        self.summaries += 1;
        if self.fail_summary {
            return Err(ReportError::Write(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "synthetic summary fault",
            )));
        }
        Ok(())
    }
}
fn request(path: &Path) -> ValidationRequest {
    ValidationRequest {
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: TraceId("T1".into()),
            source: InputSource::Path(path.into()),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    }
}

#[test]
fn sink_fault_at_each_callback_aborts_service_without_completed_outcome() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("synthetic.dat");
    fs::write(&path, b"x\n".repeat(64)).unwrap();
    let mut repository = InMemorySchemaRepository::new();
    repository.insert(raw()).unwrap();
    let service = ValidationService::new(Arc::new(SchemaRegistry::new(
        Arc::new(repository),
        SchemaCompiler,
        RawResolvedEncoding::Utf8,
    )));
    for fail_at in 1..=64 {
        let mut sink = FaultSink {
            fail_error_at: Some(fail_at),
            fail_summary: false,
            errors: 0,
            summaries: 0,
        };
        let mut completed = false;
        let result = service.validate(
            request(&path),
            &mut sink,
            &mut |event| completed |= event == ProgressEvent::RunCompleted,
            &CancellationToken::default(),
        );
        assert!(
            matches!(result, Err(ValidationRunError::Report(ReportError::Write(error))) if error.kind() == io::ErrorKind::BrokenPipe)
        );
        assert_eq!(sink.errors, fail_at);
        assert_eq!(sink.summaries, 0);
        assert!(!completed);
    }
    for fail_summary in [false, true] {
        let mut sink = FaultSink {
            fail_error_at: None,
            fail_summary,
            errors: 0,
            summaries: 0,
        };
        let mut completed = false;
        let result = service.validate(
            request(&path),
            &mut sink,
            &mut |event| completed |= event == ProgressEvent::RunCompleted,
            &CancellationToken::default(),
        );
        assert_eq!((sink.errors, sink.summaries), (64, 1));
        assert_eq!(completed, !fail_summary);
        if fail_summary {
            assert!(matches!(result, Err(ValidationRunError::Report(_))));
        } else {
            assert_eq!(result.unwrap().status, RunStatus::CompletedInvalid);
        }
    }
    fs::remove_file(&path).unwrap();
    let mut sink = FaultSink {
        fail_error_at: None,
        fail_summary: false,
        errors: 0,
        summaries: 0,
    };
    assert!(matches!(
        service.validate(
            request(&path),
            &mut sink,
            &mut |_| {},
            &CancellationToken::default()
        ),
        Err(ValidationRunError::Input(_))
    ));
    assert_eq!((sink.errors, sink.summaries), (0, 0));
}
