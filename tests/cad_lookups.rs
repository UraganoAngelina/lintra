//! CAD-11: real reference provider/service over explicitly scoped byte layouts.
//! Only the declared component fields are validated; filler is synthetic.
use lintra::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, sync::Arc};

const COMPONENT: &str =
    include_str!("../schemas/cad/components/2024-06.clinical-lookups.1/component.json");
const ROOT: &str = "schemas/cad/components/2024-06.clinical-lookups.1";
fn component() -> Value {
    serde_json::from_str(COMPONENT).unwrap()
}
fn source(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(ROOT)
                .join("source")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn filler(id: &str, start: usize, length: usize) -> Value {
    json!({"id":id,"name":"Synthetic bytes outside component scope",
        "physical":{"start":start,"length":length,"type":"alphanumeric",
        "alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
        "missing_value":{"kind":"spaces"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})
}
/// Assemble full physical coverage, keeping every supplied normative coordinate.
/// The synthetic PK is needed by the compiler, not a substitute for the CAD PK.
fn model() -> Value {
    let c = component();
    let mut v: Value =
        serde_json::from_slice(include_bytes!("golden/synthetic/v1/schema.json")).unwrap();
    v["flow_id"] = json!("CAD-LOOKUP-COMPONENT-TEST");
    v["schema_version"] = c["component_version"].clone();
    v["source"] = json!({"scope":"lookup/conditional component only; not full CAD compliance"});
    v["required_capabilities"] = c["required_capabilities"].clone();
    v["reference_data"] = c["reference_data"].clone();
    let template = v["traces"][0].clone();
    v["traces"] = json!([]);
    for id in ["B1", "D"] {
        let declaration = &c["traces"][id];
        let mut t = template.clone();
        t["id"] = json!(id);
        t["record_length"] = declaration["record_length"].clone();
        t["identity"] = json!([]);
        t["named_conditions"] = declaration["named_conditions"].clone();
        t["record_key"]["fields"] = json!([declaration["fields"][0]["id"]]);
        let mut fields = Vec::new();
        let mut next = 1;
        for field in declaration["fields"].as_array().unwrap() {
            let start = field["physical"]["start"].as_u64().unwrap() as usize;
            assert!(start >= next);
            if start > next {
                fields.push(filler(&format!("SYNTHETIC-{next}"), next, start - next));
            }
            next = start + field["physical"]["length"].as_u64().unwrap() as usize;
            fields.push(field.clone());
        }
        let length = t["record_length"].as_u64().unwrap() as usize;
        if next <= length {
            fields.push(filler("SYNTHETIC-TAIL", next, length + 1 - next));
        }
        t["fields"] = json!(fields);
        v["traces"].as_array_mut().unwrap().push(t);
    }
    v
}
struct Fixture {
    dir: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let dir =
            tempfile::tempdir_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
                .unwrap();
        fs::create_dir(dir.path().join("data")).unwrap();
        for name in ["icd9-membership.json", "prestations-membership.json"] {
            fs::copy(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(ROOT)
                    .join("data")
                    .join(name),
                dir.path().join("data").join(name),
            )
            .unwrap();
        }
        Self { dir }
    }
    fn service(&self, value: Value, policy: DatasetPreparationPolicy) -> ValidationService {
        let raw: RawFlowSchema = serde_json::from_value(value).unwrap();
        let mut repo = InMemorySchemaRepository::new();
        repo.insert(raw).unwrap();
        ValidationService::new(Arc::new(SchemaRegistry::new(
            Arc::new(repo),
            SchemaCompiler,
            RawResolvedEncoding::Utf8,
        )))
        .with_reference_data(
            Arc::new(FileSystemReferenceDataPreparer::new(self.dir.path().to_path_buf()).unwrap()),
            policy,
        )
    }
    fn run(
        &self,
        trace: &str,
        rows: &[Vec<u8>],
        value: Value,
        policy: DatasetPreparationPolicy,
    ) -> Result<(ValidationRunOutcome, VecSink), ValidationRunError> {
        let path = self.dir.path().join("input.dat");
        let mut bytes = Vec::new();
        for row in rows {
            bytes.extend_from_slice(row);
            bytes.push(b'\n');
        }
        fs::write(&path, bytes).unwrap();
        let mut sink = VecSink::default();
        let c = component();
        let request = ValidationRequest {
            schema_key: SchemaKey {
                flow_id: FlowId("CAD-LOOKUP-COMPONENT-TEST".into()),
                version: SchemaVersion(c["component_version"].as_str().unwrap().into()),
            },
            scope: ValidationScope::Trace(TraceValidationInput {
                trace_id: TraceId(trace.into()),
                source: InputSource::Path(path),
            }),
            mode: ValidationMode::LocalOnly,
            options: ValidationOptions {
                record_length_policy: RecordLengthPolicy::StrictSkipRecord,
                newline_policy: NewlinePolicy::AutoLfOrCrlf,
                max_errors: None,
            },
        };
        let out = self.service(value, policy).validate(
            request,
            &mut sink,
            &mut |_| {},
            &CancellationToken::default(),
        )?;
        write_run_summary(Vec::new(), &out.summary).unwrap();
        Ok((out, sink))
    }
}
fn set(row: &mut [u8], start: usize, width: usize, text: &str) {
    assert!(text.len() <= width);
    row[start - 1..start - 1 + width].fill(b' ');
    row[start - 1..start - 1 + text.len()].copy_from_slice(text.as_bytes());
}
fn b1(kind: u8) -> Vec<u8> {
    let mut row = vec![b' '; 198];
    row[14] = kind;
    row[157] = if kind == b'2' { b'1' } else { b'0' };
    row[158] = if kind == b'2' { b'1' } else { b'0' };
    if kind == b'2' {
        set(&mut row, 160, 5, "V667");
    }
    row
}
fn d(code: &str) -> Vec<u8> {
    let mut row = vec![b' '; 39];
    set(&mut row, 24, 2, code);
    row
}
fn jsonl(sink: &VecSink) -> Vec<u8> {
    let mut out = JsonlSink::new(Vec::new());
    for e in sink.errors() {
        out.on_error(e).unwrap();
    }
    out.finish().unwrap()
}

#[test]
fn reviewed_source_inventories_match_exact_membership_bytes_and_preserve_prefixes() {
    let c = component();
    assert_eq!(c["full_normative_compliance"], false);
    for (name, expected) in c["source"]["dataset_sha256"].as_object().unwrap() {
        let bytes = fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(ROOT)
                .join("source")
                .join(name),
        )
        .unwrap();
        assert_eq!(
            Sha256Digest(Sha256::digest(&bytes).into()).to_string(),
            expected.as_str().unwrap()
        );
    }
    for trace in ["B1", "D"] {
        for (lowered, original) in c["traces"][trace]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["source_fields"][trace].as_array().unwrap())
        {
            assert_eq!(lowered["id"], original["id"]);
            assert_eq!(lowered["physical"]["start"], original["physical"]["start"]);
            assert_eq!(
                lowered["physical"]["length"],
                original["physical"]["length"]
            );
            assert_eq!(lowered["presence"], original["presence"]);
            assert_eq!(
                lowered["regulatory_metadata"],
                original["regulatory_metadata"]
            );
        }
    }
    let icd = source("cad_icd9_cm_symptoms_page7.json");
    let prest = source("cad_prestations_type_5_5_page5.json");
    let needs = source("cad_needs_70_positions_page6.json");
    assert_eq!(
        (
            icd["entries"].as_array().unwrap().len(),
            prest["entries"].as_array().unwrap().len(),
            needs["entries"].as_array().unwrap().len()
        ),
        (63, 33, 70)
    );
    let mut positions: Vec<_> = needs["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["position"].as_u64().unwrap())
        .collect();
    positions.sort();
    assert_eq!(positions, (1..=70).collect::<Vec<_>>());
    let codes: Vec<_> = icd["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let expected = e["source_code"].as_str().unwrap().replace('.', "");
            assert_eq!(e["transmitted_code_no_dot"], expected);
            expected
        })
        .collect();
    assert_eq!(
        codes
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        63
    );
    assert_eq!(codes.iter().filter(|c| c.starts_with('V')).count(), 7);
    assert!(codes.contains(&"V667".to_owned()) && codes.contains(&"6989".to_owned()));
    for (i, values) in [
        codes,
        prest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["code"].as_str().unwrap().to_owned())
            .collect(),
    ]
    .into_iter()
    .enumerate()
    {
        let dep = &c["reference_data"][i];
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(ROOT)
            .join(dep["path"].as_str().unwrap());
        let bytes = fs::read(path).unwrap();
        assert_eq!(
            Sha256Digest(Sha256::digest(&bytes).into()).to_string(),
            dep["sha256"].as_str().unwrap()
        );
        let wire: Value = serde_json::from_slice(&bytes).unwrap();
        let actual: std::collections::BTreeSet<_> = wire["keys_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let expected: std::collections::BTreeSet<_> = values
            .iter()
            .map(|v| {
                v.as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(wire["dataset_id"], dep["dataset_id"]);
        assert_eq!(wire["version"], dep["version"]);
    }
    // Only these two enumerated memberships are dependencies; the 70-position
    // catalog is not an invented lookup and ISTAT/ICPC remain absent.
    assert_eq!(c["reference_data"].as_array().unwrap().len(), 2);
}

#[test]
fn all_63_icd_codes_validate_in_each_of_the_five_real_b1_slots() {
    let f = Fixture::new();
    let src = source("cad_icd9_cm_symptoms_page7.json");
    let rows: Vec<_> = src["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let mut r = b1(b'2');
            for start in [160, 165, 170, 175, 180] {
                set(
                    &mut r,
                    start,
                    5,
                    e["transmitted_code_no_dot"].as_str().unwrap(),
                );
            }
            r
        })
        .collect();
    let (out, sink) = f
        .run("B1", &rows, model(), DatasetPreparationPolicy::Strict)
        .unwrap();
    assert_eq!(
        (
            out.status,
            out.summary.records_processed,
            out.summary.total_errors
        ),
        (RunStatus::CompletedValid, 63, 0)
    );
    assert!(sink.errors().is_empty());
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Complete);
    let audit = out.summary.reference_validation.as_ref().unwrap();
    assert_eq!(
        audit
            .controls
            .iter()
            .map(|c| c.evaluations.definitive_satisfied)
            .sum::<u64>(),
        63 * 5
    );
    for (id, start) in [
        ("2.29", 160),
        ("2.30", 165),
        ("2.31", 170),
        ("2.32", 175),
        ("2.33", 180),
    ] {
        let mut r = b1(b'2');
        set(&mut r, start, 5, "99999");
        let (_, sink) = f
            .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
            .unwrap();
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, id);
        assert_eq!(sink.errors()[0].code, ValidationCode::ReferenceData);
    }
}
#[test]
fn all_33_prestations_validate_at_the_real_d_coordinates_and_other_codes_fail() {
    let f = Fixture::new();
    let src = source("cad_prestations_type_5_5_page5.json");
    let rows: Vec<_> = src["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| d(e["code"].as_str().unwrap()))
        .collect();
    let (out, sink) = f
        .run("D", &rows, model(), DatasetPreparationPolicy::Strict)
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert_eq!(out.summary.valid_records, 33);
    assert!(sink.errors().is_empty());
    for code in ["00", "29", "34", "75", " 1", "A1"] {
        let (out, sink) = f
            .run("D", &[d(code)], model(), DatasetPreparationPolicy::Strict)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedInvalid, "{code}");
        assert!(
            sink.errors()
                .iter()
                .all(|e| e.field.as_ref().unwrap().id.0 == "5.5")
        );
    }
}
#[test]
fn optional_symptoms_are_fac_only_inside_ucpdom_with_primary_presence_diagnostics() {
    let f = Fixture::new();
    // The first symptom remains required; optional slots may all stay blank.
    for kind in *b"12" {
        let (out, _) = f
            .run("B1", &[b1(kind)], model(), DatasetPreparationPolicy::Strict)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedValid);
    }
    for (id, start) in [("2.30", 165), ("2.31", 170), ("2.32", 175), ("2.33", 180)] {
        let mut r = b1(b'2');
        set(&mut r, start, 5, "V667");
        assert_eq!(
            f.run(
                "B1",
                &[r.clone()],
                model(),
                DatasetPreparationPolicy::Strict
            )
            .unwrap()
            .0
            .status,
            RunStatus::CompletedValid
        );
        r[14] = b'1';
        r[157] = b'0';
        r[158] = b'0';
        set(&mut r, 160, 5, "");
        let (_, sink) = f
            .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
            .unwrap();
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(sink.errors()[0].code, ValidationCode::ForbiddenField);
        assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, id);
    }
    let mut r = b1(b'2');
    set(&mut r, 160, 5, "");
    let (_, sink) = f
        .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
        .unwrap();
    assert_eq!(sink.errors().len(), 1);
    assert_eq!(sink.errors()[0].code, ValidationCode::ConditionalRequired);
}
#[test]
fn network_type_required_and_forbidden_conditions_follow_both_controlling_fields() {
    let f = Fixture::new();
    for (kind, network, required) in [
        (b'1', b'0', false),
        (b'2', b'1', true),
        (b'2', b'2', false),
        (b'2', b'9', false),
    ] {
        for supplied in [false, true] {
            let mut r = b1(kind);
            r[157] = network;
            r[158] = if supplied { b'1' } else { b'0' };
            let (out, sink) = f
                .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
                .unwrap();
            assert_eq!(
                out.status == RunStatus::CompletedValid,
                required == supplied
            );
            if required != supplied {
                assert_eq!(sink.errors().len(), 1);
                assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, "2.28");
                assert_eq!(
                    sink.errors()[0].code,
                    if required {
                        ValidationCode::ConditionalRequired
                    } else {
                        ValidationCode::ForbiddenField
                    }
                );
            }
        }
    }
}
#[test]
fn icd_membership_is_exact_without_input_dot_removal_numeric_or_case_coercion() {
    let f = Fixture::new();
    for code in ["698.9", "V66.7", "v667", "0667", "99999"] {
        let mut r = b1(b'2');
        set(&mut r, 165, 5, code);
        let (out, sink) = f
            .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
            .unwrap();
        assert_eq!(out.status, RunStatus::CompletedInvalid, "{code}");
        assert_eq!(sink.errors().len(), 1);
        assert_eq!(sink.errors()[0].code, ValidationCode::ReferenceData);
        assert_eq!(sink.errors()[0].field.as_ref().unwrap().id.0, "2.30");
    }
    let mut r = b1(b'2');
    set(&mut r, 165, 5, "6989");
    let (_, a) = f
        .run(
            "B1",
            &[r.clone()],
            model(),
            DatasetPreparationPolicy::Strict,
        )
        .unwrap();
    assert!(a.errors().is_empty());
    set(&mut r, 165, 5, "99999");
    let (_, a) = f
        .run(
            "B1",
            &[r.clone()],
            model(),
            DatasetPreparationPolicy::Strict,
        )
        .unwrap();
    let (_, b) = f
        .run("B1", &[r], model(), DatasetPreparationPolicy::Strict)
        .unwrap();
    assert_eq!(jsonl(&a), jsonl(&b));
    assert!(!jsonl(&a).contains(&b'\r'));
}
#[test]
fn missing_or_degraded_official_membership_stays_technical_or_non_definitive() {
    let f = Fixture::new();
    let mut v = model();
    v["reference_data"][0]["sha256"] = json!("0".repeat(64));
    assert!(matches!(
        f.run(
            "B1",
            &[b1(b'2')],
            v.clone(),
            DatasetPreparationPolicy::Strict
        ),
        Err(ValidationRunError::ReferenceData(_))
    ));
    let (out, _) = f
        .run(
            "B1",
            &[b1(b'2')],
            v,
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        )
        .unwrap();
    assert_eq!(out.status, RunStatus::CompletedValid);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    assert_eq!(
        out.summary.reference_validation.unwrap().datasets[0].availability,
        DatasetAvailability::Degraded
    );
    fs::remove_file(f.dir.path().join("data/icd9-membership.json")).unwrap();
    assert!(matches!(
        f.run("B1", &[b1(b'2')], model(), DatasetPreparationPolicy::Strict),
        Err(ValidationRunError::ReferenceData(_))
    ));
    let (out, sink) = f
        .run(
            "B1",
            &[b1(b'2')],
            model(),
            DatasetPreparationPolicy::ContinueWithAvailableEvidence,
        )
        .unwrap();
    assert_eq!(out.summary.total_errors, 0);
    assert_eq!(out.summary.completeness(), ValidationCompleteness::Partial);
    assert!(
        sink.errors()
            .iter()
            .all(|e| e.code != ValidationCode::ReferenceData)
    );
}
