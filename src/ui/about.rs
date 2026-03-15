use core::fmt::Write;

use heapless::String;

use crate::display::{palette, Display, ThemeMode};
use crate::system_info;

use super::{draw_gradient_background, render_nav_back};

pub fn render_about(display: &mut Display, theme: ThemeMode, zh_mode: bool, safe_boot: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 112);
    display.panel(16, 12, 288, 34, ui.panel, ui.lime);
    render_nav_back(display, zh_mode, ui.white, &ui);
    display.text(
        74,
        20,
        if zh_mode { "關於系統" } else { "ABOUT" },
        ui.text,
        ui.panel,
        2,
    );

    display.panel(18, 56, 284, 70, ui.panel_alt, ui.cyan);
    display.text(28, 66, "FINALPROJECT_RUSTMINIOS", ui.text, ui.panel_alt, 1);
    display.text(
        28,
        82,
        if zh_mode {
            "STM32F407ZG + ILI9341 + 電阻觸控"
        } else {
            "STM32F407ZG + ILI9341 + RESISTIVE TOUCH"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    let mut version_line: String<48> = String::new();
    let _ = write!(
        &mut version_line,
        "v{}  {}  {}",
        system_info::app_version(),
        system_info::git_sha(),
        system_info::build_profile()
    );
    display.text(28, 98, &version_line, ui.amber, ui.panel_alt, 1);
    display.text(
        28,
        112,
        system_info::build_target(),
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.panel(18, 136, 136, 66, ui.panel, ui.orange);
    display.text(
        28,
        146,
        if zh_mode { "資產摘要" } else { "MEDIA" },
        ui.text,
        ui.panel,
        2,
    );
    let mut still_line: String<24> = String::new();
    let _ = write!(
        &mut still_line,
        "{} {}",
        if zh_mode { "圖片" } else { "STILLS" },
        system_info::still_count()
    );
    display.text(28, 168, &still_line, ui.text_muted, ui.panel, 1);
    let mut motion_line: String<24> = String::new();
    let _ = write!(
        &mut motion_line,
        "{} {}",
        if zh_mode { "動態" } else { "MOTION" },
        system_info::motion_clip_count()
    );
    display.text(28, 182, &motion_line, ui.text_muted, ui.panel, 1);
    display.text(
        28,
        194,
        if zh_mode {
            if system_info::album_backend() == "mac-companion" {
                "來源 Mac Companion"
            } else {
                "來源 內建媒體"
            }
        } else if system_info::album_backend() == "mac-companion" {
            "BACKEND MAC COMPANION"
        } else {
            "BACKEND EMBEDDED"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    display.panel(166, 136, 136, 66, ui.panel, ui.rose);
    display.text(
        176,
        146,
        if zh_mode { "系統摘要" } else { "SYSTEM" },
        ui.text,
        ui.panel,
        2,
    );
    let mut storage_line: String<24> = String::new();
    let _ = write!(
        &mut storage_line,
        "{} {}B",
        if zh_mode { "存檔區" } else { "STORE" },
        system_info::storage_record_bytes()
    );
    display.text(176, 168, &storage_line, ui.text_muted, ui.panel, 1);
    display.text(
        176,
        182,
        if safe_boot {
            if zh_mode {
                "本次: 安全模式"
            } else {
                "SESSION: SAFE MODE"
            }
        } else if zh_mode {
            "本次: 正常開機"
        } else {
            "SESSION: NORMAL BOOT"
        },
        if safe_boot { ui.amber } else { ui.text_muted },
        ui.panel,
        1,
    );

    display.panel(18, 212, 284, 22, ui.panel_alt, ui.white);
    display.text(
        28,
        220,
        system_info::safe_mode_hint(zh_mode),
        ui.text,
        ui.panel_alt,
        1,
    );
}
