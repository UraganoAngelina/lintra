//! Request validation uses synthetic compiled schemas and never opens a file.
use lintra::*;
use std::{error::Error, path::PathBuf};

/// The compiled schema is a real public compiler product, not a mocked plan.
fn schema() -> CompiledFlowSchema {
    let mut raw: RawFlowSchema =
        serde_json::from_str(include_str!("golden/synthetic/v1/schema.json")).unwrap();
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    SchemaCompiler.compile(raw).unwrap()
}
/// Every option and input is explicit; a nonexistent file proves no I/O occurs.
fn request(schema: &CompiledFlowSchema) -> ValidationRequest {
    ValidationRequest {
        schema_key: schema.key().clone(),
        scope: ValidationScope::Trace(TraceValidationInput {
            trace_id: schema.traces()[0].id().clone(),
            source: InputSource::Path(PathBuf::from("synthetic nonexistent à/input.txt")),
        }),
        mode: ValidationMode::LocalOnly,
        options: ValidationOptions {
            record_length_policy: RecordLengthPolicy::StrictSkipRecord,
            newline_policy: NewlinePolicy::AutoLfOrCrlf,
            max_errors: None,
        },
    }
}
/// Trace identity, path and explicit options survive unchanged and borrowed.
#[test]
fn trace_local_only_prepares_without_opening_or_rewriting_paths() {
    let schema = schema();
    for policy in [
        NewlinePolicy::AutoLfOrCrlf,
        NewlinePolicy::LfOnly,
        NewlinePolicy::CrlfOnly,
    ] {
        let mut request = request(&schema);
        request.options.newline_policy = policy;
        let plan = preflight(&request, &schema).unwrap();
        assert!(std::ptr::eq(plan.trace(), &schema.traces()[0]));
        assert_eq!(
            plan.path(),
            std::path::Path::new("synthetic nonexistent à/input.txt")
        );
        assert_eq!(plan.options(), request.options);
    }
}
/// Full is always rejected, even for a single trace with no cross dependencies.
#[test]
fn full_and_flow_capabilities_are_never_silently_downgraded() {
    let schema = schema();
    let mut request = request(&schema);
    request.mode = ValidationMode::Full;
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::UnsupportedMode(ValidationMode::Full))
    ));
    let ValidationScope::Trace(input) = request.scope else {
        panic!()
    };
    request.scope = ValidationScope::Flow(FlowValidationInput {
        traces: vec![input],
    });
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::UnsupportedMode(_))
    ));
    request.mode = ValidationMode::LocalOnly;
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::UnsupportedFlowScope)
    ));
}
/// Key mismatch and any requested max_errors bound fail before execution.
#[test]
fn schema_key_and_operational_options_are_checked() {
    let schema = schema();
    let mut request = request(&schema);
    request.schema_key.version = SchemaVersion("other".into());
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::SchemaKeyMismatch { .. })
    ));
    request.schema_key = schema.key().clone();
    for limit in [0, 1, u64::MAX] {
        request.options.max_errors = Some(limit);
        assert!(
            matches!(preflight(&request,&schema),Err(InvalidRequestError::UnsupportedMaxErrors{limit:found}) if found==limit)
        );
    }
}
/// A real path containing spaces is preserved; empty OsStr alone is forbidden.
#[test]
fn trace_and_path_failures_keep_typed_sources() {
    let schema = schema();
    let mut request = request(&schema);
    let ValidationScope::Trace(input) = &mut request.scope else {
        panic!()
    };
    input.trace_id = TraceId("absent".into());
    let error = preflight(&request, &schema).unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(
        error,
        InvalidRequestError::Input(InputPlanError::TraceNotFound(_))
    ));
    let ValidationScope::Trace(input) = &mut request.scope else {
        panic!()
    };
    input.trace_id = schema.traces()[0].id().clone();
    input.source = InputSource::Path(PathBuf::new());
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::Input(InputPlanError::EmptyPath { .. }))
    ));
    let ValidationScope::Trace(input) = &mut request.scope else {
        panic!()
    };
    input.source = InputSource::Path(" ".into());
    assert_eq!(
        preflight(&request, &schema).unwrap().path(),
        std::path::Path::new(" ")
    );
}
/// Flow structural errors are checked instead of truncating a bundle to one
/// input. Valid bundles still stop at the explicit current capability boundary.
#[test]
fn empty_and_duplicate_flow_inputs_are_rejected_deterministically() {
    let schema = schema();
    let mut request = request(&schema);
    let ValidationScope::Trace(input) = request.scope else {
        panic!()
    };
    request.scope = ValidationScope::Flow(FlowValidationInput { traces: vec![] });
    assert!(matches!(
        preflight(&request, &schema),
        Err(InvalidRequestError::EmptyFlow)
    ));
    request.scope = ValidationScope::Flow(FlowValidationInput {
        traces: vec![input.clone(), input],
    });
    for _ in 0..2 {
        assert!(matches!(
            preflight(&request, &schema),
            Err(InvalidRequestError::DuplicateTrace(_))
        ));
    }
}
#[cfg(unix)]
/// Unix non-UTF-8 paths remain Path/OsStr; preflight does not decode them.
#[test]
fn native_non_utf8_path_is_not_lossily_converted() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let schema = schema();
    let mut request = request(&schema);
    let path = PathBuf::from(OsString::from_vec(vec![b'x', 0xff]));
    let ValidationScope::Trace(input) = &mut request.scope else {
        panic!()
    };
    input.source = InputSource::Path(path.clone());
    assert_eq!(preflight(&request, &schema).unwrap().path(), path);
}
