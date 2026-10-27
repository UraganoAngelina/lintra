//! Native software-renderer evidence, never a mockup self-approval.
//! One test per process because Slint installs its platform once.
use lintra_ui_proto::{sample_rows, ErrorRow, ResultsWindow};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{
    ComponentHandle, LogicalPosition, Model, ModelNotify, ModelRc, ModelTracker, PhysicalSize,
    Rgb8Pixel, SharedPixelBuffer,
};
use std::{
    cell::Cell,
    fs,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};

struct Headless {
    window: Rc<MinimalSoftwareWindow>,
    clock: Rc<Cell<Duration>>,
}
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.clock.get()
    }
}

/// Synthetic lazy fixture: only four seed rows exist, even when the model has
/// a million entries. Counts actual row requests, rather than inferring that
/// ListView virtualizes from its name. This is not the production JSONL model.
struct LazyRows {
    count: usize,
    seed: Vec<ErrorRow>,
    requests: Rc<Cell<usize>>,
    highest: Rc<Cell<usize>>,
    notify: ModelNotify,
}
impl Model for LazyRows {
    type Data = ErrorRow;
    fn row_count(&self) -> usize {
        self.count
    }
    fn row_data(&self, index: usize) -> Option<Self::Data> {
        if index >= self.count {
            return None;
        }
        self.requests.set(self.requests.get() + 1);
        self.highest.set(self.highest.get().max(index));
        let mut row = self.seed[index % 4].clone();
        if index >= 4 {
            row.line = (1000 + index).to_string().into();
            row.record = format!("REC-{:06}", 1000 + index).into();
        }
        Some(row)
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}

fn draw(
    window: &MinimalSoftwareWindow,
    clock: &Cell<Duration>,
    buffer: &mut SharedPixelBuffer<Rgb8Pixel>,
    width: usize,
    fresh_buffer: bool,
) {
    clock.set(clock.get() + Duration::from_millis(16));
    slint::platform::update_timers_and_animations();
    window.request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        // Resizing allocates a new backing buffer. Its initial frame must be
        // complete; incremental redraw is valid only after those pixels exist.
        renderer.set_repaint_buffer_type(if fresh_buffer {
            RepaintBufferType::NewBuffer
        } else {
            RepaintBufferType::ReusedBuffer
        });
        renderer.render(buffer.make_mut_slice(), width);
    }));
}
fn png(path: &Path, width: u32, height: u32, bytes: &[u8]) {
    let mut writer = png::Encoder::new(
        std::io::BufWriter::new(fs::File::create(path).unwrap()),
        width,
        height,
    );
    writer.set_color(png::ColorType::Rgb);
    writer.set_depth(png::BitDepth::Eight);
    writer
        .write_header()
        .unwrap()
        .write_image_data(bytes)
        .unwrap();
}

fn reference_rgb(path: &Path) -> Vec<u8> {
    let decoder = png::Decoder::new(fs::File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    assert_eq!((reader.info().width, reader.info().height), (2560, 1560));
    let mut data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut data).unwrap();
    match info.color_type {
        png::ColorType::Rgb => data[..info.buffer_size()].to_vec(),
        png::ColorType::Rgba => {
            let (pixels, remainder) = data[..info.buffer_size()].as_chunks::<4>();
            assert!(remainder.is_empty());
            pixels
                .iter()
                .flat_map(|p| {
                    assert_eq!(p[3], 255, "reference must be opaque");
                    [p[0], p[1], p[2]]
                })
                .collect()
        }
        other => panic!("unsupported reference color type: {other:?}"),
    }
}

/// Derived 1× view of the author's 2× export, not an approved app capture.
/// Integer 2×2 area averaging preserves explicit, reproducible provenance.
fn half(reference: &[u8]) -> Vec<u8> {
    let mut result = vec![0; 1280 * 780 * 3];
    for y in 0..780 {
        for x in 0..1280 {
            for c in 0..3 {
                let mut sum = 0u16;
                for dy in 0..2 {
                    for dx in 0..2 {
                        sum += u16::from(reference[((2 * y + dy) * 2560 + 2 * x + dx) * 3 + c]);
                    }
                }
                result[(y * 1280 + x) * 3 + c] = ((sum + 2) / 4) as u8;
            }
        }
    }
    result
}

/// Locate panel borders in narrow bands, excluding glyphs. This reports each
/// measured edge; no global pixel percentage can conceal a geometry mismatch.
fn geometry(reference: &[u8], actual: &[u8]) -> String {
    let anchors = [
        ("table-left", true, 600, 48),
        ("table-right", true, 600, 1648),
        ("detail-left", true, 600, 1680),
        ("detail-right", true, 600, 2512),
        ("table-top", false, 200, 248),
        ("table-bottom", false, 200, 800),
        ("detail-top", false, 1800, 248),
        ("detail-bottom", false, 1800, 794),
    ];
    let mut report =
        String::from("anchor\treference_physical_px\tactual_physical_px\tdelta_physical_px\n");
    for (name, horizontal, fixed, center) in anchors {
        let locate = |bytes: &[u8]| {
            (center - 8usize..=center + 8)
                .filter(|&variable| {
                    let (x, y) = if horizontal {
                        (variable, fixed)
                    } else {
                        (fixed, variable)
                    };
                    let pixel = &bytes[(y * 2560 + x) * 3..(y * 2560 + x) * 3 + 3];
                    pixel
                        .iter()
                        .zip([217u8, 222, 229])
                        // The supplied PNG has anti-aliased one-pixel edges:
                        // e2e6ec is the border blended with its white surface.
                        // This identifies the edge; it does not relax the
                        // position measurement or the cross-OS pixel equality.
                        .all(|(&a, b)| a.abs_diff(b) <= 15)
                })
                .min_by_key(|&p| p.abs_diff(center))
                .unwrap_or_else(|| panic!("{name}: border missing near expected location"))
        };
        let expected = locate(reference);
        let measured = locate(actual);
        assert!(
            expected.abs_diff(measured) <= 2,
            "{name}: panel edge moved more than one logical pixel from supplied mockup"
        );
        report.push_str(&format!(
            "{name}\t{expected}\t{measured}\t{}\n",
            expected.abs_diff(measured)
        ));
    }
    report
}

#[test]
fn native_captures_and_large_lazy_table_produce_measured_evidence() {
    // A persistent backing buffer models incremental software redraw. The
    // earlier NewBuffer measurements are retained as a full-redraw worst case.
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    let clock = Rc::new(Cell::new(Duration::ZERO));
    slint::platform::set_platform(Box::new(Headless {
        window: window.clone(),
        clock: clock.clone(),
    }))
    .unwrap();
    let ui = ResultsWindow::new().unwrap();
    ui.show().unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-prototype-visual");
    fs::create_dir_all(&output).unwrap();
    for name in [
        "results-1x.png",
        "results-2x.png",
        "results-1x.rgb",
        "results-2x.rgb",
        "scroll.tsv",
        "mockup-geometry.tsv",
        "mockup-derived-1x.png",
    ] {
        match fs::remove_file(output.join(name)) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => panic!("cannot clear prior owned diagnostic {name}: {e}"),
        }
    }
    let mut measurements = String::from(
        "scale\trows\trow_requests\thighest_requested_row\tframes\tmedian_ms\tp95_ms\n",
    );
    let reference = reference_rgb(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reference/results-mockup-2x.png"),
    );
    png(
        &output.join("mockup-derived-1x.png"),
        1280,
        780,
        &half(&reference),
    );
    for scale in [1u32, 2] {
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        let (width, height) = (1280 * scale, 780 * scale);
        window.set_size(PhysicalSize::new(width, height));
        let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
        for count in [4usize, 300_000, 1_000_000] {
            let requests = Rc::new(Cell::new(0));
            let highest = Rc::new(Cell::new(0));
            ui.set_rows(ModelRc::new(LazyRows {
                count,
                seed: sample_rows(4),
                requests: requests.clone(),
                highest: highest.clone(),
                notify: ModelNotify::default(),
            }));
            ui.set_selected(0);
            draw(&window, &clock, &mut buffer, width as usize, true);
            if count == 4 {
                // These are actual captures for review, not approved regression
                // baselines. Keep raw RGB so CI can compare pixels directly.
                let name = format!("results-{scale}x");
                png(
                    &output.join(format!("{name}.png")),
                    width,
                    height,
                    buffer.as_bytes(),
                );
                fs::write(output.join(format!("{name}.rgb")), buffer.as_bytes()).unwrap();
                if scale == 2 {
                    // Diagnostics, not automatic visual baseline approval.
                    fs::write(
                        output.join("mockup-geometry.tsv"),
                        geometry(&reference, buffer.as_bytes()),
                    )
                    .unwrap();
                }
            }
            let mut times = Vec::new();
            for _ in 0..30 {
                let start = Instant::now();
                window.dispatch_event(WindowEvent::PointerScrolled {
                    position: LogicalPosition::new(350., 280.),
                    delta_x: 0.,
                    delta_y: -340.,
                });
                draw(&window, &clock, &mut buffer, width as usize, false);
                times.push(start.elapsed().as_secs_f64() * 1000.);
            }
            times.sort_by(f64::total_cmp);
            let row_requests = requests.get();
            if count > 4 {
                assert!(
                    row_requests > 0 && row_requests < 5_000,
                    "model requested {row_requests} rows for {count}"
                );
                assert!(
                    highest.get() > 100,
                    "scroll events did not advance the viewport"
                );
            }
            measurements.push_str(&format!(
                "{scale}\t{count}\t{row_requests}\t{}\t30\t{:.3}\t{:.3}\n",
                highest.get(),
                times[15],
                times[28]
            ));
        }
    }
    fs::write(output.join("scroll.tsv"), measurements).unwrap();
    ui.hide().unwrap();
    // A rendered image or measured frame latency alone does not establish
    // geometric congruence, human-perceived scrolling or cross-OS identity.
}
