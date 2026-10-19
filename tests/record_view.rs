use lintra::{
    CompiledFlowSchema, FieldSlice, RawFlowSchema, RawResolvedEncoding, RecordView, SchemaCompiler,
};
use serde_json::json;
fn schema() -> CompiledFlowSchema {
    let mut raw:RawFlowSchema=serde_json::from_value(json!({
        "schema_format_version":1,"flow_id":"EXAMPLE-VIEW","schema_version":"v1","regulatory_version":"synthetic",
        "source":{},"encoding":{"mode":"runtime_required","position_unit":"byte"},"required_capabilities":["local.v1"],
        "parameters":[],"reference_data":[],"persistence_policy":null,
        "traces":[{"id":"T","name":"Synthetic","record_length":6,
            "fields":(0..3).map(|i|json!({"id":format!("F{i}"),"name":"Synthetic field",
                "physical":{"type":"alphanumeric","start":1+i*2,"length":2,"alignment":{"kind":"exact"},"lexical":{"kind":"encoded_text"}},
                "missing_value":{"kind":"never_missing"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]})).collect::<Vec<_>>(),
            "record_key":{"fields":["F0"],"uniqueness_scope":"trace","error_code":"E_DUPLICATE_RECORD_KEY"},
            "joins":[],"identity":[],"named_conditions":{},"derived_values":{},"repeated_groups":[],"rules":[],"documented_non_executable_rules":[]}]
    })).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
#[test]
fn field_slices_borrow_original_bytes_and_keep_padding_multibyte_nul_and_invalid_bytes() {
    let schema = schema();
    let fields = schema.traces()[0].fields();
    let raw = [b'A', b' ', 0xc3, 0xa8, 0, 0xff];
    let view = RecordView::new(&raw);
    assert_eq!(view.raw().as_ptr(), raw.as_ptr());
    for (index, spec) in fields.iter().enumerate() {
        let field = view.field(spec).unwrap();
        assert_eq!(field.as_bytes(), &raw[index * 2..index * 2 + 2]);
        assert_eq!(field.raw.as_ptr(), raw[index * 2..].as_ptr());
        assert_eq!(field.len(), 2);
        assert!(!field.is_empty());
        assert_eq!(field.is_ascii(), index == 0);
    }
    assert_eq!(view.field(&fields[0]).unwrap().raw, b"A ");
}
#[test]
fn every_outside_range_is_none_without_requiring_whole_record_length_validation() {
    let schema = schema();
    let fields = schema.traces()[0].fields();
    let raw = b"ABCDEF";
    for length in 0..=raw.len() {
        let view = RecordView::new(&raw[..length]);
        assert_eq!(view.field(&fields[0]).is_some(), length >= 2);
        assert_eq!(view.field(&fields[1]).is_some(), length >= 4);
        assert_eq!(view.field(&fields[2]).is_some(), length >= 6);
    }
    let empty = FieldSlice { raw: &[] };
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(empty.is_ascii());
}
#[test]
fn field_slice_lifetime_depends_only_on_data_not_view_or_compiled_metadata() {
    let data = b"ABCDEF";
    let field = {
        let schema = schema();
        let view = RecordView::new(data);
        view.field(&schema.traces()[0].fields()[1]).unwrap()
    };
    assert_eq!(field.as_bytes(), b"CD");
    assert_eq!(field.raw.as_ptr(), data[2..].as_ptr());
}
