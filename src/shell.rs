#[path = "shell/benchmark.rs"]
mod benchmark;
#[path = "shell/calibration.rs"]
mod calibration;
#[path = "shell/persistence.rs"]
mod persistence;
#[path = "shell/render.rs"]
mod render;
#[path = "shell/showcase.rs"]
mod showcase;
#[path = "shell/update.rs"]
mod update;

pub use render::boot_sequence;

use crate::app_registry::{self, home_apps, AppId};
use crate::apps::{
    AlbumAction, AlbumApp, AlbumRedraw, AlbumState, AutoBattleAction, AutoBattleApp,
    AutoBattleRedraw, GameCenterAction, GameCenterApp, GraphicsLabAction, GraphicsLabApp,
    PaintAction, PaintApp, PaintRedraw, PaintState, PseudoRacerAction, PseudoRacerApp,
    TapRushAction, TapRushApp,
};
use crate::board::{delay_ms, millis, Board, ButtonSnapshot};
use crate::display::{color, palette, Display, ThemeMode, SCREEN_WIDTH};
use crate::dungeon::{DungeonAction, DungeonApp, RenderStrategy};
use crate::storage::{
    self, PersistedAppData, PersistedState, PersistedSystemSettings, STATION_HUNTER_STAGE_COUNT,
};
use crate::touch::{Touch, TouchCalibration, TouchState};
use crate::ui::{
    desktop_icon_rect, render_about, render_control_room, render_diagnostics, render_home,
    render_map_select, render_performance_console, render_safe_mode, render_settings,
    render_showcase_overlay, render_touch_calibration, settings_item_at_point,
    settings_list_contains, settings_max_scroll_top, settings_visual_row_for_item,
    DiagnosticsNotice, DIAG_ACTION_COUNT, DIAG_ACTION_H, DIAG_ACTION_W, DIAG_ACTION_Y,
    DIAG_CLEAR_X, DIAG_RESET_X, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y, PERF_BENCH_H,
    PERF_BENCH_W, PERF_BENCH_X, PERF_BENCH_Y, SETTINGS_ROW_HEIGHT, SETTINGS_VISIBLE_ROWS,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Album,
    GameCenter,
    MapSelect,
    Settings,
    PerformanceConsole,
    Benchmark,
    About,
    Diagnostics,
    SafeMode,
    TouchCalibrate,
    ControlRoom,
    DungeonCore,
    AutoBattle,
    Paint,
    TapRush,
    PseudoRacer,
    GraphicsLab,
}

impl Screen {
    fn label(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::Home, true) => "首頁",
            (Self::Album, true) => "相簿",
            (Self::GameCenter, true) => "遊戲中心",
            (Self::MapSelect, true) => "地圖選單",
            (Self::Settings, true) => "設定",
            (Self::PerformanceConsole, true) => "效能儀表",
            (Self::Benchmark, true) => "效能測試",
            (Self::About, true) => "關於系統",
            (Self::Diagnostics, true) => "系統診斷",
            (Self::SafeMode, true) => "安全模式",
            (Self::TouchCalibrate, true) => "觸控校正",
            (Self::ControlRoom, true) => "控制室",
            (Self::DungeonCore, true) => "地城核心",
            (Self::AutoBattle, true) => "定點獵手",
            (Self::Paint, true) => "像素畫板",
            (Self::TapRush, true) => "Tap Rush",
            (Self::PseudoRacer, true) => "假 3D 賽車",
            (Self::GraphicsLab, true) => "圖學實驗室",
            (Self::Home, false) => "HOME",
            (Self::Album, false) => "ALBUM",
            (Self::GameCenter, false) => "GAME CENTER",
            (Self::MapSelect, false) => "MAP SELECT",
            (Self::Settings, false) => "SETTINGS",
            (Self::PerformanceConsole, false) => "PERFORMANCE",
            (Self::Benchmark, false) => "BENCHMARK",
            (Self::About, false) => "ABOUT",
            (Self::Diagnostics, false) => "DIAGNOSTICS",
            (Self::SafeMode, false) => "SAFE MODE",
            (Self::TouchCalibrate, false) => "TOUCH CALIBRATION",
            (Self::ControlRoom, false) => "CONTROL ROOM",
            (Self::DungeonCore, false) => "DUNGEON CORE",
            (Self::AutoBattle, false) => "STATION HUNTER",
            (Self::Paint, false) => "PIXEL PAINT",
            (Self::TapRush, false) => "TAP RUSH",
            (Self::PseudoRacer, false) => "PSEUDO RACER",
            (Self::GraphicsLab, false) => "GRAPHICS LAB",
        }
    }
}

const SETTINGS_ITEM_COUNT: usize = 9;
const BENCH_COUNT: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BenchmarkState {
    Menu,
    Running,
    Results,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BenchmarkCase {
    UiFill,
    RgbBlit,
    PseudoRacer,
    GraphicsLab,
}

const BENCH_CASES: [BenchmarkCase; BENCH_COUNT] = [
    BenchmarkCase::UiFill,
    BenchmarkCase::RgbBlit,
    BenchmarkCase::PseudoRacer,
    BenchmarkCase::GraphicsLab,
];

#[derive(Clone, Copy)]
struct BenchmarkResult {
    avg_fps: u16,
    min_fps: u16,
    duration_ms: u32,
}

const EMPTY_BENCH_RESULT: BenchmarkResult = BenchmarkResult {
    avg_fps: 0,
    min_fps: 0,
    duration_ms: 0,
};

#[derive(Clone, Copy)]
struct BenchmarkMode {
    state: BenchmarkState,
    case_index: usize,
    case_elapsed_ms: u32,
    fps_sum: u32,
    fps_samples: u16,
    min_fps: u16,
    stage_full_redraw: bool,
    rgb_phase: u16,
    results: [BenchmarkResult; BENCH_COUNT],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShowcaseScene {
    Home,
    Album,
    AutoBattle,
    PseudoRacer,
    GraphicsLab,
    Diagnostics,
}

impl ShowcaseScene {
    fn title(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::Home, true) => "復古桌面",
            (Self::Album, true) => "媒體相簿",
            (Self::AutoBattle, true) => "定點獵手",
            (Self::PseudoRacer, true) => "假 3D 賽車",
            (Self::GraphicsLab, true) => "圖學實驗室",
            (Self::Diagnostics, true) => "系統診斷",
            (Self::Home, false) => "RETRO DESKTOP",
            (Self::Album, false) => "MEDIA ALBUM",
            (Self::AutoBattle, false) => "STATION HUNTER",
            (Self::PseudoRacer, false) => "PSEUDO RACER",
            (Self::GraphicsLab, false) => "GRAPHICS LAB",
            (Self::Diagnostics, false) => "DIAGNOSTICS",
        }
    }

    fn subtitle(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::Home, true) => "桌面 / 系統入口 / 復古 shell",
            (Self::Album, true) => "圖片 / 動圖 / 媒體流程",
            (Self::AutoBattle, true) => "2D 機制 / 關卡 / 養成",
            (Self::PseudoRacer, true) => "假 3D 路面 / 速度感 / viewport",
            (Self::GraphicsLab, true) => "數學效果 / framebuffer / demo scene",
            (Self::Diagnostics, true) => "系統監看 / 恢復 / benchmark",
            (Self::Home, false) => "desktop / shell / retro gui",
            (Self::Album, false) => "stills / motion / media path",
            (Self::AutoBattle, false) => "2d systems / stages / growth",
            (Self::PseudoRacer, false) => "pseudo 3d road / speed / viewport",
            (Self::GraphicsLab, false) => "math fx / framebuffer / demo scene",
            (Self::Diagnostics, false) => "system watch / recovery / benchmark",
        }
    }

    const fn duration_ms(self) -> u32 {
        match self {
            Self::Home => 4_500,
            Self::Album => 6_500,
            Self::AutoBattle => 10_000,
            Self::PseudoRacer => 10_000,
            Self::GraphicsLab => 9_000,
            Self::Diagnostics => 6_500,
        }
    }
}

const SHOWCASE_SCENES: [ShowcaseScene; 6] = [
    ShowcaseScene::Home,
    ShowcaseScene::Album,
    ShowcaseScene::AutoBattle,
    ShowcaseScene::PseudoRacer,
    ShowcaseScene::GraphicsLab,
    ShowcaseScene::Diagnostics,
];

#[derive(Clone, Copy)]
struct ShowcaseMode {
    scene_index: usize,
    scene_elapsed_ms: u32,
    cycle_count: u16,
    last_countdown_sec: u8,
    paused: bool,
}

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
    settings_scroll_top_row: usize,
    settings_drag_anchor_y: u16,
    settings_drag_origin_row: usize,
    settings_drag_active: bool,
    map_index: usize,
    dungeon: DungeonApp,
    album: AlbumApp,
    game_center: GameCenterApp,
    auto_battle: AutoBattleApp,
    paint: PaintApp,
    tap_rush: TapRushApp,
    pseudo_racer: PseudoRacerApp,
    graphics_lab: GraphicsLabApp,
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
    performance_focus_app: Option<AppId>,
    benchmark_mode: BenchmarkMode,
    touch_calibration: TouchCalibration,
    touch_return_screen: Screen,
    diagnostics_return_screen: Screen,
    diagnostics_action_index: usize,
    diagnostics_armed: bool,
    diagnostics_notice: Option<DiagnosticsNotice>,
    safe_mode_index: usize,
    safe_boot_session: bool,
    showcase_mode: Option<ShowcaseMode>,
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
            settings_scroll_top_row: 0,
            settings_drag_anchor_y: 0,
            settings_drag_origin_row: 0,
            settings_drag_active: false,
            map_index: 0,
            dungeon: DungeonApp::new(),
            album: AlbumApp::new(),
            game_center: GameCenterApp::new(),
            auto_battle: AutoBattleApp::new(),
            paint: PaintApp::new(),
            tap_rush: TapRushApp::new(),
            pseudo_racer: PseudoRacerApp::new(),
            graphics_lab: GraphicsLabApp::new(),
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
            performance_focus_app: None,
            benchmark_mode: BenchmarkMode {
                state: BenchmarkState::Menu,
                case_index: 0,
                case_elapsed_ms: 0,
                fps_sum: 0,
                fps_samples: 0,
                min_fps: u16::MAX,
                stage_full_redraw: true,
                rgb_phase: 0,
                results: [EMPTY_BENCH_RESULT; BENCH_COUNT],
            },
            touch_calibration: TouchCalibration::default(),
            touch_return_screen: Screen::Home,
            diagnostics_return_screen: Screen::Settings,
            diagnostics_action_index: 0,
            diagnostics_armed: false,
            diagnostics_notice: None,
            safe_mode_index: 0,
            safe_boot_session: false,
            showcase_mode: None,
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

    pub fn touch_ready(&self) -> bool {
        self.touch_ready
    }
}
