//! JSONL browsing uses synthetic canonical diagnostics and disk offsets only.
use lintra::*;
use lintra_ui::results::*;
use slint::Model;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    sync::atomic::AtomicBool,
    time::{Duration, UNIX_EPOCH},
};

fn summary(total: u64, codes: &[(&str, u64)]) -> ValidationSummary {
    ValidationSummary {
        started_at: UNIX_EPOCH,
        validation_mode: ValidationMode::LocalOnly,
        run_status: RunStatus::CompletedInvalid,
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope_summary: ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
        records_processed: total,
        valid_records: 0,
        invalid_records: total,
        total_errors: total,
        errors_by_code: codes
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), *v))
            .collect(),
        skipped_rules: 0,
        truncated: false,
        validator_version: "synthetic".into(),
        schema_hash: "synthetic".into(),
        reference_data_versions: BTreeMap::new(),
        reference_validation: None,
        full_validation: None,
        elapsed: Duration::ZERO,
        inputs: Vec::new(),
    }
}
fn small_summary() -> ValidationSummary {
    summary(2, &[("E_SYNTHETIC_RULE", 1), ("E_RECORD_LENGTH", 1)])
}
fn fixture() -> Vec<u8> {
    include_bytes!("../../tests/fixtures/diagnostics.errors.jsonl").to_vec()
}
fn index(
    bytes: &[u8],
    summary: &ValidationSummary,
    filter: &ResultsFilter,
) -> Result<ReportIndex, ResultsError> {
    let mut f = tempfile::tempfile().unwrap();
    f.write_all(bytes).unwrap();
    ReportIndex::scan(f, summary, filter, &AtomicBool::new(false))
}
#[test]
fn canonical_report_is_lazy_ordered_and_preserves_field_record_and_raw_context() {
    let index = index(&fixture(), &small_summary(), &ResultsFilter::default()).unwrap();
    assert_eq!(index.row_count(), 2);
    assert_eq!(index.total(), 2);
    assert_eq!(index.offset_bytes(), 16);
    let model = JsonlModel::new(index);
    assert_eq!(model.decoded_rows(), 0);
    let first = model.read_row(0).unwrap().unwrap();
    assert_eq!(first.line, "7");
    assert_eq!(first.field, "F1");
    assert_eq!(first.field_name, "Campo à");
    assert_eq!(first.position, "1–3");
    assert!(first.actual.contains("00 C3 A9"));
    assert!(first.context.contains("T2.F2 = 01"));
    assert!(first.context.contains("SYNTHETIC-Z"));
    let second = model.read_row(1).unwrap().unwrap();
    assert_eq!(second.line, "8");
    assert_eq!(second.field, "—");
    assert_eq!(model.read_row(2).unwrap(), None);
    assert_eq!(model.read_row(0).unwrap().unwrap(), first);
    assert_eq!(model.decoded_rows(), 2);
    assert_eq!(model.position_of(model.offset_at(1).unwrap()), Some(1));
    assert_eq!(model.position_of(1), None);
}
#[test]
fn all_five_filters_are_exact_combinable_and_do_not_reinterpret_null_or_case() {
    let mut rows: Vec<serde_json::Value> = fixture()
        .split(|v| *v == b'\n')
        .filter(|v| !v.is_empty())
        .map(|v| serde_json::from_slice(v).unwrap())
        .collect();
    rows[0]["record_identity"]["sioc_record_id"] = "SYNTH-RECORD".into();
    rows[0]["record_identity"]["fiscal_code"] = "SYNTH-FISCAL".into();
    let mut bytes = Vec::new();
    for row in rows {
        serde_json::to_writer(&mut bytes, &row).unwrap();
        bytes.push(b'\n');
    }
    for filter in [
        ResultsFilter {
            code: "E_SYNTHETIC_RULE".into(),
            ..Default::default()
        },
        ResultsFilter {
            field: "F1".into(),
            ..Default::default()
        },
        ResultsFilter {
            record: "SYNTH-RECORD".into(),
            ..Default::default()
        },
        ResultsFilter {
            fiscal_code: "SYNTH-FISCAL".into(),
            ..Default::default()
        },
        ResultsFilter {
            line: Some(7),
            ..Default::default()
        },
        ResultsFilter {
            code: "E_SYNTHETIC_RULE".into(),
            field: "F1".into(),
            record: "SYNTH-RECORD".into(),
            fiscal_code: "SYNTH-FISCAL".into(),
            line: Some(7),
        },
    ] {
        let i = index(&bytes, &small_summary(), &filter).unwrap();
        assert_eq!(i.row_count(), 1);
        assert_eq!(i.total(), 2);
    }
    for filter in [
        ResultsFilter {
            field: "f1".into(),
            ..Default::default()
        },
        ResultsFilter {
            record: "SYNTH-RECORD ".into(),
            ..Default::default()
        },
        ResultsFilter {
            code: "E_RECORD_LENGTH".into(),
            line: Some(7),
            ..Default::default()
        },
        ResultsFilter {
            fiscal_code: "null".into(),
            ..Default::default()
        },
    ] {
        assert_eq!(
            index(&bytes, &small_summary(), &filter)
                .unwrap()
                .row_count(),
            0
        );
    }
    for text in ["0", "-1", " 7", "7 ", "1.0", "18446744073709551616"] {
        assert!(matches!(
            ResultsFilter::parse_line(text),
            Err(ResultsError::InvalidLineFilter)
        ));
    }
    assert_eq!(ResultsFilter::parse_line("").unwrap(), None);
    assert_eq!(ResultsFilter::parse_line("7").unwrap(), Some(7));
}
#[test]
fn malformed_or_incomplete_diagnostics_fail_instead_of_disappearing_from_the_table() {
    let bytes = fixture();
    for bad in [
        b"\n".to_vec(),
        b"\xef\xbb\xbf{}\n".to_vec(),
        b"{\xff}\n".to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
        bytes
            .iter()
            .flat_map(|b| {
                if *b == b'\n' {
                    vec![b'\r', b'\n']
                } else {
                    vec![*b]
                }
            })
            .collect(),
    ] {
        assert!(matches!(
            index(&bad, &small_summary(), &ResultsFilter::default()),
            Err(ResultsError::Malformed)
        ));
    }
    let first = bytes.split(|v| *v == b'\n').next().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(first).unwrap();
    value["field"]["start"] = 0.into();
    let mut bad = serde_json::to_vec(&value).unwrap();
    bad.push(b'\n');
    assert!(matches!(
        index(&bad, &small_summary(), &ResultsFilter::default()),
        Err(ResultsError::Malformed)
    ));
}
#[test]
fn scope_count_histogram_and_cancellation_are_verified_before_publication() {
    let mut s = small_summary();
    s.total_errors = 3;
    assert!(matches!(
        index(&fixture(), &s, &ResultsFilter::default()),
        Err(ResultsError::CountMismatch)
    ));
    s = small_summary();
    s.errors_by_code = BTreeMap::from([("E_RECORD_LENGTH".parse().unwrap(), 2)]);
    assert!(matches!(
        index(&fixture(), &s, &ResultsFilter::default()),
        Err(ResultsError::CountMismatch)
    ));
    s = small_summary();
    s.schema_key.version.0 = "other".into();
    assert!(matches!(
        index(&fixture(), &s, &ResultsFilter::default()),
        Err(ResultsError::ScopeMismatch)
    ));
    let mut f = tempfile::tempfile().unwrap();
    f.write_all(&fixture()).unwrap();
    assert!(matches!(
        ReportIndex::scan(
            f,
            &small_summary(),
            &ResultsFilter::default(),
            &AtomicBool::new(true)
        ),
        Err(ResultsError::Cancelled)
    ));
    let empty = index(b"", &summary(0, &[]), &ResultsFilter::default()).unwrap();
    assert_eq!(empty.row_count(), 0);
}
#[test]
fn a_truncated_report_after_indexing_surfaces_a_technical_read_failure() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("synthetic.jsonl");
    fs::write(&path, fixture()).unwrap();
    let model = JsonlModel::new(
        ReportIndex::scan(
            File::open(&path).unwrap(),
            &small_summary(),
            &ResultsFilter::default(),
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    fs::write(&path, b"").unwrap();
    assert!(model.read_row(0).is_err());
    assert!(model.row_data(0).unwrap().message.contains("Errore"));
    assert!(model.failed());
}
#[test]
fn three_hundred_thousand_and_one_million_rows_keep_only_offsets_and_a_bounded_cache() {
    // A canonical record-level diagnostic repeated as declared synthetic load.
    // This is a browsing workload, not a fabricated clinical validation result.
    let row = fixture().split(|v| *v == b'\n').nth(1).unwrap().to_vec();
    for count in [300_000usize, 1_000_000] {
        let mut file = tempfile::tempfile().unwrap();
        {
            let mut writer = BufWriter::new(&mut file);
            for _ in 0..count {
                writer.write_all(&row).unwrap();
                writer.write_all(b"\n").unwrap();
            }
            writer.flush().unwrap();
        }
        let s = summary(count as u64, &[("E_RECORD_LENGTH", count as u64)]);
        let index = ReportIndex::scan(file, &s, &ResultsFilter::default(), &AtomicBool::new(false))
            .unwrap();
        assert_eq!(index.row_count(), count);
        assert_eq!(index.offset_bytes(), count * 8);
        let model = JsonlModel::new(index);
        assert_eq!(model.decoded_rows(), 0);
        for i in 0..300 {
            let n = i * (count - 1) / 299;
            assert_eq!(model.read_row(n).unwrap().unwrap().code, "E_RECORD_LENGTH");
        }
        assert!(model.cache_len() <= 128);
        assert_eq!(model.decoded_rows(), 300);
        assert_eq!(model.read_row(count - 1).unwrap().unwrap().line, "8");
    }
}
