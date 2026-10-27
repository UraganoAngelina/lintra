//! Workspace boundary only. The actual Slint bootstrap belongs to T-P16-02.
//! Exit with an explicit unsupported status instead of pretending to open a UI.
fn main() -> std::process::ExitCode {
    eprintln!("Lintra UI: interfaccia desktop non ancora disponibile in questa build.");
    std::process::ExitCode::from(2)
}
