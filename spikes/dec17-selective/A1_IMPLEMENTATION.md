# A1 implementation and correctness — experimental branch only

No production validator body/public API/CLI/UI is changed. flow_execution.rs
includes a child module only under cfg(test). The two persistence-module test
bridges call the existing private filesystem policy; they do not access SQLite,
HMAC/key material, snapshots or repository mutation. A normal library binary
contains none of these test-only helpers or the experimental allocator.

## Data and execution

model builds one compiler-owned schema shared immutably by all variants:4traces,
ID8digits, CODE01/02 and8synthetic16-byte payloads with explicit regex constraints.
This is not a CAD/SIOC fixture. fixture writes only synthetic rows in a private
owned directory. Balanced25/25/25/25 and skewed80/10/5/5 distributions have exact
row counts. Sparse errors are every1000throw in CODE; heavy errors every4throw
also affect all8payloads. An independent count oracle checks9errors per heavy
bad record or1per sparse bad record, valid/invalid/processed and actual input
hashes. These counts are not inferred from matching the two implementations.

run_trace owns TraceValidator, reader/buffer, InputAuditSession, summary and
progress hasher. It calls the real validate_path/local pipeline and freezes real
input metadata only after observed completion. No fake Complete is constructed.
Summary uses the existing caller-supplied metadata/test-clock contract for a fixed
started_at; real monotonic elapsed is retained. The software label includes the
experimental source commit. Static generic bounds actually compile Send+Sync for
CompiledFlowSchema, PreparedReferenceData and PreparedReferenceEvidence.

Every run has canonical JSONL and summary files in the protected staging root.
Private-directory policy is the qualified persistence FS helper, reused without
changing it. Each NamedTempFile is checked against the existing file ownership/
mode/link/ACL rules. Root/report handles and files are RAII-owned; owner.close
must succeed before successful final publication of measurement JSON. This is
intended synthetic output/staging, not storage of whole medical records.

sample uses a true sequential variant or scoped standard workers. A worker owns
at most a fixed subset of the four independent traces, with no shared mutable
summary/sink/audit/reader/progress observer. Joining collects only four artifacts
and summaries, not error vectors. Results are ordered by schema trace index and
merged using bounded streaming copy. No completion-order output or global mutex
around validation. Sequential pays the same staging+merge costs for a fair A1
comparison. This does not model a single Full run or prove Full cancellation.

## Measurements and failure ownership

Meter forwards unchanged allocator arguments to System and atomically counts
successful allocations/live requested bytes. Peak additional requested heap is
relative to live fixtures/previous measurement metadata at sample entry. It is
not OS RSS, allocator overhead, reserved thread stack space or memory bandwidth.
MeterGuard disables sampling on unwind. Counter logic never allocates or reads
allocation contents. This instrumentation affects throughput equally in both
variants and is not directly comparable to older uninstrumented timings.

One owned sampler observes current RSS/working set every2ms through worker work
and the merge. Starting RSS, sampled absolute peak and sampled extra RSS are
reported separately. A brief OS peak may be missed; exact requested-heap peak
and sampled RSS are not interchangeable. StopSampler sets the stop flag during
unwinding before thread::scope joins, preventing a validation failure from
stranding a sampling thread. All worker handles belong to the same foreground
scope. Validation-worker count excludes coordinator and the one sampler.

Timing starts before thread coordination and includes actual read/audit, local
validation, JSONL/summary serialization/flush, ownership handoff and ordered
merge. Metadata readback inside run_trace is charged too. Compilation, fixture
generation, correctness comparisons and cleanup are outside the timed sample.
Staging plus final report sizes quantify extra disk storage/I/O; retained reference
reports are setup/verification costs, not hidden worker memory. No other local
build/benchmark may run during measurement.

report_digest and same_bytes read bounded64KiB chunks. No entire report is read
into RAM. Direct streamed comparison proves JSONL bytes, not merely equal hashes.
Progress digest must match per trace. compare_summary checks all keys/values
recursively; only the previously audited monotonic elapsed path may differ and
its exact before/after values are recorded. Original UTF-8 summary bytes are
stored verbatim as strings alongside parsed values; they are not rewritten to
zero, stripped, or silently normalized. Each input digest is independently
checked against its fixture file outside timing.

## Qualification state

Initial compilation found a struct-variant match typo; corrected. Two debug
smokes passed all8small samples (sequential/1/2/4 ×balanced sparse/skewed heavy),
with exact diagnostics/progress and deterministic summary fields. Clippy with
-Dwarnings passed after preserving original summary bytes. These are correctness
checks, not release performance evidence. Final release smoke and repeated
measurements must use a committed/frozen harness before A1 conclusions.

Full A2/A3 are separate sequential activities after A1. No production adoption,
new dependency, scheduling framework, Full speedup or treatment of max_errors/
cancellation is claimed by this implementation.
