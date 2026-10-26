//! Window sizing against the monitor work area (the monitor minus the taskbar).
//!
//! Units:
//! - Physical pixels: what Windows reports (`rcWork`, window rects) to this per-monitor
//!   DPI aware process. Virtual-screen coordinates, negative on monitors left of or above
//!   the primary one.
//! - Points: egui units, physical pixels ÷ `pixels_per_point` (DPI / 96 × egui zoom).
//! - Inner size: the client area egui draws into. Outer size: inner plus decorations
//!   (title bar and the invisible resize borders, which Windows counts in window rects).
//!
//! The policy (`plan_startup`) is `min(880×600, usable) <= size <= usable`, centred in the
//! work area. It runs in two steps with different authority:
//!
//! 1. **Estimate, before the window exists** (eframe `window_builder` hook). No window
//!    exists yet, so `GetDpiForWindow` cannot be used. The only per-monitor DPI source
//!    without a window is `GetDpiForMonitor`, the same call winit's
//!    `MonitorHandle::scale_factor` and eframe's own centring make at this point. The
//!    estimate sets only the initial size and position, never the minimum, so a wrong DPI
//!    can never make Windows enforce a minimum larger than the work area. It is skipped
//!    (and reported) when the thread is not per-monitor aware or a query fails.
//! 2. **Authoritative, on the first frame.** The window exists, and egui's
//!    `pixels_per_point` comes from winit's per-window scale (`GetDpiForWindow`) and egui's
//!    zoom. The real decoration size is `outer - inner`. This step sets the minimum and, if
//!    the estimate was missing or wrong, moves and resizes the window to the plan. eframe
//!    shows the window right after the first paint and applies these commands immediately
//!    afterwards, so a wrong estimate can be visible for about one frame. With a correct
//!    estimate no correction is sent.
//!
//! The window is never sized or positioned again after the first frame, so a window the
//! user moves or resizes is left alone.

use std::sync::OnceLock;

use eframe::egui::{self, Pos2, Rect, Vec2, ViewportBuilder};

/// Inner size the window asks for when there is room.
pub(crate) const PREFERRED_INNER_SIZE: Vec2 = Vec2::new(1200.0, 760.0);
/// Minimum inner size when there is room; smaller work areas lower it (see module docs).
pub(crate) const PREFERRED_MIN_INNER_SIZE: Vec2 = Vec2::new(880.0, 600.0);
/// Fallback when the work area is unknown: assume a bottom taskbar of this height.
const FALLBACK_TASKBAR_ALLOWANCE: f32 = 48.0;
/// Integer physical pixels and fractional points differ by under one point after rounding.
const SIZE_TOLERANCE: f32 = 1.0;
/// DPI values Windows can report for an effective scale of 100 % to 800 %.
const PLAUSIBLE_DPI: std::ops::RangeInclusive<u32> = 96..=768;
/// `DPI_AWARENESS_PER_MONITOR_AWARE`, the value winit's per-monitor v2 context reports.
const PER_MONITOR_AWARE: i32 = 2;

/// Startup geometry in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WindowPlan {
    pub(crate) inner: Vec2,
    pub(crate) min_inner: Vec2,
    pub(crate) outer_position: Pos2,
}

/// Why the pre-creation estimate was not applied. The window then keeps eframe's
/// defaults until the first frame sizes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EstimateSkipped {
    /// No native query on this platform.
    #[cfg_attr(windows, allow(dead_code))]
    Unsupported,
    /// The hook thread is not per-monitor DPI aware (the value is its `DPI_AWARENESS`).
    NotPerMonitorAware(i32),
    WorkAreaUnavailable,
    DpiUnavailable,
    DpiOutOfRange(u32),
    FrameUnavailable,
}

/// What the pre-creation hook did, kept for the first-frame check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum StartupEstimate {
    Applied { dpi: u32, plan: WindowPlan },
    Skipped(EstimateSkipped),
}

#[derive(Clone, Copy, Debug)]
struct Startup {
    desired_inner: Vec2,
    estimate: StartupEstimate,
}

static STARTUP: OnceLock<Startup> = OnceLock::new();

/// Raw values from Windows before validation, in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RawMonitorMetrics {
    pub(crate) awareness: i32,
    pub(crate) work: Option<[i32; 4]>,
    pub(crate) dpi: Option<u32>,
    /// Decoration extents for that DPI: left, top, right, bottom offsets.
    pub(crate) frame: Option<[i32; 4]>,
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

/// Validates raw monitor values and converts them to points (work area and decorations)
/// at the reported DPI, with egui zoom 1 (the zoom at startup).
pub(crate) fn estimate_metrics(
    raw: RawMonitorMetrics,
) -> Result<(Rect, Vec2, u32), EstimateSkipped> {
    if raw.awareness != PER_MONITOR_AWARE {
        return Err(EstimateSkipped::NotPerMonitorAware(raw.awareness));
    }
    let work = raw.work.ok_or(EstimateSkipped::WorkAreaUnavailable)?;
    let dpi = raw.dpi.ok_or(EstimateSkipped::DpiUnavailable)?;
    if !PLAUSIBLE_DPI.contains(&dpi) {
        return Err(EstimateSkipped::DpiOutOfRange(dpi));
    }
    let [left, top, right, bottom] = raw.frame.ok_or(EstimateSkipped::FrameUnavailable)?;
    let scale = dpi as f32 / 96.0;
    let frame = egui::vec2((right - left) as f32, (bottom - top) as f32) / scale;
    Ok((physical_to_points(work, scale), frame, dpi))
}

/// eframe `window_builder` hook (step 1, estimate): initial size and position only. The
/// minimum is left unset; the first frame sets it from the window's own DPI.
pub(crate) fn startup_viewport(builder: ViewportBuilder, desired_inner: Vec2) -> ViewportBuilder {
    let estimate = match native_primary_metrics().and_then(estimate_metrics) {
        Ok((work, frame, dpi)) => StartupEstimate::Applied {
            dpi,
            plan: plan_startup(work, frame, desired_inner),
        },
        Err(reason) => StartupEstimate::Skipped(reason),
    };
    // Observable without a debugger: printed when the app runs from a console.
    eprintln!("encryptdrive window: startup estimate {estimate:?}");
    let _ = STARTUP.set(Startup {
        desired_inner,
        estimate,
    });
    match estimate {
        StartupEstimate::Applied { plan, .. } => builder
            .with_inner_size(plan.inner)
            .with_position(plan.outer_position),
        StartupEstimate::Skipped(_) => builder,
    }
}

/// First-frame decision, from the real window geometry in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FirstFrameFit {
    pub(crate) min_inner: Vec2,
    /// New inner size and outer position, when the window does not match the plan or is
    /// not inside the work area.
    pub(crate) correction: Option<(Vec2, Pos2)>,
}

/// Step 2 (authoritative): the plan for this window's monitor and real decorations. A
/// window that already has the planned size and lies inside the work area is untouched;
/// one with the planned size that overlaps an edge is nudged inside; anything else is
/// resized and centred to the plan.
pub(crate) fn first_frame_fit(
    outer: Rect,
    inner: Vec2,
    work: Rect,
    desired_inner: Vec2,
) -> FirstFrameFit {
    let frame = (outer.size() - inner).max(Vec2::ZERO);
    let min_inner = minimum_inner_size(work, frame);
    if work.width() <= 0.0 || work.height() <= 0.0 {
        return FirstFrameFit {
            min_inner,
            correction: None,
        };
    }
    let plan = plan_startup(work, frame, desired_inner);
    let size_matches = (inner - plan.inner).abs().max_elem() <= SIZE_TOLERANCE;
    let inside = work.expand(SIZE_TOLERANCE / 2.0).contains_rect(outer);
    let correction = match (size_matches, inside) {
        (true, true) => None,
        (true, false) => Some((inner, nudge_inside(outer, work))),
        (false, _) => Some((plan.inner, plan.outer_position)),
    };
    FirstFrameFit {
        min_inner,
        correction,
    }
}

/// The closest position that keeps `outer` inside `work` (top-left wins if it cannot fit).
fn nudge_inside(outer: Rect, work: Rect) -> Pos2 {
    egui::pos2(
        outer
            .min
            .x
            .clamp(work.min.x, (work.max.x - outer.width()).max(work.min.x)),
        outer
            .min
            .y
            .clamp(work.min.y, (work.max.y - outer.height()).max(work.min.y)),
    )
}

/// First-frame fit, run once (see module docs, step 2).
pub(crate) fn fit_window_to_monitor(ctx: &egui::Context, done: &mut bool) {
    if *done {
        return;
    }
    let (monitor, inner, outer, native_pixels_per_point) = ctx.input(|input| {
        let viewport = input.viewport();
        (
            viewport.monitor_size,
            viewport.inner_rect,
            viewport.outer_rect,
            viewport.native_pixels_per_point,
        )
    });
    let (Some(monitor), Some(inner), Some(outer)) = (monitor, inner, outer) else {
        return;
    };
    *done = true;
    // Window DPI (winit, GetDpiForWindow) times egui zoom: the factor egui used for these
    // rects and will use for the commands below.
    let pixels_per_point = ctx.pixels_per_point();
    let work = native_work_area(outer, pixels_per_point).unwrap_or_else(|| {
        Rect::from_min_size(
            Pos2::ZERO,
            monitor - egui::vec2(0.0, FALLBACK_TASKBAR_ALLOWANCE),
        )
    });
    let startup = STARTUP.get().copied();
    let desired_inner = startup.map_or(inner.size(), |startup| startup.desired_inner);
    let fit = first_frame_fit(outer, inner.size(), work, desired_inner);
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(fit.min_inner));
    if let Some((inner_size, outer_position)) = fit.correction {
        if inner_size != inner.size() {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(inner_size));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(outer_position));
    }
    let window_dpi = native_pixels_per_point.map(|scale| (scale * 96.0).round() as u32);
    eprintln!(
        "encryptdrive window: first frame window_dpi={window_dpi:?} estimate={:?} correction={:?} min={:?}",
        startup.map(|startup| startup.estimate),
        fit.correction,
        fit.min_inner
    );
}

/// Converts a physical-pixel rectangle (`left, top, right, bottom`) to points.
pub(crate) fn physical_to_points(rect: [i32; 4], pixels_per_point: f32) -> Rect {
    let [left, top, right, bottom] = rect.map(|value| value as f32 / pixels_per_point);
    Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom))
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

/// Primary monitor values for the estimate. Runs inside the eframe hook, after winit made
/// the process per-monitor DPI aware.
#[cfg(windows)]
fn native_primary_metrics() -> Result<RawMonitorMetrics, EstimateSkipped> {
    let (work, dpi) = windows::primary_work_area_and_dpi();
    Ok(RawMonitorMetrics {
        awareness: windows::thread_dpi_awareness(),
        work,
        dpi,
        frame: dpi.and_then(windows::window_frame),
    })
}

#[cfg(not(windows))]
fn native_primary_metrics() -> Result<RawMonitorMetrics, EstimateSkipped> {
    Err(EstimateSkipped::Unsupported)
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
        fn GetThreadDpiAwarenessContext() -> *mut c_void;
        fn GetAwarenessFromDpiAwarenessContext(context: *mut c_void) -> i32;
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

    /// `DPI_AWARENESS` of the calling thread: -1 invalid, 0 unaware, 1 system aware,
    /// 2 per-monitor aware (v1 and v2 both report 2).
    pub(super) fn thread_dpi_awareness() -> i32 {
        // SAFETY: both calls take no pointers to our memory; the context returned by the
        // first is only passed to the second.
        unsafe { GetAwarenessFromDpiAwarenessContext(GetThreadDpiAwarenessContext()) }
    }

    /// Work area of the monitor nearest to a physical point (read-only query; nothing in
    /// the display configuration changes).
    pub(super) fn monitor_work_area(x: i32, y: i32) -> Option<[i32; 4]> {
        // SAFETY: MonitorFromPoint takes a POINT by value and has no other preconditions.
        work_area(unsafe { MonitorFromPoint(Point { x, y }, MONITOR_DEFAULTTONEAREST) })
    }

    /// Primary monitor work area (physical pixels) and its effective DPI, each `None` when
    /// its query fails.
    pub(super) fn primary_work_area_and_dpi() -> (Option<[i32; 4]>, Option<u32>) {
        // SAFETY: as above; (0, 0) with MONITOR_DEFAULTTOPRIMARY selects the primary monitor.
        let monitor = unsafe { MonitorFromPoint(Point { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
        let work = work_area(monitor);
        if work.is_none() {
            return (None, None);
        }
        let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
        // SAFETY: `monitor` was validated by `work_area`; both out-pointers are live locals.
        // Pre-window estimate only: see the module docs for why this call is used here and
        // why the result never sets the minimum size.
        let hr = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
        (work, (hr >= 0).then_some(dpi_y))
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
        EstimateSkipped, PREFERRED_INNER_SIZE, PREFERRED_MIN_INNER_SIZE, RawMonitorMetrics,
        WindowPlan, estimate_metrics, first_frame_fit, physical_to_points, plan_startup,
    };
    use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

    /// Decorations measured with `AdjustWindowRectExForDpi(WS_OVERLAPPEDWINDOW)` on this
    /// Windows 11 machine: left, top, right, bottom offsets in physical pixels.
    fn frame_px(dpi: u32) -> [i32; 4] {
        match dpi {
            96 => [-8, -31, 8, 8],
            120 => [-9, -38, 9, 9],
            144 => [-11, -45, 11, 11],
            _ => [-13, -58, 13, 13],
        }
    }

    fn frame_points(scale: f32) -> Vec2 {
        let [left, top, right, bottom] = frame_px((scale * 96.0).round() as u32);
        vec2((right - left) as f32, (bottom - top) as f32) / scale
    }

    fn raw(work: [i32; 4], dpi: u32) -> RawMonitorMetrics {
        RawMonitorMetrics {
            awareness: 2,
            work: Some(work),
            dpi: Some(dpi),
            frame: Some(frame_px(dpi)),
        }
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

    /// What winit and Windows do with a requested inner size and outer position at the
    /// window's DPI: round to whole physical pixels, add the real decorations. Returns the
    /// outer and inner rects in points at that DPI, as egui reports them on frame one.
    fn create_window(inner: Vec2, outer_position: Pos2, window_dpi: u32) -> (Rect, Vec2) {
        let scale = window_dpi as f32 / 96.0;
        let [left, top, right, bottom] = frame_px(window_dpi);
        let inner_px = (inner * scale).round();
        let outer_px = inner_px + vec2((right - left) as f32, (bottom - top) as f32);
        let position_px = (outer_position.to_vec2() * scale).round();
        (
            Rect::from_min_size(position_px.to_pos2(), outer_px) / scale,
            inner_px / scale,
        )
    }

    /// Full startup in production code: estimate at `estimate_dpi` (or none), window
    /// created at `window_dpi`, authoritative first frame. Returns the final outer rect and
    /// the first-frame decision.
    fn startup(
        work_px: [i32; 4],
        estimate_dpi: Option<u32>,
        window_dpi: u32,
        desired: Vec2,
    ) -> (Rect, Vec2, super::FirstFrameFit) {
        let window_scale = window_dpi as f32 / 96.0;
        let (inner, position) = match estimate_dpi {
            Some(dpi) => {
                let (work, frame, _) = estimate_metrics(raw(work_px, dpi)).unwrap();
                let plan = plan_startup(work, frame, desired);
                (plan.inner, plan.outer_position)
            }
            // eframe's default: desired size, centred on the monitor ignoring the taskbar.
            None => (desired, pos2(0.0, 0.0)),
        };
        let (outer, inner) = create_window(inner, position, window_dpi);
        let work = physical_to_points(work_px, window_scale);
        let fit = first_frame_fit(outer, inner, work, desired);
        let (final_inner, final_position) = fit.correction.unwrap_or((inner, outer.min));
        let frame = outer.size() - inner;
        let final_outer = Rect::from_min_size(final_position, final_inner + frame);
        (final_outer, final_inner, fit)
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
    fn estimate_converts_work_area_and_frame_at_96_120_144_and_192_dpi() {
        for (dpi, work_px, expected_work) in [
            (96, [0, 0, 1920, 1032], vec2(1920.0, 1032.0)),
            (120, [0, 0, 1920, 1020], vec2(1536.0, 816.0)),
            (144, [0, 0, 1920, 1008], vec2(1280.0, 672.0)),
            (192, [0, 0, 1920, 1020], vec2(960.0, 510.0)),
        ] {
            let (work, frame, reported) = estimate_metrics(raw(work_px, dpi)).unwrap();
            assert_eq!(reported, dpi);
            assert_eq!(work.size(), expected_work, "{dpi}");
            let scale = dpi as f32 / 96.0;
            let [left, top, right, bottom] = frame_px(dpi);
            assert_eq!(
                frame,
                vec2((right - left) as f32, (bottom - top) as f32) / scale
            );
        }
    }

    #[test]
    fn missing_or_invalid_native_data_skips_the_estimate() {
        let good = raw([0, 0, 1920, 1032], 96);
        for (label, metrics, expected) in [
            (
                "system-aware thread",
                RawMonitorMetrics {
                    awareness: 1,
                    ..good
                },
                EstimateSkipped::NotPerMonitorAware(1),
            ),
            (
                "unaware thread",
                RawMonitorMetrics {
                    awareness: 0,
                    ..good
                },
                EstimateSkipped::NotPerMonitorAware(0),
            ),
            (
                "no work area",
                RawMonitorMetrics { work: None, ..good },
                EstimateSkipped::WorkAreaUnavailable,
            ),
            (
                "DPI query failed",
                RawMonitorMetrics { dpi: None, ..good },
                EstimateSkipped::DpiUnavailable,
            ),
            (
                "zero DPI",
                RawMonitorMetrics {
                    dpi: Some(0),
                    ..good
                },
                EstimateSkipped::DpiOutOfRange(0),
            ),
            (
                "absurd DPI",
                RawMonitorMetrics {
                    dpi: Some(4_000),
                    ..good
                },
                EstimateSkipped::DpiOutOfRange(4_000),
            ),
            (
                "no frame",
                RawMonitorMetrics {
                    frame: None,
                    ..good
                },
                EstimateSkipped::FrameUnavailable,
            ),
        ] {
            assert_eq!(estimate_metrics(metrics), Err(expected), "{label}");
        }
    }

    #[test]
    fn a_correct_estimate_needs_no_first_frame_correction_at_any_dpi() {
        for (label, work_px, dpi) in [
            ("100 % bottom taskbar", [0, 0, 1920, 1032], 96),
            ("125 % bottom taskbar", [0, 0, 1920, 1020], 120),
            ("150 % bottom taskbar", [0, 0, 1920, 1008], 144),
            ("200 % bottom taskbar", [0, 0, 1920, 984], 192),
            ("200 % 1020 px work area", [0, 0, 1920, 1020], 192),
            ("150 % small vertical work area", [0, 0, 1366, 720], 144),
            ("125 % top taskbar", [0, 60, 1920, 1080], 120),
            ("125 % left taskbar", [76, 0, 1920, 1080], 120),
            ("125 % right taskbar", [0, 0, 1844, 1080], 120),
            ("100 % monitor left of primary", [-1920, 0, 0, 1032], 96),
            ("200 % monitor above primary", [0, -2160, 3840, -96], 192),
        ] {
            let (outer, inner, fit) = startup(work_px, Some(dpi), dpi, PREFERRED_INNER_SIZE);
            let work = physical_to_points(work_px, dpi as f32 / 96.0);
            assert_eq!(fit.correction, None, "{label}: {fit:?}");
            assert!(
                work.expand(1.0).contains_rect(outer),
                "{label}: {outer:?} vs {work:?}"
            );
            assert!(
                fit.min_inner.x <= inner.x + 1.0 && fit.min_inner.y <= inner.y + 1.0,
                "{label}"
            );
        }
    }

    #[test]
    fn a_wrong_or_missing_estimate_is_corrected_from_the_window_dpi() {
        let work_px = [0, 0, 1920, 1020];
        // Estimate too low (96 for a 200 % window): window too large, then shrunk.
        let (outer, inner, fit) = startup(work_px, Some(96), 192, PREFERRED_INNER_SIZE);
        let work = physical_to_points(work_px, 2.0);
        assert!(fit.correction.is_some());
        assert!(work.expand(1.0).contains_rect(outer), "{outer:?}");
        assert!(fit.min_inner.y < 600.0 && fit.min_inner.y <= inner.y);
        // Estimate too high (192 for a 100 % window): window too small, then grown.
        let (outer, inner, fit) = startup(work_px, Some(192), 96, PREFERRED_INNER_SIZE);
        assert!(fit.correction.is_some());
        assert_eq!(inner, PREFERRED_INNER_SIZE);
        assert!(physical_to_points(work_px, 1.0).contains_rect(outer));
        assert_eq!(fit.min_inner, PREFERRED_MIN_INNER_SIZE);
        // No estimate at all (query failed): eframe's default is corrected the same way.
        for dpi in [96, 120, 144, 192] {
            let (outer, _, fit) = startup(work_px, None, dpi, PREFERRED_INNER_SIZE);
            let work = physical_to_points(work_px, dpi as f32 / 96.0);
            assert!(work.expand(1.0).contains_rect(outer), "{dpi}: {fit:?}");
        }
    }

    #[test]
    fn the_first_frame_minimum_never_exceeds_the_usable_area() {
        for (work_px, dpi) in [
            ([0, 0, 1920, 1020], 192),
            ([0, 0, 1280, 672], 144),
            ([0, 0, 1024, 600], 192),
        ] {
            let (_, _, fit) = startup(work_px, Some(dpi), dpi, PREFERRED_INNER_SIZE);
            let scale = dpi as f32 / 96.0;
            let work = physical_to_points(work_px, scale);
            let usable = work.size() - frame_points(scale);
            // The frame comes from rounded physical pixels: allow float error only.
            assert!(
                fit.min_inner.x <= usable.x + 1e-3 && fit.min_inner.y <= usable.y + 1e-3,
                "{work_px:?}: min {:?} usable {usable:?}",
                fit.min_inner
            );
        }
    }

    // First-frame behaviour without a startup record (desired = current size), which is
    // the UX6.1 fit: preserved cases.

    #[test]
    fn a_window_that_fits_is_left_alone() {
        let work = Rect::from_min_size(pos2(0.0, 0.0), vec2(1536.0, 816.0));
        let outer = Rect::from_min_size(pos2(100.0, 10.0), vec2(1200.0, 792.0));
        let inner = vec2(1200.0, 760.0);
        assert_eq!(first_frame_fit(outer, inner, work, inner).correction, None);
    }

    #[test]
    fn an_oversized_window_shrinks_inside_the_work_area() {
        let work = Rect::from_min_size(pos2(0.0, 0.0), vec2(1536.0, 816.0));
        // 1920×1080 requested; 32 points of title bar.
        let outer = Rect::from_min_size(pos2(-192.0, -148.0), vec2(1920.0, 1112.0));
        let requested = vec2(1920.0, 1080.0);
        let (inner, position) = first_frame_fit(outer, requested, work, requested)
            .correction
            .unwrap();
        assert_eq!(inner, vec2(1536.0, 784.0));
        assert!(work.contains_rect(Rect::from_min_size(position, inner + vec2(0.0, 32.0))));
        // Only the axis that overflows shrinks.
        let tall = Rect::from_min_size(pos2(100.0, 0.0), vec2(1180.0, 824.0));
        let requested = vec2(1180.0, 792.0);
        let (inner, _) = first_frame_fit(tall, requested, work, requested)
            .correction
            .unwrap();
        assert_eq!(inner, vec2(1180.0, 784.0));
    }

    #[test]
    fn a_window_partly_behind_the_taskbar_moves_without_resizing() {
        // Top taskbar: work area starts at y = 40.
        let work = Rect::from_min_max(pos2(0.0, 40.0), pos2(1536.0, 864.0));
        let outer = Rect::from_min_size(pos2(168.0, 20.0), vec2(1200.0, 792.0));
        let inner = vec2(1200.0, 760.0);
        let (size, position) = first_frame_fit(outer, inner, work, inner)
            .correction
            .unwrap();
        assert_eq!(size, inner);
        assert_eq!(position, pos2(168.0, 40.0));
    }

    #[test]
    fn windows_on_a_secondary_monitor_stay_on_that_monitor() {
        let work = Rect::from_min_max(pos2(-1920.0, 0.0), pos2(0.0, 1032.0));
        let outer = Rect::from_min_size(pos2(-400.0, 100.0), vec2(1200.0, 792.0));
        let inner = vec2(1200.0, 760.0);
        let (_, position) = first_frame_fit(outer, inner, work, inner)
            .correction
            .unwrap();
        assert_eq!(position, pos2(-1200.0, 100.0));
    }

    #[test]
    fn a_degenerate_work_area_changes_nothing() {
        let outer = Rect::from_min_size(pos2(0.0, 0.0), vec2(1200.0, 792.0));
        let inner = vec2(1200.0, 760.0);
        assert_eq!(
            first_frame_fit(outer, inner, Rect::NOTHING, inner).correction,
            None
        );
        // The startup plan degrades to zero-sized values rather than a negative size.
        let plan = plan_startup(Rect::NOTHING, vec2(16.0, 39.0), PREFERRED_INNER_SIZE);
        assert!(plan.inner.x >= 0.0 && plan.min_inner.y >= 0.0);
    }

    /// The real hook in this (DPI-unaware) test process: the awareness guard skips the
    /// estimate and the builder keeps no minimum and the requested size.
    #[test]
    fn the_hook_never_sets_a_minimum_and_skips_without_per_monitor_awareness() {
        let builder = super::startup_viewport(
            eframe::egui::ViewportBuilder::default().with_inner_size(PREFERRED_INNER_SIZE),
            PREFERRED_INNER_SIZE,
        );
        assert_eq!(builder.min_inner_size, None);
        #[cfg(windows)]
        {
            let estimate = super::STARTUP.get().unwrap().estimate;
            println!("test-process estimate: {estimate:?}");
            if let super::StartupEstimate::Skipped(EstimateSkipped::NotPerMonitorAware(_)) =
                estimate
            {
                assert_eq!(builder.inner_size, Some(PREFERRED_INNER_SIZE));
            }
        }
    }

    /// Read-only queries of the real system; a session without a display may return None.
    /// This test process is not DPI aware, so Windows reports values virtualised to 96 DPI.
    #[cfg(windows)]
    #[test]
    fn native_queries_are_well_formed() {
        println!(
            "thread DPI awareness: {}",
            super::windows::thread_dpi_awareness()
        );
        if let Some([left, top, right, bottom]) = super::windows::monitor_work_area(0, 0) {
            println!("primary work area (physical px): {left},{top} - {right},{bottom}");
            assert!(right > left && bottom > top);
        }
        for dpi in [96, 120, 144, 192] {
            let measured = super::windows::window_frame(dpi);
            println!("frame at {dpi} dpi: {measured:?}");
            if let Some(frame) = measured {
                assert_eq!(frame, frame_px(dpi), "test fixture matches this machine");
            }
        }
    }
}
