//! Repeatable presentation captures of the production components. These inputs
//! are explicitly synthetic, never evidence of an executed regulatory check.
//! The same harness is used before/after so window, content and DPI agree.
use lintra_ui::{BrowserItem, ErrorRow, MainWindow};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, ModelRc, PhysicalSize, Rgb8Pixel, SharedPixelBuffer, VecModel};
use std::{fs, path::Path, rc::Rc};

struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    fn duration_since_start(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

#[test]
fn signature_production_screens_at_desktop_sizes() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    let ui = MainWindow::new().unwrap();
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_flows(ModelRc::new(VecModel::from(vec![
        "Flusso sintetico".into(),
    ])));
    ui.set_versions(ModelRc::new(VecModel::from(vec![
        "v1 / revisione 2".into(),
    ])));
    ui.set_traces(ModelRc::new(VecModel::from(vec![
        "T1 · Anagrafica sintetica".into(),
    ])));
    ui.set_bundle_ready(true);
    ui.set_worker_available(true);
    ui.set_origin_label("Bundle incorporato · fixture sintetica".into());
    ui.set_schema_label("Flusso sintetico / v1 · revisione 2".into());
    ui.set_info_schema("Schema: Flusso sintetico / v1 · revisione 2".into());
    ui.set_info_bundle("Bundle incorporato · fixture sintetica".into());
    ui.set_results_heading("Analisi con violazioni".into());
    ui.set_results_summary("Fixture sintetica: 240 record · 18 diagnostiche · durata 0 s".into());
    ui.set_results_scope("Flusso sintetico / v1 · controlli locali sul tracciato T1".into());
    ui.set_results_note(
        "Controlli locali. Questo risultato non attesta la conformità complessiva del flusso."
            .into(),
    );
    ui.set_results_invalid(true);
    let rows: Vec<_> = (0..18).map(|index| ErrorRow {
        trace: "T1".into(), line: (index * 7 + 1).to_string().into(),
        record: format!("SYN-{index:04}").into(), field: "F2".into(),
        field_name: "Codice sintetico".into(), code: "E_INVALID_DOMAIN".into(),
        position: "4–6".into(), actual: "Z␠␠".into(), expected: "A, B".into(),
        context: "SYN-0000 · slot 4–6".into(),
        message: "Valore fuori dominio. Tipo AN; larghezza 3 byte. Allineamento a sinistra, riempimento con spazi a destra.".into(),
    }).collect();
    ui.set_result_detail(rows[0].clone());
    ui.set_result_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_domain_text(
        "Dominio dichiarato: A, B. Fixture sintetica, nessun dato personale.".into(),
    );
    ui.set_selected_result(0);
    ui.set_export_run("Flusso sintetico / v1 · controlli locali · 18 diagnostiche. Report completo, indipendente dai filtri.".into());
    ui.set_export_path("/report/analisi-sintetica.xlsx".into());
    ui.set_export_note("Seleziona il formato e la destinazione del report.".into());
    ui.set_browser_directory("/fixture-sintetiche".into());
    ui.set_browser_entries(ModelRc::new(VecModel::from(vec![
        BrowserItem {
            label: "archivio-sintetico".into(),
            directory: true,
            selectable: true,
        },
        BrowserItem {
            label: "tracciato-sintetico.txt".into(),
            directory: false,
            selectable: true,
        },
    ])));
    ui.show().unwrap();
    let output = std::env::var_os("LINTRA_SIGNATURE_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/ui-signature"));
    fs::create_dir_all(&output).unwrap();
    for (width, height, scale) in [
        (1366, 768, 1),
        (1920, 1080, 1),
        (1920, 1200, 1),
        (1366, 768, 2),
        (640, 480, 1),
    ] {
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(width * scale, height * scale));
        for dark in [false, true] {
            ui.set_dark_mode(dark);
            for screen in [
                "analysis",
                "running",
                "technical",
                "results",
                "empty",
                "export",
                "information",
                "browser",
            ] {
                ui.set_export_open(screen == "export");
                ui.set_show_info(screen == "information");
                ui.set_show_results(screen == "results" || screen == "empty");
                ui.set_browser_open(screen == "browser");
                ui.set_busy(screen == "running");
                ui.set_configuration_error(screen == "technical");
                ui.set_execution_label(
                    match screen {
                        "running" => "Validazione in corso",
                        "technical" => "Errore tecnico",
                        _ => "Analisi non avviata",
                    }
                    .into(),
                );
                ui.set_execution_detail(match screen {
                    "running" => "Lettura del tracciato · 240 record elaborati. Fixture di presentazione.",
                    "technical" => "Impossibile leggere il file. Verifica il percorso e i permessi, poi ripeti la selezione.",
                    _ => "Seleziona file, flusso, versione e tracciato per preparare l’analisi.",
                }.into());
                if screen == "empty" {
                    ui.set_result_rows(ModelRc::default());
                    ui.set_selected_result(-1);
                    ui.set_results_note("Nessuna diagnostica corrisponde ai filtri. Modifica i valori e applica di nuovo i filtri.".into());
                } else if screen == "results" {
                    // Each dimension/theme starts with the same populated model.
                    let row = ui.get_result_detail();
                    ui.set_result_rows(ModelRc::new(VecModel::from(vec![row; 18])));
                    ui.set_selected_result(0);
                    ui.set_results_note("Controlli locali. Questo risultato non attesta la conformità complessiva del flusso.".into());
                }
                slint::platform::update_timers_and_animations();
                let (w, h) = (width * scale, height * scale);
                let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(w, h);
                window.request_redraw();
                assert!(window.draw_if_needed(|renderer| {
                    renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
                    renderer.render(pixels.make_mut_slice(), w as usize);
                }));
                let theme = if dark { "dark" } else { "light" };
                let file = fs::File::create(
                    output.join(format!("{screen}-{theme}-{width}x{height}-{scale}x.png")),
                )
                .unwrap();
                let mut encoder = png::Encoder::new(file, w, h);
                encoder.set_color(png::ColorType::Rgb);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(pixels.as_bytes())
                    .unwrap();
            }
        }
    }
}
