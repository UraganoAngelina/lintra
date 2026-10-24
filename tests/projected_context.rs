//! Scalar related contexts use actual compiled joins, projections and indexes.
use lintra::*;
use serde_json::{Value, json};
use std::{cell::Cell, collections::BTreeMap, error::Error};

fn raw_schema() -> Value {
    let mut value: Value =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    let mut trace = value["traces"][0].clone();
    let widths = [2, 2, 2, 4, 2];
    let mut start = 1;
    trace["record_length"] = json!(12);
    trace["fields"]=json!(["PK","KEY","VALUE","TEXT","UNUSED"].into_iter().zip(widths).map(|(id,width)| {
        let mut field=trace["fields"][0].clone();field["id"]=json!(id);
        field["physical"]["start"]=json!(start);field["physical"]["length"]=json!(width);start+=width;
        if id=="TEXT" {field["physical"]["alignment"]=json!({"kind":"right_pad","fill_byte":32,"content_policy":"strip_trailing_fill"});}
        if id=="VALUE" || id=="KEY" {field["missing_value"]=json!({"kind":"spaces"});}
        field
    }).collect::<Vec<_>>());
    trace["record_key"]["fields"] = json!(["PK"]);
    trace["identity"] = json!([{"target":"sioc_record_id","field":"VALUE"},{"target":"technical","technical_name":"text","field":"TEXT"}]);
    value["traces"] = json!(["S", "T", "U"].map(|id| {
        let mut t = trace.clone();
        t["id"] = json!(id);
        t
    }));
    value["required_capabilities"] = json!(["local.v1", "joins.v1", "derived.substring.v1"]);
    value["parameters"] = json!([{"type":"string","id":"org","allowed_values":["001","002"]},{"type":"integer","id":"year","min":2020,"max":2030}]);
    value["traces"][0]["derived_values"] = json!({"PART":{"kind":"substring","source_ref":{"field":"TEXT"},"target":"logical","start":1,"length":2}});
    value["traces"][0]["joins"] = json!([{"id":"S_T","target_trace":"T","source_fields":["KEY","PK"],"target_fields":["KEY","PK"],
        "cardinality":"zero_or_one","scope":"current_submission","namespace_parameters":["org","year"],
        "on_missing_record":{"policy":"not_applicable","diagnostic":{"code":"SYNTHETIC_ABSENT","message_template":"Synthetic target absent"}},
        "on_missing_trace":{"policy":"unknown","diagnostic":{"code":"SYNTHETIC_INPUT","message_template":"Synthetic input unavailable"}}}]);
    value
}
fn compile(value: Value) -> CompiledFlowSchema {
    let mut raw: RawFlowSchema = serde_json::from_value(value).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
fn parameters() -> BTreeMap<String, JoinParameterValue> {
    BTreeMap::from([
        ("org".into(), JoinParameterValue::Text("001".into())),
        (
            "year".into(),
            JoinParameterValue::Integer("2026".parse().unwrap()),
        ),
    ])
}
fn project(flow: &CompiledFlowSchema, id: &str, bytes: &[u8], line: u64) -> ProjectedRecord {
    project_record(
        &RecordView::new(bytes),
        flow.trace(&TraceId(id.into())).unwrap(),
        RecordIdentity::default(),
        line,
    )
    .unwrap()
}
fn reference(flow: &CompiledFlowSchema, target: &str, field: &str) -> ResolvedFieldRef {
    let raw: RawRef =
        serde_json::from_value(json!({"trace":target,"field":field,"join":"S_T"})).unwrap();
    resolve_field_reference(&raw, &TraceId("S".into()), flow).unwrap()
}

#[test]
fn projected_and_related_bytes_follow_declared_missing_raw_logical_and_derived_plans() {
    let flow = compile(raw_schema());
    let source = project(&flow, "S", b"00AA05Ab  ZZ", 1);
    let mut store = InMemoryContextStore::new();
    store
        .insert(project(&flow, "T", b"00AA07Cd  ZZ", 2))
        .unwrap();
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let params = parameters();
    let ctx = ProjectedEvaluationContext::new(
        &flow,
        &source,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &params,
        },
    )
    .unwrap();
    let value = reference(&flow, "T", "VALUE");
    let text = reference(&flow, "T", "TEXT");
    assert_eq!(
        ctx.resolve_field(&value),
        ResolvedFieldValue::Present(b"07")
    );
    assert_eq!(ctx.resolve_field(&text), ResolvedFieldValue::Present(b"Cd"));
    assert_eq!(
        ctx.resolve_raw_field(&text),
        ResolvedFieldValue::Present(b"Cd  ")
    );
    let local: RawRef = serde_json::from_value(json!({"field":"TEXT"})).unwrap();
    let local = resolve_field_reference(&local, &TraceId("S".into()), &flow).unwrap();
    assert_eq!(
        ctx.resolve_field(&local),
        ResolvedFieldValue::Present(b"Ab")
    );
    assert_eq!(
        ctx.resolve_raw_field(&local),
        ResolvedFieldValue::Present(b"Ab  ")
    );
    assert_eq!(ctx.resolve_derived(0), DerivedValue::Bytes(b"Ab"));
    assert_eq!(
        ExpressionEvaluator.evaluate(
            &CompiledExpr::Eq(value, b"07".to_vec()),
            &EvaluationContext { fields: &ctx }
        ),
        TruthValue::True
    );
    // A field missing on a present related record remains ordinary Missing.
    let mut missing = InMemoryContextStore::new();
    missing
        .insert(project(&flow, "T", b"00AA  Cd  ZZ", 2))
        .unwrap();
    let ctx = ProjectedEvaluationContext::new(
        &flow,
        &source,
        &missing,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &params,
        },
    )
    .unwrap();
    assert_eq!(
        ctx.resolve_field(&reference(&flow, "T", "VALUE")),
        ResolvedFieldValue::Missing
    );
}

#[test]
fn unavailable_relations_preserve_each_unknown_reason_and_the_boolean_truth_tables() {
    let flow = compile(raw_schema());
    let source = project(&flow, "S", b"00AA05Ab  ZZ", 1);
    let params = parameters();
    for (count, target_available, reason) in [
        (0, true, UnknownReason::RelatedRecordMissing),
        (2, true, UnknownReason::AmbiguousJoin),
        (1, false, UnknownReason::TraceInputUnavailable),
    ] {
        let mut store = InMemoryContextStore::new();
        for line in 1..=count {
            store
                .insert(project(&flow, "T", b"00AA07Cd  ZZ", line))
                .unwrap();
        }
        let traces = if target_available {
            vec![TraceId("S".into()), TraceId("T".into())]
        } else {
            vec![TraceId("S".into())]
        };
        let ctx = ProjectedEvaluationContext::new(
            &flow,
            &source,
            &store,
            CrossRecordInputs {
                available_traces: &traces,
                parameters: &params,
            },
        )
        .unwrap();
        let r = reference(&flow, "T", "VALUE");
        assert_eq!(
            ctx.resolve_field(&r),
            ResolvedFieldValue::Unavailable(reason)
        );
        let unknown = CompiledExpr::Eq(r.clone(), b"07".to_vec());
        let expressions = [
            (
                CompiledExpr::IsPresent(r.clone()),
                TruthValue::Unknown(reason),
            ),
            (CompiledExpr::IsMissing(r), TruthValue::Unknown(reason)),
            (
                CompiledExpr::Not(Box::new(unknown.clone())),
                TruthValue::Unknown(reason),
            ),
            (
                CompiledExpr::And(vec![CompiledExpr::Const(false), unknown.clone()]),
                TruthValue::False,
            ),
            (
                CompiledExpr::And(vec![unknown.clone(), CompiledExpr::Const(false)]),
                TruthValue::False,
            ),
            (
                CompiledExpr::And(vec![CompiledExpr::Const(true), unknown.clone()]),
                TruthValue::Unknown(reason),
            ),
            (
                CompiledExpr::Or(vec![CompiledExpr::Const(true), unknown.clone()]),
                TruthValue::True,
            ),
            (
                CompiledExpr::Or(vec![unknown.clone(), CompiledExpr::Const(true)]),
                TruthValue::True,
            ),
            (
                CompiledExpr::Or(vec![CompiledExpr::Const(false), unknown]),
                TruthValue::Unknown(reason),
            ),
        ];
        for (expr, expected) in expressions {
            assert_eq!(
                ExpressionEvaluator.evaluate(&expr, &EvaluationContext { fields: &ctx }),
                expected
            );
        }
        let lookup = ctx.related(0).unwrap();
        assert!(matches!(
            lookup.join.on_missing_record,
            RelatedPolicy::NotApplicable { .. }
        ));
        assert_eq!(lookup.join.id, JoinId("S_T".into()));
        if count == 0 {
            assert!(
                matches!(&lookup.resolution,RelatedResolution::Records(records) if records.is_empty())
            );
        }
        if !target_available {
            assert!(matches!(
                lookup.resolution,
                RelatedResolution::Unavailable(UnknownReason::TraceInputUnavailable)
            ));
        }
    }
}

struct CountingStore {
    inner: InMemoryContextStore,
    calls: Cell<usize>,
    fail: bool,
}
impl ContextStore for CountingStore {
    fn insert(&mut self, r: ProjectedRecord) -> Result<(), ContextError> {
        self.inner.insert(r)
    }
    fn find(
        &self,
        t: &TraceId,
        j: &JoinId,
        k: &JoinKey,
    ) -> Result<RelatedRecordSet<'_>, ContextError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(ContextError::InvalidJoinId)
        } else {
            self.inner.find(t, j, k)
        }
    }
    fn iter_trace<'a>(&'a self, t: &TraceId) -> Box<dyn Iterator<Item = &'a ProjectedRecord> + 'a> {
        self.inner.iter_trace(t)
    }
}

#[test]
fn namespace_and_input_preflight_happen_before_lookup_and_store_failures_keep_the_cause() {
    let flow = compile(raw_schema());
    let source = project(&flow, "S", b"00AA05Ab  ZZ", 1);
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let store = CountingStore {
        inner: InMemoryContextStore::new(),
        calls: Cell::new(0),
        fail: true,
    };
    for (id, value) in [
        ("org", JoinParameterValue::Text("1".into())),
        ("org", JoinParameterValue::Integer("1".parse().unwrap())),
        ("year", JoinParameterValue::Integer("2031".parse().unwrap())),
        ("unknown", JoinParameterValue::Text("001".into())),
    ] {
        let mut params = parameters();
        params.insert(id.into(), value);
        assert!(matches!(
            ProjectedEvaluationContext::new(
                &flow,
                &source,
                &store,
                CrossRecordInputs {
                    available_traces: &traces,
                    parameters: &params
                }
            ),
            Err(RelatedContextError::InvalidParameter)
        ));
    }
    let empty = BTreeMap::new();
    assert!(matches!(
        ProjectedEvaluationContext::new(
            &flow,
            &source,
            &store,
            CrossRecordInputs {
                available_traces: &traces,
                parameters: &empty
            }
        ),
        Err(RelatedContextError::MissingNamespaceParameter)
    ));
    let params = parameters();
    for (ids, expected) in [
        (vec![TraceId("S".into()), TraceId("S".into())], "duplicate"),
        (vec![TraceId("UNKNOWN".into())], "unknown"),
        (vec![TraceId("T".into())], "source"),
    ] {
        let err = ProjectedEvaluationContext::new(
            &flow,
            &source,
            &store,
            CrossRecordInputs {
                available_traces: &ids,
                parameters: &params,
            },
        )
        .err()
        .unwrap();
        assert!(matches!(
            (&err, expected),
            (RelatedContextError::DuplicateTraceInput, "duplicate")
                | (RelatedContextError::InvalidTraceInput, "unknown")
                | (RelatedContextError::SourceInputUnavailable, "source")
        ));
    }
    assert_eq!(store.calls.get(), 0);
    let err = ProjectedEvaluationContext::new(
        &flow,
        &source,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &params,
        },
    )
    .err()
    .unwrap();
    assert!(matches!(
        err,
        RelatedContextError::Store(ContextError::InvalidJoinId)
    ));
    assert!(err.source().unwrap().is::<ContextError>());
    assert_eq!(store.calls.get(), 1);
    assert!(!format!("{:?}", params["org"]).contains("001"));
}

#[test]
fn missing_source_key_does_not_invent_a_related_record_or_invoke_the_store() {
    let flow = compile(raw_schema());
    let source = project(&flow, "S", b"00  05Ab  ZZ", 1);
    let params = parameters();
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let store = CountingStore {
        inner: InMemoryContextStore::new(),
        calls: Cell::new(0),
        fail: true,
    };
    let ctx = ProjectedEvaluationContext::new(
        &flow,
        &source,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &params,
        },
    )
    .unwrap();
    assert_eq!(
        ctx.resolve_field(&reference(&flow, "T", "VALUE")),
        ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable)
    );
    assert_eq!(store.calls.get(), 0);
}

#[test]
fn reference_resolution_checks_source_target_join_and_field_and_scalar_cardinality() {
    let flow = compile(raw_schema());
    for (source, raw, expected) in [
        (
            "UNKNOWN",
            json!({"field":"VALUE"}),
            FieldReferenceError::SourceTraceMissing,
        ),
        (
            "S",
            json!({"field":"UNKNOWN"}),
            FieldReferenceError::FieldMissing,
        ),
        (
            "S",
            json!({"trace":"T","field":"VALUE","join":"UNKNOWN"}),
            FieldReferenceError::JoinMissing,
        ),
        (
            "T",
            json!({"trace":"T","field":"VALUE","join":"S_T"}),
            FieldReferenceError::JoinSourceMismatch,
        ),
        (
            "S",
            json!({"trace":"UNKNOWN","field":"VALUE","join":"S_T"}),
            FieldReferenceError::TargetTraceMissing,
        ),
        (
            "S",
            json!({"trace":"U","field":"VALUE","join":"S_T"}),
            FieldReferenceError::JoinTargetMismatch,
        ),
        (
            "S",
            json!({"trace":"T","field":"UNKNOWN","join":"S_T"}),
            FieldReferenceError::FieldMissing,
        ),
    ] {
        assert_eq!(
            resolve_field_reference(
                &serde_json::from_value(raw).unwrap(),
                &TraceId(source.into()),
                &flow
            ),
            Err(expected)
        );
    }
    let mut raw = raw_schema();
    raw["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!("joins.sets.v1"));
    raw["traces"][0]["joins"][0]["cardinality"] = json!("zero_or_many");
    let set_flow = compile(raw);
    let r: RawRef =
        serde_json::from_value(json!({"trace":"T","field":"VALUE","join":"S_T"})).unwrap();
    assert_eq!(
        resolve_field_reference(&r, &TraceId("S".into()), &set_flow),
        Err(FieldReferenceError::ScalarRequiresSingleCardinality)
    );
}

#[test]
fn self_join_and_inconsistent_direct_refs_never_choose_arbitrary_data() {
    let mut raw = raw_schema();
    raw["traces"][0]["joins"][0]["target_trace"] = json!("S");
    let flow = compile(raw);
    let source = project(&flow, "S", b"00AA05Ab  ZZ", 1);
    let mut store = InMemoryContextStore::new();
    store.insert(source.clone()).unwrap();
    let params = parameters();
    let traces = [TraceId("S".into())];
    let ctx = ProjectedEvaluationContext::new(
        &flow,
        &source,
        &store,
        CrossRecordInputs {
            available_traces: &traces,
            parameters: &params,
        },
    )
    .unwrap();
    let r = reference(&flow, "S", "VALUE");
    assert_eq!(ctx.resolve_field(&r), ResolvedFieldValue::Present(b"05"));
    let mut bad = r.clone();
    if let ResolvedFieldRef::External { join_id, .. } = &mut bad {
        *join_id = JoinId("WRONG".into());
    }
    assert_eq!(
        ctx.resolve_field(&bad),
        ResolvedFieldValue::Unavailable(UnknownReason::UnsupportedPlan)
    );
    let mut bad = r;
    if let ResolvedFieldRef::External { field_index, .. } = &mut bad {
        *field_index = usize::MAX;
    }
    assert_eq!(
        ctx.resolve_field(&bad),
        ResolvedFieldValue::Unavailable(UnknownReason::FieldUnavailable)
    );
    let mut invalid = source.clone();
    invalid.join_keys.clear();
    assert!(matches!(
        ProjectedEvaluationContext::new(
            &flow,
            &invalid,
            &store,
            CrossRecordInputs {
                available_traces: &traces,
                parameters: &params
            }
        ),
        Err(RelatedContextError::InvalidJoinBinding)
    ));
}

#[test]
fn unsupported_consensus_is_an_explicit_preflight_error_not_a_false_negative() {
    let mut raw = raw_schema();
    raw["required_capabilities"]
        .as_array_mut()
        .unwrap()
        .extend([json!("joins.consensus.v1")]);
    raw["traces"][0]["joins"][0]["reduction"] = json!({"kind":"consensus","fields":["VALUE"],"include_if":{"const":true},"on_conflict":"unknown","diagnostic":{"code":"SYNTHETIC_CONFLICT","message_template":"Synthetic conflict"}});
    let flow = compile(raw);
    let source = project(&flow, "S", b"00AA05Ab  ZZ", 1);
    let params = parameters();
    let traces = [TraceId("S".into()), TraceId("T".into())];
    let store = CountingStore {
        inner: InMemoryContextStore::new(),
        calls: Cell::new(0),
        fail: true,
    };
    assert!(matches!(
        ProjectedEvaluationContext::new(
            &flow,
            &source,
            &store,
            CrossRecordInputs {
                available_traces: &traces,
                parameters: &params
            }
        ),
        Err(RelatedContextError::UnsupportedReduction)
    ));
    assert_eq!(store.calls.get(), 0);
    let r: RawRef =
        serde_json::from_value(json!({"trace":"T","field":"VALUE","join":"S_T"})).unwrap();
    assert_eq!(
        resolve_field_reference(&r, &TraceId("S".into()), &flow),
        Err(FieldReferenceError::UnsupportedReduction)
    );
}
