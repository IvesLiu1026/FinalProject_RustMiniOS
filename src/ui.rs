#[path = "ui/about.rs"]
mod about;
#[path = "ui/calibration.rs"]
mod calibration;
#[path = "ui/control_room.rs"]
mod control_room;
#[path = "ui/diagnostics.rs"]
mod diagnostics;
#[path = "ui/home.rs"]
mod home;
#[path = "ui/map_select.rs"]
mod map_select;
#[path = "ui/performance_console.rs"]
mod performance_console;
#[path = "ui/safe_mode.rs"]
mod safe_mode;
#[path = "ui/settings.rs"]
mod settings;
#[path = "ui/shared.rs"]
mod shared;
#[path = "ui/showcase.rs"]
mod showcase;

pub const NAV_BACK_X: u16 = 18;
pub const NAV_BACK_Y: u16 = 14;
pub const NAV_BACK_W: u16 = 56;
pub const NAV_BACK_H: u16 = 16;
pub const DIAG_ACTION_COUNT: usize = 2;
pub const DIAG_CLEAR_X: u16 = 18;
pub const DIAG_RESET_X: u16 = 166;
pub const DIAG_ACTION_Y: u16 = 184;
pub const DIAG_ACTION_W: u16 = 136;
pub const DIAG_ACTION_H: u16 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsNotice {
    ClearReady,
    Cleared,
    ClearFailed,
    ResetReady,
    ResetFailed,
}

pub use about::render_about;
pub use calibration::render_touch_calibration;
pub use control_room::render_control_room;
pub use diagnostics::render_diagnostics;
pub use home::{desktop_icon_rect, render_home};
pub use map_select::render_map_select;
pub use performance_console::{
    render_performance_console, PERF_BENCH_H, PERF_BENCH_W, PERF_BENCH_X, PERF_BENCH_Y,
};
pub use safe_mode::render_safe_mode;
pub use settings::{
    render_settings, settings_item_at_point, settings_list_contains, settings_max_scroll_top,
    settings_visual_row_for_item, SETTINGS_ROW_HEIGHT, SETTINGS_VISIBLE_ROWS,
};
pub use shared::{
    draw_app_icon, draw_desktop_shortcut, draw_footer_hint, draw_gradient_background,
    draw_info_strip, draw_scrollbar, draw_shell_window, draw_title_bar, fit_text_to_width,
    render_nav_back, theme_mode_label, SHELL_CONTENT_X,
};
pub use showcase::render_showcase_overlay;
