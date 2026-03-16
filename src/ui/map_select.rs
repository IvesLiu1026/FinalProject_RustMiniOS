use crate::display::{color, palette, Display, ThemeMode};
use crate::dungeon::DungeonApp;

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width, render_nav_back,
};

pub fn render_map_select(display: &mut Display, map_index: usize, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 38);
    draw_shell_window(display, ui.cyan, &ui);
    draw_title_bar(
        display,
        if zh_mode {
            "選擇地圖"
        } else {
            "MAP SELECT"
        },
        if zh_mode {
            "dungeon cartridge loader"
        } else {
            "dungeon cartridge loader"
        },
        ui.cyan,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.orange, &ui);

    draw_info_strip(
        display,
        18,
        46,
        132,
        if zh_mode { "卡匣" } else { "CARTRIDGE" },
        if zh_mode {
            "地城裝載器"
        } else {
            "DUNGEON LOADER"
        },
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        164,
        46,
        138,
        if zh_mode { "數量" } else { "MAPS" },
        match DungeonApp::map_count() {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            _ => "6+",
        },
        ui.amber,
        &ui,
    );

    display.panel(18, 62, 98, 134, ui.panel, ui.cyan);
    draw_map_cartridge(display, 40, 78, &ui);
    display.text(
        28,
        142,
        if zh_mode {
            "選中地圖"
        } else {
            "SELECTED MAP"
        },
        ui.text,
        ui.panel,
        1,
    );
    display.text(
        28,
        156,
        &fit_text_to_width(display, DungeonApp::map_name(map_index, zh_mode), 72, 1),
        ui.text_muted,
        ui.panel,
        1,
    );
    display.fill_rect(28, 172, 78, 12, color::mix(ui.panel_alt, ui.cyan, 22));
    display.stroke_rect(28, 172, 78, 12, 1, ui.cyan);
    display.centered_text(
        67,
        175,
        if zh_mode { "K1 啟動" } else { "K1 START" },
        ui.text,
        color::mix(ui.panel_alt, ui.cyan, 22),
        1,
    );

    for idx in 0..DungeonApp::map_count() {
        let y = 70 + idx as u16 * 42;
        let selected = idx == map_index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let accent = if selected { ui.cyan } else { ui.steel };
        display.panel(126, y, 174, 34, fill, accent);
        display.fill_rect(136, y + 8, 16, 16, color::mix(fill, accent, 22));
        display.stroke_rect(136, y + 8, 16, 16, 1, accent);
        display.fill_rect(140, y + 12, 8, 8, ui.text);
        display.fill_rect(142, y + 14, 4, 4, color::mix(ui.panel_alt, ui.amber, 22));
        let map_name = fit_text_to_width(display, DungeonApp::map_name(idx, zh_mode), 78, 1);
        display.text(160, y + 10, &map_name, ui.text, fill, 1);
        let state_text = fit_text_to_width(
            display,
            if selected {
                if zh_mode {
                    "進入"
                } else {
                    "ENTER"
                }
            } else if zh_mode {
                "已選"
            } else {
                "SELECT"
            },
            36,
            1,
        );
        display.text(248, y + 12, &state_text, ui.text_muted, fill, 1);
    }

    draw_footer_hint(
        display,
        if zh_mode {
            "K1 START  K0+WK RETURNS TO GAME CENTER"
        } else {
            "K1 START  K0+WK RETURNS TO GAME CENTER"
        },
        ui.amber,
        &ui,
    );
}

fn draw_map_cartridge(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 52, 40, color::mix(ui.panel_alt, ui.cyan, 20));
    display.stroke_rect(x, y, 52, 40, 1, ui.cyan);
    display.fill_rect(x + 8, y + 6, 36, 18, ui.text);
    display.fill_rect(x + 12, y + 10, 28, 10, color::mix(ui.cyan, ui.white, 72));
    display.fill_rect(x + 18, y + 28, 16, 4, ui.amber);
    display.fill_rect(x + 6, y + 34, 40, 2, ui.shadow);
}
