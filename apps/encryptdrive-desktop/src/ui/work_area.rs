//! Fits the window to the monitor work area once at startup.
//!
//! Coordinates: egui reports window rects in points, which are physical desktop pixels
//! divided by `Context::pixels_per_point` (native DPI scale × zoom). Windows reports the
//! work area in physical virtual-screen pixels, which are negative on monitors left of or
//! above the primary one. Converting with the same factor egui uses avoids double scaling.

use eframe::egui::{self, Pos2, Rect, Vec2};

/// Fallback when the work area is unknown: assume a bottom taskbar of this height.
const FALLBACK_TASKBAR_ALLOWANCE: f32 = 48.0;

/// Shrinks and/or moves the window once so it lies inside the work area of the monitor
/// it opened on (taskbar excluded). It never enlarges the window, and it only runs on the
/// first frames, so a window the user later moves or resizes is left alone.
pub(crate) fn fit_window_to_monitor(ctx: &egui::Context, done: &mut bool) {
    if *done {
        return;
    }
    let (monitor, inner, outer) = ctx.input(|input| {
        let viewport = input.viewport();
        (
            viewport.monitor_size,
            viewport.inner_rect,
            viewport.outer_rect,
        )
    });
    let (Some(monitor), Some(inner), Some(outer)) = (monitor, inner, outer) else {
        return;
    };
    *done = true;
    let pixels_per_point = ctx.pixels_per_point();
    let work = native_work_area(outer, pixels_per_point).unwrap_or_else(|| {
        Rect::from_min_size(
            Pos2::ZERO,
            monitor - egui::vec2(0.0, FALLBACK_TASKBAR_ALLOWANCE),
        )
    });
    if let Some((inner_size, outer_position)) = fit_to_work_area(outer, inner.size(), work) {
        if inner_size != inner.size() {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(inner_size));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(outer_position));
    }
}

/// Converts a physical-pixel rectangle (`left, top, right, bottom`) to points.
pub(crate) fn physical_to_points(rect: [i32; 4], pixels_per_point: f32) -> Rect {
    let [left, top, right, bottom] = rect.map(|value| value as f32 / pixels_per_point);
    Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom))
}

/// Returns the inner size and outer position that keep the window inside `work`, or
/// `None` when it already fits. Sizes only shrink; the window is centred in the work area
/// when it has to shrink and otherwise only nudged inside it.
pub(crate) fn fit_to_work_area(outer: Rect, inner_size: Vec2, work: Rect) -> Option<(Vec2, Pos2)> {
    if work.width() <= 0.0 || work.height() <= 0.0 || work.contains_rect(outer) {
        return None;
    }
    let frame = (outer.size() - inner_size).max(Vec2::ZERO);
    let fitted_inner = inner_size.min((work.size() - frame).max(Vec2::ZERO));
    let fitted_outer = fitted_inner + frame;
    let position = if fitted_inner == inner_size {
        egui::pos2(
            outer
                .min
                .x
                .clamp(work.min.x, (work.max.x - fitted_outer.x).max(work.min.x)),
            outer
                .min
                .y
                .clamp(work.min.y, (work.max.y - fitted_outer.y).max(work.min.y)),
        )
    } else {
        work.center() - fitted_outer / 2.0
    };
    Some((fitted_inner, position))
}

/// Work area (in points) of the monitor that contains the window centre.
#[cfg(windows)]
fn native_work_area(outer: Rect, pixels_per_point: f32) -> Option<Rect> {
    let centre = outer.center() * pixels_per_point;
    windows::monitor_work_area(centre.x.round() as i32, centre.y.round() as i32)
        .map(|rect| physical_to_points(rect, pixels_per_point))
}

#[cfg(not(windows))]
fn native_work_area(_outer: Rect, _pixels_per_point: f32) -> Option<Rect> {
    None
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows {
    use std::ffi::c_void;

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct RectPx {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct MonitorInfo {
        size: u32,
        monitor: RectPx,
        work: RectPx,
        flags: u32,
    }

    const MONITOR_DEFAULTTONEAREST: u32 = 2;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MonitorFromPoint(point: Point, flags: u32) -> *mut c_void;
        fn GetMonitorInfoW(monitor: *mut c_void, info: *mut MonitorInfo) -> i32;
    }

    /// Work area of the monitor nearest to a physical point (read-only query; nothing in
    /// the display configuration changes).
    pub(super) fn monitor_work_area(x: i32, y: i32) -> Option<[i32; 4]> {
        let mut info = MonitorInfo {
            size: std::mem::size_of::<MonitorInfo>() as u32,
            ..MonitorInfo::default()
        };
        // SAFETY: `MonitorFromPoint` takes a POINT by value and never fails with
        // MONITOR_DEFAULTTONEAREST. `GetMonitorInfoW` writes into `info`, a live, correctly
        // sized MONITORINFO whose `cbSize` is set, and the handle is checked first.
        let ok = unsafe {
            let monitor = MonitorFromPoint(Point { x, y }, MONITOR_DEFAULTTONEAREST);
            !monitor.is_null() && GetMonitorInfoW(monitor, &mut info) != 0
        };
        let work = info.work;
        (ok && work.right > work.left && work.bottom > work.top).then_some([
            work.left,
            work.top,
            work.right,
            work.bottom,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::{fit_to_work_area, physical_to_points};
    use eframe::egui::{Rect, pos2, vec2};

    #[test]
    fn physical_work_area_converts_with_the_same_scale_as_egui() {
        // 1920×1080 at 125 % with a 60 px bottom taskbar: 1536×816 points.
        let work = physical_to_points([0, 0, 1920, 1020], 1.25);
        assert_eq!(
            work,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1536.0, 816.0))
        );
        // Secondary monitor to the left keeps its negative virtual coordinates.
        let left = physical_to_points([-2560, 0, 0, 1392], 1.0);
        assert_eq!(left.min, pos2(-2560.0, 0.0));
        // Taskbar at the top: the work area starts below it.
        assert_eq!(physical_to_points([0, 60, 1920, 1080], 1.5).min.y, 40.0);
    }

    #[test]
    fn a_window_that_fits_is_left_alone() {
        let work = Rect::from_min_size(pos2(0.0, 0.0), vec2(1536.0, 816.0));
        let outer = Rect::from_min_size(pos2(100.0, 10.0), vec2(1200.0, 792.0));
        assert_eq!(fit_to_work_area(outer, vec2(1200.0, 760.0), work), None);
    }

    #[test]
    fn an_oversized_window_shrinks_inside_the_work_area_and_never_grows() {
        let work = Rect::from_min_size(pos2(0.0, 0.0), vec2(1536.0, 816.0));
        // 1920×1080 requested; 32 points of title bar.
        let outer = Rect::from_min_size(pos2(-192.0, -148.0), vec2(1920.0, 1112.0));
        let (inner, position) = fit_to_work_area(outer, vec2(1920.0, 1080.0), work).unwrap();
        assert_eq!(inner, vec2(1536.0, 784.0));
        assert!(work.contains_rect(Rect::from_min_size(position, inner + vec2(0.0, 32.0))));
        // Only the axis that overflows shrinks.
        let tall = Rect::from_min_size(pos2(100.0, 0.0), vec2(1180.0, 824.0));
        let (inner, _) = fit_to_work_area(tall, vec2(1180.0, 792.0), work).unwrap();
        assert_eq!(inner, vec2(1180.0, 784.0));
    }

    #[test]
    fn a_window_partly_behind_the_taskbar_moves_without_resizing() {
        // Top taskbar: work area starts at y = 40.
        let work = Rect::from_min_max(pos2(0.0, 40.0), pos2(1536.0, 864.0));
        let outer = Rect::from_min_size(pos2(168.0, 20.0), vec2(1200.0, 792.0));
        let (inner, position) = fit_to_work_area(outer, vec2(1200.0, 760.0), work).unwrap();
        assert_eq!(inner, vec2(1200.0, 760.0));
        assert_eq!(position, pos2(168.0, 40.0));
    }

    #[test]
    fn windows_on_a_secondary_monitor_stay_on_that_monitor() {
        let work = Rect::from_min_max(pos2(-1920.0, 0.0), pos2(0.0, 1032.0));
        let outer = Rect::from_min_size(pos2(-400.0, 100.0), vec2(1200.0, 792.0));
        let (_, position) = fit_to_work_area(outer, vec2(1200.0, 760.0), work).unwrap();
        assert_eq!(position, pos2(-1200.0, 100.0));
    }

    /// Read-only query of the real monitor; a session without a display may return None.
    #[cfg(windows)]
    #[test]
    fn native_work_area_query_is_well_formed() {
        if let Some([left, top, right, bottom]) = super::windows::monitor_work_area(0, 0) {
            println!("primary work area (physical px): {left},{top} - {right},{bottom}");
            assert!(right > left && bottom > top);
        }
    }

    #[test]
    fn a_degenerate_work_area_changes_nothing() {
        let outer = Rect::from_min_size(pos2(0.0, 0.0), vec2(1200.0, 792.0));
        assert_eq!(
            fit_to_work_area(outer, vec2(1200.0, 760.0), Rect::NOTHING),
            None
        );
    }
}
