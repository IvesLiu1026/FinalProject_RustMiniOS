use crate::app_registry::AppId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
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
    pub fn label(self, zh_mode: bool) -> &'static str {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AppRoute {
    pub entry_screen: Screen,
    pub exit_screen: Screen,
    pub track_performance: bool,
    pub pipeline_label: &'static str,
    pub cadence_label: &'static str,
}

pub const fn app_route(app_id: AppId) -> AppRoute {
    match app_id {
        AppId::Album => AppRoute {
            entry_screen: Screen::Album,
            exit_screen: Screen::Home,
            track_performance: true,
            pipeline_label: "RGB565 STILL / MOTION",
            cadence_label: "still / clip cadence",
        },
        AppId::GameCenter => AppRoute {
            entry_screen: Screen::GameCenter,
            exit_screen: Screen::Home,
            track_performance: false,
            pipeline_label: "RETRO LAUNCHER UI",
            cadence_label: "ui event redraw",
        },
        AppId::Paint => AppRoute {
            entry_screen: Screen::Paint,
            exit_screen: Screen::Home,
            track_performance: true,
            pipeline_label: "PIXEL DIRTY RECT",
            cadence_label: "touch event redraw",
        },
        AppId::Settings => AppRoute {
            entry_screen: Screen::Settings,
            exit_screen: Screen::Home,
            track_performance: false,
            pipeline_label: "CONTROL PANEL UI",
            cadence_label: "ui event redraw",
        },
        AppId::DungeonCore => AppRoute {
            entry_screen: Screen::MapSelect,
            exit_screen: Screen::GameCenter,
            track_performance: true,
            pipeline_label: "RAYCAST 3D + HUD",
            cadence_label: "dynamic / strategy driven",
        },
        AppId::AutoBattle => AppRoute {
            entry_screen: Screen::AutoBattle,
            exit_screen: Screen::GameCenter,
            track_performance: true,
            pipeline_label: "DIRTY RECT ARENA",
            cadence_label: "event + dirty redraw",
        },
        AppId::TapRush => AppRoute {
            entry_screen: Screen::TapRush,
            exit_screen: Screen::GameCenter,
            track_performance: true,
            pipeline_label: "REACTION PANEL",
            cadence_label: "ui event redraw",
        },
        AppId::PseudoRacer => AppRoute {
            entry_screen: Screen::PseudoRacer,
            exit_screen: Screen::GameCenter,
            track_performance: true,
            pipeline_label: "71x37 ROAD BUF x4",
            cadence_label: "20 FPS target",
        },
        AppId::GraphicsLab => AppRoute {
            entry_screen: Screen::GraphicsLab,
            exit_screen: Screen::GameCenter,
            track_performance: true,
            pipeline_label: "60x36 FB x5",
            cadence_label: "15 FPS target",
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HostedAppNavigation {
    Stay,
    Launch(AppId),
    Switch(Screen),
    Exit { app_id: AppId, persist_state: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameCenterIntent {
    Idle,
    Previous,
    Next,
    LaunchCurrent,
    ExitHome,
    SelectSlot(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GameCenterOutcome {
    pub next_selected: usize,
    pub dirty: bool,
    pub navigation: HostedAppNavigation,
}

impl GameCenterOutcome {
    const fn stay(next_selected: usize, dirty: bool) -> Self {
        Self {
            next_selected,
            dirty,
            navigation: HostedAppNavigation::Stay,
        }
    }

    const fn launch(next_selected: usize, app_id: AppId) -> Self {
        Self {
            next_selected,
            dirty: true,
            navigation: HostedAppNavigation::Launch(app_id),
        }
    }

    const fn exit_home(next_selected: usize) -> Self {
        Self {
            next_selected,
            dirty: true,
            navigation: HostedAppNavigation::Exit {
                app_id: AppId::GameCenter,
                persist_state: false,
            },
        }
    }
}

pub fn reduce_game_center(
    current_selected: usize,
    app_ids: &[AppId],
    intent: GameCenterIntent,
) -> GameCenterOutcome {
    if app_ids.is_empty() {
        return GameCenterOutcome::stay(0, false);
    }

    let selected = current_selected % app_ids.len();
    match intent {
        GameCenterIntent::Idle => GameCenterOutcome::stay(selected, false),
        GameCenterIntent::Previous => {
            GameCenterOutcome::stay((selected + app_ids.len() - 1) % app_ids.len(), true)
        }
        GameCenterIntent::Next => GameCenterOutcome::stay((selected + 1) % app_ids.len(), true),
        GameCenterIntent::LaunchCurrent => {
            GameCenterOutcome::launch(selected, app_ids[selected])
        }
        GameCenterIntent::ExitHome => GameCenterOutcome::exit_home(selected),
        GameCenterIntent::SelectSlot(index) => {
            let next_selected = index % app_ids.len();
            if next_selected == selected {
                GameCenterOutcome::launch(selected, app_ids[selected])
            } else {
                GameCenterOutcome::stay(next_selected, true)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DungeonHostIntent {
    Stay,
    ExitToGameCenter,
    OpenMapSelect,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DungeonHostSignals {
    pub animation_active: bool,
    pub redraw_requested: bool,
    pub k0_just_pressed: bool,
    pub k1_just_pressed: bool,
    pub wkup_just_pressed: bool,
    pub home_chord: bool,
    pub touch_just_pressed: bool,
    pub touch_just_released: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DungeonHostOutcome {
    pub dirty: bool,
    pub navigation: HostedAppNavigation,
}

pub fn reduce_dungeon_host(
    intent: DungeonHostIntent,
    signals: DungeonHostSignals,
) -> DungeonHostOutcome {
    match intent {
        DungeonHostIntent::ExitToGameCenter => DungeonHostOutcome {
            dirty: true,
            navigation: HostedAppNavigation::Exit {
                app_id: AppId::DungeonCore,
                persist_state: false,
            },
        },
        DungeonHostIntent::OpenMapSelect => DungeonHostOutcome {
            dirty: true,
            navigation: HostedAppNavigation::Switch(Screen::MapSelect),
        },
        DungeonHostIntent::Stay => DungeonHostOutcome {
            dirty: signals.animation_active
                || signals.redraw_requested
                || signals.k0_just_pressed
                || signals.k1_just_pressed
                || signals.wkup_just_pressed
                || signals.home_chord
                || signals.touch_just_pressed
                || signals.touch_just_released,
            navigation: HostedAppNavigation::Stay,
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapSelectIntent {
    Idle,
    Previous,
    Next,
    LaunchCurrent,
    ExitToGameCenter,
    SelectMap(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapSelectOutcome {
    pub next_map_index: usize,
    pub dirty: bool,
    pub navigation: HostedAppNavigation,
    pub prepare_dungeon_launch: bool,
}

impl MapSelectOutcome {
    const fn stay(next_map_index: usize, dirty: bool) -> Self {
        Self {
            next_map_index,
            dirty,
            navigation: HostedAppNavigation::Stay,
            prepare_dungeon_launch: false,
        }
    }

    const fn launch_current(next_map_index: usize) -> Self {
        Self {
            next_map_index,
            dirty: true,
            navigation: HostedAppNavigation::Switch(Screen::DungeonCore),
            prepare_dungeon_launch: true,
        }
    }
}

pub fn reduce_map_select(
    current_map_index: usize,
    map_count: usize,
    intent: MapSelectIntent,
) -> MapSelectOutcome {
    if map_count == 0 {
        return MapSelectOutcome::stay(current_map_index, false);
    }

    match intent {
        MapSelectIntent::Idle => MapSelectOutcome::stay(current_map_index, false),
        MapSelectIntent::Previous => {
            MapSelectOutcome::stay((current_map_index + map_count - 1) % map_count, true)
        }
        MapSelectIntent::Next => MapSelectOutcome::stay((current_map_index + 1) % map_count, true),
        MapSelectIntent::LaunchCurrent => MapSelectOutcome::launch_current(current_map_index),
        MapSelectIntent::ExitToGameCenter => MapSelectOutcome {
            next_map_index: current_map_index,
            dirty: true,
            navigation: HostedAppNavigation::Exit {
                app_id: AppId::DungeonCore,
                persist_state: false,
            },
            prepare_dungeon_launch: false,
        },
        MapSelectIntent::SelectMap(index) => {
            let next_map_index = index % map_count;
            if next_map_index == current_map_index {
                MapSelectOutcome::launch_current(current_map_index)
            } else {
                MapSelectOutcome::stay(next_map_index, true)
            }
        }
    }
}
