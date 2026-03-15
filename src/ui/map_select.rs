use crate::display::{palette, Display, ThemeMode};
use crate::dungeon::DungeonApp;

use super::{draw_gradient_background, render_nav_back};

pub fn render_map_select(display: &mut Display, map_index: usize, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 38);
    display.panel(16, 12, 288, 34, ui.panel, ui.cyan);
    render_nav_back(display, zh_mode, ui.orange, &ui);
    display.text(
        74,
        20,
        if zh_mode {
            "選擇地圖"
        } else {
            "MAP SELECT"
        },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        186,
        22,
        if zh_mode { "K1 開始" } else { "K1 START" },
        ui.text_muted,
        ui.panel,
        1,
    );

    for idx in 0..DungeonApp::map_count() {
        let y = 72 + idx as u16 * 44;
        let selected = idx == map_index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let accent = if selected { ui.cyan } else { ui.steel };
        display.panel(20, y, 280, 36, fill, accent);
        display.text(
            30,
            y + 10,
            DungeonApp::map_name(idx, zh_mode),
            ui.text,
            fill,
            2,
        );
        display.text(
            202,
            y + 14,
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
            ui.text_muted,
            fill,
            1,
        );
    }

    display.panel(22, 208, 276, 20, ui.panel, ui.amber);
    display.text(
        30,
        214,
        if zh_mode {
            "按 K0+WK 返回遊戲中心"
        } else {
            "PRESS K0+WK TO RETURN GAME CENTER"
        },
        ui.text,
        ui.panel,
        1,
    );
}
