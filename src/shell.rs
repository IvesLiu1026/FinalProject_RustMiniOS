#[path = "shell/calibration.rs"]
mod calibration;
#[path = "shell/persistence.rs"]
mod persistence;
#[path = "shell/render.rs"]
mod render;
#[path = "shell/update.rs"]
mod update;

pub use render::boot_sequence;

use crate::app_registry::{self, home_apps, AppId};
use crate::apps::{
    AlbumAction, AlbumApp, AlbumRedraw, AlbumState, AutoBattleAction, AutoBattleApp,
    AutoBattleRedraw, GameCenterAction, GameCenterApp, PaintAction, PaintApp, PaintRedraw,
    PaintState, TapRushAction, TapRushApp,
};
use crate::board::{delay_ms, millis, Board, ButtonSnapshot};
use crate::display::{color, palette, Display, ThemeMode, SCREEN_WIDTH};
use crate::dungeon::{DungeonAction, DungeonApp, RenderStrategy};
use crate::storage::{self, PersistedAppData, PersistedState, PersistedSystemSettings};
use crate::touch::{Touch, TouchCalibration, TouchState};
use crate::ui::{
    render_about, render_control_room, render_diagnostics, render_home, render_map_select,
    render_safe_mode, render_settings, render_touch_calibration, DiagnosticsNotice,
    DIAG_ACTION_COUNT, DIAG_ACTION_H, DIAG_ACTION_W, DIAG_ACTION_Y, DIAG_CLEAR_X, DIAG_RESET_X,
    NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Album,
    GameCenter,
    MapSelect,
    Settings,
    About,
    Diagnostics,
    SafeMode,
    TouchCalibrate,
    ControlRoom,
    DungeonCore,
    AutoBattle,
    Paint,
    TapRush,
}

impl Screen {
    fn label(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::Home, true) => "首頁",
            (Self::Album, true) => "相簿",
            (Self::GameCenter, true) => "遊戲中心",
            (Self::MapSelect, true) => "地圖選單",
            (Self::Settings, true) => "設定",
            (Self::About, true) => "關於系統",
            (Self::Diagnostics, true) => "系統診斷",
            (Self::SafeMode, true) => "安全模式",
            (Self::TouchCalibrate, true) => "觸控校正",
            (Self::ControlRoom, true) => "控制室",
            (Self::DungeonCore, true) => "地城核心",
            (Self::AutoBattle, true) => "自動獵手",
            (Self::Paint, true) => "像素畫板",
            (Self::TapRush, true) => "Tap Rush",
            (Self::Home, false) => "HOME",
            (Self::Album, false) => "ALBUM",
            (Self::GameCenter, false) => "GAME CENTER",
            (Self::MapSelect, false) => "MAP SELECT",
            (Self::Settings, false) => "SETTINGS",
            (Self::About, false) => "ABOUT",
            (Self::Diagnostics, false) => "DIAGNOSTICS",
            (Self::SafeMode, false) => "SAFE MODE",
            (Self::TouchCalibrate, false) => "TOUCH CALIBRATION",
            (Self::ControlRoom, false) => "CONTROL ROOM",
            (Self::DungeonCore, false) => "DUNGEON CORE",
            (Self::AutoBattle, false) => "AUTO HUNTER",
            (Self::Paint, false) => "PIXEL PAINT",
            (Self::TapRush, false) => "TAP RUSH",
        }
    }
}

const SETTINGS_ITEM_COUNT: usize = 7;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    ZhTw,
}

impl Language {
    fn toggle(&mut self) {
        *self = match self {
            Self::English => Self::ZhTw,
            Self::ZhTw => Self::English,
        };
    }

    fn is_zh(self) -> bool {
        matches!(self, Self::ZhTw)
    }
}

pub struct MiniOs {
    screen: Screen,
    home_index: usize,
    settings_index: usize,
    map_index: usize,
    dungeon: DungeonApp,
    album: AlbumApp,
    game_center: GameCenterApp,
    auto_battle: AutoBattleApp,
    paint: PaintApp,
    tap_rush: TapRushApp,
    theme: ThemeMode,
    language: Language,
    render_strategy: RenderStrategy,
    last_uptime_second: u32,
    fps_estimate: u16,
    force_full_redraw: bool,
    calibration_step: u8,
    calibration_raw_x: [u16; 5],
    calibration_raw_y: [u16; 5],
    touch_ready: bool,
    recent_app: Option<AppId>,
    touch_calibration: TouchCalibration,
    touch_return_screen: Screen,
    diagnostics_return_screen: Screen,
    diagnostics_action_index: usize,
    diagnostics_armed: bool,
    diagnostics_notice: Option<DiagnosticsNotice>,
    safe_mode_index: usize,
    safe_boot_session: bool,
    album_redraw: Option<AlbumRedraw>,
    paint_redraw: Option<PaintRedraw>,
    auto_battle_redraw: Option<AutoBattleRedraw>,
}

impl MiniOs {
    pub fn new() -> Self {
        Self {
            screen: Screen::TouchCalibrate,
            home_index: 0,
            settings_index: 0,
            map_index: 0,
            dungeon: DungeonApp::new(),
            album: AlbumApp::new(),
            game_center: GameCenterApp::new(),
            auto_battle: AutoBattleApp::new(),
            paint: PaintApp::new(),
            tap_rush: TapRushApp::new(),
            theme: ThemeMode::Dark,
            language: Language::English,
            render_strategy: RenderStrategy::Balanced,
            last_uptime_second: 0,
            fps_estimate: 0,
            force_full_redraw: true,
            calibration_step: 0,
            calibration_raw_x: [0; 5],
            calibration_raw_y: [0; 5],
            touch_ready: false,
            recent_app: None,
            touch_calibration: TouchCalibration::default(),
            touch_return_screen: Screen::Home,
            diagnostics_return_screen: Screen::Settings,
            diagnostics_action_index: 0,
            diagnostics_armed: false,
            diagnostics_notice: None,
            safe_mode_index: 0,
            safe_boot_session: false,
            album_redraw: None,
            paint_redraw: None,
            auto_battle_redraw: None,
        }
    }

    pub fn enter_safe_mode(&mut self) {
        self.safe_boot_session = true;
        self.safe_mode_index = 0;
        self.diagnostics_return_screen = Screen::SafeMode;
        self.screen = Screen::SafeMode;
        self.force_full_redraw = true;
    }
}
