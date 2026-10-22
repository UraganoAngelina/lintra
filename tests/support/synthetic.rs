//! Synthetic adapter using production LocalOnly validation and report components.
//! Profiles are explicit test fixtures, never normative acceptance.
use super::golden::{GoldenOutput, HarnessResult};
use lintra::{
    CompiledFlowSchema, InputAuditSession, JsonlSink, NewlinePolicy, RawFlowSchema,
    RawResolvedEncoding, RecordReader, RecordValidator, RunMetadata, SchemaCompiler, ScopeSummary,
    Sha256Digest, ValidationSink, ValidationSummaryBuilder, write_summary,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::BufReader,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

/// Compile the external synthetic JSON and hash its exact source bytes.
/// Encoding is an explicit test binding, never a locale/platform inference.
pub fn load_schema(path: &Path) -> HarnessResult<(CompiledFlowSchema, String)> {
    let bytes = fs::read(path)?;
    let mut raw: RawFlowSchema = serde_json::from_slice(&bytes)?;
    raw.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    Ok((
        SchemaCompiler.compile(raw)?,
        // Reuse the domain formatter: sha2's array type has no LowerHex impl.
        Sha256Digest(Sha256::digest(&bytes).into()).to_string(),
    ))
}

/// Stream input through the real audited reader and real report components.
/// A single reused record buffer preserves byte lengths; only small report
/// outputs are held in memory by this test adapter, never production code.
pub fn produce(
    directory: &Path,
    schema: &CompiledFlowSchema,
    schema_hash: &str,
) -> HarnessResult<GoldenOutput> {
    let trace = &schema.traces()[0];
    let key = schema.key();
    let mut validator = RecordValidator::new(schema, trace.id())?;
    let audits = InputAuditSession::new();
    let source = audits.open(trace.id().clone(), &directory.join("input.txt"))?;
    let mut reader = RecordReader::new(BufReader::new(source), NewlinePolicy::AutoLfOrCrlf);
    let mut builder = ValidationSummaryBuilder::new(
        key.clone(),
        ScopeSummary::Trace {
            trace_id: trace.id().clone(),
        },
    );
    let mut sink = JsonlSink::new(Vec::new());
    let mut buffer = Vec::new();
    while let Some(record) = reader.read_next(&mut buffer)? {
        validator.validate_local(record, &mut sink, &mut builder)?;
    }
    // The loop must observe real EOF before freezing the authoritative audits.
    // Closing the reader now also closes the input handle on Windows.
    drop(reader);
    let metadata = audits.finish(RunMetadata {
        validation_mode: lintra::ValidationMode::LocalOnly,
        validator_version: "golden-test".into(),
        schema_hash: schema_hash.into(),
        reference_data_versions: BTreeMap::new(),
        started_at: UNIX_EPOCH,
        inputs: vec![],
    })?;
    let mut summary = builder.finish(metadata)?;
    // Test-only fixture time, set BEFORE serialization. No report bytes are
    // parsed, masked, normalized or rewritten after encoder output is obtained.
    summary.elapsed = Duration::ZERO;
    sink.on_summary(&summary)?;
    Ok(GoldenOutput {
        errors: sink.finish()?,
        summary: write_summary(Vec::new(), &summary, UNIX_EPOCH)?,
    })
}
