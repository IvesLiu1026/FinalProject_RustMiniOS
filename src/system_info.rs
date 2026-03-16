use crate::media;
use crate::storage_codec;

const FLASH_ORIGIN: usize = 0x0800_0000;
const PROGRAM_FLASH_BYTES: usize = 896 * 1024;
const SRAM_BYTES: usize = 128 * 1024;

unsafe extern "C" {
    static __sidata: u8;
    static __sdata: u8;
    static __edata: u8;
    static __sbss: u8;
    static __ebss: u8;
}

pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn git_sha() -> &'static str {
    option_env!("MINIOS_GIT_SHA").unwrap_or("nogit")
}

pub fn build_profile() -> &'static str {
    option_env!("MINIOS_BUILD_PROFILE").unwrap_or("unknown")
}

pub fn build_target() -> &'static str {
    option_env!("MINIOS_BUILD_TARGET").unwrap_or("unknown-target")
}

pub fn album_backend() -> &'static str {
    option_env!("MINIOS_ALBUM_BACKEND").unwrap_or("embedded")
}

pub fn still_count() -> usize {
    media::stills().len()
}

pub fn motion_clip_count() -> usize {
    media::motion_clips().len()
}

pub fn storage_record_bytes() -> usize {
    storage_codec::STORAGE_BYTES
}

pub fn flash_used_bytes() -> usize {
    text_rodata_bytes().saturating_add(data_bytes())
}

pub fn flash_capacity_bytes() -> usize {
    PROGRAM_FLASH_BYTES
}

pub fn text_rodata_bytes() -> usize {
    unsafe { (&__sidata as *const u8 as usize).saturating_sub(FLASH_ORIGIN) }
}

pub fn data_bytes() -> usize {
    unsafe { (&__edata as *const u8 as usize).saturating_sub(&__sdata as *const u8 as usize) }
}

pub fn bss_bytes() -> usize {
    unsafe { (&__ebss as *const u8 as usize).saturating_sub(&__sbss as *const u8 as usize) }
}

pub fn ram_capacity_bytes() -> usize {
    SRAM_BYTES
}

pub fn safe_mode_hint(zh_mode: bool) -> &'static str {
    if zh_mode {
        "開機時按住 K1 進入安全模式"
    } else {
        "HOLD K1 DURING BOOT FOR SAFE MODE"
    }
}
