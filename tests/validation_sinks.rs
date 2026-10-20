use lintra::{
    CompositeSink, FlowId, InvalidDiagnostic, RecordIdentity, ReportError, RunStatus, SchemaKey,
    SchemaVersion, ScopeSummary, TraceId, ValidationCode, ValidationError, ValidationSink,
    ValidationSummary, VecSink,
};
use std::{cell::RefCell, collections::BTreeMap, error::Error, io, time::Duration};

fn error() -> ValidationError {
    ValidationError {
        code: ValidationCode::RecordLength,
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T1".into()),
        line_number: 1,
        record_identity: RecordIdentity::default(),
        field: None,
        actual: None,
        expected: Some("6 byte".into()),
        context: vec![],
        message: "Synthetic test error".into(),
    }
}
fn summary() -> ValidationSummary {
    ValidationSummary {
        run_status: RunStatus::CompletedInvalid,
        schema_key: SchemaKey {
            flow_id: FlowId("SYNTHETIC".into()),
            version: SchemaVersion("v1".into()),
        },
        scope_summary: ScopeSummary::Trace {
            trace_id: TraceId("T1".into()),
        },
        records_processed: 1,
        valid_records: 0,
        invalid_records: 1,
        total_errors: 1,
        errors_by_code: BTreeMap::from([(ValidationCode::RecordLength, 1)]),
        skipped_rules: 0,
        truncated: false,
        validator_version: "synthetic-build".into(),
        schema_hash: "synthetic-hash".into(),
        reference_data_versions: BTreeMap::new(),
        elapsed: Duration::from_millis(3),
    }
}

#[test]
fn vec_sink_keeps_owned_errors_and_summaries_in_callback_order() {
    let mut sink = VecSink::default();
    let mut first = error();
    sink.on_error(&first).unwrap();
    first.line_number = 2;
    sink.on_error(&first).unwrap();
    first.line_number = 99;
    let mut result = summary();
    sink.on_summary(&result).unwrap();
    result.run_status = RunStatus::Cancelled;
    sink.on_summary(&result).unwrap();
    result.total_errors = 99;
    assert_eq!(
        sink.errors()
            .iter()
            .map(|e| e.line_number)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(sink.summaries()[0], summary());
    assert_eq!(sink.summaries()[1].run_status, RunStatus::Cancelled);
    assert_eq!(sink.summaries()[1].total_errors, 1);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Event {
    Error,
    Summary,
}
struct Spy<'a> {
    index: usize,
    calls: &'a RefCell<Vec<(usize, Event)>>,
    fail_on: Option<Event>,
}
impl Spy<'_> {
    fn record(&self, event: Event) -> Result<(), ReportError> {
        self.calls.borrow_mut().push((self.index, event));
        if self.fail_on == Some(event) {
            Err(ReportError::Write(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "synthetic sink failure",
            )))
        } else {
            Ok(())
        }
    }
}
impl ValidationSink for Spy<'_> {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.record(Event::Error)
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        self.record(Event::Summary)
    }
}

#[test]
fn composite_delivers_both_callbacks_in_configured_order_with_borrowed_sinks() {
    let calls = RefCell::new(Vec::new());
    let sinks = (0..3)
        .map(|index| {
            Box::new(Spy {
                index,
                calls: &calls,
                fail_on: None,
            }) as Box<dyn ValidationSink>
        })
        .collect();
    let mut composite = CompositeSink::new(sinks);
    composite.on_error(&error()).unwrap();
    composite.on_summary(&summary()).unwrap();
    assert_eq!(
        *calls.borrow(),
        [
            (0, Event::Error),
            (1, Event::Error),
            (2, Event::Error),
            (0, Event::Summary),
            (1, Event::Summary),
            (2, Event::Summary)
        ]
    );
}

#[test]
fn either_callback_failure_at_each_position_stops_fanout_and_all_later_callbacks() {
    for event in [Event::Error, Event::Summary] {
        for failing_index in 0..3 {
            let calls = RefCell::new(Vec::new());
            let sinks = (0..3)
                .map(|index| {
                    Box::new(Spy {
                        index,
                        calls: &calls,
                        fail_on: (index == failing_index).then_some(event),
                    }) as Box<dyn ValidationSink>
                })
                .collect();
            let mut composite = CompositeSink::new(sinks);
            let failure = match event {
                Event::Error => composite.on_error(&error()),
                Event::Summary => composite.on_summary(&summary()),
            }
            .unwrap_err();
            assert!(matches!(failure, ReportError::Write(_)));
            assert_eq!(
                failure
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap()
                    .kind(),
                io::ErrorKind::PermissionDenied
            );
            let expected: Vec<_> = (0..=failing_index).map(|index| (index, event)).collect();
            assert_eq!(*calls.borrow(), expected);
            assert!(matches!(
                composite.on_error(&error()),
                Err(ReportError::PreviousFailure)
            ));
            assert!(matches!(
                composite.on_summary(&summary()),
                Err(ReportError::PreviousFailure)
            ));
            assert_eq!(*calls.borrow(), expected);
        }
    }
}

#[test]
fn malformed_diagnostics_fail_before_any_output_and_are_not_normative_errors() {
    let mut invalid = error();
    invalid.line_number = 0;
    let mut sink = VecSink::default();
    let failure = sink.on_error(&invalid).unwrap_err();
    assert!(matches!(
        failure,
        ReportError::InvalidDiagnostic(InvalidDiagnostic::ZeroLineNumber)
    ));
    assert_eq!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<InvalidDiagnostic>(),
        Some(&InvalidDiagnostic::ZeroLineNumber)
    );
    assert!(sink.errors().is_empty());
    assert!(matches!(
        sink.on_summary(&summary()),
        Err(ReportError::PreviousFailure)
    ));
    let calls = RefCell::new(Vec::new());
    let mut composite = CompositeSink::new(vec![Box::new(Spy {
        index: 0,
        calls: &calls,
        fail_on: None,
    })]);
    assert!(matches!(
        composite.on_error(&invalid),
        Err(ReportError::InvalidDiagnostic(_))
    ));
    assert!(calls.borrow().is_empty());
}

#[test]
fn empty_composite_is_an_explicit_noop_without_inventing_output() {
    let mut composite = CompositeSink::new(vec![]);
    composite.on_error(&error()).unwrap();
    composite.on_summary(&summary()).unwrap();
}
