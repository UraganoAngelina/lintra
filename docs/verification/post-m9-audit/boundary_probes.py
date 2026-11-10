"""Reproduce bounded, synthetic operational risks against the built Rust library.

Run from the repository root after cargo test --workspace --all-features.
Creates no operational archive, key or patient data. The harmless '=1+1'
sentinel verifies CSV bytes only; it does not launch spreadsheet software.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SOURCE = r'''
use lintra::*;
use std::{io::Cursor, path::Path};
use std::os::unix::fs::PermissionsExt;
fn main() {
    let directory = std::env::args_os().nth(1).unwrap();
    let destination = Path::new(&directory).join("synthetic-report.jsonl");
    let mut pending = AtomicReportFile::create_replacing(
        &destination, PartialOutputPolicy::Delete).unwrap();
    pending.writer().write_all(b"public synthetic sentinel\n").unwrap();
    pending.complete(ReportCompletion {
        run_status: RunStatus::CompletedInvalid, truncated: false
    }).unwrap();
    let mode = std::fs::metadata(&destination).unwrap().permissions().mode() & 0o777;
    let error = ValidationError {
        code: ValidationCode::InvalidDomain,
        flow_id: FlowId("SYNTHETIC".into()),
        schema_version: SchemaVersion("v1".into()),
        trace_id: TraceId("T".into()), line_number: 1,
        record_identity: RecordIdentity::default(), field: None,
        actual: Some(DiagnosticValue { display: "=1+1".into(), raw_hex: None }),
        expected: None, context: vec![], message: "public synthetic probe".into(),
    };
    let mut csv = CsvSink::new(Vec::new()).unwrap();
    csv.on_error(&error).unwrap();
    let csv = String::from_utf8(csv.finish().unwrap()).unwrap();
    let mut reader = RecordReader::new(
        Cursor::new(vec![b'A'; 8 * 1024 * 1024]), NewlinePolicy::AutoLfOrCrlf);
    let mut buffer = Vec::new();
    let length = reader.read_next(&mut buffer).unwrap().unwrap().bytes.len();
    println!("{{\"report_mode_with_umask_022\":\"{:04o}\",\"formula_prefix_preserved\":{},\"single_record_bytes\":{},\"record_buffer_capacity\":{}}}",
        mode, csv.contains(",=1+1,"), length, buffer.capacity());
}
'''

if __name__ == "__main__":
    if os.name != "posix":
        raise SystemExit("This permissions/allocation probe is Linux-only.")
    libraries = list((ROOT / "target/debug/deps").glob("liblintra-*.rlib"))
    library = max(libraries, key=lambda path: path.stat().st_mtime_ns)
    with tempfile.TemporaryDirectory(prefix="lintra-audit-probe-") as temp:
        temp = Path(temp)
        source = temp / "probe.rs"
        source.write_text(SOURCE)
        subprocess.run([
            "rustc", "--edition=2024", "-Dwarnings", str(source), "--extern", f"lintra={library}",
            "-L", f"dependency={ROOT / 'target/debug/deps'}", "-o", str(temp / "probe"),
        ], cwd=ROOT, check=True)
        previous = os.umask(0o022)
        try:
            output = subprocess.check_output([str(temp / "probe"), str(temp)], text=True)
        finally:
            os.umask(previous)
        result = json.loads(output)
        result["library"] = library.name
        print(json.dumps(result, indent=2))
