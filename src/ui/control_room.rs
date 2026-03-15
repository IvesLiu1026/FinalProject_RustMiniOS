use core::fmt::Write;

use heapless::String;

use crate::board::{millis, Board};
use crate::display::{palette, Display, Palette, ThemeMode};

use super::{draw_gradient_background, render_nav_back};

pub fn render_control_room(display: &mut Display, board: &Board, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 62);
    display.panel(14, 10, 292, 34, ui.panel, ui.orange);
    render_nav_back(display, zh_mode, ui.white, &ui);
    display.text(
        74,
        18,
        if zh_mode { "控制室" } else { "CONTROL ROOM" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        186,
        20,
        if zh_mode {
            "K1 切換 LED"
        } else {
            "K1 TOGGLE LED"
        },
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
            if zh_mode {
                "LED 亮起"
            } else {
                "LED STATE ON"
            }
        } else if zh_mode {
            "LED 熄滅"
        } else {
            "LED STATE OFF"
        },
        if board.led_on() {
            ui.lime
        } else {
            ui.text_muted
        },
        ui.panel_alt,
        2,
    );

    render_button_card(
        display,
        20,
        138,
        "K1",
        if zh_mode {
            "切換/開啟"
        } else {
            "MOVE / OPEN"
        },
        ui.cyan,
        &ui,
    );
    render_button_card(
        display,
        111,
        138,
        "K0",
        if zh_mode {
            "返回/左移"
        } else {
            "BACK / LEFT"
        },
        ui.orange,
        &ui,
    );
    render_button_card(
        display,
        202,
        138,
        "WK",
        if zh_mode {
            "下一步/首頁"
        } else {
            "NEXT / HOME"
        },
        ui.rose,
        &ui,
    );

    display.panel(18, 206, 284, 24, ui.panel, ui.amber);
    display.text(
        26,
        214,
        if zh_mode {
            "按 K0+WK 返回設定"
        } else {
            "PRESS K0+WK TO RETURN SETTINGS"
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
