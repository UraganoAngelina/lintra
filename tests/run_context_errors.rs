//! T-P2-03 deferred contract: actual typed context causes at the application boundary.
use lintra::*;
use std::{error::Error, io};

fn assert_context(error: ValidationRunError, expected: ContextError) {
    assert!(matches!(&error, ValidationRunError::Context(actual) if *actual == expected));
    assert_eq!(
        error.source().unwrap().downcast_ref::<ContextError>(),
        Some(&expected)
    );
    assert_eq!(error.to_string(), "run context storage failed");
}

#[test]
fn context_errors_keep_their_real_payload_at_each_execution_boundary() {
    let cause = ContextError::DuplicatePosition {
        trace_id: TraceId("SYNTHETIC".into()),
        line_number: 7,
    };
    assert_context(cause.clone().into(), cause.clone());
    assert_context(
        TraceValidationError::Collection(Box::new(cause.clone())).into(),
        cause.clone(),
    );
    assert_context(
        FlowValidationError::Trace(TraceValidationError::Collection(Box::new(cause.clone())))
            .into(),
        cause.clone(),
    );
    assert_context(
        FlowValidationError::Inputs(RelatedContextError::Store(cause.clone())).into(),
        cause.clone(),
    );
    assert_context(
        FlowValidationError::Cross(CrossRecordValidationError::Context(
            RelatedContextError::Store(cause.clone()),
        ))
        .into(),
        cause,
    );
}

#[test]
fn other_collection_failures_and_context_binding_errors_keep_their_categories() {
    let error: ValidationRunError = TraceValidationError::Collection(Box::new(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "synthetic collector failure",
    )))
    .into();
    assert!(matches!(&error, ValidationRunError::Trace(_)));
    let io = error.source().unwrap().source().unwrap();
    assert_eq!(
        io.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::BrokenPipe
    );
    let binding: ValidationRunError =
        FlowValidationError::Inputs(RelatedContextError::MissingNamespaceParameter).into();
    assert!(matches!(
        binding,
        ValidationRunError::Flow(FlowValidationError::Inputs(
            RelatedContextError::MissingNamespaceParameter
        ))
    ));
}
