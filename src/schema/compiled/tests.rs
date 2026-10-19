use super::*;
use crate::domain::{FlowId, SchemaVersion, ValidationCode};

fn field(id: &str, start: usize, len: usize) -> CompiledFieldSpec {
    CompiledFieldSpec {
        id: FieldId(id.into()),
        name: format!("Synthetic {id}").into(),
        range: ByteRange::new(start, len).unwrap(),
        field_type: FieldType::Alphanumeric,
        missing_value_policy: MissingValuePolicy::Spaces,
        presence_policy: CompiledPresencePolicy {
            base: BasePresence::Required,
            required_if: vec![],
            forbidden_if: vec![],
        },
        alignment: AlignmentRule::RightPad {
            fill: b' ',
            content_policy: ContentBoundaryPolicy::StripTrailingFill,
        },
        lexical: Some(LexicalPlan::EncodedText),
        constraints: vec![],
        regulatory_metadata: None,
    }
}

fn record_parts(id: &str) -> RecordParts {
    RecordParts {
        id: TraceId(id.into()),
        name: "Synthetic trace".into(),
        record_length: 5,
        // Deliberately different physical and lexical identifier orders.
        fields: vec![field("Z", 0, 3), field("A", 3, 2)],
        local_rules: vec![],
        derived_values: vec![],
        identity_plan: IdentityExtractionPlan {
            bindings: vec![IdentityBinding {
                target: IdentityTarget::Technical("example".into()),
                field_index: 0,
            }],
        },
        projection_plan: ContextProjectionPlan {
            field_indexes: vec![0, 1],
            join_indexes: vec![],
        },
        record_key: RecordKeyPlan {
            field_indexes: vec![0, 1],
            error_code: ValidationCode::DuplicateRecordKey,
        },
        repeated_groups: vec![],
        documented_non_executable_rules: vec![],
    }
}

fn flow_parts(traces: Vec<CompiledRecordSchema>) -> FlowParts {
    FlowParts {
        key: SchemaKey {
            flow_id: FlowId("EXAMPLE".into()),
            version: SchemaVersion("v1".into()),
        },
        schema_format_version: 1,
        regulatory_version: "synthetic-only".into(),
        source: Default::default(),
        encoding: EncodingSpec::Utf8,
        traces,
        required_reference_data: vec![],
        cross_record_plan: CrossRecordValidationPlan {
            joins: vec![],
            rules: vec![],
            required_traces: vec![],
        },
        parameters: vec![],
        required_capabilities: vec![Capability::LocalV1],
        persistence_policy: None,
    }
}

#[test]
fn index_is_built_from_physical_order_and_every_entry_resolves_to_its_field() {
    let record = CompiledRecordSchema::from_parts(record_parts("T")).unwrap();
    assert_eq!(
        record
            .fields()
            .iter()
            .map(|f| f.id().0.as_str())
            .collect::<Vec<_>>(),
        ["Z", "A"]
    );
    assert_eq!(record.field_index.len(), record.fields.len());
    for (id, index) in &record.field_index {
        assert_eq!(record.fields().get(*index).unwrap().id(), id);
        assert_eq!(record.field(id).unwrap().id(), id);
    }
    assert_eq!(record.field_index(&FieldId("Z".into())), Some(0));
    assert_eq!(record.field_index(&FieldId("A".into())), Some(1));
    assert_eq!(record.field_index(&FieldId("MISSING".into())), None);
    assert!(record.field(&FieldId("MISSING".into())).is_none());
    assert_eq!(
        record
            .field(&FieldId("A".into()))
            .unwrap()
            .range()
            .slice(b"ABCDE"),
        Some(&b"DE"[..])
    );
}

#[test]
fn duplicate_ids_and_outside_ranges_cannot_create_an_inconsistent_record() {
    let mut parts = record_parts("T");
    parts.fields[1].id = parts.fields[0].id.clone();
    assert_eq!(
        CompiledRecordSchema::from_parts(parts).unwrap_err(),
        ModelInvariantError::DuplicateFieldId(FieldId("Z".into()))
    );
    let mut parts = record_parts("T");
    parts.record_length = 4;
    assert_eq!(
        CompiledRecordSchema::from_parts(parts).unwrap_err(),
        ModelInvariantError::FieldOutsideRecord(FieldId("A".into()))
    );
}

#[test]
fn invalid_key_projection_and_identity_indexes_are_rejected() {
    for kind in 0..3 {
        let mut parts = record_parts("T");
        match kind {
            0 => parts.record_key.field_indexes = vec![usize::MAX],
            1 => parts.projection_plan.field_indexes = vec![usize::MAX],
            _ => parts.identity_plan.bindings[0].field_index = usize::MAX,
        }
        assert_eq!(
            CompiledRecordSchema::from_parts(parts).unwrap_err(),
            ModelInvariantError::InvalidFieldIndex(usize::MAX)
        );
    }
    let mut parts = record_parts("T");
    parts.record_key.field_indexes.clear();
    assert_eq!(
        CompiledRecordSchema::from_parts(parts).unwrap_err(),
        ModelInvariantError::EmptyRecordKey
    );
}

#[test]
fn empty_records_and_flows_are_not_usable_compiled_models() {
    for empty_fields in [true, false] {
        let mut parts = record_parts("T");
        if empty_fields {
            parts.fields.clear();
        } else {
            parts.record_length = 0;
        }
        assert_eq!(
            CompiledRecordSchema::from_parts(parts).unwrap_err(),
            ModelInvariantError::EmptyRecord
        );
    }
    assert_eq!(
        CompiledFlowSchema::from_parts(flow_parts(vec![])).unwrap_err(),
        ModelInvariantError::EmptyFlow
    );
}

#[test]
fn flow_preserves_trace_order_and_rejects_duplicate_trace_ids() {
    let z = CompiledRecordSchema::from_parts(record_parts("Z")).unwrap();
    let a = CompiledRecordSchema::from_parts(record_parts("A")).unwrap();
    let flow = CompiledFlowSchema::from_parts(flow_parts(vec![z.clone(), a])).unwrap();
    assert_eq!(
        flow.traces()
            .iter()
            .map(|t| t.id().0.as_str())
            .collect::<Vec<_>>(),
        ["Z", "A"]
    );
    assert_eq!(flow.trace(&TraceId("A".into())).unwrap().record_length(), 5);
    assert!(flow.trace(&TraceId("MISSING".into())).is_none());
    let key: &crate::SchemaKey = flow.key();
    assert_eq!(key.flow_id, FlowId("EXAMPLE".into()));
    assert_eq!(key.version, SchemaVersion("v1".into()));
    assert!(flow.persistence_policy().is_none());
    assert_eq!(
        CompiledFlowSchema::from_parts(flow_parts(vec![z.clone(), z])).unwrap_err(),
        ModelInvariantError::DuplicateTraceId(TraceId("Z".into()))
    );
}

#[test]
fn compiled_payloads_own_a_regex_and_keep_non_applicability_distinct() {
    let mut parts = record_parts("T");
    parts.fields[0].constraints.push(CompiledConstraint::Regex {
        regex: regex::bytes::Regex::new("(?-u)^[0-9]+$").unwrap(),
        mode: RegexMatch::Required,
        target: ValueTarget::Logical,
    });
    let record = CompiledRecordSchema::from_parts(parts).unwrap();
    let CompiledConstraint::Regex { regex, .. } = &record.fields()[0].constraints()[0] else {
        panic!("wrong plan")
    };
    assert_eq!(regex.as_str(), "(?-u)^[0-9]+$");
    let policy = RelatedPolicy::NotApplicable {
        diagnostic: RelatedDiagnostic {
            code: "EXAMPLE".into(),
            message_template: "Synthetic explanation".into(),
        },
    };
    assert!(matches!(policy, RelatedPolicy::NotApplicable { .. }));
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CompiledFlowSchema>();
}

#[test]
fn compiled_integer_literals_are_exact_beyond_machine_limits() {
    let huge: IntegerLiteral = "12345678901234567890123456789012345678901234567890"
        .parse()
        .unwrap();
    let small: IntegerLiteral = "2".parse().unwrap();
    assert!(huge > small);
    assert_eq!(
        huge.magnitude(),
        b"12345678901234567890123456789012345678901234567890"
    );
    assert!(
        "-100000000000000000000000000000000000000000000000000"
            .parse::<IntegerLiteral>()
            .unwrap()
            < "-10".parse().unwrap()
    );
    assert!("-10".parse::<IntegerLiteral>().unwrap() < "0".parse().unwrap());
    assert_eq!(
        "-000".parse::<IntegerLiteral>().unwrap(),
        "0".parse().unwrap()
    );
    assert!(!"-000".parse::<IntegerLiteral>().unwrap().is_negative());
    for invalid in ["", "-", "+1", " 1", "1 ", "1.0", "1e3", "١"] {
        assert_eq!(
            invalid.parse::<IntegerLiteral>(),
            Err(InvalidIntegerLiteral)
        );
    }
}

#[test]
fn physical_order_gaps_overlaps_and_trailing_uncovered_bytes_are_rejected() {
    let mut reversed = record_parts("T");
    reversed.fields.swap(0, 1);
    assert_eq!(
        CompiledRecordSchema::from_parts(reversed).unwrap_err(),
        ModelInvariantError::PhysicalOrderMismatch(FieldId("A".into()))
    );
    for (start, len) in [(2, 2), (4, 1)] {
        let mut parts = record_parts("T");
        parts.fields[1].range = ByteRange::new(start, len).unwrap();
        assert_eq!(
            CompiledRecordSchema::from_parts(parts).unwrap_err(),
            ModelInvariantError::PhysicalOrderMismatch(FieldId("A".into()))
        );
    }
    let mut parts = record_parts("T");
    parts.record_length = 6;
    assert_eq!(
        CompiledRecordSchema::from_parts(parts).unwrap_err(),
        ModelInvariantError::RecordLengthMismatch
    );
}
