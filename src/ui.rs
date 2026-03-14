use core::fmt::Write;

use heapless::String;

use crate::board::{millis, Board};
use crate::display::{color, palette, Display, Palette, ThemeMode, SCREEN_WIDTH};
use crate::dungeon::DungeonApp;
use crate::dungeon::RenderStrategy;

pub fn render_home(display: &mut Display, home_index: usize, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 12);

    display.panel(16, 12, 288, 48, ui.panel, ui.cyan);
    display.text(26, 22, "RUST MINI OS", ui.text, ui.panel, 3);
    display.text(
        28,
        44,
        if zh_mode {
            "多地圖地城與系統中心"
        } else {
            "DUNGEON + SYSTEM HUB"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    let apps = if zh_mode {
        [
            ("地圖選擇", "進入關卡", ui.cyan),
            ("系統設定", "主題與語言", ui.orange),
            ("控制中心", "板子與狀態", ui.rose),
            ("觸控校正", "校正觸控", ui.lime),
        ]
    } else {
        [
            ("PLAY MAPS", "ENTER CAMPAIGN", ui.cyan),
            ("SETTINGS", "THEME + LANG", ui.orange),
            ("CONTROL", "BOARD STATUS", ui.rose),
            ("TOUCH LAB", "CALIBRATE PEN", ui.lime),
        ]
    };

    for (index, (title, subtitle, accent)) in apps.iter().enumerate() {
        let y = 64 + index as u16 * 39;
        let selected = index == home_index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { *accent } else { ui.steel };
        display.panel(20, y, 280, 35, fill, border);
        display.text(28, y + 5, title, ui.text, fill, 2);
        display.text(30, y + 23, subtitle, ui.text_muted, fill, 1);
    }

    display.panel(18, 226, 284, 12, ui.panel, ui.white);
    display.text(
        34,
        228,
        if zh_mode {
            "K0 上一項  WK 下一項  K1 開啟"
        } else {
            "K0 PREV  WK NEXT  K1 OPEN"
        },
        ui.text_muted,
        ui.panel,
        1,
    );
}

pub fn render_map_select(display: &mut Display, map_index: usize, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 38);
    display.panel(16, 12, 288, 34, ui.panel, ui.cyan);
    display.text(
        24,
        20,
        if zh_mode { "選擇地圖" } else { "MAP SELECT" },
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
        display.text(30, y + 10, DungeonApp::map_name(idx, zh_mode), ui.text, fill, 2);
        display.text(
            202,
            y + 14,
            if selected {
                if zh_mode { "進入" } else { "ENTER" }
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
            "按 K0+WK 返回主頁"
        } else {
            "PRESS K0+WK TO RETURN HOME"
        },
        ui.text,
        ui.panel,
        1,
    );
}

pub fn render_settings(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    render_strategy: RenderStrategy,
) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 84);
    display.panel(16, 12, 288, 34, ui.panel, ui.orange);
    display.text(
        24,
        20,
        if zh_mode { "系統設定" } else { "SETTINGS" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        170,
        22,
        if zh_mode {
            "WK 主題  K1 語言"
        } else {
            "WK THEME  K1 LANG"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    display.panel(22, 60, 276, 50, ui.panel_alt, ui.cyan);
    display.text(
        32,
        70,
        if zh_mode { "主題模式" } else { "THEME MODE" },
        ui.text,
        ui.panel_alt,
        2,
    );
    display.text(
        34,
        92,
        match theme {
            ThemeMode::Dark => {
                if zh_mode { "深色模式" } else { "DARK MODE" }
            }
            ThemeMode::Light => {
                if zh_mode { "淺色模式" } else { "LIGHT MODE" }
            }
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.panel(22, 118, 276, 50, ui.panel_alt, ui.rose);
    display.text(
        32,
        128,
        if zh_mode { "語言" } else { "LANGUAGE" },
        ui.text,
        ui.panel_alt,
        2,
    );
    display.text(
        34,
        150,
        if zh_mode { "繁體中文" } else { "ENGLISH" },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.panel(22, 176, 276, 34, ui.panel_alt, ui.lime);
    display.text(
        32,
        184,
        "RENDER STRATEGY",
        ui.text,
        ui.panel_alt,
        2,
    );
    display.text(
        34,
        200,
        render_strategy.label(),
        ui.text_muted,
        ui.panel_alt,
        1,
    );
    display.text(
        132,
        200,
        render_strategy.detail(),
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.panel(22, 216, 276, 14, ui.panel, ui.amber);
    display.text(
        34,
        219,
        if zh_mode {
            "按 K0+WK 返回主頁"
        } else {
            "PRESS K0+WK TO RETURN HOME"
        },
        ui.text,
        ui.panel,
        1,
    );
}

pub fn render_touch_calibration(display: &mut Display, step: u8, theme: ThemeMode, zh_mode: bool) {
    const TARGET_X: [u16; 5] = [28, 292, 160, 292, 28];
    const TARGET_Y: [u16; 5] = [40, 40, 122, 210, 210];
    const TARGET_LABELS_EN: [&str; 5] =
        ["TOP LEFT", "TOP RIGHT", "CENTER", "BOTTOM RIGHT", "BOTTOM LEFT"];
    const TARGET_LABELS_ZH: [&str; 5] = ["左上", "右上", "中央", "右下", "左下"];

    let ui = palette(theme);
    draw_gradient_background(display, theme, 120);
    display.panel(16, 12, 288, 34, ui.panel, ui.orange);
    display.text(
        24,
        20,
        if zh_mode {
            "觸控校正"
        } else {
            "TOUCH CALIBRATION"
        },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        192,
        22,
        if zh_mode { "K0 取消" } else { "K0 CANCEL" },
        ui.text_muted,
        ui.panel,
        1,
    );

    display.panel(18, 58, 284, 46, ui.panel_alt, ui.cyan);
    display.text(
        28,
        68,
        if zh_mode {
            "依序點擊四角與中央"
        } else {
            "TAP EACH CROSS INCLUDING CENTER"
        },
        ui.text,
        ui.panel_alt,
        2,
    );
    display.text(
        28,
        88,
        if zh_mode {
            "重開機後需要重新校正"
        } else {
            "RUNTIME ONLY, RECALIBRATE AFTER RESET"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    let safe_step = (step.min(4)) as usize;
    let mut line: String<24> = String::new();
    let _ = write!(
        &mut line,
        "{} {}/5",
        if zh_mode { "步驟" } else { "STEP" },
        safe_step + 1
    );
    display.panel(108, 116, 104, 24, ui.panel, ui.cyan);
    display.centered_text(160, 124, &line, ui.text, ui.panel, 2);
    display.panel(88, 146, 144, 24, ui.panel, ui.rose);
    display.centered_text(
        160,
        154,
        if zh_mode {
            TARGET_LABELS_ZH[safe_step]
        } else {
            TARGET_LABELS_EN[safe_step]
        },
        ui.text,
        ui.panel,
        1,
    );

    let tx = TARGET_X[safe_step];
    let ty = TARGET_Y[safe_step];
    display.fill_rect(tx.saturating_sub(20), ty, 40, 2, ui.rose);
    display.fill_rect(tx, ty.saturating_sub(20), 2, 40, ui.rose);

    display.panel(18, 206, 284, 24, ui.panel, ui.white);
    display.text(
        28,
        214,
        if zh_mode {
            "點一下記錄座標，完成後自動返回"
        } else {
            "TAP ONCE TO CAPTURE, AUTO RETURN WHEN DONE"
        },
        ui.text_muted,
        ui.panel,
        1,
    );
}

pub fn render_control_room(display: &mut Display, board: &Board, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 62);
    display.panel(14, 10, 292, 34, ui.panel, ui.orange);
    display.text(
        24,
        18,
        if zh_mode { "控制室" } else { "CONTROL ROOM" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        186,
        20,
        if zh_mode { "K1 切換 LED" } else { "K1 TOGGLE LED" },
        ui.text_muted,
        ui.panel,
        1,
    );

    display.panel(18, 56, 284, 70, ui.panel_alt, ui.cyan);
    let mut uptime: String<24> = String::new();
    let seconds = millis() / 1000;
    let _ = write!(
        &mut uptime,
        "{} {}S",
        if zh_mode { "開機時間" } else { "UPTIME" },
        seconds
    );
    display.text(32, 72, &uptime, ui.text, ui.panel_alt, 2);
    display.text(
        32,
        102,
        if board.led_on() {
            if zh_mode { "LED 亮起" } else { "LED STATE ON" }
        } else if zh_mode {
            "LED 熄滅"
        } else {
            "LED STATE OFF"
        },
        if board.led_on() { ui.lime } else { ui.text_muted },
        ui.panel_alt,
        2,
    );

    render_button_card(
        display,
        20,
        138,
        "K1",
        if zh_mode { "切換/開啟" } else { "MOVE / OPEN" },
        ui.cyan,
        &ui,
    );
    render_button_card(
        display,
        111,
        138,
        "K0",
        if zh_mode { "返回/左移" } else { "BACK / LEFT" },
        ui.orange,
        &ui,
    );
    render_button_card(
        display,
        202,
        138,
        "WK",
        if zh_mode { "下一步/首頁" } else { "NEXT / HOME" },
        ui.rose,
        &ui,
    );

    display.panel(18, 206, 284, 24, ui.panel, ui.amber);
    display.text(
        26,
        214,
        if zh_mode {
            "按 K0+WK 返回主頁"
        } else {
            "PRESS K0+WK TO RETURN HOME"
        },
        ui.text,
        ui.panel,
        1,
    );
}

fn render_button_card(
    display: &mut Display,
    x: u16,
    y: u16,
    title: &str,
    subtitle: &str,
    accent: u16,
    ui: &Palette,
) {
    display.panel(x, y, 85, 52, ui.panel, accent);
    display.text(x + 18, y + 10, title, ui.text, ui.panel, 3);
    display.text(x + 10, y + 34, subtitle, ui.text_muted, ui.panel, 1);
}

fn draw_gradient_background(display: &mut Display, theme: ThemeMode, shift: u8) {
    let ui = palette(theme);
    for band in 0..12u16 {
        let tint = (band * 20) as u8;
        let top = color::mix(ui.canvas, ui.indigo, tint.wrapping_add(shift));
        let bottom = color::mix(ui.panel, ui.sky, tint);
        let fill = color::mix(top, bottom, 120);
        display.fill_rect(0, band * 20, SCREEN_WIDTH, 20, fill);
    }
}
