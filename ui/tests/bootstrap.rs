//! Native headless software pixels; one platform/test per process.
use lintra_ui::{MainWindow, Tok};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PointerEventButton, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, PhysicalSize, Rgb8Pixel, SharedPixelBuffer};
use std::{fs, path::Path, rc::Rc};

struct Headless(Rc<MinimalSoftwareWindow>);
impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
    fn duration_since_start(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}
fn click(window: &MinimalSoftwareWindow, x: f32, y: f32) {
    window.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, y),
    });
    for event in [
        WindowEvent::PointerPressed {
            position: LogicalPosition::new(x, y),
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, y),
            button: PointerEventButton::Left,
        },
    ] {
        window.dispatch_event(event);
    }
}
fn capture(
    window: &MinimalSoftwareWindow,
    output: &Path,
    name: &str,
    scale: u32,
    background: [u8; 3],
) {
    let (width, height) = (1280 * scale, 780 * scale);
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    slint::platform::update_timers_and_animations();
    window.request_redraw();
    assert!(window.draw_if_needed(|renderer| {
        renderer.set_repaint_buffer_type(RepaintBufferType::NewBuffer);
        renderer.render(buffer.make_mut_slice(), width as usize);
    }));
    let bytes = buffer.as_bytes();
    let index = ((400 * scale * width + 100 * scale) * 3) as usize;
    assert_eq!(&bytes[index..index + 3], background);
    fs::write(output.join(format!("{name}-{scale}x.rgb")), bytes).unwrap();
    let mut writer = png::Encoder::new(
        fs::File::create(output.join(format!("{name}-{scale}x.png"))).unwrap(),
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
fn contrast(a: slint::Color, b: slint::Color) -> f64 {
    let luminance = |color: slint::Color| {
        let channel = |byte: u8| {
            let x = f64::from(byte) / 255.0;
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.red())
            + 0.7152 * channel(color.green())
            + 0.0722 * channel(color.blue())
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
#[test]
fn native_software_bootstrap_has_real_theme_controls_fonts_and_attribution() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    let ui = MainWindow::new().unwrap();
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.show().unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/ui-bootstrap-visual");
    fs::create_dir_all(&output).unwrap();
    assert!(!ui.get_dark_mode());
    let tokens = ui.global::<Tok>();
    assert_eq!(tokens.get_sans().as_str(), "IBM Plex Sans");
    assert_eq!(tokens.get_mono().as_str(), "IBM Plex Mono");
    let mut measurements =
        String::from("scale\ttheme\tbody_contrast\tmuted_contrast\tselected_contrast\n");
    for scale in [1u32, 2] {
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32,
        });
        window.set_size(PhysicalSize::new(1280 * scale, 780 * scale));
        // Coordinates are logical: header controls stay at their token positions
        // across DPI scales. Use actual pointer events rather than setting a theme
        // and claiming the selector was exercised.
        let dark_x = 1280.0 - 24.0 - 120.0 - 12.0 - 40.0;
        let light_x = dark_x - 80.0 - 12.0;
        for (dark, name, color, x) in [
            (false, "light", [244, 246, 248], light_x),
            (true, "dark", [11, 13, 15], dark_x),
        ] {
            click(&window, x, 28.0);
            assert_eq!(ui.get_dark_mode(), dark);
            let ratios = [
                contrast(tokens.get_ink(), tokens.get_bg()),
                contrast(tokens.get_muted(), tokens.get_bg()),
                contrast(tokens.get_accent(), tokens.get_accent_tint()),
            ];
            assert!(
                ratios.iter().all(|ratio| *ratio >= 4.5),
                "{name}: {ratios:?}"
            );
            measurements.push_str(&format!(
                "{scale}\t{name}\t{}\t{}\t{}\n",
                ratios[0], ratios[1], ratios[2]
            ));
            capture(&window, &output, name, scale, color);
        }
        click(&window, light_x, 28.0); // focus the light choice
        ui.set_dark_mode(true);
        window.dispatch_event(WindowEvent::KeyPressed { text: " ".into() });
        window.dispatch_event(WindowEvent::KeyReleased { text: " ".into() });
        assert!(
            !ui.get_dark_mode(),
            "focused control must activate from keyboard"
        );
        for text in ["\t", " "] {
            window.dispatch_event(WindowEvent::KeyPressed { text: text.into() });
            window.dispatch_event(WindowEvent::KeyReleased { text: text.into() });
        }
        assert!(ui.get_dark_mode(), "Tab must reach the next theme choice");
        click(&window, 1280.0 - 24.0 - 60.0, 28.0);
        assert!(
            ui.get_show_info(),
            "top-level attribution screen must be reachable"
        );
        // Instantiate/render the actual AboutSlint resource in both palettes.
        for dark in [false, true] {
            ui.set_dark_mode(dark);
            slint::platform::update_timers_and_animations();
            window.request_redraw();
            let width = 1280 * scale;
            let mut about = SharedPixelBuffer::<Rgb8Pixel>::new(width, 780 * scale);
            assert!(window.draw_if_needed(|r| {
                r.render(about.make_mut_slice(), width as usize);
            }));
            let index = ((100 * scale * width + 410 * scale) * 3) as usize;
            let surface = if dark { [20, 24, 28] } else { [255, 255, 255] };
            assert_eq!(&about.as_bytes()[index..index + 3], surface);
        }
        ui.set_show_info(false);
        ui.set_dark_mode(false);
    }
    // Smallest supported viewport: controls remain reachable after native resize.
    window.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    window.set_size(PhysicalSize::new(640, 480));
    slint::platform::update_timers_and_animations();
    let mut small = SharedPixelBuffer::<Rgb8Pixel>::new(640, 480);
    window.request_redraw();
    assert!(window.draw_if_needed(|r| {
        r.render(small.make_mut_slice(), 640);
    }));
    click(&window, 640.0 - 24.0 - 120.0 - 12.0 - 40.0, 28.0);
    assert!(
        ui.get_dark_mode(),
        "theme control must remain reachable at minimum size"
    );
    fs::write(output.join("contrast.tsv"), measurements).unwrap();
}
