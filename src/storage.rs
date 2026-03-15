use core::ptr::{read_volatile, write_volatile};

use cortex_m::interrupt;

use crate::app_registry::AppId;
use crate::display::ThemeMode;
use crate::dungeon::RenderStrategy;
use crate::storage_codec::{self, STORAGE_BYTES};
pub use crate::storage_codec::{StorageStatus, PAINT_STORAGE_BYTES};
use crate::touch::TouchCalibration;

const STORAGE_BASE: usize = 0x080E_0000;

const FLASH_REG_BASE: usize = 0x4002_3C00;
const FLASH_KEYR: *mut u32 = (FLASH_REG_BASE + 0x04) as *mut u32;
const FLASH_SR: *mut u32 = (FLASH_REG_BASE + 0x0C) as *mut u32;
const FLASH_CR: *mut u32 = (FLASH_REG_BASE + 0x10) as *mut u32;

const FLASH_KEY1: u32 = 0x4567_0123;
const FLASH_KEY2: u32 = 0xCDEF_89AB;

const FLASH_SR_EOP: u32 = 1 << 0;
const FLASH_SR_OPERR: u32 = 1 << 1;
const FLASH_SR_WRPERR: u32 = 1 << 4;
const FLASH_SR_PGAERR: u32 = 1 << 5;
const FLASH_SR_PGPERR: u32 = 1 << 6;
const FLASH_SR_PGSERR: u32 = 1 << 7;
const FLASH_SR_BSY: u32 = 1 << 16;
const FLASH_STATUS_ERRORS: u32 = FLASH_SR_EOP
    | FLASH_SR_OPERR
    | FLASH_SR_WRPERR
    | FLASH_SR_PGAERR
    | FLASH_SR_PGPERR
    | FLASH_SR_PGSERR;

const FLASH_CR_PG: u32 = 1 << 0;
const FLASH_CR_SER: u32 = 1 << 1;
const FLASH_CR_SNB_SHIFT: u32 = 3;
const FLASH_CR_PSIZE_SHIFT: u32 = 8;
const FLASH_CR_STRT: u32 = 1 << 16;
const FLASH_CR_LOCK: u32 = 1 << 31;
const FLASH_PSIZE_X32: u32 = 0b10 << FLASH_CR_PSIZE_SHIFT;
const STORAGE_SECTOR: u32 = 11;

#[derive(Clone, Copy)]
pub struct PersistedSystemSettings {
    pub theme: ThemeMode,
    pub language_zh: bool,
    pub render_strategy: RenderStrategy,
    pub touch_ready: bool,
    pub touch_calibration: TouchCalibration,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PersistedAppData {
    pub recent_app: Option<AppId>,
    pub album_motion_tab: bool,
    pub album_still_index: u16,
    pub album_motion_index: u16,
    pub album_playing: bool,
    pub paint_selected_color: u8,
    pub paint_pixels: [u8; PAINT_STORAGE_BYTES],
    pub auto_battle_best_kills: u16,
    pub tap_rush_best_score: u16,
}

#[derive(Clone, Copy)]
pub struct PersistedState {
    pub system: PersistedSystemSettings,
    pub apps: PersistedAppData,
}

pub fn load() -> Option<PersistedState> {
    let bytes = unsafe { core::slice::from_raw_parts(STORAGE_BASE as *const u8, STORAGE_BYTES) };
    storage_codec::decode(bytes)
}

pub fn save(state: &PersistedState) -> bool {
    let bytes = storage_codec::encode(state);
    interrupt::free(|_| unsafe {
        if !flash_unlock() {
            return false;
        }
        let result = flash_erase_sector(STORAGE_SECTOR)
            && flash_program_words(STORAGE_BASE as *mut u32, &bytes)
            && flash_verify(STORAGE_BASE as *const u8, &bytes);
        flash_lock();
        result
    })
}

pub fn erase_all() -> bool {
    interrupt::free(|_| unsafe {
        if !flash_unlock() {
            return false;
        }
        let result = flash_erase_sector(STORAGE_SECTOR);
        flash_lock();
        result
    })
}

pub fn inspect() -> StorageStatus {
    let bytes = unsafe { core::slice::from_raw_parts(STORAGE_BASE as *const u8, STORAGE_BYTES) };
    storage_codec::inspect_bytes(bytes)
}

#[allow(dead_code)]
pub fn default_app_data() -> PersistedAppData {
    storage_codec::default_app_data()
}

unsafe fn flash_wait_ready() -> bool {
    while read_volatile(FLASH_SR) & FLASH_SR_BSY != 0 {}
    (read_volatile(FLASH_SR)
        & (FLASH_SR_OPERR | FLASH_SR_WRPERR | FLASH_SR_PGAERR | FLASH_SR_PGPERR | FLASH_SR_PGSERR))
        == 0
}

unsafe fn flash_clear_status() {
    write_volatile(FLASH_SR, FLASH_STATUS_ERRORS);
}

unsafe fn flash_unlock() -> bool {
    if read_volatile(FLASH_CR) & FLASH_CR_LOCK == 0 {
        return true;
    }
    write_volatile(FLASH_KEYR, FLASH_KEY1);
    write_volatile(FLASH_KEYR, FLASH_KEY2);
    read_volatile(FLASH_CR) & FLASH_CR_LOCK == 0
}

unsafe fn flash_lock() {
    write_volatile(FLASH_CR, read_volatile(FLASH_CR) | FLASH_CR_LOCK);
}

unsafe fn flash_erase_sector(sector: u32) -> bool {
    if !flash_wait_ready() {
        return false;
    }
    flash_clear_status();

    let mut cr = read_volatile(FLASH_CR);
    cr &= !((0xF << FLASH_CR_SNB_SHIFT) | FLASH_CR_PG);
    cr |= FLASH_PSIZE_X32 | FLASH_CR_SER | (sector << FLASH_CR_SNB_SHIFT);
    write_volatile(FLASH_CR, cr);
    write_volatile(FLASH_CR, cr | FLASH_CR_STRT);
    let ok = flash_wait_ready();

    let mut clear = read_volatile(FLASH_CR);
    clear &= !((0xF << FLASH_CR_SNB_SHIFT) | FLASH_CR_SER | FLASH_CR_PG);
    write_volatile(FLASH_CR, clear);
    flash_clear_status();
    ok
}

unsafe fn flash_program_words(base: *mut u32, bytes: &[u8; STORAGE_BYTES]) -> bool {
    let mut offset = 0usize;
    while offset < bytes.len() {
        let word = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);

        if !flash_wait_ready() {
            return false;
        }
        flash_clear_status();
        let mut cr = read_volatile(FLASH_CR);
        cr &= !((0xF << FLASH_CR_SNB_SHIFT) | FLASH_CR_SER);
        cr |= FLASH_PSIZE_X32 | FLASH_CR_PG;
        write_volatile(FLASH_CR, cr);
        write_volatile(base.add(offset / 4), word);
        if !flash_wait_ready() {
            return false;
        }

        let mut clear = read_volatile(FLASH_CR);
        clear &= !FLASH_CR_PG;
        write_volatile(FLASH_CR, clear);
        offset += 4;
    }
    flash_clear_status();
    true
}

unsafe fn flash_verify(base: *const u8, bytes: &[u8; STORAGE_BYTES]) -> bool {
    for (index, expected) in bytes.iter().enumerate() {
        if read_volatile(base.add(index)) != *expected {
            return false;
        }
    }
    true
}
