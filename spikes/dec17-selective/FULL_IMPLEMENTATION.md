# DEC-17 A2/A3 — implementation and research boundaries

This branch is an experiment. All entry points, helpers and selection state are
`cfg(test)`; normal libraries have the original sequential execution. No public
API, CLI/UI, dependencies, persistence format or normative algorithm changed.

`full.rs` selects a bounded number of scoped standard Rust workers through
thread-local test configuration on the caller. Historical producers, prepared
reference dependencies, configured error limits and pre-cancelled runs retain the
original sequential path. These are explicit scope limits, not evidence that
these capabilities have been parallelized. There is no parallel pass two.

Each worker owns its TraceValidator, reader, audit session, local summary,
ProjectionCollector/FlowState and PK map. The original FullPassOne is used;
LocalOnly is not substituted. PK checking follows the existing compiled policy;
no dependent rule is moved to a local stage. The original project_record retains
only the compiler's projection plan. No raw record is retained after its callback.

Each declared trace has a FIFO bounded at 64 messages. Messages own either one
real ValidationError, one progress event, or the completed trace state. Worker
assignments are round robin. The coordinator drains FIFOs in schema order and
calls the real caller's sink and progress observer synchronously. Later workers
backpressure. Capacity bounds diagnostic staging by trace count and compiled
message/record width; there is no global error Vec or worker disk spool. Completed
minimal projections still require O(n) context memory, as does sequential Full.

After consuming a trace's messages the coordinator moves its invalid-position
set, trace/key evidence and projections. InMemoryContextStore's test-only consuming
iterator first releases secondary Arc indexes and then unwraps each sole payload
owner; it does not clone projections. Insertion into the final context rebuilds
indexes with the original algorithm, in physical line order. The aggregate cost
is measured. There can be transient old/new index headers, never duplicated raw
records. All worker completions and merges precede the existing sequential
cross-record loop: a full context barrier, not a third normative pass.

The private SummaryBuilder merger accepts disjoint trace-local totals only with
no error budget, no prior failure and matching schema key; all additions check
overflow. It leaves the coordinator's Full expectation intact. Invalid positions
are moved too, so FullCrossObserver's original transition from valid to invalid
occurs once even with multiple cross violations. Limits use the unchanged
sequential path to preserve DEC-16's exact prefix; no race chooses retained errors.

The private audit adopter transfers actual slot/hash/EOF state after readers are
dropped, requiring exclusive ownership and active states. It never invents a
Complete digest, rereads input, or assembles independent metadata. Coordinator
order is schema order. This avoids the single audit mutex spanning all concurrent
source reads, a concrete bottleneck identified before implementation.

All exit paths close receivers before joining workers, releasing blocked sends.
Sink and input failures preserve application error classification. A panic is a
technical failure. Workers are scoped foreground threads and cannot outlive the
run. No worker accesses a PersistentContextRepository.

## Known failed condition: dynamic cancellation

A cancellation arriving during parallel pass one aborts the prototype with a
technical error and publishes no summary or complete context. This is deliberately
NOT claimed equivalent to sequential Cancellation/Partial. Worker speculative
counts and audits cannot simply be represented as the sequential canonical prefix.
The dedicated test records this failed adoption property, rather than blessing a
new production contract. Therefore this prototype is not eligible for adoption,
regardless of timing. A later study needs record-level acknowledged checkpoints
and bounded ordered aggregation before claiming full observable equivalence.
Pre-cancellation and configured max_errors are tested via sequential fallback.

## Measurement and equivalence

Both variants enter the real ValidationService with an uncached in-memory schema
repository and fixed injected wall clock. Schema resolution/compilation,
fingerprint, preflight, parameter preparation, input read/hash, local validation,
projection/indexing, callbacks, sequential cross validation, summary serialization
and sink finalization are inside elapsed time. Rust compilation, synthetic fixture
generation and equivalence checks are outside. Sequential mode receives no worker
staging cost. Measurements contain original unmodified summary UTF8; only real
monotonic `/elapsed` differences are allowed by the previously recorded audit.
Canonical JSONL is compared byte for byte with streaming fixed-size buffers.

Conformance separately compares every minimal projection at the pass barrier
(identity, trace/line, sorted field values and join-key evidence), as well as
summary and JSONL. The capture hook is off during benchmarks. Two trace pairs
exercise both self-joins and cross-trace joins, missing/unique/ambiguous related
records and duplicate PK evidence. Local payload errors coexist with cross errors.

The harness reports pass-one wall time including aggregate overhead, aggregate
CPU wall intervals, pass-two duration and application total. Phase measurements
are not additive CPU times: workers overlap, and preflight/finalization remain in
total. RSS is sampled every 2ms by one owned sampler, so very brief peaks can be
missed. Requested heap accounting is optional and distinct from RSS; off-mode
removes shared accounting writes but retains the readonly allocator flag check.
Linux I/O counters are actual process snapshots including sampler reads, not a
claim about uncached physical storage traffic. No worker spool exists; report
output size and fixture bytes are recorded separately. Files are synthetic,
inside the existing qualified private-directory policy. `/tmp` tmpfs was rejected
by that policy during development and was not bypassed: staging uses the qualified
workspace filesystem instead.
