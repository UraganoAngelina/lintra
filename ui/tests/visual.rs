//! Production geometry against the explicitly reviewed Signature reference.
//! Historic author-approved mockup PNGs remain unchanged under docs/ui/review/.
//! Fixture text is synthetic presentation input, not a regulatory outcome.
use lintra_ui::{ErrorRow, MainWindow};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, ModelRc, PhysicalSize, Rgb8Pixel, SharedPixelBuffer, VecModel};
use std::{fs, path::Path, rc::Rc};
struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    // The presentation fixture has no asynchronous controller work. Freeze the
    // clock so cursor/animation timing cannot vary across native runner speeds.
    fn duration_since_start(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }

    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
/// Bounds are in logical pixels, with an explicit maximum channel delta.
/// Exact rectangles cover surfaces/straight edges; text permits antialiasing.
#[derive(Clone, Copy)]
struct Zone {
    name: &'static str,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    delta: u8,
}
const ZONES: &[Zone] = &[
    Zone {
        name: "header surface",
        x: 0,
        y: 0,
        w: 1280,
        h: 8,
        delta: 0,
    },
    Zone {
        name: "background",
        x: 0,
        y: 405,
        w: 20,
        h: 320,
        delta: 0,
    },
    Zone {
        name: "table upper edge",
        x: 33,
        y: 204,
        w: 796,
        h: 1,
        delta: 0,
    },
    Zone {
        name: "table left edge",
        x: 24,
        y: 214,
        w: 1,
        h: 250,
        delta: 0,
    },
    Zone {
        name: "detail upper edge",
        x: 859,
        y: 204,
        w: 390,
        h: 1,
        delta: 0,
    },
    Zone {
        name: "detail left edge",
        x: 852,
        y: 214,
        w: 1,
        h: 250,
        delta: 0,
    },
    Zone {
        name: "table right edge",
        x: 835,
        y: 214,
        w: 1,
        h: 250,
        delta: 0,
    },
    Zone {
        name: "detail right edge",
        x: 1255,
        y: 214,
        w: 1,
        h: 250,
        delta: 0,
    },
    Zone {
        name: "table rounded corner",
        x: 24,
        y: 204,
        w: 6,
        h: 6,
        delta: 0,
    },
    Zone {
        name: "detail rounded corner",
        x: 852,
        y: 204,
        w: 6,
        h: 6,
        delta: 0,
    },
    Zone {
        name: "brand text",
        x: 64,
        y: 20,
        w: 55,
        h: 25,
        delta: 16,
    },
];
fn compare(actual: &[u8], expected: &[u8], scale: usize, zone: Zone) -> Result<(), String> {
    let stride = 1280 * scale * 3;
    if actual.len() != 780 * scale * stride || expected.len() != actual.len() {
        return Err("pixel dimensions differ".into());
    }
    for y in zone.y * scale..(zone.y + zone.h) * scale {
        for x in zone.x * scale..(zone.x + zone.w) * scale {
            for c in 0..3 {
                let i = y * stride + x * 3 + c;
                if actual[i].abs_diff(expected[i]) > zone.delta {
                    return Err(format!(
                        "{}: pixel ({x},{y}) channel {c}: {} vs {}",
                        zone.name, actual[i], expected[i]
                    ));
                }
            }
        }
    }
    Ok(())
}
fn reference(path: &Path, scale: u32) -> Vec<u8> {
    let decoder = png::Decoder::new(fs::File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    assert_eq!((info.width, info.height), (1280 * scale, 780 * scale));
    match info.color_type {
        png::ColorType::Rgb => buffer[..info.buffer_size()].to_vec(),
        png::ColorType::Rgba => buffer[..info.buffer_size()]
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| p[..3].iter().copied())
            .collect(),
        other => panic!("unsupported reference pixels: {other:?}"),
    }
}
#[test]
fn production_geometry_retains_signature_zones_at_both_scales() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    let ui = MainWindow::new().unwrap();
    ui.set_show_results(true);
    ui.set_results_invalid(true);
    ui.set_results_heading("Analisi con violazioni".into());
    ui.set_results_summary("Fixture sintetica: 1 record, 1 diagnostica, tempo 0 s".into());
    ui.set_results_scope("Controlli locali — fixture visiva sintetica".into());
    let row = ErrorRow {
        trace: "T1".into(),
        line: "1".into(),
        field: "F2".into(),
        field_name: "Codice sintetico".into(),
        code: "E_INVALID_DOMAIN".into(),
        actual: "Z␠␠".into(),
        position: "4–6".into(),
        expected: "A, B".into(),
        message: "Valore fuori dominio. Slot di 3 byte; testo a sinistra, spazi a destra.".into(),
        ..Default::default()
    };
    ui.set_result_rows(ModelRc::new(VecModel::from(vec![row.clone()])));
    ui.set_result_detail(row);
    ui.set_selected_result(0);
    ui.set_domain_text("Dichiarazione sintetica: A, B. Nessun dato paziente.".into());
    ui.show().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("../target/ui-regression");
    fs::create_dir_all(&output).unwrap();
    for scale in [1u32, 2] {
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(1280 * scale, 780 * scale));
        for (dark, theme) in [(false, "light"), (true, "dark")] {
            ui.set_dark_mode(dark);
            slint::platform::update_timers_and_animations();
            let mut pixels = SharedPixelBuffer::<Rgb8Pixel>::new(1280 * scale, 780 * scale);
            window.request_redraw();
            assert!(window.draw_if_needed(|renderer| {
                renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
                renderer.render(pixels.make_mut_slice(), (1280 * scale) as usize);
            }));
            let bytes = pixels.as_bytes();
            fs::write(output.join(format!("{theme}-{scale}x.rgb")), bytes).unwrap();
            let mut encoder = png::Encoder::new(
                fs::File::create(output.join(format!("{theme}-{scale}x.png"))).unwrap(),
                1280 * scale,
                780 * scale,
            );
            encoder.set_color(png::ColorType::Rgb);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(bytes)
                .unwrap();
            if !dark {
                let expected = reference(
                    &root.join(format!("../docs/ui/signature/reference/light-{scale}x.png")),
                    scale,
                );
                for zone in ZONES {
                    compare(bytes, &expected, scale as usize, *zone).unwrap();
                }
            }
        }
    }
    // The strict geometry comparator must reject a one-channel edge regression.
    let bytes = vec![0u8; 1280 * 780 * 3];
    let mut changed = bytes.clone();
    changed[(204 * 1280 + 33) * 3] = 1;
    assert!(compare(&changed, &bytes, 1, ZONES[2]).is_err());
    assert!(compare(&bytes, &bytes, 1, ZONES[2]).is_ok());
}
