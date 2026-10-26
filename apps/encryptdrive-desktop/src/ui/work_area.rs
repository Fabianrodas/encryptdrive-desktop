//! Window sizing against the monitor work area (the monitor minus the taskbar).
//!
//! Units:
//! - Physical pixels: what Windows reports (`rcWork`, window rects). Virtual-screen
//!   coordinates, negative on monitors left of or above the primary one.
//! - Points: egui/winit logical units, physical pixels ÷ scale (DPI / 96, times egui zoom).
//! - Inner size: the client area egui draws into. Outer size: inner plus decorations
//!   (title bar and the invisible resize borders, which Windows counts in window rects).
//!
//! Policy, applied twice:
//! 1. Before the window exists (eframe `window_builder` hook): size, minimum and position
//!    from the primary monitor's work area, DPI and decoration size.
//! 2. On the first frame: the same rules from the real window geometry. This also covers
//!    windows that opened elsewhere, and systems where step 1 had no data.
//!
//! The minimum is `min(880×600, usable inner area)`. A fixed 880×600 minimum is larger
//! than the work area at high scaling (for example 1920×1080 at 200 % leaves about
//! 960×478 points), and Windows enforces the minimum, which would push the window behind
//! the taskbar. The window is never sized or positioned again after startup.

use eframe::egui::{self, Pos2, Rect, Vec2, ViewportBuilder};

/// Inner size the window asks for when there is room.
pub(crate) const PREFERRED_INNER_SIZE: Vec2 = Vec2::new(1200.0, 760.0);
/// Minimum inner size when there is room; smaller work areas lower it (see module docs).
pub(crate) const PREFERRED_MIN_INNER_SIZE: Vec2 = Vec2::new(880.0, 600.0);
/// Fallback when the work area is unknown: assume a bottom taskbar of this height.
const FALLBACK_TASKBAR_ALLOWANCE: f32 = 48.0;

/// Startup geometry in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WindowPlan {
    pub(crate) inner: Vec2,
    pub(crate) min_inner: Vec2,
    pub(crate) outer_position: Pos2,
}

/// Inner area that fits in `work` once the decorations (`frame`) are added.
fn usable_inner(work: Rect, frame: Vec2) -> Vec2 {
    (work.size() - frame).max(Vec2::ZERO)
}

/// Minimum inner size compatible with the work area.
pub(crate) fn minimum_inner_size(work: Rect, frame: Vec2) -> Vec2 {
    PREFERRED_MIN_INNER_SIZE.min(usable_inner(work, frame))
}

/// Startup plan: the desired size, raised to the minimum and capped by the work area,
/// centred in the work area. Every value satisfies `min_inner <= inner <= usable`.
pub(crate) fn plan_startup(work: Rect, frame: Vec2, desired_inner: Vec2) -> WindowPlan {
    let min_inner = minimum_inner_size(work, frame);
    let inner = desired_inner.max(min_inner).min(usable_inner(work, frame));
    let outer = inner + frame;
    WindowPlan {
        inner,
        min_inner,
        outer_position: (work.center() - outer / 2.0).floor(),
    }
}

/// eframe `window_builder` hook: applies [`plan_startup`] for the primary monitor (where
/// eframe opens the window). Without native data the builder is returned unchanged and
/// the first-frame fit does the work.
pub(crate) fn startup_viewport(builder: ViewportBuilder, desired_inner: Vec2) -> ViewportBuilder {
    let Some((work, frame)) = native_primary_metrics() else {
        return builder;
    };
    let plan = plan_startup(work, frame, desired_inner);
    builder
        .with_inner_size(plan.inner)
        .with_min_inner_size(plan.min_inner)
        .with_position(plan.outer_position)
}

/// First-frame fit, run once: sets the compatible minimum first (so Windows cannot hold
/// the window at a larger one), then shrinks and/or moves the window only if it is not
/// inside the work area of the monitor it is on. It never enlarges the window, and a
/// window the user moves or resizes later is left alone.
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
    let frame = (outer.size() - inner.size()).max(Vec2::ZERO);
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(minimum_inner_size(
        work, frame,
    )));
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
    let fitted_inner = inner_size.min(usable_inner(work, frame));
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

/// Primary monitor work area and standard window decoration size, both in points.
/// Only meaningful once the process is DPI aware, which winit makes it when the event
/// loop is created (before eframe calls the `window_builder` hook).
#[cfg(windows)]
fn native_primary_metrics() -> Option<(Rect, Vec2)> {
    let (work, dpi) = windows::primary_work_area_and_dpi()?;
    let [left, top, right, bottom] = windows::window_frame(dpi)?;
    let scale = dpi as f32 / 96.0;
    let frame = egui::vec2((right - left) as f32, (bottom - top) as f32) / scale;
    Some((physical_to_points(work, scale), frame))
}

#[cfg(not(windows))]
fn native_primary_metrics() -> Option<(Rect, Vec2)> {
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

    const MONITOR_DEFAULTTOPRIMARY: u32 = 1;
    const MONITOR_DEFAULTTONEAREST: u32 = 2;
    const MDT_EFFECTIVE_DPI: i32 = 0;
    /// `WS_OVERLAPPEDWINDOW`: caption, system menu, thick frame, minimize and maximize,
    /// the decorations winit uses for a normal resizable window.
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MonitorFromPoint(point: Point, flags: u32) -> *mut c_void;
        fn GetMonitorInfoW(monitor: *mut c_void, info: *mut MonitorInfo) -> i32;
        fn AdjustWindowRectExForDpi(
            rect: *mut RectPx,
            style: u32,
            menu: i32,
            ex_style: u32,
            dpi: u32,
        ) -> i32;
    }

    #[link(name = "shcore")]
    unsafe extern "system" {
        fn GetDpiForMonitor(
            monitor: *mut c_void,
            kind: i32,
            dpi_x: *mut u32,
            dpi_y: *mut u32,
        ) -> i32;
    }

    fn work_area(monitor: *mut c_void) -> Option<[i32; 4]> {
        if monitor.is_null() {
            return None;
        }
        let mut info = MonitorInfo {
            size: std::mem::size_of::<MonitorInfo>() as u32,
            ..MonitorInfo::default()
        };
        // SAFETY: `monitor` is a non-null HMONITOR from MonitorFromPoint, and `info` is a
        // live, correctly sized MONITORINFO with `cbSize` set.
        let ok = unsafe { GetMonitorInfoW(monitor, &mut info) != 0 };
        let work = info.work;
        (ok && work.right > work.left && work.bottom > work.top).then_some([
            work.left,
            work.top,
            work.right,
            work.bottom,
        ])
    }

    /// Work area of the monitor nearest to a physical point (read-only query; nothing in
    /// the display configuration changes).
    pub(super) fn monitor_work_area(x: i32, y: i32) -> Option<[i32; 4]> {
        // SAFETY: MonitorFromPoint takes a POINT by value and has no other preconditions.
        work_area(unsafe { MonitorFromPoint(Point { x, y }, MONITOR_DEFAULTTONEAREST) })
    }

    /// Primary monitor work area (physical pixels) and its effective DPI.
    pub(super) fn primary_work_area_and_dpi() -> Option<([i32; 4], u32)> {
        // SAFETY: as above; (0, 0) with MONITOR_DEFAULTTOPRIMARY selects the primary monitor.
        let monitor = unsafe { MonitorFromPoint(Point { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
        let work = work_area(monitor)?;
        let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
        // SAFETY: `monitor` was validated by `work_area`; both out-pointers are live locals.
        let hr = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
        (hr >= 0 && dpi_y >= 96 / 2).then_some((work, dpi_y))
    }

    /// Decoration extents (left, top, right, bottom offsets from the client rect) for a
    /// standard window at `dpi`, in physical pixels.
    pub(super) fn window_frame(dpi: u32) -> Option<[i32; 4]> {
        let mut rect = RectPx::default();
        // SAFETY: `rect` is a live RECT; the call only reads the style values and writes it.
        let ok = unsafe { AdjustWindowRectExForDpi(&mut rect, WS_OVERLAPPEDWINDOW, 0, 0, dpi) };
        (ok != 0).then_some([rect.left, rect.top, rect.right, rect.bottom])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PREFERRED_INNER_SIZE, PREFERRED_MIN_INNER_SIZE, WindowPlan, fit_to_work_area,
        physical_to_points, plan_startup,
    };
    use eframe::egui::{Rect, Vec2, pos2, vec2};

    /// Decorations measured with `AdjustWindowRectExForDpi(WS_OVERLAPPEDWINDOW)` on this
    /// Windows 11 machine, in physical pixels: (width, height) at 96, 120, 144 and 192 DPI.
    fn frame_points(scale: f32) -> Vec2 {
        let physical = match (scale * 100.0) as u32 {
            100 => vec2(16.0, 39.0),
            125 => vec2(18.0, 47.0),
            150 => vec2(22.0, 56.0),
            _ => vec2(26.0, 71.0),
        };
        physical / scale
    }

    /// Checks every invariant of a plan against its work area.
    fn assert_plan_is_coherent(plan: WindowPlan, work: Rect, frame: Vec2, label: &str) {
        let outer = Rect::from_min_size(plan.outer_position, plan.inner + frame);
        assert!(
            work.expand(1.0).contains_rect(outer),
            "{label}: outer {outer:?} not inside work area {work:?}"
        );
        assert!(
            plan.min_inner.x <= plan.inner.x && plan.min_inner.y <= plan.inner.y,
            "{label}: minimum {:?} exceeds size {:?}",
            plan.min_inner,
            plan.inner
        );
        assert!(
            plan.min_inner.x <= PREFERRED_MIN_INNER_SIZE.x
                && plan.min_inner.y <= PREFERRED_MIN_INNER_SIZE.y,
            "{label}"
        );
    }

    #[test]
    fn startup_plans_fit_the_work_area_at_every_common_scale() {
        // (label, physical work area with a 48-logical-point bottom taskbar, scale)
        for (label, work_px, scale) in [
            ("1920x1080 @100%", [0, 0, 1920, 1032], 1.0),
            ("1920x1080 @125%", [0, 0, 1920, 1020], 1.25),
            ("1920x1080 @150%", [0, 0, 1920, 1008], 1.5),
            ("1920x1080 @200%", [0, 0, 1920, 984], 2.0),
            ("1920x1020 work @200%", [0, 0, 1920, 1020], 2.0),
        ] {
            let work = physical_to_points(work_px, scale);
            let frame = frame_points(scale);
            let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
            assert_plan_is_coherent(plan, work, frame, label);
        }
    }

    #[test]
    fn the_preferred_minimum_is_kept_when_there_is_room() {
        let work = physical_to_points([0, 0, 1920, 1032], 1.0);
        let plan = plan_startup(work, frame_points(1.0), PREFERRED_INNER_SIZE);
        assert_eq!(plan.min_inner, PREFERRED_MIN_INNER_SIZE);
        assert_eq!(plan.inner, PREFERRED_INNER_SIZE);
        let work = physical_to_points([0, 0, 1920, 1020], 1.25);
        let plan = plan_startup(work, frame_points(1.25), PREFERRED_INNER_SIZE);
        // 816 points of work area minus a 37.6-point frame still holds 760.
        assert_eq!(plan.min_inner, PREFERRED_MIN_INNER_SIZE);
        assert_eq!(plan.inner, PREFERRED_INNER_SIZE);
        // At 150 % (672 points of work area) the height is capped; 634 points still allow
        // the full 600-point minimum.
        let work = physical_to_points([0, 0, 1920, 1008], 1.5);
        let frame = frame_points(1.5);
        let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
        assert_eq!(plan.inner.y, 672.0 - frame.y);
        assert_eq!(plan.min_inner.y, 600.0);
    }

    #[test]
    fn at_200_percent_the_minimum_drops_below_600_points_instead_of_overflowing() {
        // 1920×1080 at 200 % with a 1020 px work area: 960×510 points of work area.
        let work = physical_to_points([0, 0, 1920, 1020], 2.0);
        let frame = frame_points(2.0);
        let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
        assert_eq!(plan.min_inner.x, 880.0, "width still fits");
        assert_eq!(plan.min_inner.y, 510.0 - frame.y);
        assert_eq!(plan.inner.y, plan.min_inner.y);
        assert_eq!(plan.outer_position.y, 0.0);
    }

    #[test]
    fn a_work_area_under_600_points_high_gets_a_compatible_minimum() {
        // 1366×768 at 150 % with a 48 px taskbar: about 910×480 points.
        let work = physical_to_points([0, 0, 1366, 720], 1.5);
        let frame = frame_points(1.5);
        let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
        assert!(plan.min_inner.y < 600.0 && plan.min_inner.x == 880.0);
        assert_plan_is_coherent(plan, work, frame, "1366x768 @150%");
        // Narrower than 880 points too: both axes shrink.
        let work = physical_to_points([0, 0, 1280, 672], 1.5);
        let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
        assert!(plan.min_inner.x < 880.0);
        assert_plan_is_coherent(plan, work, frame, "1280x720 @150%");
    }

    #[test]
    fn taskbars_on_any_edge_and_secondary_monitors_keep_the_window_inside() {
        let frame = frame_points(1.25);
        for (label, work_px) in [
            ("top taskbar", [0, 60, 1920, 1080]),
            ("left taskbar", [76, 0, 1920, 1080]),
            ("right taskbar", [0, 0, 1844, 1080]),
            ("secondary monitor left of primary", [-1920, 0, 0, 1020]),
            ("secondary monitor above primary", [0, -1080, 1920, -60]),
        ] {
            let work = physical_to_points(work_px, 1.25);
            let plan = plan_startup(work, frame, PREFERRED_INNER_SIZE);
            assert_plan_is_coherent(plan, work, frame, label);
        }
    }

    #[test]
    fn a_smaller_requested_window_is_kept_but_never_below_the_minimum() {
        let work = physical_to_points([0, 0, 1920, 1032], 1.0);
        let frame = frame_points(1.0);
        let plan = plan_startup(work, frame, vec2(1000.0, 650.0));
        assert_eq!(plan.inner, vec2(1000.0, 650.0));
        let plan = plan_startup(work, frame, vec2(700.0, 500.0));
        assert_eq!(
            plan.inner, PREFERRED_MIN_INNER_SIZE,
            "raised to the minimum"
        );
    }

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

    #[test]
    fn a_degenerate_work_area_changes_nothing() {
        let outer = Rect::from_min_size(pos2(0.0, 0.0), vec2(1200.0, 792.0));
        assert_eq!(
            fit_to_work_area(outer, vec2(1200.0, 760.0), Rect::NOTHING),
            None
        );
        // The startup plan degrades to zero-sized values rather than a negative size.
        let plan = plan_startup(Rect::NOTHING, vec2(16.0, 39.0), PREFERRED_INNER_SIZE);
        assert!(plan.inner.x >= 0.0 && plan.min_inner.y >= 0.0);
    }

    /// Read-only queries of the real system; a session without a display may return None.
    /// These run in a test process that is not DPI aware, so Windows reports values
    /// virtualised to 96 DPI.
    #[cfg(windows)]
    #[test]
    fn native_queries_are_well_formed() {
        if let Some([left, top, right, bottom]) = super::windows::monitor_work_area(0, 0) {
            println!("primary work area (physical px): {left},{top} - {right},{bottom}");
            assert!(right > left && bottom > top);
        }
        for dpi in [96, 120, 144, 192] {
            if let Some([left, top, right, bottom]) = super::windows::window_frame(dpi) {
                println!("frame at {dpi} dpi: {}x{}", right - left, bottom - top);
                assert!(right - left > 0 && bottom - top > 0);
            }
        }
    }
}
