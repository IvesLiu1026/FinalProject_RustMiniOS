use crate::app_registry::AppId;
use crate::display::ThemeMode;
use crate::dungeon::RenderStrategy;
use crate::storage::{PersistedAppData, PersistedState, PersistedSystemSettings};
use crate::touch::TouchCalibration;

pub const PAINT_STORAGE_BYTES: usize = 24 * 20;
pub const STORAGE_BYTES: usize = 552;

const CHECKSUM_OFFSET: usize = STORAGE_BYTES - 4;
const MAGIC: u32 = 0x4D4F_5332; // "MOS2"
const VERSION: u16 = 2;

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

pub fn default_app_data() -> PersistedAppData {
    PersistedAppData {
        recent_app: None,
        album_motion_tab: false,
        album_still_index: 0,
        album_motion_index: 0,
        album_playing: true,
        paint_selected_color: 1,
        paint_pixels: [0; PAINT_STORAGE_BYTES],
        auto_battle_best_kills: 0,
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
    write_u16(&mut out, 16, state.apps.auto_battle_best_kills);
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
    out[64..64 + PAINT_STORAGE_BYTES].copy_from_slice(&state.apps.paint_pixels);

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
    paint_pixels.copy_from_slice(&bytes[64..64 + PAINT_STORAGE_BYTES]);

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
            auto_battle_best_kills: read_u16(bytes, 16),
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
