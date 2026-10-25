use lintra::*;
use rust_xlsxwriter::{Formula, Workbook};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

const NOTE: &str = "  Nota originale 00123\r\nSeconda riga è invariata  ";
fn blueprint() -> RawFlowSchema {
    let mut v: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let original = v["traces"][0].clone();
    v["traces"] = json!(["TRACE-A", "TRACE-B"].map(|id| {
        let mut t = original.clone();
        t["id"] = json!(id);
        let mut a = t["fields"][0].clone();
        a["id"] = json!("001.01");
        let mut b = a.clone();
        b["id"] = json!("001.02");
        b["physical"]["start"] = json!(7);
        b["physical"]["length"] = json!(2);
        t["fields"] = json!([a, b]);
        t["record_length"] = json!(8);
        t["record_key"]["fields"] = json!(["001.01"]);
        t
    }));
    serde_json::from_value(v).unwrap()
}
fn profile() -> XlsxImportProfile {
    let names = [
        "N",
        "Chiave ",
        "Descrizione Campo",
        "LUNGH.",
        "POS. INIZ.",
        "TIPO",
        "OBB",
        "Istruzioni per la codifica",
    ];
    let columns = names
        .into_iter()
        .enumerate()
        .map(|(i, h)| XlsxColumn {
            number: i as u32 + 1,
            header: h.into(),
        })
        .collect::<Vec<_>>();
    XlsxImportProfile {
        flow_id: "SYNTHETIC".into(),
        blueprint_version: "v1".into(),
        sheet: "Tracciato".into(),
        header_row: 5,
        columns: XlsxColumns {
            field_id: columns[0].clone(),
            key_marker: columns[1].clone(),
            name: columns[2].clone(),
            length: columns[3].clone(),
            start: columns[4].clone(),
            source_type: columns[5].clone(),
            obligation: columns[6].clone(),
            notes: columns[7].clone(),
        },
        traces: [("TRACE-A", 7), ("TRACE-B", 12)]
            .map(|(id, row)| XlsxTraceBlock {
                trace_id: id.into(),
                heading_row: row,
                heading: format!("{id} — sezione dichiarata"),
                fields: [("001.01", row + 1), ("001.02", row + 2)]
                    .map(|(id, row)| XlsxFieldRow {
                        row,
                        field_id: id.into(),
                        source_type: Some("AN".into()),
                        obligation: "OBB".into(),
                        key_marker: None,
                        reviewed_physical_type: "alphanumeric".into(),
                    })
                    .to_vec(),
            })
            .to_vec(),
        coordinate_formulas: CoordinateFormulaPolicy::UseCachedValues,
        annotation_sheets: vec!["Indicazioni operative".into()],
    }
}
fn workbook(change: impl FnOnce(&mut Workbook)) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.xlsx");
    let p = profile();
    let mut book = Workbook::new();
    let sheet = book.add_worksheet();
    sheet.set_name(&p.sheet).unwrap();
    for (i, header) in [
        "N",
        "Chiave ",
        "Descrizione Campo",
        "LUNGH.",
        "POS. INIZ.",
        "TIPO",
        "OBB",
        "Istruzioni per la codifica",
    ]
    .iter()
    .enumerate()
    {
        sheet.write_string(4, i as u16, *header).unwrap();
    }
    for block in &p.traces {
        sheet
            .write_string(block.heading_row - 1, 0, &block.heading)
            .unwrap();
        for (index, f) in block.fields.iter().enumerate() {
            let row = f.row - 1;
            sheet.write_string(row, 0, &f.field_id).unwrap();
            sheet.write_string(row, 2, "Titolo da XLSX 00123").unwrap();
            sheet
                .write_number(row, 3, if index == 0 { 6.0 } else { 2.0 })
                .unwrap();
            if index == 0 {
                sheet.write_number(row, 4, 1.0).unwrap();
            } else {
                sheet
                    .write_formula(
                        row,
                        4,
                        Formula::new(format!("E{}+D{}", row, row)).set_result("7"),
                    )
                    .unwrap();
            }
            sheet.write_string(row, 5, "AN").unwrap();
            sheet.write_string(row, 6, "OBB").unwrap();
            sheet.write_string(row, 7, NOTE).unwrap();
        }
    }
    book.add_worksheet()
        .set_name("Indicazioni operative")
        .unwrap()
        .write_string(6, 2, "  Note generali\noriginali  ")
        .unwrap();
    change(&mut book);
    book.save(&path).unwrap();
    (dir, path)
}
fn importer() -> XlsxSchemaImporter {
    XlsxSchemaImporter::new(blueprint(), profile()).unwrap()
}
fn patch_xml(path: &Path, change: impl FnOnce(String) -> String) {
    let bytes = fs::read(path).unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut change = Some(change);
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let name = file.name().unwrap().into_owned();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if name == "xl/worksheets/sheet1.xml" {
            data = change.take().unwrap()(String::from_utf8(data).unwrap()).into_bytes();
        }
        output
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        output.write_all(&data).unwrap();
    }
    fs::write(path, output.finish().unwrap().into_inner()).unwrap();
}
#[test]
fn real_xlsx_layout_import_preserves_original_notes_semantics_and_provenance() {
    let (_dir, path) = workbook(|_| {});
    let before = blueprint();
    let raw = importer().import(&path).unwrap();
    assert_eq!(raw.traces.len(), 2);
    assert_eq!(raw.traces[0].fields[0].name, "Titolo da XLSX 00123");
    for (trace, original) in raw.traces.iter().zip(&before.traces) {
        assert_eq!(trace.record_key, original.record_key);
        assert_eq!(trace.rules, original.rules);
        assert_eq!(trace.joins, original.joins);
        for (field, previous) in trace.fields.iter().zip(&original.fields) {
            assert_eq!(field.id, previous.id);
            assert_eq!(field.physical, previous.physical);
            assert_eq!(field.presence, previous.presence);
            assert_eq!(field.constraints, previous.constraints);
            let meta = field.regulatory_metadata.as_ref().unwrap();
            assert_eq!(meta["source_note"], NOTE);
            assert_eq!(meta["xlsx_source"]["notes_original"], NOTE);
        }
    }
    let audit = &raw.source["xlsx_import"];
    assert_eq!(audit["status"], "requires_review");
    assert_eq!(audit["formula_cache_verified"], false);
    assert_eq!(
        audit["source_size_bytes"],
        fs::metadata(&path).unwrap().len()
    );
    assert_eq!(
        audit["blueprint_sha256"],
        schema_fingerprint(&before).unwrap().to_string()
    );
    assert_eq!(audit["annotations"][0]["row"], 7);
    assert_eq!(audit["annotations"][0]["column"], 3);
    assert_eq!(
        audit["annotations"][0]["text"],
        "  Note generali\noriginali  "
    );
    assert_eq!(
        raw.traces[0].fields[1]
            .regulatory_metadata
            .as_ref()
            .unwrap()["xlsx_source"]["start_formula"],
        "E8+D8"
    );
    assert_eq!(raw, importer().import(&path).unwrap());
    assert_eq!(before, blueprint());
}
#[test]
fn source_coordinates_and_descriptions_are_read_not_returned_from_blueprint() {
    let (_dir, path) = workbook(|book| {
        let s = book.worksheet_from_name("Tracciato").unwrap();
        s.write_number(7, 4, 0.0).unwrap();
        s.write_string(7, 2, "Changed name").unwrap();
    });
    let mut raw = importer().import(&path).unwrap();
    assert_eq!(raw.traces[0].fields[0].name, "Changed name");
    let field = serde_json::to_value(&raw.traces[0].fields[0]).unwrap();
    assert_eq!(field["physical"]["start"], 0);
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    assert!(SchemaCompiler.compile(raw).is_err());
}
#[test]
fn absent_cells_wrong_headers_and_binding_drift_are_rejected() {
    for (row, col, text) in [
        (4, 0, "Wrong header"),
        (6, 0, "Wrong section"),
        (7, 0, "001.1"),
        (7, 5, "N"),
        (7, 6, "FAC"),
        (7, 1, "X"),
    ] {
        let (_dir, path) = workbook(|book| {
            book.worksheet_from_name("Tracciato")
                .unwrap()
                .write_string(row, col, text)
                .unwrap();
        });
        assert!(
            matches!(importer().import(&path), Err(ImportError::Cell { .. })),
            "row{row}/col{col}"
        );
    }
    let (_dir, path) = workbook(|_| {});
    patch_xml(&path, |xml| {
        let pattern = regex::Regex::new(r#"(?s)<c r="A8"[^>]*>.*?</c>"#).unwrap();
        assert!(pattern.is_match(&xml));
        pattern.replace(&xml, "").into_owned()
    });
    assert!(importer().import(&path).is_err());
}
#[test]
fn numeric_ids_and_text_formulas_are_not_coerced_to_original_strings() {
    let (_dir, path) = workbook(|book| {
        book.worksheet_from_name("Tracciato")
            .unwrap()
            .write_number(7, 0, 1.01)
            .unwrap();
    });
    assert!(matches!(
        importer().import(&path),
        Err(ImportError::Cell { .. })
    ));
    let (_dir, path) = workbook(|book| {
        book.worksheet_from_name("Tracciato")
            .unwrap()
            .write_formula(7, 7, Formula::new("=1+1").set_result("2"))
            .unwrap();
    });
    assert!(matches!(
        importer().import(&path),
        Err(ImportError::Cell { .. })
    ));
}
#[test]
fn appended_unbound_fields_and_geometry_are_not_silently_omitted() {
    for (col, value) in [(0, "EXTRA"), (3, "4")] {
        let (_dir, path) = workbook(|book| {
            book.worksheet_from_name("Tracciato")
                .unwrap()
                .write_string(99, col, value)
                .unwrap();
        });
        assert!(matches!(
            importer().import(&path),
            Err(ImportError::UnexpectedRow { row: 100, .. })
        ));
    }
}
#[test]
fn formula_policy_is_explicit_and_noninteger_or_absent_cache_is_rejected() {
    let (_dir, path) = workbook(|_| {});
    let mut p = profile();
    p.coordinate_formulas = CoordinateFormulaPolicy::Reject;
    let reader = XlsxSchemaImporter::new(blueprint(), p).unwrap();
    assert!(matches!(
        reader.import(&path),
        Err(ImportError::Cell { .. })
    ));
    for result in ["1.5", "-1", "4294967296"] {
        let (_dir, path) = workbook(|book| {
            book.worksheet_from_name("Tracciato")
                .unwrap()
                .write_formula(8, 4, Formula::new("=1+1").set_result(result))
                .unwrap();
        });
        assert!(
            matches!(importer().import(&path), Err(ImportError::Cell { .. })),
            "result{result}"
        );
    }
    let (_dir, path) = workbook(|_| {});
    patch_xml(&path, |xml| {
        assert!(xml.contains("<f>E8+D8</f><v>7</v>"));
        xml.replace("<f>E8+D8</f><v>7</v>", "<f>E8+D8</f>")
    });
    assert!(matches!(
        importer().import(&path),
        Err(ImportError::Cell { .. })
    ));
}
#[test]
fn integer_coordinates_reject_strings_and_fractions() {
    let (_dir, path) = workbook(|book| {
        book.worksheet_from_name("Tracciato")
            .unwrap()
            .write_string(7, 3, "6")
            .unwrap();
    });
    assert!(importer().import(&path).is_err());
    let (_dir, path) = workbook(|book| {
        book.worksheet_from_name("Tracciato")
            .unwrap()
            .write_number(7, 3, 6.25)
            .unwrap();
    });
    assert!(importer().import(&path).is_err());
}
#[test]
fn incomplete_ambiguous_out_of_bounds_profiles_and_wrong_blueprints_fail_before_io() {
    let base = profile();
    for p in [
        {
            let mut p = base.clone();
            p.traces.pop();
            p
        },
        {
            let mut p = base.clone();
            p.traces[0].fields.pop();
            p
        },
        {
            let mut p = base.clone();
            p.traces[0].fields[1].row = 8;
            p
        },
        {
            let mut p = base.clone();
            p.traces[0].fields[1].field_id = "001.01".into();
            p
        },
        {
            let mut p = base.clone();
            p.columns.notes.number = 0;
            p
        },
        {
            let mut p = base.clone();
            p.columns.notes.number = 1;
            p
        },
        {
            let mut p = base.clone();
            p.traces[1].heading_row = 2;
            p
        },
        {
            let mut p = base.clone();
            p.traces[0].fields[0].reviewed_physical_type = "numeric".into();
            p
        },
        {
            let mut p = base.clone();
            p.flow_id = "OTHER".into();
            p
        },
        {
            let mut p = base.clone();
            p.blueprint_version = "other".into();
            p
        },
    ] {
        assert!(matches!(
            XlsxSchemaImporter::new(blueprint(), p),
            Err(ImportError::InvalidProfile { .. })
        ));
    }
    let mut b = blueprint();
    b.source.insert("xlsx_import".into(), json!({}));
    assert!(XlsxSchemaImporter::new(b, base.clone()).is_err());
    let mut b = blueprint();
    b.traces[0].fields[0].regulatory_metadata = Some(std::collections::BTreeMap::from([(
        "xlsx_source".into(),
        json!({}),
    )]));
    assert!(XlsxSchemaImporter::new(b, base).is_err());
}
#[test]
fn io_and_malformed_workbook_keep_the_original_technical_cause() {
    use std::error::Error;
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        importer().import(&dir.path().join("absent.xlsx")),
        Err(ImportError::Io(_))
    ));
    let path = dir.path().join("malformed.xlsx");
    fs::write(&path, b"not a workbook").unwrap();
    let error = importer().import(&path).unwrap_err();
    assert!(matches!(error, ImportError::Workbook(_)));
    assert!(error.source().is_some());
    let (_dir, path) = workbook(|_| {});
    let mut p = profile();
    p.sheet = "Not present".into();
    let error = XlsxSchemaImporter::new(blueprint(), p)
        .unwrap()
        .import(&path)
        .unwrap_err();
    assert!(error.source().is_some());
}
#[test]
fn general_notes_are_preserved_but_annotation_formulas_are_rejected() {
    let (_dir, path) = workbook(|book| {
        book.worksheet_from_name("Indicazioni operative")
            .unwrap()
            .write_formula(0, 0, "=1+1")
            .unwrap();
    });
    assert!(importer().import(&path).is_err());
}
#[test]
fn source_hash_tracks_the_actual_snapshot_and_not_the_blueprint() {
    use sha2::{Digest, Sha256};
    let (_dir, path) = workbook(|_| {});
    let a = importer().import(&path).unwrap();
    let digest: String = Sha256::digest(fs::read(&path).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(a.source["xlsx_import"]["source_sha256"], digest);
    patch_xml(&path, |xml| xml.replace("<f>E8+D8</f>", "<f>7</f>"));
    let b = importer().import(&path).unwrap();
    assert_ne!(
        a.source["xlsx_import"]["source_sha256"],
        b.source["xlsx_import"]["source_sha256"]
    );
    assert_eq!(
        a.source["xlsx_import"]["blueprint_sha256"],
        b.source["xlsx_import"]["blueprint_sha256"]
    );
}
#[test]
fn supplied_sioc_profile_covers_all_reviewed_fields_without_changing_semantics() {
    let b: RawFlowSchema = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    let p: XlsxImportProfile = serde_json::from_str(include_str!(
        "../docs/schema-import/sioc-2026-04-20.xlsx-profile.json"
    ))
    .unwrap();
    assert_eq!(
        p.traces.iter().map(|t| t.fields.len()).collect::<Vec<_>>(),
        [18, 50, 36, 20, 11]
    );
    assert!(XlsxSchemaImporter::new(b, p).is_ok());
}
#[test]
#[ignore = "manual qualification against the author-provided normative XLSX; no external files required by CI"]
fn original_sioc_workbook_is_imported_through_the_production_adapter() {
    let path =
        std::env::var_os("LINTRA_SIOC_XLSX").expect("explicit normative workbook path required");
    let b: RawFlowSchema = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    let p: XlsxImportProfile = serde_json::from_str(include_str!(
        "../docs/schema-import/sioc-2026-04-20.xlsx-profile.json"
    ))
    .unwrap();
    let mut raw = XlsxSchemaImporter::new(b.clone(), p)
        .unwrap()
        .import(Path::new(&path))
        .unwrap();
    assert_eq!(
        raw.source["xlsx_import"]["source_sha256"],
        "6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3"
    );
    let mut formulas = 0;
    let mut changed_notes = 0;
    for (t, old) in raw.traces.iter().zip(&b.traces) {
        assert_eq!(t.joins, old.joins);
        assert_eq!(t.rules, old.rules);
        assert_eq!(t.record_key, old.record_key);
        for (f, old) in t.fields.iter().zip(&old.fields) {
            assert_eq!(f.physical, old.physical);
            assert_eq!(f.presence, old.presence);
            assert_eq!(f.constraints, old.constraints);
            let m = f.regulatory_metadata.as_ref().unwrap();
            if !m["xlsx_source"]["start_formula"].is_null() {
                formulas += 1;
            }
            if m["source_note"] != old.regulatory_metadata.as_ref().unwrap()["source_note"] {
                changed_notes += 1;
            }
        }
    }
    assert_eq!(formulas, 130);
    assert_eq!(changed_notes, 8);
    assert!(
        raw.source["xlsx_import"]["annotations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["text"]
                .as_str()
                .unwrap()
                .contains("separazione delle migliaia"))
    );
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let error = SchemaCompiler.compile(raw).unwrap_err();
    assert!(format!("{error:?}").contains("PersistenceAuditV1"));
    println!(
        "Real SIOC:135 fields,130 cached formulas,8 exact original-note updates; unsupported full capability still rejected"
    );
}
