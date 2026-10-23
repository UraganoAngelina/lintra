use super::*;
use crate::domain::{FlowId, SchemaVersion, ValidationCode};

#[path = "derived_presence_tests.rs"]
mod derived_presence_tests;

/// A private synthetic field lets the runtime boundaries be tested without
/// admitting unimplemented plans through the real production compiler.
fn run_private_pipeline_fixture(
    spec: &CompiledFieldSpec,
) -> Result<crate::FieldPipelineResult, crate::FieldPipelineError> {
    let key = SchemaKey {
        flow_id: FlowId("EXAMPLE".into()),
        version: SchemaVersion("v1".into()),
    };
    let trace = TraceId("T".into());
    let mut summary = crate::ValidationSummaryBuilder::new(
        key.clone(),
        crate::ScopeSummary::Trace {
            trace_id: trace.clone(),
        },
    );
    let mut sink = crate::VecSink::default();
    crate::FieldPipeline::new(
        crate::ErrorFactory {
            flow_id: &key.flow_id,
            schema_version: &key.version,
            trace_id: &trace,
            encoding: &EncodingSpec::Utf8,
        },
        &mut sink,
        &mut summary,
    )
    .run(
        spec,
        crate::FieldSlice { raw: b"ABC" },
        1,
        &crate::RecordIdentity::default(),
    )
}

/// Unknown stops before malformed later plans; once presence is resolved,
/// bad extraction/datatype/constraint plans remain typed technical failures.
#[test]
fn pipeline_unknown_and_unsupported_plan_boundaries_are_explicit() {
    use crate::{FieldPipelineError as Failure, FieldPipelineResult as Outcome};
    let mut spec = field("F", 0, 3);
    spec.presence_policy
        .required_if
        .push(CompiledExpr::Const(true));
    spec.alignment = AlignmentRule::RightPad {
        fill: b' ',
        content_policy: ContentBoundaryPolicy::StripLeadingFill,
    };
    assert_eq!(
        run_private_pipeline_fixture(&spec).unwrap(),
        Outcome::SkippedDueToUnknownPresence
    );
    spec.presence_policy.required_if.clear();
    assert!(matches!(
        run_private_pipeline_fixture(&spec),
        Err(Failure::Logical(
            crate::LogicalValueError::IncompatibleContentPolicy
        ))
    ));
    spec.alignment = AlignmentRule::Exact;
    spec.field_type = FieldType::Date(DateFormat::DdMmYyyy);
    assert_eq!(
        run_private_pipeline_fixture(&spec).unwrap(),
        Outcome::Invalid
    );
    spec.field_type = FieldType::Alphanumeric;
    spec.constraints
        .push(CompiledConstraint::LogicalLength { equals: 3 });
    assert!(
        matches!(run_private_pipeline_fixture(&spec), Err(Failure::UnsupportedConstraint { field_id, constraint_index: 0 }) if field_id.0 == "F")
    );
}

/// Numeric/AN require a lexical plan; Date has its own explicit ASCII format.
/// Unknown encoding still fails technically before interpreting record bytes.
#[test]
fn datatype_plan_invariants_and_date_representation_are_explicit() {
    use crate::{DatatypeError, FieldSlice, validate_datatype};
    let mut spec = field("F", 0, 3);
    spec.lexical = None;
    assert_eq!(
        validate_datatype(&spec, FieldSlice { raw: b"ABC" }, &EncodingSpec::Utf8),
        Err(DatatypeError::MissingLexicalPlan)
    );
    spec.field_type = FieldType::Date(DateFormat::DdMmYyyy);
    assert_eq!(
        validate_datatype(&spec, FieldSlice { raw: b"01012026" }, &EncodingSpec::Utf8),
        Ok(crate::DatatypeValidationResult::Valid)
    );
    assert!(matches!(
        validate_datatype(
            &spec,
            FieldSlice { raw: b"01012026" },
            &EncodingSpec::Other("unsupported".into())
        ),
        Err(DatatypeError::UnsupportedEncoding(_))
    ));
}

/// Private fixture access can construct future conditional plans without
/// relaxing compiled-model constructors or enabling unsupported compiler ASTs.
/// The base presence stage must refuse both kinds, for present/missing values.
#[test]
fn base_presence_never_ignores_compiled_conditions() {
    use crate::{FieldSlice, PresenceValidationResult, validate_presence};
    for required in [false, true] {
        let mut spec = field("F", 0, 3);
        if required {
            spec.presence_policy
                .required_if
                .push(CompiledExpr::Const(true));
        } else {
            spec.presence_policy
                .forbidden_if
                .push(CompiledExpr::Const(false));
        }
        for raw in [b"   ".as_slice(), b"ABC"] {
            assert_eq!(
                validate_presence(&spec, FieldSlice { raw }),
                Ok(PresenceValidationResult::Unknown)
            );
        }
    }
}

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

/// Execute the actual record boundary with private malformed/future models.
/// Production constructors remain closed: no unsupported AST is enabled just
/// to make a runtime failure test reachable.
fn run_private_record(
    schema: &CompiledFlowSchema,
    bytes: &[u8],
) -> Result<crate::LocalRecordValidationArtifacts, crate::RecordValidationError> {
    let trace = &schema.traces()[0];
    let mut sink = crate::VecSink::default();
    let mut summary = crate::ValidationSummaryBuilder::new(
        schema.key().clone(),
        crate::ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let result = crate::RecordValidator::new(schema, trace.id())?.validate_local(
        crate::RawRecord {
            line_number: 1,
            bytes,
        },
        &mut sink,
        &mut summary,
    );
    if result.is_err() {
        assert!(
            sink.errors().is_empty(),
            "technical failure must not become a normative error"
        );
    }
    result
}

/// A true identity indexing failure and a real pipeline extraction failure
/// keep their original payloads; malformed width never reaches either stage.
#[test]
fn local_record_preserves_identity_and_pipeline_technical_causes() {
    use crate::{FieldPipelineError, RecordValidationError as Failure};
    let trace = CompiledRecordSchema::from_parts(record_parts("T")).unwrap();
    let mut schema = CompiledFlowSchema::from_parts(flow_parts(vec![trace])).unwrap();
    schema.traces[0].identity_plan.bindings[0].field_index = usize::MAX;
    assert!(matches!(
        run_private_record(&schema, b"ABCDE"),
        Err(Failure::Identity(
            crate::IdentityExtractionError::InvalidFieldIndex {
                field_index: usize::MAX
            }
        ))
    ));
    let malformed = run_private_record(&schema, b"x").unwrap();
    assert!(!malformed.valid && malformed.identity.is_none());
    schema.traces[0].identity_plan.bindings[0].field_index = 0;
    schema.traces[0].fields[0].alignment = AlignmentRule::RightPad {
        fill: b' ',
        content_policy: ContentBoundaryPolicy::StripLeadingFill,
    };
    assert!(matches!(
        run_private_record(&schema, b"ABCDE"),
        Err(Failure::Pipeline(FieldPipelineError::Logical(
            crate::LogicalValueError::IncompatibleContentPolicy
        )))
    ));
    schema.traces[0].fields[0].alignment = AlignmentRule::Exact;
    schema.traces[0].fields[0].range = ByteRange::new(20, 3).unwrap();
    // Skip identity so the record's own checked field access is exercised.
    schema.traces[0].identity_plan.bindings.clear();
    assert!(
        matches!(run_private_record(&schema,b"ABCDE"),Err(Failure::FieldOutOfBounds{field_id}) if field_id.0=="Z")
    );
}

/// Future compiled local obligations cannot silently pass the current runtime,
/// even on blank fields where the field pipeline would otherwise stop early.
#[test]
fn local_record_rejects_every_unimplemented_local_plan_before_processing() {
    use crate::{RecordValidationError as Failure, UnsupportedLocalFeature as Feature};
    let trace = CompiledRecordSchema::from_parts(record_parts("T")).unwrap();
    let base = CompiledFlowSchema::from_parts(flow_parts(vec![trace])).unwrap();
    for feature in [
        Feature::Rules,
        Feature::DerivedValues,
        Feature::RepeatedGroups,
        Feature::ConditionalPresence,
        Feature::Constraint,
    ] {
        let mut schema = base.clone();
        let trace = &mut schema.traces[0];
        match feature {
            Feature::Rules => trace.local_rules.push(CompiledRule {
                id: "synthetic".into(),
                when: CompiledExpr::Const(true),
                assertions: vec![],
                error_code: ValidationCode::CrossField,
                message_template: "synthetic".into(),
            }),
            Feature::DerivedValues => trace.derived_values.push(DerivedValuePlan::Substring {
                source: ResolvedFieldRef::Local {
                    field_index: 0,
                    field_id: FieldId("Z".into()),
                },
                target: ValueTarget::Raw,
                // An actual invalid plan remains rejected after the executor
                // is delivered; a valid substring is now supported.
                range: ByteRange::new(0, 4).unwrap(),
            }),
            Feature::RepeatedGroups => trace.repeated_groups.push(RepeatedGroupPlan {
                id: "synthetic".into(),
                fields: vec![0],
                required_slots: vec![],
                optional_slots: vec![],
                contiguity_required: false,
                unique_non_missing: false,
                unused_slot_value: None,
            }),
            Feature::ConditionalPresence => {
                trace.fields[0]
                    .presence_policy
                    .forbidden_if
                    .push(CompiledExpr::DerivedEq {
                        field: ResolvedFieldRef::Local {
                            field_index: 0,
                            field_id: FieldId("Z".into()),
                        },
                        derived_index: 0,
                    })
            }
            Feature::Constraint => trace.fields[0]
                .constraints
                .push(CompiledConstraint::LogicalLength { equals: 3 }),
        }
        assert!(
            matches!(run_private_record(&schema,b"     "),Err(Failure::UnsupportedLocalPlan{feature:found,..}) if found==feature)
        );
    }
}

/// Invalid scalar metadata never becomes a DecimalSpec; the public compiled
/// type exposes no setters. This tests structural safety beyond the v2 profile.
#[test]
fn decimal_spec_rejects_impossible_width_scale_and_separator_metadata() {
    for (width, scale, separator) in [
        (0, 2, b'.'),
        (3, 2, b'.'),
        (7, 0, b'.'),
        (7, usize::MAX, b'.'),
        (7, 2, b'0'),
        (7, 2, b'+'),
        (7, 2, b'-'),
        (7, 2, b' '),
        (7, 2, 0xff),
    ] {
        assert!(DecimalSpec::new(width, scale, separator, SignPolicy::Unsigned).is_none());
    }
}

/// The engine obeys verified metadata rather than hardcoding the monetary
/// profile. These private plans are not additional formats accepted by authoring.
#[test]
fn decimal_validator_uses_the_plan_scale_separator_and_physical_width() {
    let spec = DecimalSpec::new(7, 3, b',', SignPolicy::Unsigned).unwrap();
    assert!(
        crate::validators::decimal::validate_decimal(&spec, crate::FieldSlice { raw: b"012,345" })
            .is_ok()
    );
    for raw in [b"0012,34".as_slice(), b"012.345", b"012,34", b"012,3450"] {
        assert!(
            crate::validators::decimal::validate_decimal(&spec, crate::FieldSlice { raw }).is_err()
        );
    }
}

/// Positive wiring check of the real local resolver, without opening a public
/// constructor or bypassing the still-closed production derived-schema gate.
#[test]
fn local_context_uses_the_stored_derived_plan_and_real_record_bytes() {
    use crate::{DerivedValue, FieldValueResolver, LocalEvaluationContext, RecordView};
    let mut raw: crate::RawFlowSchema = serde_json::from_slice(include_bytes!(
        "../../../tests/golden/synthetic/v1/schema.json"
    ))
    .unwrap();
    raw.encoding.resolved = Some(crate::RawResolvedEncoding::Utf8);
    let mut schema = crate::SchemaCompiler.compile(raw).unwrap();
    schema.traces[0]
        .derived_values
        .push(DerivedValuePlan::Substring {
            source: ResolvedFieldRef::Local {
                field_index: 0,
                field_id: FieldId("F1".into()),
            },
            target: ValueTarget::Raw,
            range: ByteRange::new(1, 3).unwrap(),
        });
    let view = RecordView::new(b"ABCDEF");
    let local = LocalEvaluationContext {
        record: &view,
        schema: &schema.traces[0],
    };
    assert_eq!(local.resolve_derived(0), DerivedValue::Bytes(b"BCD"));
    assert_eq!(
        local.resolve_derived(1),
        DerivedValue::Unknown(crate::UnknownReason::FieldUnavailable)
    );
}
