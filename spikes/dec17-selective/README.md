# DEC-17 — isolated experiment branch

Branch: spike/dec17-selective-pass1. Product adoption is not authorized.
The production functions and public API remain unchanged. A cfg(test) child
module grants the experiment access to existing internal contracts; the small
cfg(test) filesystem bridges reuse existing private-directory/opened-file checks.
No experimental module is compiled in a normal product library/binary.

A1 is implemented first: four independent LocalOnly traces share one immutable
compiled schema. Every invocation owns its reader/buffer, validator, audit,
summary, progress digest and private canonical JSONL/summary reports. Scoped
workers are bounded to1/2/4 where available. Sequential uses the same staging
and ordered merge; neither variant keeps a global error vector or whole report
in RAM. Direct streamed byte comparison, source input hashes and independent
fixture counts verify correctness. Reports are merged in schema trace order.

Measurement includes local validation, actual input audit/read, JSONL and summary
serialization/flush, thread coordination and ordered report merge. Compilation,
fixture generation and equivalence checking are outside timing. Private staging
and final report sizes are recorded; no persistent repository is opened.
Peak requested heap is exact allocator accounting, not RSS. A joined internal
sampler records current RSS/working set every2ms through merge; it can miss a
short OS peak, so the metric is explicitly sampled. Starting RSS and additional
sampled RSS are retained, not treated as identical to heap metrics. One sampler
thread is additional to validation workers; its overhead is included for all.

Raw summaries retain real monotonic elapsed. The comparison records the exact
elapsed-only differences after the source-contract audit; it does not rewrite
files or discard other fields. Started_at uses the existing test-clock contract.
JSONL has no normalization or filtering. The benchmark version labels the
experimental commit and does not attribute parallelism to the published rc.2.

Smoke: LINTRA_DEC17_SMOKE=1 LINTRA_DEC17_A1_OUTPUT=<new absolute JSON path>
cargo test --lib --release --locked --offline selective_parallelism::a1_local_only
-- --ignored --nocapture. Full runs omit SMOKE:7workloads,3repetitions and
sequential/1/2/4 variants, rotating order. Keep other local builds/benchmarks
stopped. Smoke timing is not a performance claim. Output is exclusive/no-clobber.

A2 Full and A3 end-to-end are not implemented by A1. No conclusion about Full
speedup, max_errors, cancellation or adoption follows from these LocalOnly runs.

Instrumentation control: LINTRA_DEC17_HEAP_MODE=off disables all shared live/peak
counter writes once before fixtures/workers are created. Heap peak is then null,
not zero; RSS sampling stays active. The allocator still delegates to System
with a read-only mode-flag check, so this is a counter-disabled control rather
than an assertion of zero instrumentation overhead. Never re-enable tracking
in that process. LINTRA_DEC17_A1_CONTROL=1 selects the two500k balanced workloads
with sparse/heavy errors, still three repetitions and the same four variants.
Paired on/off comparisons use the exact same release executable and inputs.

A2/A3 is in `full.rs`; read `FULL_IMPLEMENTATION.md`, including the failed dynamic
cancellation condition, before interpreting performance. It remains `cfg(test)`
and cannot enable parallel execution in a product build. Run the ignored
`hybrid::full_benchmark` only in an isolated foreground process, with exclusive
`LINTRA_DEC17_FULL_OUTPUT` and optionally `LINTRA_DEC17_SMOKE=1`. No concurrent
build or benchmark may run during measurement.
