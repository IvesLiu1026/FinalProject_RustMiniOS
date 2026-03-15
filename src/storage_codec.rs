use crate::app_registry::AppId;
use crate::display::ThemeMode;
use crate::dungeon::RenderStrategy;
use crate::storage::{
    PersistedAppData, PersistedPseudoRacerData, PersistedState, PersistedStationHunterData,
    PersistedSystemSettings,
};
use crate::touch::TouchCalibration;

pub const PAINT_STORAGE_BYTES: usize = 24 * 20;
pub const STATION_HUNTER_STAGE_COUNT: usize = 5;
pub const STORAGE_BYTES: usize = 608;

const CHECKSUM_OFFSET: usize = STORAGE_BYTES - 4;
const MAGIC: u32 = 0x4D4F_5332; // "MOS2"
const VERSION: u16 = 4;
const PAINT_OFFSET: usize = 64;
const STATION_OFFSET: usize = PAINT_OFFSET + PAINT_STORAGE_BYTES;
const RACER_OFFSET: usize = STATION_OFFSET + 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageStatus {
    pub found_magic: bool,
    pub valid_record: bool,
    pub checksum_ok: bool,
    pub version: u16,
    pub record_bytes: u16,
    pub recent_app: Option<AppId>,
    pub paint_pixels_used: u16,
    pub has_app_saves: bool,
}

pub fn inspect_bytes(bytes: &[u8]) -> StorageStatus {
    let found_magic = bytes.len() >= 4 && read_u32(bytes, 0) == MAGIC;
    let version = if found_magic && bytes.len() >= 6 {
        read_u16(bytes, 4)
    } else {
        0
    };
    let record_bytes = if found_magic && bytes.len() >= 8 {
        read_u16(bytes, 6)
    } else {
        0
    };
    let checksum_ok = bytes.len() >= STORAGE_BYTES
        && read_u32(bytes, CHECKSUM_OFFSET) == fnv1a(&bytes[..CHECKSUM_OFFSET]);
    if let Some(state) = decode(bytes) {
        let defaults = default_app_data();
        let paint_pixels_used = state
            .apps
            .paint_pixels
            .iter()
            .filter(|pixel| **pixel != 0)
            .count()
            .min(u16::MAX as usize) as u16;
        StorageStatus {
            found_magic,
            valid_record: true,
            checksum_ok,
            version,
            record_bytes,
            recent_app: state.apps.recent_app,
            paint_pixels_used,
            has_app_saves: state.apps != defaults,
        }
    } else {
        StorageStatus {
            found_magic,
            valid_record: false,
            checksum_ok,
            version,
            record_bytes,
            recent_app: None,
            paint_pixels_used: 0,
            has_app_saves: false,
        }
    }
}

pub fn default_station_hunter_data() -> PersistedStationHunterData {
    PersistedStationHunterData {
        selected_stage: 1,
        player_level: 1,
        player_xp: 0,
        upgrade_points: 0,
        unlocked_stage: 1,
        base_attack: 0,
        base_hp: 0,
        base_fire_rate: 0,
        base_move_speed: 0,
        best_kills: 0,
        stage_best_wave: [0; STATION_HUNTER_STAGE_COUNT],
        stage_best_kills: [0; STATION_HUNTER_STAGE_COUNT],
        stage_clear_count: [0; STATION_HUNTER_STAGE_COUNT],
    }
}

pub fn default_pseudo_racer_data() -> PersistedPseudoRacerData {
    PersistedPseudoRacerData {
        selected_track: 0,
        best_time_ms: [0; 3],
    }
}

pub fn default_app_data() -> PersistedAppData {
    PersistedAppData {
        recent_app: None,
        album_motion_tab: false,
        album_still_index: 0,
        album_motion_index: 0,
        album_playing: true,
        paint_selected_color: 1,
        paint_pixels: [0; PAINT_STORAGE_BYTES],
        station_hunter: default_station_hunter_data(),
        pseudo_racer: default_pseudo_racer_data(),
        tap_rush_best_score: 0,
    }
}

pub fn encode(state: &PersistedState) -> [u8; STORAGE_BYTES] {
    let mut out = [0xFFu8; STORAGE_BYTES];
    write_u32(&mut out, 0, MAGIC);
    write_u16(&mut out, 4, VERSION);
    write_u16(&mut out, 6, STORAGE_BYTES as u16);
    out[8] = encode_theme(state.system.theme);
    out[9] = if state.system.language_zh { 1 } else { 0 };
    out[10] = encode_render_strategy(state.system.render_strategy);
    out[11] = if state.system.touch_ready { 1 } else { 0 };
    out[12] = encode_app_id(state.apps.recent_app);
    out[13] = if state.apps.album_motion_tab { 1 } else { 0 };
    out[14] = if state.apps.album_playing { 1 } else { 0 };
    out[15] = state.apps.paint_selected_color;
    write_u16(&mut out, 16, state.apps.station_hunter.best_kills);
    write_u16(&mut out, 18, state.apps.tap_rush_best_score);
    write_u16(&mut out, 20, state.apps.album_still_index);
    write_u16(&mut out, 22, state.apps.album_motion_index);
    write_u16(&mut out, 24, state.system.touch_calibration.x_min);
    write_u16(&mut out, 26, state.system.touch_calibration.x_max);
    write_u16(&mut out, 28, state.system.touch_calibration.y_min);
    write_u16(&mut out, 30, state.system.touch_calibration.y_max);
    out[32] = if state.system.touch_calibration.swap_xy {
        1
    } else {
        0
    };
    out[33] = if state.system.touch_calibration.invert_x {
        1
    } else {
        0
    };
    out[34] = if state.system.touch_calibration.invert_y {
        1
    } else {
        0
    };
    out[35] = if state.system.touch_calibration.valid {
        1
    } else {
        0
    };
    out[36] = if state.system.touch_calibration.affine {
        1
    } else {
        0
    };
    write_f32(&mut out, 40, state.system.touch_calibration.ax);
    write_f32(&mut out, 44, state.system.touch_calibration.bx);
    write_f32(&mut out, 48, state.system.touch_calibration.cx);
    write_f32(&mut out, 52, state.system.touch_calibration.ay);
    write_f32(&mut out, 56, state.system.touch_calibration.by);
    write_f32(&mut out, 60, state.system.touch_calibration.cy);
    out[PAINT_OFFSET..PAINT_OFFSET + PAINT_STORAGE_BYTES].copy_from_slice(&state.apps.paint_pixels);

    let hunter = state.apps.station_hunter;
    out[STATION_OFFSET] = hunter.selected_stage;
    out[STATION_OFFSET + 1] = hunter.player_level;
    write_u16(&mut out, STATION_OFFSET + 2, hunter.player_xp);
    out[STATION_OFFSET + 4] = hunter.upgrade_points;
    out[STATION_OFFSET + 5] = hunter.unlocked_stage;
    out[STATION_OFFSET + 6] = hunter.base_attack;
    out[STATION_OFFSET + 7] = hunter.base_hp;
    out[STATION_OFFSET + 8] = hunter.base_fire_rate;
    out[STATION_OFFSET + 9] = hunter.base_move_speed;
    out[STATION_OFFSET + 10..STATION_OFFSET + 10 + STATION_HUNTER_STAGE_COUNT]
        .copy_from_slice(&hunter.stage_best_wave);
    out[STATION_OFFSET + 15..STATION_OFFSET + 15 + STATION_HUNTER_STAGE_COUNT]
        .copy_from_slice(&hunter.stage_clear_count);
    for (index, kills) in hunter.stage_best_kills.iter().copied().enumerate() {
        write_u16(&mut out, STATION_OFFSET + 20 + index * 2, kills);
    }

    let racer = state.apps.pseudo_racer;
    out[RACER_OFFSET] = racer.selected_track;
    for (index, best_time) in racer.best_time_ms.iter().copied().enumerate() {
        write_u32(&mut out, RACER_OFFSET + 4 + index * 4, best_time);
    }

    let checksum = fnv1a(&out[..CHECKSUM_OFFSET]);
    write_u32(&mut out, CHECKSUM_OFFSET, checksum);
    out
}

pub fn decode(bytes: &[u8]) -> Option<PersistedState> {
    if bytes.len() < STORAGE_BYTES {
        return None;
    }
    if read_u32(bytes, 0) != MAGIC {
        return None;
    }
    if read_u16(bytes, 4) != VERSION || read_u16(bytes, 6) as usize != STORAGE_BYTES {
        return None;
    }
    if read_u32(bytes, CHECKSUM_OFFSET) != fnv1a(&bytes[..CHECKSUM_OFFSET]) {
        return None;
    }

    let mut paint_pixels = [0u8; PAINT_STORAGE_BYTES];
    paint_pixels.copy_from_slice(&bytes[PAINT_OFFSET..PAINT_OFFSET + PAINT_STORAGE_BYTES]);

    let mut stage_best_wave = [0u8; STATION_HUNTER_STAGE_COUNT];
    stage_best_wave.copy_from_slice(
        &bytes[STATION_OFFSET + 10..STATION_OFFSET + 10 + STATION_HUNTER_STAGE_COUNT],
    );
    let mut stage_clear_count = [0u8; STATION_HUNTER_STAGE_COUNT];
    stage_clear_count.copy_from_slice(
        &bytes[STATION_OFFSET + 15..STATION_OFFSET + 15 + STATION_HUNTER_STAGE_COUNT],
    );
    let mut stage_best_kills = [0u16; STATION_HUNTER_STAGE_COUNT];
    for (index, kills) in stage_best_kills.iter_mut().enumerate() {
        *kills = read_u16(bytes, STATION_OFFSET + 20 + index * 2);
    }
    let mut racer_best_time_ms = [0u32; 3];
    for (index, best_time) in racer_best_time_ms.iter_mut().enumerate() {
        *best_time = read_u32(bytes, RACER_OFFSET + 4 + index * 4);
    }

    Some(PersistedState {
        system: PersistedSystemSettings {
            theme: decode_theme(bytes[8]),
            language_zh: bytes[9] != 0,
            render_strategy: decode_render_strategy(bytes[10]),
            touch_ready: bytes[11] != 0,
            touch_calibration: TouchCalibration {
                x_min: read_u16(bytes, 24),
                x_max: read_u16(bytes, 26),
                y_min: read_u16(bytes, 28),
                y_max: read_u16(bytes, 30),
                swap_xy: bytes[32] != 0,
                invert_x: bytes[33] != 0,
                invert_y: bytes[34] != 0,
                valid: bytes[35] != 0,
                affine: bytes[36] != 0,
                ax: read_f32(bytes, 40),
                bx: read_f32(bytes, 44),
                cx: read_f32(bytes, 48),
                ay: read_f32(bytes, 52),
                by: read_f32(bytes, 56),
                cy: read_f32(bytes, 60),
            },
        },
        apps: PersistedAppData {
            recent_app: decode_app_id(bytes[12]),
            album_motion_tab: bytes[13] != 0,
            album_playing: bytes[14] != 0,
            paint_selected_color: bytes[15],
            station_hunter: PersistedStationHunterData {
                selected_stage: bytes[STATION_OFFSET].clamp(1, STATION_HUNTER_STAGE_COUNT as u8),
                player_level: bytes[STATION_OFFSET + 1].max(1),
                player_xp: read_u16(bytes, STATION_OFFSET + 2),
                upgrade_points: bytes[STATION_OFFSET + 4],
                unlocked_stage: bytes[STATION_OFFSET + 5]
                    .clamp(1, STATION_HUNTER_STAGE_COUNT as u8),
                base_attack: bytes[STATION_OFFSET + 6],
                base_hp: bytes[STATION_OFFSET + 7],
                base_fire_rate: bytes[STATION_OFFSET + 8],
                base_move_speed: bytes[STATION_OFFSET + 9],
                best_kills: read_u16(bytes, 16),
                stage_best_wave,
                stage_best_kills,
                stage_clear_count,
            },
            pseudo_racer: PersistedPseudoRacerData {
                selected_track: bytes[RACER_OFFSET].min(2),
                best_time_ms: racer_best_time_ms,
            },
            tap_rush_best_score: read_u16(bytes, 18),
            album_still_index: read_u16(bytes, 20),
            album_motion_index: read_u16(bytes, 22),
            paint_pixels,
        },
    })
}

fn encode_app_id(app_id: Option<AppId>) -> u8 {
    match app_id {
        Some(AppId::Album) => 0,
        Some(AppId::GameCenter) => 1,
        Some(AppId::Paint) => 2,
        Some(AppId::Settings) => 3,
        Some(AppId::DungeonCore) => 4,
        Some(AppId::AutoBattle) => 5,
        Some(AppId::TapRush) => 6,
        Some(AppId::PseudoRacer) => 7,
        Some(AppId::GraphicsLab) => 8,
        None => 0xFF,
    }
}

fn decode_app_id(value: u8) -> Option<AppId> {
    match value {
        0 => Some(AppId::Album),
        1 => Some(AppId::GameCenter),
        2 => Some(AppId::Paint),
        3 => Some(AppId::Settings),
        4 => Some(AppId::DungeonCore),
        5 => Some(AppId::AutoBattle),
        6 => Some(AppId::TapRush),
        7 => Some(AppId::PseudoRacer),
        8 => Some(AppId::GraphicsLab),
        _ => None,
    }
}

fn encode_theme(theme: ThemeMode) -> u8 {
    match theme {
        ThemeMode::Dark => 0,
        ThemeMode::Light => 1,
    }
}

fn decode_theme(value: u8) -> ThemeMode {
    match value {
        1 => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

fn encode_render_strategy(strategy: RenderStrategy) -> u8 {
    match strategy {
        RenderStrategy::Quality => 0,
        RenderStrategy::Balanced => 1,
        RenderStrategy::Performance => 2,
    }
}

fn decode_render_strategy(value: u8) -> RenderStrategy {
    match value {
        0 => RenderStrategy::Quality,
        2 => RenderStrategy::Performance,
        _ => RenderStrategy::Balanced,
    }
}

fn fnv1a(bytes: &[u8]) -> u32 {
    let mut hash = 0x811C_9DC5u32;
    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn write_u16(buffer: &mut [u8], offset: usize, value: u16) {
    buffer[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_f32(buffer: &mut [u8], offset: usize, value: f32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(buffer: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buffer[offset], buffer[offset + 1]])
}

fn read_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

fn read_f32(buffer: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}
