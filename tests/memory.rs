//! Requested heap bytes, not RSS. This integration executable has one test,
//! and production validation remains synchronous. No allocator dependency.
use lintra::*;
use serde_json::{Value, json};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs::File,
    io::{BufWriter, Write},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
#[path = "support/full_fixture.rs"]
mod full_fixture;
#[path = "support/full_schema.rs"]
mod full_schema;

struct Meter;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static MEASURING: AtomicBool = AtomicBool::new(false);
fn acquired(size: usize) {
    let current = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    if MEASURING.load(Ordering::Relaxed) {
        PEAK.fetch_max(current, Ordering::Relaxed);
    }
}
// SAFETY: every allocation operation is forwarded to System with its original
// layout/pointer. Only successful allocations affect atomic counters; no heap
// allocation, callback or pointer dereference occurs in the accounting itself.
unsafe impl GlobalAlloc for Meter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: caller provides GlobalAlloc's valid allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            acquired(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same layout/preconditions as the caller's zeroed allocation.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            acquired(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer/layout are forwarded unchanged to the owning allocator.
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: System receives the exact pointer/layout and requested size.
        let next = unsafe { System.realloc(pointer, layout, new_size) };
        if !next.is_null() {
            if new_size >= layout.size() {
                acquired(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Meter = Meter;

#[derive(Default)]
struct CountSink {
    errors: u64,
    summaries: u64,
}
impl ValidationSink for CountSink {
    fn on_error(&mut self, _: &ValidationError) -> Result<(), ReportError> {
        self.errors += 1;
        Ok(())
    }
    fn on_summary(&mut self, _: &ValidationSummary) -> Result<(), ReportError> {
        self.summaries += 1;
        Ok(())
    }
}
fn schema(width: usize) -> Value {
    assert!(width > 18);
    let mut value = full_schema::raw();
    for trace in value["traces"].as_array_mut().unwrap() {
        let original = trace["fields"][0].clone();
        trace["record_length"] = json!(width);
        trace["fields"] = json!(
            [
                ("PK", 1, 8),
                ("KEY", 9, 8),
                ("DATA", 17, 2),
                ("FILLER", 19, width - 18)
            ]
            .map(|(id, start, length)| {
                let mut field = original.clone();
                field["id"] = json!(id);
                field["physical"]["start"] = json!(start);
                field["physical"]["length"] = json!(length);
                field
            })
        );
    }
    value
}
fn write_records(path: &std::path::Path, records: usize, width: usize) {
    let mut output = BufWriter::new(File::create(path).unwrap());
    let mut record = vec![b' '; width];
    record[16..18].copy_from_slice(b"OK");
    for index in 0..records {
        let key = format!("{index:08}");
        record[..8].copy_from_slice(key.as_bytes());
        record[8..16].copy_from_slice(key.as_bytes());
        output.write_all(&record).unwrap();
        output.write_all(b"\n").unwrap();
    }
    output.flush().unwrap();
}
#[derive(Debug)]
struct Sample {
    records: usize,
    width: usize,
    context_bytes: usize,
    peak_bytes: usize,
}
fn sample(records: usize, width: usize) -> Sample {
    let raw = schema(width);
    // Inspect the real plan before measuring: filler must remain unprojected.
    let mut authored: RawFlowSchema = serde_json::from_value(raw.clone()).unwrap();
    authored.encoding.resolved = Some(RawResolvedEncoding::Utf8);
    let compiled = SchemaCompiler.compile(authored).unwrap();
    for trace in compiled.traces() {
        assert_eq!(trace.projection_plan().field_indexes, [0, 1, 2]);
    }
    drop(compiled);
    let fixture = full_fixture::Fixture::new(raw, b"", b"");
    write_records(&fixture.source, records, width);
    write_records(&fixture.target, records, width);
    let service = fixture.service();
    let request = fixture.request();
    let token = CancellationToken::default();
    let mut sink = CountSink::default();
    let mut context = None;
    let baseline = LIVE.load(Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let outcome = service.validate(
        request,
        &mut sink,
        &mut |event| {
            if event == ProgressEvent::PhaseStarted(ValidationPhase::CrossRecordValidation) {
                assert!(context.is_none());
                context = Some(LIVE.load(Ordering::Relaxed).saturating_sub(baseline));
            }
        },
        &token,
    );
    MEASURING.store(false, Ordering::Relaxed);
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    let outcome = outcome.unwrap();
    assert_eq!(outcome.status, RunStatus::CompletedValid);
    assert_eq!(outcome.summary.records_processed, records as u64 * 2);
    assert_eq!(outcome.summary.valid_records, records as u64 * 2);
    assert_eq!(
        outcome.summary.completeness(),
        ValidationCompleteness::Complete
    );
    assert_eq!(
        outcome.summary.full_validation.as_ref().unwrap().controls[0].satisfied,
        records as u64
    );
    assert_eq!((sink.errors, sink.summaries), (0, 1));
    Sample {
        records,
        width,
        context_bytes: context.expect("actual cross pass checkpoint"),
        peak_bytes: peak,
    }
}
#[test]
fn full_context_heap_tracks_selected_fields_and_record_count_not_raw_row_width() {
    let small = sample(1024, 32);
    let large = sample(4096, 32);
    let wide = sample(4096, 4096);
    // Fixed compilation/IO/harness overhead is allowed; a fourfold context
    // must still grow with records rather than retaining a fixed-size fake.
    assert!(
        large.context_bytes > small.context_bytes * 2,
        "{small:?} {large:?}"
    );
    assert!(
        large.context_bytes < small.context_bytes * 6 + 256 * 1024,
        "{small:?} {large:?}"
    );
    // Row width grows 128x at the same record count. After pass one the readers
    // are gone; the tolerance covers allocator/harness noise, not raw rows.
    const TOLERANCE: usize = 256 * 1024;
    assert!(
        wide.context_bytes.abs_diff(large.context_bytes) < TOLERANCE,
        "{large:?} {wide:?}"
    );
    assert!(
        wide.peak_bytes < large.peak_bytes + TOLERANCE,
        "{large:?} {wide:?}"
    );
    let raw_bytes = wide.records * 2 * wide.width;
    assert!(
        wide.context_bytes < raw_bytes / 2,
        "context retained a substantial fraction of full rows: {wide:?}"
    );
    println!("Full heap requested bytes (not RSS): {small:?}; {large:?}; {wide:?}");
}
