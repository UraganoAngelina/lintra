use serde_json::{Value, json};
pub fn raw() -> Value {
    let mut raw: Value =
        serde_json::from_str(include_str!("../golden/synthetic/v1/schema.json")).unwrap();
    let mut trace = raw["traces"][0].clone();
    trace["record_length"] = json!(6);
    let field = trace["fields"][0].clone();
    trace["fields"] = json!(
        ["PK", "KEY", "DATA"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut field = field.clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(i * 2 + 1);
                field["physical"]["length"] = json!(2);
                field
            })
            .collect::<Vec<_>>()
    );
    trace["record_key"]["fields"] = json!(["PK"]);
    raw["traces"] = json!(["S", "T"].map(|id| {
        let mut trace = trace.clone();
        trace["id"] = json!(id);
        trace
    }));
    raw["required_capabilities"] = json!(["local.v1", "expressions.v1", "joins.v1"]);
    let policy = json!({"policy":"violation","error_code":"E_RELATED_RECORD_MISSING","diagnostic":{"code":"SYNTHETIC_MISSING","message_template":"Missing related record for {rule_id}"}});
    raw["traces"][0]["joins"] = json!([{"id":"S_T","target_trace":"T","source_fields":["KEY"],"target_fields":["KEY"],"cardinality":"zero_or_one","scope":"current_submission","namespace_parameters":[],"on_missing_record":policy,"on_missing_trace":policy}]);
    raw["traces"][0]["rules"] = json!([{"id":"COMPARE","when":{"const":true},"assertions":[{"kind":"field_compare","left":{"ref":{"field":"DATA"}},"operator":"==","right":{"ref":{"trace":"T","field":"DATA","join":"S_T"}},"value_type":"bytes"}],"error_code":"E_CROSS_FIELD","message_template":"Synthetic {rule_id}: {condition}"}]);
    raw
}
