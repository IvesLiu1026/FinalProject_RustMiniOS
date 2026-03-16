use crate::display::{color, palette, Display, ThemeMode};
use crate::dungeon::RenderStrategy;

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_scrollbar, draw_shell_window,
    draw_title_bar, fit_text_to_width, render_nav_back, theme_mode_label, SHELL_CONTENT_X,
};

pub const SETTINGS_VIEWPORT_X: u16 = SHELL_CONTENT_X;
pub const SETTINGS_VIEWPORT_Y: u16 = 66;
pub const SETTINGS_VIEWPORT_W: u16 = 272;
pub const SETTINGS_VIEWPORT_H: u16 = 140;
pub const SETTINGS_SCROLLBAR_X: u16 = SETTINGS_VIEWPORT_X + SETTINGS_VIEWPORT_W + 4;
pub const SETTINGS_ROW_HEIGHT: u16 = 24;
pub const SETTINGS_VISIBLE_ROWS: usize = 5;
pub const SETTINGS_TOTAL_ROWS: usize = 12;

pub fn settings_max_scroll_top() -> usize {
    SETTINGS_TOTAL_ROWS.saturating_sub(SETTINGS_VISIBLE_ROWS)
}

pub fn settings_visual_row_for_item(item_index: usize) -> usize {
    match item_index {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 5,
        4 => 6,
        5 => 7,
        6 => 9,
        7 => 10,
        _ => 11,
    }
}

pub fn settings_item_for_visual_row(visual_row: usize) -> Option<usize> {
    match visual_row {
        1 => Some(0),
        2 => Some(1),
        3 => Some(2),
        5 => Some(3),
        6 => Some(4),
        7 => Some(5),
        9 => Some(6),
        10 => Some(7),
        11 => Some(8),
        _ => None,
    }
}

pub fn settings_list_contains(x: u16, y: u16) -> bool {
    x >= SETTINGS_VIEWPORT_X
        && x < SETTINGS_VIEWPORT_X.saturating_add(SETTINGS_VIEWPORT_W)
        && y >= SETTINGS_VIEWPORT_Y + 20
        && y < SETTINGS_VIEWPORT_Y + 20 + SETTINGS_VISIBLE_ROWS as u16 * SETTINGS_ROW_HEIGHT
}

pub fn settings_item_at_point(x: u16, y: u16, scroll_top_row: usize) -> Option<usize> {
    if !settings_list_contains(x, y) {
        return None;
    }
    let row_in_view = ((y - (SETTINGS_VIEWPORT_Y + 20)) / SETTINGS_ROW_HEIGHT) as usize;
    let visual_row = scroll_top_row + row_in_view;
    settings_item_for_visual_row(visual_row)
}

pub fn render_settings(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    render_strategy: RenderStrategy,
    selected_index: usize,
    scroll_top_row: usize,
) {
    let ui = palette(theme);
    let accent = ui.orange;
    draw_gradient_background(display, theme, 84);
    draw_shell_window(display, accent, &ui);
    draw_title_bar(
        display,
        if zh_mode { "系統設定" } else { "SETTINGS" },
        if zh_mode {
            "control panel / appearance / tools"
        } else {
            "control panel / appearance / tools"
        },
        accent,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.white, &ui);

    draw_info_strip(
        display,
        SHELL_CONTENT_X,
        46,
        136,
        if zh_mode { "主題" } else { "THEME" },
        theme_mode_label(theme, zh_mode),
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        160,
        46,
        74,
        if zh_mode { "語言" } else { "LANG" },
        if zh_mode { "繁中" } else { "EN" },
        ui.rose,
        &ui,
    );
    draw_info_strip(
        display,
        238,
        46,
        64,
        if zh_mode { "捲動" } else { "SCROLL" },
        if zh_mode { "列對齊" } else { "SNAP" },
        ui.amber,
        &ui,
    );

    display.fill_rect(
        SETTINGS_VIEWPORT_X,
        SETTINGS_VIEWPORT_Y,
        SETTINGS_VIEWPORT_W,
        SETTINGS_VIEWPORT_H,
        ui.panel_alt,
    );
    display.stroke_rect(
        SETTINGS_VIEWPORT_X,
        SETTINGS_VIEWPORT_Y,
        SETTINGS_VIEWPORT_W,
        SETTINGS_VIEWPORT_H,
        1,
        ui.steel,
    );
    display.fill_rect(
        SETTINGS_VIEWPORT_X + 4,
        SETTINGS_VIEWPORT_Y + 4,
        SETTINGS_VIEWPORT_W - 8,
        16,
        ui.panel,
    );
    display.stroke_rect(
        SETTINGS_VIEWPORT_X + 4,
        SETTINGS_VIEWPORT_Y + 4,
        SETTINGS_VIEWPORT_W - 8,
        16,
        1,
        ui.steel,
    );
    let header_left = fit_text_to_width(
        display,
        if zh_mode {
            "控制台項目"
        } else {
            "CONTROL PANEL ITEMS"
        },
        132,
        1,
    );
    let header_right =
        fit_text_to_width(display, if zh_mode { "K1 套用" } else { "K1 APPLY" }, 66, 1);
    display.text(
        SETTINGS_VIEWPORT_X + 10,
        SETTINGS_VIEWPORT_Y + 8,
        &header_left,
        ui.text,
        ui.panel,
        1,
    );
    display.text(
        SETTINGS_VIEWPORT_X + 224,
        SETTINGS_VIEWPORT_Y + 8,
        &header_right,
        ui.text_muted,
        ui.panel,
        1,
    );

    for row_in_view in 0..SETTINGS_VISIBLE_ROWS {
        let visual_row = scroll_top_row + row_in_view;
        let y = SETTINGS_VIEWPORT_Y + 20 + row_in_view as u16 * SETTINGS_ROW_HEIGHT;
        render_settings_row(
            display,
            y,
            visual_row,
            theme,
            zh_mode,
            render_strategy,
            selected_index,
            &ui,
        );
    }

    draw_scrollbar(
        display,
        SETTINGS_SCROLLBAR_X,
        SETTINGS_VIEWPORT_Y + 20,
        SETTINGS_VISIBLE_ROWS as u16 * SETTINGS_ROW_HEIGHT,
        scroll_top_row,
        SETTINGS_VISIBLE_ROWS,
        SETTINGS_TOTAL_ROWS,
        accent,
        &ui,
    );

    draw_footer_hint(
        display,
        if zh_mode {
            "列對齊捲動  可拖曳或用 K0/WK 切換"
        } else {
            "ROW-SNAPPED SCROLL  DRAG OR USE K0/WK"
        },
        accent,
        &ui,
    );
}

fn render_settings_row(
    display: &mut Display,
    y: u16,
    visual_row: usize,
    theme: ThemeMode,
    zh_mode: bool,
    render_strategy: RenderStrategy,
    selected_index: usize,
    ui: &crate::display::Palette,
) {
    if let Some((title, accent)) = section_for_row(visual_row, zh_mode) {
        let accent = match accent {
            1 => ui.cyan,
            2 => ui.amber,
            _ => ui.rose,
        };
        display.fill_rect(
            SETTINGS_VIEWPORT_X + 6,
            y + 4,
            SETTINGS_VIEWPORT_W - 12,
            16,
            ui.panel,
        );
        display.stroke_rect(
            SETTINGS_VIEWPORT_X + 6,
            y + 4,
            SETTINGS_VIEWPORT_W - 12,
            16,
            1,
            accent,
        );
        display.fill_rect(SETTINGS_VIEWPORT_X + 10, y + 7, 12, 10, accent);
        display.stroke_rect(SETTINGS_VIEWPORT_X + 10, y + 7, 12, 10, 1, ui.white);
        display.text(SETTINGS_VIEWPORT_X + 30, y + 9, title, ui.text, ui.panel, 1);
        return;
    }

    if let Some(item_index) = settings_item_for_visual_row(visual_row) {
        let selected = item_index == selected_index;
        let (title, detail, accent) = setting_row(item_index, theme, zh_mode, render_strategy, ui);
        let fill = if selected { ui.panel } else { ui.panel_alt };
        let border = if selected { accent } else { ui.steel };
        display.fill_rect(
            SETTINGS_VIEWPORT_X + 6,
            y + 2,
            SETTINGS_VIEWPORT_W - 12,
            20,
            fill,
        );
        display.stroke_rect(
            SETTINGS_VIEWPORT_X + 6,
            y + 2,
            SETTINGS_VIEWPORT_W - 12,
            20,
            1,
            border,
        );
        display.fill_rect(SETTINGS_VIEWPORT_X + 10, y + 5, 10, 14, accent);
        display.stroke_rect(SETTINGS_VIEWPORT_X + 10, y + 5, 10, 14, 1, ui.white);
        if selected {
            display.fill_rect(SETTINGS_VIEWPORT_X + 22, y + 4, 2, 16, accent);
        }
        let title_text = fit_text_to_width(display, title, 140, 1);
        display.text(
            SETTINGS_VIEWPORT_X + 30,
            y + 8,
            &title_text,
            ui.text,
            fill,
            1,
        );
        display.fill_rect(
            SETTINGS_VIEWPORT_X + 182,
            y + 6,
            72,
            12,
            color_box(fill, accent, ui),
        );
        display.stroke_rect(SETTINGS_VIEWPORT_X + 182, y + 6, 72, 12, 1, accent);
        let detail_text = fit_text_to_width(display, detail, 58, 1);
        let detail_width = display.measure_text(&detail_text, 1);
        let detail_x = SETTINGS_VIEWPORT_X + 182 + 36 - detail_width / 2;
        display.text(
            detail_x,
            y + 9,
            &detail_text,
            ui.text,
            color_box(fill, accent, ui),
            1,
        );
        display.text(SETTINGS_VIEWPORT_X + 258, y + 8, ">", border, fill, 1);
    }
}

fn color_box(fill: u16, accent: u16, ui: &crate::display::Palette) -> u16 {
    color::mix(fill, accent, if accent == ui.white { 28 } else { 22 })
}

fn section_for_row(visual_row: usize, zh_mode: bool) -> Option<(&'static str, u16)> {
    match visual_row {
        0 => Some((if zh_mode { "外觀" } else { "APPEARANCE" }, 1)),
        4 => Some((if zh_mode { "系統" } else { "SYSTEM" }, 2)),
        8 => Some((
            if zh_mode {
                "修復與資訊"
            } else {
                "RECOVERY / INFO"
            },
            3,
        )),
        _ => None,
    }
}

fn setting_row(
    item_index: usize,
    theme: ThemeMode,
    zh_mode: bool,
    render_strategy: RenderStrategy,
    ui: &crate::display::Palette,
) -> (&'static str, &'static str, u16) {
    match item_index {
        0 => (
            if zh_mode {
                "主題模式"
            } else {
                "THEME MODE"
            },
            match (theme, zh_mode) {
                (ThemeMode::Light, true) => "日間",
                (ThemeMode::Dark, true) => "夜間",
                (ThemeMode::Light, false) => "DAY",
                (ThemeMode::Dark, false) => "NITE",
            },
            ui.cyan,
        ),
        1 => (
            if zh_mode { "語言" } else { "LANGUAGE" },
            if zh_mode { "繁中" } else { "EN" },
            ui.rose,
        ),
        2 => (
            if zh_mode {
                "渲染策略"
            } else {
                "RENDER STRATEGY"
            },
            render_strategy.label(),
            ui.lime,
        ),
        3 => (
            if zh_mode { "控制室" } else { "CONTROL ROOM" },
            if zh_mode { "LED+IO" } else { "LED+IO" },
            ui.amber,
        ),
        4 => (
            if zh_mode {
                "觸控校正"
            } else {
                "TOUCH CALIBRATION"
            },
            if zh_mode { "精靈" } else { "WIZARD" },
            ui.orange,
        ),
        5 => (
            if zh_mode {
                "展示模式"
            } else {
                "SHOWCASE MODE"
            },
            if zh_mode { "輪播" } else { "AUTO" },
            ui.amber,
        ),
        6 => (
            if zh_mode {
                "效能儀表"
            } else {
                "PERFORMANCE"
            },
            if zh_mode { "監看" } else { "LIVE" },
            ui.lime,
        ),
        7 => (
            if zh_mode {
                "系統診斷"
            } else {
                "DIAGNOSTICS"
            },
            if zh_mode { "檢查" } else { "CHECK" },
            ui.white,
        ),
        _ => (
            if zh_mode { "關於系統" } else { "ABOUT" },
            if zh_mode { "版本" } else { "INFO" },
            ui.cyan,
        ),
    }
}
