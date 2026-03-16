use core::fmt::Write;

use heapless::String;

use crate::display::{color, palette, Display, ThemeMode};
use crate::system_info;

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width, render_nav_back,
};

pub fn render_about(display: &mut Display, theme: ThemeMode, zh_mode: bool, safe_boot: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 112);
    draw_shell_window(display, ui.lime, &ui);
    draw_title_bar(
        display,
        if zh_mode { "關於系統" } else { "ABOUT" },
        if zh_mode {
            "version / build / media / hardware"
        } else {
            "version / build / media / hardware"
        },
        ui.lime,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.white, &ui);

    display.panel(18, 52, 284, 50, ui.panel_alt, ui.cyan);
    display.fill_rect(28, 62, 24, 18, ui.canvas);
    display.stroke_rect(28, 62, 24, 18, 1, ui.white);
    display.fill_rect(31, 65, 18, 9, color::mix(ui.cyan, ui.white, 74));
    display.stroke_rect(31, 65, 18, 9, 1, ui.indigo);
    display.fill_rect(36, 76, 8, 2, ui.shadow);
    display.text(62, 62, "MiniOS 95", ui.text, ui.panel_alt, 1);
    let hardware_line = fit_text_to_width(
        display,
        if zh_mode {
            "STM32F407ZG + ILI9341 + 電阻觸控"
        } else {
            "STM32F407ZG + ILI9341 + RESISTIVE TOUCH"
        },
        164,
        1,
    );
    display.text(62, 76, &hardware_line, ui.text_muted, ui.panel_alt, 1);
    let build_target = fit_text_to_width(display, system_info::build_target(), 164, 1);
    display.text(62, 88, &build_target, ui.text_muted, ui.panel_alt, 1);
    draw_about_lamps(display, 248, 62, safe_boot, &ui);

    let mut version_line: String<48> = String::new();
    let _ = write!(
        &mut version_line,
        "v{}  {}",
        system_info::app_version(),
        system_info::git_sha()
    );
    draw_info_strip(
        display,
        18,
        108,
        136,
        if zh_mode { "版本" } else { "VERSION" },
        &version_line,
        ui.amber,
        &ui,
    );
    draw_info_strip(
        display,
        166,
        108,
        136,
        if zh_mode { "模式" } else { "SESSION" },
        &{
            let mut session_line: String<24> = String::new();
            let _ = write!(
                &mut session_line,
                "{} {}",
                if safe_boot { "SAFE" } else { "NORMAL" },
                system_info::build_profile()
            );
            session_line
        },
        if safe_boot { ui.amber } else { ui.lime },
        &ui,
    );

    display.panel(18, 128, 136, 78, ui.panel, ui.orange);
    let media_title =
        fit_text_to_width(display, if zh_mode { "媒體庫" } else { "MEDIA LIB" }, 74, 1);
    display.text(28, 140, &media_title, ui.text, ui.panel, 1);
    draw_media_shelf_icon(display, 114, 138, &ui);
    let mut still_line: String<24> = String::new();
    let _ = write!(&mut still_line, "{}", system_info::still_count());
    draw_info_strip(
        display,
        28,
        160,
        116,
        if zh_mode { "圖片" } else { "STILLS" },
        &still_line,
        ui.cyan,
        &ui,
    );
    let mut motion_line: String<24> = String::new();
    let _ = write!(&mut motion_line, "{}", system_info::motion_clip_count());
    draw_info_strip(
        display,
        28,
        178,
        116,
        if zh_mode { "動圖" } else { "MOTION" },
        &motion_line,
        ui.rose,
        &ui,
    );

    display.panel(166, 128, 136, 78, ui.panel, ui.rose);
    let hardware_title = fit_text_to_width(
        display,
        if zh_mode { "機器資訊" } else { "HARDWARE" },
        74,
        1,
    );
    display.text(176, 140, &hardware_title, ui.text, ui.panel, 1);
    draw_chip_icon(display, 264, 138, &ui);
    draw_info_strip(
        display,
        176,
        160,
        116,
        if zh_mode { "儲存" } else { "STORE" },
        &{
            let mut s: String<24> = String::new();
            let _ = write!(&mut s, "{}B", system_info::storage_record_bytes());
            s
        },
        ui.amber,
        &ui,
    );
    draw_info_strip(
        display,
        176,
        178,
        116,
        if zh_mode { "相簿" } else { "ALBUM" },
        if system_info::album_backend() == "mac-companion" {
            "MAC LINK"
        } else {
            "EMBEDDED"
        },
        ui.cyan,
        &ui,
    );

    draw_footer_hint(display, system_info::safe_mode_hint(zh_mode), ui.white, &ui);
}

fn draw_about_lamps(
    display: &mut Display,
    x: u16,
    y: u16,
    safe_boot: bool,
    ui: &crate::display::Palette,
) {
    display.fill_rect(x, y, 42, 16, ui.panel);
    display.stroke_rect(x, y, 42, 16, 1, ui.steel);
    display.fill_rect(x + 6, y + 6, 5, 5, ui.cyan);
    display.stroke_rect(x + 6, y + 6, 5, 5, 1, ui.white);
    display.fill_rect(
        x + 18,
        y + 6,
        5,
        5,
        if safe_boot { ui.amber } else { ui.lime },
    );
    display.stroke_rect(x + 18, y + 6, 5, 5, 1, ui.white);
    display.fill_rect(x + 30, y + 6, 5, 5, ui.rose);
    display.stroke_rect(x + 30, y + 6, 5, 5, 1, ui.white);
}

fn draw_media_shelf_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 24, 18, color::mix(ui.panel_alt, ui.amber, 18));
    display.stroke_rect(x, y, 24, 18, 1, ui.orange);
    display.fill_rect(x + 3, y + 3, 9, 7, ui.white);
    display.stroke_rect(x + 3, y + 3, 9, 7, 1, ui.cyan);
    display.fill_rect(x + 5, y + 5, 5, 3, color::mix(ui.cyan, ui.white, 80));
    display.fill_rect(x + 14, y + 4, 7, 9, ui.text);
    display.fill_rect(x + 16, y + 6, 3, 3, color::mix(ui.panel_alt, ui.rose, 20));
    display.fill_rect(x + 2, y + 14, 20, 1, ui.steel);
}

fn draw_chip_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 22, 18, color::mix(ui.panel_alt, ui.rose, 18));
    display.stroke_rect(x, y, 22, 18, 1, ui.rose);
    display.fill_rect(x + 5, y + 4, 12, 10, ui.text);
    display.fill_rect(x + 8, y + 7, 6, 4, ui.white);
    let mut pin_x = x + 3;
    while pin_x <= x + 17 {
        display.fill_rect(pin_x, y + 1, 1, 2, ui.amber);
        display.fill_rect(pin_x, y + 15, 1, 2, ui.amber);
        pin_x += 3;
    }
}
