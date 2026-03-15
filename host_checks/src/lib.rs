pub mod display {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ThemeMode {
        Dark,
        Light,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Palette {
        pub canvas: u16,
        pub panel: u16,
        pub panel_alt: u16,
        pub shadow: u16,
        pub text: u16,
        pub text_muted: u16,
        pub steel: u16,
        pub sky: u16,
        pub floor: u16,
        pub cyan: u16,
        pub orange: u16,
        pub rose: u16,
        pub lime: u16,
        pub amber: u16,
        pub indigo: u16,
        pub white: u16,
    }
}

pub mod dungeon {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum RenderStrategy {
        Quality,
        Balanced,
        Performance,
    }
}

pub mod touch {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct TouchCalibration {
        pub x_min: u16,
        pub x_max: u16,
        pub y_min: u16,
        pub y_max: u16,
        pub swap_xy: bool,
        pub invert_x: bool,
        pub invert_y: bool,
        pub valid: bool,
        pub affine: bool,
        pub ax: f32,
        pub bx: f32,
        pub cx: f32,
        pub ay: f32,
        pub by: f32,
        pub cy: f32,
    }
}

#[path = "../../src/app_registry.rs"]
pub mod app_registry;

pub mod storage {
    use crate::app_registry::AppId;
    use crate::display::ThemeMode;
    use crate::dungeon::RenderStrategy;
    use crate::touch::TouchCalibration;

    pub const PAINT_STORAGE_BYTES: usize = 24 * 20;
    pub const PSEUDO_RACER_TRACK_COUNT: usize = 3;
    pub const STATION_HUNTER_STAGE_COUNT: usize = 5;

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct PersistedSystemSettings {
        pub theme: ThemeMode,
        pub language_zh: bool,
        pub render_strategy: RenderStrategy,
        pub touch_ready: bool,
        pub touch_calibration: TouchCalibration,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct PersistedStationHunterData {
        pub selected_stage: u8,
        pub player_level: u8,
        pub player_xp: u16,
        pub upgrade_points: u8,
        pub unlocked_stage: u8,
        pub base_attack: u8,
        pub base_hp: u8,
        pub base_fire_rate: u8,
        pub base_move_speed: u8,
        pub best_kills: u16,
        pub stage_best_wave: [u8; STATION_HUNTER_STAGE_COUNT],
        pub stage_best_kills: [u16; STATION_HUNTER_STAGE_COUNT],
        pub stage_clear_count: [u8; STATION_HUNTER_STAGE_COUNT],
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct PersistedPseudoRacerData {
        pub selected_track: u8,
        pub best_time_ms: [u32; PSEUDO_RACER_TRACK_COUNT],
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct PersistedAppData {
        pub recent_app: Option<AppId>,
        pub album_motion_tab: bool,
        pub album_still_index: u16,
        pub album_motion_index: u16,
        pub album_playing: bool,
        pub paint_selected_color: u8,
        pub paint_pixels: [u8; PAINT_STORAGE_BYTES],
        pub station_hunter: PersistedStationHunterData,
        pub pseudo_racer: PersistedPseudoRacerData,
        pub tap_rush_best_score: u16,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct PersistedState {
        pub system: PersistedSystemSettings,
        pub apps: PersistedAppData,
    }
}

#[path = "../../src/storage_codec.rs"]
pub mod storage_codec;

#[path = "../../shared/media_manifest.rs"]
pub mod media_manifest;
