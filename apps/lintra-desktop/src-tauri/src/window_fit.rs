//! Keeps the main window inside the usable area of its monitor (T-MIG-P5-01).
//!
//! The configured size (1100x760 logical, minimum 640x480) is larger than the
//! work area of common high-DPI desktops: at 200% on 1920x1080 the area is
//! about 960x492 logical, so the window would open partly under the taskbar or
//! off the monitor. The Tauri brief forbids native sizes that hide commands
//! outside the monitor, so at startup the client area is shrunk to fit and, if
//! even the configured minimum cannot fit, the minimum is lowered to what the
//! monitor can show. Layout below the minimum stays usable: the shell reflows
//! and the page scrolls. Applied on Windows only (see main.rs).

#[cfg(target_os = "windows")]
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

/// Size and position changes needed to fit a window, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fit {
    pub inner: (u32, u32),
    pub min_inner: (u32, u32),
    pub position: (i32, i32),
}

/// Current window geometry and monitor work area, in physical pixels.
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub outer_position: (i32, i32),
    pub outer: (u32, u32),
    pub inner: (u32, u32),
    pub min_inner: (u32, u32),
    pub area_position: (i32, i32),
    pub area: (u32, u32),
}

/// Returns the changes that make the outer window fit the work area, or
/// `None` when it already fits or the area is unusable (no monitor data).
/// The decoration size (outer minus inner) is preserved; sizes never grow.
pub fn fit_to_work_area(g: Geometry) -> Option<Fit> {
    let frame = (
        g.outer.0.saturating_sub(g.inner.0),
        g.outer.1.saturating_sub(g.inner.1),
    );
    let max_inner = (
        g.area.0.saturating_sub(frame.0),
        g.area.1.saturating_sub(frame.1),
    );
    if max_inner.0 == 0 || max_inner.1 == 0 {
        return None;
    }
    let inner = (g.inner.0.min(max_inner.0), g.inner.1.min(max_inner.1));
    let min_inner = (
        g.min_inner.0.min(max_inner.0),
        g.min_inner.1.min(max_inner.1),
    );
    let outer = (inner.0 + frame.0, inner.1 + frame.1);
    // Keep the position when the window fits there; otherwise clamp it into
    // the area. i64 avoids overflow on extreme virtual-desktop coordinates.
    let clamp = |pos: i32, len: u32, start: i32, span: u32| -> i32 {
        let last = i64::from(start) + i64::from(span) - i64::from(len);
        let value = i64::from(pos).clamp(i64::from(start), last.max(i64::from(start)));
        value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    };
    let position = (
        clamp(g.outer_position.0, outer.0, g.area_position.0, g.area.0),
        clamp(g.outer_position.1, outer.1, g.area_position.1, g.area.1),
    );
    let fit = Fit {
        inner,
        min_inner,
        position,
    };
    let unchanged = inner == g.inner && min_inner == g.min_inner && position == g.outer_position;
    (!unchanged).then_some(fit)
}

/// Applies [`fit_to_work_area`] to the built main window. `min_logical` is
/// the configured minimum inner size in logical pixels.
#[cfg(target_os = "windows")]
pub fn fit_window(window: &WebviewWindow, min_logical: (f64, f64)) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let scale = monitor.scale_factor();
    let (outer_position, outer, inner) = (
        window.outer_position()?,
        window.outer_size()?,
        window.inner_size()?,
    );
    let area = monitor.work_area();
    let geometry = Geometry {
        outer_position: (outer_position.x, outer_position.y),
        outer: (outer.width, outer.height),
        inner: (inner.width, inner.height),
        min_inner: (
            (min_logical.0 * scale).round() as u32,
            (min_logical.1 * scale).round() as u32,
        ),
        area_position: (area.position.x, area.position.y),
        area: (area.size.width, area.size.height),
    };
    let Some(fit) = fit_to_work_area(geometry) else {
        return Ok(());
    };
    // Lower the minimum first: the platform clamps set_size to it.
    window.set_min_size(Some(PhysicalSize::new(fit.min_inner.0, fit.min_inner.1)))?;
    window.set_size(PhysicalSize::new(fit.inner.0, fit.inner.1))?;
    window.set_position(PhysicalPosition::new(fit.position.0, fit.position.1))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geometry() -> Geometry {
        // 1100x760 logical at 200% with a 16x78 physical frame, on 1920x1080
        // with a 96 px taskbar: the observed Windows 11 VM case.
        Geometry {
            outer_position: (0, 0),
            outer: (2216, 1598),
            inner: (2200, 1520),
            min_inner: (1280, 960),
            area_position: (0, 0),
            area: (1920, 984),
        }
    }

    #[test]
    fn a_window_larger_than_the_work_area_shrinks_and_lowers_the_minimum() {
        let fit = fit_to_work_area(geometry()).expect("must fit");
        assert_eq!(fit.inner, (1904, 906));
        assert_eq!(fit.min_inner, (1280, 906));
        assert_eq!(fit.position, (0, 0));
        // The resulting outer window is exactly inside the area.
        assert_eq!((fit.inner.0 + 16, fit.inner.1 + 78), (1920, 984));
    }

    #[test]
    fn a_window_that_already_fits_is_left_alone() {
        let g = Geometry {
            outer_position: (100, 50),
            outer: (1116, 798),
            inner: (1100, 760),
            min_inner: (640, 480),
            area_position: (0, 0),
            area: (1920, 1032),
        };
        assert_eq!(fit_to_work_area(g), None);
    }

    #[test]
    fn a_fitting_window_placed_outside_the_area_is_moved_back() {
        let g = Geometry {
            outer_position: (-40, 900),
            ..fit_inside(1920, 1032)
        };
        let fit = fit_to_work_area(g).expect("must move");
        assert_eq!(fit.inner, g.inner);
        assert_eq!(fit.min_inner, g.min_inner);
        assert_eq!(fit.position, (0, 1032 - 798));
    }

    #[test]
    fn work_area_offset_by_a_left_or_top_panel_is_respected() {
        let g = Geometry {
            area_position: (120, 20),
            area: (1780, 1040),
            ..geometry()
        };
        let fit = fit_to_work_area(g).expect("must fit");
        assert_eq!(fit.position, (120, 20));
        assert!(fit.inner.0 + 16 <= 1780 && fit.inner.1 + 78 <= 1040);
    }

    #[test]
    fn sizes_never_grow_and_unusable_areas_are_ignored() {
        let small = Geometry {
            outer: (700, 520),
            inner: (690, 490),
            min_inner: (640, 480),
            ..fit_inside(1920, 1032)
        };
        assert_eq!(fit_to_work_area(small), None);
        for area in [(0, 0), (16, 1000), (1000, 78), (10, 10)] {
            assert_eq!(
                fit_to_work_area(Geometry { area, ..geometry() }),
                None,
                "{area:?}"
            );
        }
    }

    #[test]
    fn extreme_coordinates_do_not_overflow() {
        let g = Geometry {
            outer_position: (i32::MAX, i32::MIN),
            area_position: (i32::MAX - 100, i32::MIN),
            ..geometry()
        };
        let fit = fit_to_work_area(g).expect("must fit");
        assert!(fit.position.0 >= i32::MAX - 100);
        assert_eq!(fit.position.1, i32::MIN);
    }

    fn fit_inside(width: u32, height: u32) -> Geometry {
        Geometry {
            outer_position: (0, 0),
            outer: (1116, 798),
            inner: (1100, 760),
            min_inner: (640, 480),
            area_position: (0, 0),
            area: (width, height),
        }
    }
}
