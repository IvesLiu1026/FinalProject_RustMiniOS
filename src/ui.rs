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
#[path = "ui/safe_mode.rs"]
mod safe_mode;
#[path = "ui/settings.rs"]
mod settings;
#[path = "ui/shared.rs"]
mod shared;

pub const NAV_BACK_X: u16 = 20;
pub const NAV_BACK_Y: u16 = 14;
pub const NAV_BACK_W: u16 = 44;
pub const NAV_BACK_H: u16 = 16;
pub const DIAG_ACTION_COUNT: usize = 2;
pub const DIAG_CLEAR_X: u16 = 18;
pub const DIAG_RESET_X: u16 = 166;
pub const DIAG_ACTION_Y: u16 = 190;
pub const DIAG_ACTION_W: u16 = 136;
pub const DIAG_ACTION_H: u16 = 22;

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
pub use home::render_home;
pub use map_select::render_map_select;
pub use safe_mode::render_safe_mode;
pub use settings::render_settings;
pub use shared::{draw_gradient_background, render_nav_back};
