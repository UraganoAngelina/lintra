use lintra::*;
use serde_json::{Value, json};
fn raw() -> RawFlowSchema {
    let mut r: RawFlowSchema =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    r.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    r
}
fn noted() -> RawFlowSchema {
    let mut r = raw();
    r.traces[0].fields[0].regulatory_metadata = Some(std::collections::BTreeMap::from([
        ("source_note".into(), json!("  Nota originale\n00123  ")),
        (
            "xlsx_source".into(),
            json!({"notes_original":"  Nota originale\n00123  "}),
        ),
    ]));
    r
}
fn reason() -> ImportedNoteDisposition {
    ImportedNoteDisposition::Metadata {
        reason: "Explicit source-description review".into(),
    }
}
fn domains(required: bool, same_target: bool) -> RawFlowSchema {
    let mut v = serde_json::to_value(raw()).unwrap();
    v["traces"][0]["fields"][0]["presence"]["base"] =
        json!(if required { "required" } else { "optional" });
    v["traces"][0]["fields"][0]["constraints"] = json!([{"kind":"allowed_values","target":"logical","values":["000001","000002"]},{"kind":"allowed_values","target":if same_target {"logical"} else {"raw"},"values":["000003"]}]);
    {
        let mut raw: RawFlowSchema = serde_json::from_value(v).unwrap();
        raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
        raw
    }
}
#[test]
fn confirmed_aliases_normalize_only_reviewed_tags_and_preserve_original_tokens() {
    let p: XlsxImportProfile = serde_json::from_str(include_str!(
        "../docs/schema-import/sioc-2026-04-20.xlsx-profile.json"
    ))
    .unwrap();
    let aliases: Vec<TypeAlias> = serde_json::from_str(include_str!(
        "../docs/schema-import/sioc-2026-04-20.type-aliases.json"
    ))
    .unwrap();
    let mut source = p.clone();
    for t in &mut source.traces {
        for f in &mut t.fields {
            if let Some(alias) = &f.source_type {
                f.reviewed_physical_type = alias.clone();
            }
        }
    }
    let result = normalize_import_profile(&source, &aliases).unwrap();
    assert_eq!(result.profile, p);
    assert_eq!(result.changes.len(), 134);
    assert_ne!(source, p);
    assert!(
        normalize_import_profile(&p, &aliases)
            .unwrap()
            .changes
            .is_empty()
    );
    let b: RawFlowSchema = serde_json::from_str(include_str!(
        "../schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json"
    ))
    .unwrap();
    assert!(XlsxSchemaImporter::new(b, result.profile).is_ok());
}
#[test]
fn unknown_aliases_duplicates_and_canonical_rebindings_fail() {
    let mut p: XlsxImportProfile = serde_json::from_str(include_str!(
        "../docs/schema-import/sioc-2026-04-20.xlsx-profile.json"
    ))
    .unwrap();
    for alias in ["A", " N ", "IMPORTO", "Unknown"] {
        p.traces[0].fields[0].reviewed_physical_type = alias.into();
        let old = p.clone();
        assert!(matches!(
            normalize_import_profile(&p, &[]),
            Err(SchemaAuthoringError::UnmappedAlias { .. })
        ));
        assert_eq!(p, old);
    }
    for aliases in [
        vec![TypeAlias {
            alias: "numeric".into(),
            canonical: ImportedPhysicalType::Date,
        }],
        vec![TypeAlias {
            alias: "".into(),
            canonical: ImportedPhysicalType::Numeric,
        }],
        vec![
            TypeAlias {
                alias: "N".into(),
                canonical: ImportedPhysicalType::Numeric
            };
            2
        ],
    ] {
        assert!(matches!(
            normalize_import_profile(&p, &aliases),
            Err(SchemaAuthoringError::InvalidAlias(_))
        ));
    }
}
#[test]
fn unreviewed_imported_notes_warn_without_inventing_rules_or_preventing_compile() {
    let r = noted();
    let issues = lint_raw_schema(&r);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, SchemaIssueCode::UnreviewedImportedNote);
    assert_eq!(issues[0].severity, SchemaIssueSeverity::Warning);
    assert_eq!(
        issues[0].source_note.as_deref(),
        Some("  Nota originale\n00123  ")
    );
    assert!(r.traces[0].rules.is_empty());
    let compiled = SchemaCompiler.compile(r);
    assert!(compiled.is_ok(), "{compiled:?}");
}
#[test]
fn metadata_review_is_bound_to_exact_text_and_becomes_stale_after_text_changes() {
    let mut r = noted();
    review_imported_note(&mut r, "T1", "F1", reason()).unwrap();
    assert!(lint_raw_schema(&r).is_empty());
    let m = r.traces[0].fields[0].regulatory_metadata.as_mut().unwrap();
    m.insert("source_note".into(), json!("Nota originale\n00123"));
    m.get_mut("xlsx_source").unwrap()["notes_original"] = json!("Nota originale\n00123");
    let i = lint_raw_schema(&r);
    assert_eq!(i.len(), 1);
    assert_eq!(i[0].code, SchemaIssueCode::UnreviewedImportedNote);
    assert_eq!(i[0].severity, SchemaIssueSeverity::Warning);
    review_imported_note(&mut r, "T1", "F1", reason()).unwrap();
    assert!(lint_raw_schema(&r).is_empty());
}
#[test]
fn mapped_notes_require_unique_existing_rules_without_creating_them() {
    let mut r = noted();
    let d = ImportedNoteDisposition::Mapped {
        rule_ids: vec!["R".into()],
    };
    assert!(review_imported_note(&mut r, "T1", "F1", d.clone()).is_err());
    r.traces[0].rules.push(serde_json::from_value(json!({"id":"R","when":{"const":true},"assertions":[{"kind":"field_present","field":{"field":"F1"}}],"error_code":"E_CONDITIONAL_REQUIRED","message_template":"Reviewed source note"})).unwrap());
    review_imported_note(&mut r, "T1", "F1", d).unwrap();
    assert!(lint_raw_schema(&r).is_empty());
    assert_eq!(r.traces[0].rules.len(), 1);
    r.traces[0].rules.clear();
    let i = lint_raw_schema(&r);
    assert_eq!(i[0].code, SchemaIssueCode::InvalidImportedNoteReview);
    assert_eq!(i[0].severity, SchemaIssueSeverity::Error);
    assert!(SchemaCompiler.compile(r).is_err());
}
#[test]
fn malformed_reviews_and_invalid_classifications_fail() {
    let mut r = noted();
    assert!(
        review_imported_note(
            &mut r,
            "T1",
            "F1",
            ImportedNoteDisposition::Metadata {
                reason: "  ".into()
            }
        )
        .is_err()
    );
    assert!(review_imported_note(&mut r, "absent", "F1", reason()).is_err());
    assert!(review_imported_note(&mut r, "T1", "absent", reason()).is_err());
    r.traces[0].fields[0]
        .regulatory_metadata
        .as_mut()
        .unwrap()
        .insert("xlsx_note_review".into(), json!({"made_up":true}));
    let i = lint_raw_schema(&r);
    assert_eq!(i[0].code, SchemaIssueCode::InvalidImportedNoteReview);
    assert_eq!(i[0].severity, SchemaIssueSeverity::Error);
    assert!(review_imported_note(&mut raw(), "T1", "F1", reason()).is_err());
}
#[test]
fn disjoint_domains_are_errors_when_always_required() {
    let r = domains(true, true);
    let i = lint_raw_schema(&r);
    assert_eq!(i.len(), 1);
    assert_eq!(i[0].code, SchemaIssueCode::ContradictoryConstraints);
    assert_eq!(i[0].severity, SchemaIssueSeverity::Error);
    assert!(SchemaCompiler.compile(r).is_err());
}
#[test]
fn optional_and_conditionally_forbidden_domains_warn_but_targets_are_not_mixed() {
    let r = domains(false, true);
    assert_eq!(
        lint_raw_schema(&r)[0].severity,
        SchemaIssueSeverity::Warning
    );
    let compiled = SchemaCompiler.compile(r);
    assert!(compiled.is_ok(), "{compiled:?}");
    assert!(lint_raw_schema(&domains(true, false)).is_empty());
    let mut v: Value = serde_json::to_value(domains(true, true)).unwrap();
    v["traces"][0]["fields"][0]["presence"]["forbidden_if"] = json!([{"const":true}]);
    let r = serde_json::from_value(v).unwrap();
    assert_eq!(
        lint_raw_schema(&r)[0].severity,
        SchemaIssueSeverity::Warning
    );
}
#[test]
fn domain_intersection_is_exact_and_issue_order_is_deterministic() {
    let mut v = serde_json::to_value(domains(true, true)).unwrap();
    v["traces"][0]["fields"][0]["constraints"][1]["values"] = json!(["000002", "000004"]);
    assert!(lint_raw_schema(&serde_json::from_value(v).unwrap()).is_empty());
    let mut r = domains(true, true);
    r.traces[0].fields[0].regulatory_metadata =
        noted().traces[0].fields[0].regulatory_metadata.clone();
    let a = lint_raw_schema(&r);
    assert_eq!(a, lint_raw_schema(&r));
    assert_eq!(
        a.iter().map(|i| i.code).collect::<Vec<_>>(),
        [
            SchemaIssueCode::UnreviewedImportedNote,
            SchemaIssueCode::ContradictoryConstraints
        ]
    );
    let mut r = raw();
    r.traces[0].fields[0].regulatory_metadata = Some(std::collections::BTreeMap::from([(
        "source_note".into(),
        json!("Legacy arbitrary metadata"),
    )]));
    assert!(lint_raw_schema(&r).is_empty());
}

#[test]
fn changed_displayed_note_cannot_keep_an_old_review_or_hide_behind_original_text() {
    let mut r = noted();
    review_imported_note(&mut r, "T1", "F1", reason()).unwrap();
    r.traces[0].fields[0]
        .regulatory_metadata
        .as_mut()
        .unwrap()
        .insert("source_note".into(), json!("Different text"));
    assert_eq!(
        lint_raw_schema(&r)[0].code,
        SchemaIssueCode::InvalidImportedNoteReview
    );
    assert!(review_imported_note(&mut r, "T1", "F1", reason()).is_err());
}
