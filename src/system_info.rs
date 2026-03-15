use crate::media;
use crate::storage_codec;

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

pub fn safe_mode_hint(zh_mode: bool) -> &'static str {
    if zh_mode {
        "開機時按住 K1 進入安全模式"
    } else {
        "HOLD K1 DURING BOOT FOR SAFE MODE"
    }
}
