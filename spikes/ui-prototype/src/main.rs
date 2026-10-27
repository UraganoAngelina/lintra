use lintra_ui_proto::{sample_rows, ResultsWindow};
use slint::{ComponentHandle, ModelRc, VecModel};

fn main() -> Result<(), slint::PlatformError> {
    // Uso: lintra-ui-proto [numero_righe]   (default 4 = righe del mockup)
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(4);

    let ui = ResultsWindow::new()?;
    ui.set_rows(ModelRc::new(VecModel::from(sample_rows(n))));
    if n != 4 {
        ui.set_table_note(
            format!("Righe 1–{n} di {n} · l'ordine è lo stesso del file JSONL e dell'XLSX.").into(),
        );
    }
    ui.run()
}
