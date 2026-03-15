use core::fmt::Write;

use heapless::String;

use crate::app_registry::{descriptor, home_apps};
use crate::display::{color, palette, Display, ThemeMode};

use super::draw_desktop_shortcut;

const ICON_X: [u16; 4] = [18, 108, 18, 108];
const ICON_Y: [u16; 4] = [42, 42, 110, 110];
const ICON_W: u16 = 76;
const ICON_H: u16 = 56;

pub fn desktop_icon_rect(index: usize) -> (u16, u16, u16, u16) {
    (ICON_X[index], ICON_Y[index], ICON_W, ICON_H)
}

pub fn render_home(
    display: &mut Display,
    home_index: usize,
    theme: ThemeMode,
    zh_mode: bool,
    fps: u16,
    uptime_seconds: u32,
) {
    let ui = palette(theme);
    draw_desktop_background(display, &ui);
    draw_top_bar(display, theme, zh_mode, fps, uptime_seconds, &ui);
    draw_desk_widgets(display, theme, zh_mode, &ui);

    for (index, app_id) in home_apps().iter().copied().enumerate() {
        let app = descriptor(app_id);
        let (x, y, width, height) = desktop_icon_rect(index);
        draw_desktop_shortcut(
            display,
            x,
            y,
            width,
            height,
            app.icon,
            app.desktop_label(zh_mode),
            app.accent.resolve(&ui),
            index == home_index,
            &ui,
        );
    }

    let selected = descriptor(home_apps()[home_index]);
    draw_selection_panel(
        display,
        selected.title(zh_mode),
        selected.subtitle(zh_mode),
        fps,
        selected.accent.resolve(&ui),
        &ui,
    );
    draw_taskbar(
        display,
        zh_mode,
        selected.title(zh_mode),
        selected.accent.resolve(&ui),
        &ui,
    );
}

fn draw_desktop_background(display: &mut Display, ui: &crate::display::Palette) {
    let desktop = color::rgb565(0, 128, 128);
    display.fill_rect(0, 0, 320, 240, desktop);

    for y in (28..194).step_by(22) {
        display.fill_rect(0, y, 320, 1, color::mix(desktop, ui.white, 16));
    }
    for x in (10..320).step_by(28) {
        display.fill_rect(x, 28, 1, 164, color::mix(desktop, ui.white, 10));
    }

    display.fill_rect(196, 32, 112, 138, color::mix(ui.canvas, ui.white, 16));
    display.stroke_rect(196, 32, 112, 138, 1, ui.white);
    display.stroke_rect(197, 33, 110, 136, 1, color::mix(ui.shadow, ui.canvas, 46));
    display.fill_rect(198, 34, 108, 5, color::mix(ui.white, ui.cyan, 12));
}

fn draw_top_bar(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    _fps: u16,
    uptime_seconds: u32,
    ui: &crate::display::Palette,
) {
    let bar_fill = match theme {
        ThemeMode::Light => color::rgb565(192, 192, 192),
        ThemeMode::Dark => color::rgb565(148, 148, 148),
    };
    let tray_fill = color::mix(bar_fill, ui.white, 18);

    display.fill_rect(0, 0, 320, 24, bar_fill);
    display.stroke_rect(0, 0, 320, 24, 1, ui.white);
    display.fill_rect(0, 23, 320, 2, color::mix(ui.shadow, bar_fill, 44));
    display.fill_rect(0, 24, 320, 1, color::mix(ui.white, bar_fill, 30));

    draw_menu_chip(
        display,
        4,
        4,
        58,
        "MiniOS",
        color::mix(ui.indigo, ui.shadow, 18),
        ui.white,
        ui,
    );
    draw_menu_chip(
        display,
        66,
        4,
        38,
        if zh_mode { "應用" } else { "APPS" },
        tray_fill,
        ui.text,
        ui,
    );
    draw_menu_chip(
        display,
        108,
        4,
        28,
        if zh_mode { "中" } else { "EN" },
        tray_fill,
        ui.text,
        ui,
    );
    draw_menu_chip(
        display,
        140,
        4,
        34,
        top_bar_theme_label(theme, zh_mode),
        tray_fill,
        ui.text,
        ui,
    );

    let hours = (uptime_seconds / 3600) % 100;
    let minutes = (uptime_seconds / 60) % 60;
    let mut clock_line: String<8> = String::new();
    let _ = write!(&mut clock_line, "{:02}:{:02}", hours, minutes);
    display.fill_rect(178, 4, 138, 16, tray_fill);
    display.stroke_rect(178, 4, 138, 16, 1, ui.steel);
    draw_status_lamp(display, 184, 10, ui.cyan, ui);
    draw_status_lamp(display, 192, 10, ui.amber, ui);
    draw_status_lamp(display, 200, 10, ui.lime, ui);
    draw_tray_icon_mail(display, 210, 4, tray_fill, ui);
    draw_tray_icon_disk(display, 226, 4, tray_fill, ui);
    display.fill_rect(244, 6, 1, 12, ui.steel);
    display.text(252, 9, &clock_line, ui.text, tray_fill, 1);
}

fn draw_menu_chip(
    display: &mut Display,
    x: u16,
    y: u16,
    width: u16,
    label: &str,
    fill: u16,
    text: u16,
    ui: &crate::display::Palette,
) {
    display.fill_rect(x, y, width, 16, fill);
    display.stroke_rect(x, y, width, 16, 1, ui.steel);
    display.text(x + 6, y + 5, label, text, fill, 1);
}

fn draw_status_lamp(
    display: &mut Display,
    x: u16,
    y: u16,
    accent: u16,
    ui: &crate::display::Palette,
) {
    display.fill_rect(x, y, 5, 5, accent);
    display.stroke_rect(x, y, 5, 5, 1, ui.white);
}

fn top_bar_theme_label(theme: ThemeMode, zh_mode: bool) -> &'static str {
    match (theme, zh_mode) {
        (ThemeMode::Light, true) => "日間",
        (ThemeMode::Dark, true) => "夜間",
        (ThemeMode::Light, false) => "DAY",
        (ThemeMode::Dark, false) => "NITE",
    }
}

fn draw_tray_icon_mail(
    display: &mut Display,
    x: u16,
    y: u16,
    bg: u16,
    ui: &crate::display::Palette,
) {
    display.fill_rect(x, y, 14, 16, bg);
    display.stroke_rect(x, y, 14, 16, 1, ui.steel);
    display.fill_rect(x + 3, y + 5, 8, 5, ui.white);
    display.stroke_rect(x + 3, y + 5, 8, 5, 1, ui.cyan);
    display.fill_rect(x + 4, y + 6, 3, 1, ui.cyan);
    display.fill_rect(x + 7, y + 6, 3, 1, ui.rose);
}

fn draw_tray_icon_disk(
    display: &mut Display,
    x: u16,
    y: u16,
    bg: u16,
    ui: &crate::display::Palette,
) {
    display.fill_rect(x, y, 14, 16, bg);
    display.stroke_rect(x, y, 14, 16, 1, ui.steel);
    display.fill_rect(x + 3, y + 4, 8, 7, ui.amber);
    display.stroke_rect(x + 3, y + 4, 8, 7, 1, ui.orange);
    display.fill_rect(x + 5, y + 6, 4, 2, ui.white);
    display.fill_rect(x + 6, y + 9, 2, 1, ui.orange);
}

fn draw_desk_widgets(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    ui: &crate::display::Palette,
) {
    let note_fill = color::mix(ui.white, ui.amber, 46);
    display.fill_rect(208, 42, 44, 24, note_fill);
    display.stroke_rect(208, 42, 44, 24, 1, ui.orange);
    display.text(216, 50, "TODO", ui.text, note_fill, 1);

    display.fill_rect(258, 42, 34, 18, color::mix(ui.panel_alt, ui.cyan, 28));
    display.stroke_rect(258, 42, 34, 18, 1, ui.cyan);
    display.text(
        266,
        48,
        "ROM",
        ui.text,
        color::mix(ui.panel_alt, ui.cyan, 28),
        1,
    );

    display.fill_rect(208, 74, 88, 16, color::mix(ui.panel, ui.canvas, 12));
    display.stroke_rect(208, 74, 88, 16, 1, ui.steel);
    display.text(
        216,
        79,
        if zh_mode { "桌面小物" } else { "DESK TOYS" },
        ui.text,
        color::mix(ui.panel, ui.canvas, 12),
        1,
    );

    draw_mini_plant(display, 208, 98, &ui);
    draw_mini_computer(display, 232, 98, theme, &ui);
    draw_mini_floppy(display, 270, 98, &ui);

    display.fill_rect(210, 138, 84, 14, color::mix(ui.white, ui.cyan, 20));
    display.stroke_rect(210, 138, 84, 14, 1, ui.cyan);
    display.text(
        218,
        143,
        if zh_mode { "收件匣 0" } else { "INBOX 0" },
        ui.text,
        color::mix(ui.white, ui.cyan, 20),
        1,
    );
}

fn draw_mini_plant(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x + 6, y + 14, 12, 8, ui.orange);
    display.stroke_rect(x + 6, y + 14, 12, 8, 1, ui.white);
    display.fill_rect(x + 10, y + 4, 4, 10, ui.lime);
    display.fill_rect(x + 6, y + 8, 4, 3, ui.lime);
    display.fill_rect(x + 14, y + 7, 4, 3, ui.lime);
    display.fill_rect(x + 8, y + 2, 2, 3, ui.lime);
    display.fill_rect(x + 14, y + 2, 2, 3, ui.lime);
}

fn draw_mini_computer(
    display: &mut Display,
    x: u16,
    y: u16,
    theme: ThemeMode,
    ui: &crate::display::Palette,
) {
    let shell = match theme {
        ThemeMode::Light => color::mix(ui.white, ui.panel, 16),
        ThemeMode::Dark => color::mix(ui.panel_alt, ui.white, 30),
    };
    display.fill_rect(x, y, 32, 24, shell);
    display.stroke_rect(x, y, 32, 24, 1, ui.white);
    display.stroke_rect(x + 1, y + 1, 30, 22, 1, color::mix(ui.shadow, shell, 44));
    display.fill_rect(x + 4, y + 4, 24, 12, color::mix(ui.cyan, ui.white, 74));
    display.stroke_rect(x + 4, y + 4, 24, 12, 1, ui.indigo);
    display.fill_rect(x + 9, y + 8, 3, 3, ui.text);
    display.fill_rect(x + 20, y + 8, 3, 3, ui.text);
    display.fill_rect(x + 12, y + 13, 8, 1, ui.rose);
    display.fill_rect(x + 11, y + 19, 10, 2, ui.shadow);
}

fn draw_mini_floppy(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 20, 22, ui.amber);
    display.stroke_rect(x, y, 20, 22, 1, ui.orange);
    display.fill_rect(x + 4, y + 4, 10, 6, ui.white);
    display.fill_rect(x + 6, y + 13, 8, 3, ui.orange);
    display.fill_rect(x + 14, y + 4, 2, 5, ui.orange);
}

fn draw_selection_panel(
    display: &mut Display,
    title: &str,
    subtitle: &str,
    fps: u16,
    accent: u16,
    ui: &crate::display::Palette,
) {
    let panel_fill = color::mix(ui.panel, ui.canvas, 20);
    display.fill_rect(12, 172, 296, 26, panel_fill);
    display.stroke_rect(12, 172, 296, 26, 1, ui.steel);
    display.fill_rect(16, 176, 70, 16, accent);
    display.stroke_rect(16, 176, 70, 16, 1, ui.white);
    display.text(22, 181, title, ui.white, accent, 1);
    display.fill_rect(92, 176, 126, 16, ui.panel_alt);
    display.stroke_rect(92, 176, 126, 16, 1, ui.steel);
    display.text(98, 181, subtitle, ui.text, ui.panel_alt, 1);
    let mut fps_line: String<12> = String::new();
    if fps == 0 {
        let _ = fps_line.push_str("-- FPS");
    } else {
        let _ = write!(&mut fps_line, "{} FPS", fps);
    }
    display.fill_rect(224, 176, 36, 16, ui.panel_alt);
    display.stroke_rect(224, 176, 36, 16, 1, ui.steel);
    let fps_width = display.measure_text(&fps_line, 1);
    let fps_x = 242u16.saturating_sub(fps_width / 2);
    display.text(fps_x, 181, &fps_line, ui.text_muted, ui.panel_alt, 1);
    display.fill_rect(266, 176, 34, 16, color::mix(ui.panel_alt, accent, 22));
    display.stroke_rect(266, 176, 34, 16, 1, accent);
    display.centered_text(
        283,
        181,
        "OPEN",
        ui.text,
        color::mix(ui.panel_alt, accent, 22),
        1,
    );
}

fn draw_taskbar(
    display: &mut Display,
    zh_mode: bool,
    selected_title: &str,
    accent: u16,
    ui: &crate::display::Palette,
) {
    let bar_fill = color::rgb565(192, 192, 192);
    let slot_fill = color::mix(bar_fill, ui.white, 18);
    display.fill_rect(0, 220, 320, 20, bar_fill);
    display.stroke_rect(0, 220, 320, 20, 1, ui.white);
    display.fill_rect(0, 239, 320, 1, color::mix(ui.shadow, bar_fill, 48));

    display.fill_rect(4, 223, 46, 14, color::mix(ui.white, ui.lime, 24));
    display.stroke_rect(4, 223, 46, 14, 1, ui.shadow);
    display.text(
        12,
        227,
        if zh_mode { "開始" } else { "START" },
        ui.text,
        color::mix(ui.white, ui.lime, 24),
        1,
    );

    draw_task_quick_icon(display, 58, 223, ui.cyan, ui);
    draw_task_quick_icon(display, 74, 223, ui.amber, ui);
    draw_task_quick_icon(display, 90, 223, ui.rose, ui);

    display.fill_rect(110, 223, 88, 14, slot_fill);
    display.stroke_rect(110, 223, 88, 14, 1, accent);
    display.text(118, 227, selected_title, ui.text, slot_fill, 1);

    display.fill_rect(204, 223, 50, 14, slot_fill);
    display.stroke_rect(204, 223, 50, 14, 1, ui.shadow);
    display.text(
        212,
        227,
        if zh_mode { "就緒" } else { "READY" },
        ui.text_muted,
        slot_fill,
        1,
    );

    display.fill_rect(260, 223, 56, 14, slot_fill);
    display.stroke_rect(260, 223, 56, 14, 1, ui.shadow);
    display.fill_rect(266, 227, 4, 4, accent);
    display.fill_rect(274, 227, 4, 4, ui.lime);
    display.text(284, 227, "4 APP", ui.text_muted, slot_fill, 1);
}

fn draw_task_quick_icon(
    display: &mut Display,
    x: u16,
    y: u16,
    accent: u16,
    ui: &crate::display::Palette,
) {
    let fill = color::mix(ui.white, accent, 24);
    display.fill_rect(x, y, 12, 14, fill);
    display.stroke_rect(x, y, 12, 14, 1, ui.shadow);
    display.fill_rect(x + 3, y + 4, 6, 6, accent);
}
