use core::fmt::Write;

use heapless::String;

use crate::board::{millis, Board};
use crate::display::{color, palette, Display, Palette, ThemeMode};

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    render_nav_back,
};

pub fn render_control_room(display: &mut Display, board: &Board, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 62);
    draw_shell_window(display, ui.orange, &ui);
    draw_title_bar(
        display,
        if zh_mode { "控制室" } else { "CONTROL ROOM" },
        if zh_mode {
            "board status / led / input map"
        } else {
            "board status / led / input map"
        },
        ui.orange,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.white, &ui);

    let seconds = millis() / 1000;
    let mut uptime: String<24> = String::new();
    let _ = write!(
        &mut uptime,
        "{} {}S",
        if zh_mode { "開機" } else { "UPTIME" },
        seconds
    );
    draw_info_strip(
        display,
        18,
        46,
        132,
        if zh_mode { "機台" } else { "BOARD" },
        if board.led_on() { "ONLINE" } else { "IDLE" },
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        164,
        46,
        138,
        if zh_mode { "時間" } else { "TIME" },
        &uptime,
        ui.amber,
        &ui,
    );

    display.panel(18, 62, 136, 84, ui.panel_alt, ui.cyan);
    draw_runtime_monitor(display, 104, 72, board.led_on(), &ui);
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
        1,
    );
    draw_info_strip(
        display,
        28,
        118,
        116,
        if zh_mode { "輸入" } else { "INPUT" },
        if zh_mode {
            "K1 / K0 / WK"
        } else {
            "K1 / K0 / WK"
        },
        ui.rose,
        &ui,
    );

    display.panel(166, 62, 136, 84, ui.panel, ui.lime);
    display.text(
        176,
        74,
        if zh_mode {
            "操作提示"
        } else {
            "CONTROL MAP"
        },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        176,
        94,
        if zh_mode {
            "K1: 切換 LED"
        } else {
            "K1: TOGGLE LED"
        },
        ui.text_muted,
        ui.panel,
        1,
    );
    display.text(
        176,
        108,
        if zh_mode {
            "K0: 返回設定"
        } else {
            "K0: BACK TO SETTINGS"
        },
        ui.text_muted,
        ui.panel,
        1,
    );
    display.text(
        176,
        122,
        if zh_mode {
            "WK: 下一項 / 首頁"
        } else {
            "WK: NEXT / HOME"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    render_button_card(
        display,
        20,
        154,
        "K1",
        if zh_mode { "切換 LED" } else { "TOGGLE LED" },
        ui.cyan,
        &ui,
    );
    render_button_card(
        display,
        111,
        154,
        "K0",
        if zh_mode { "返回設定" } else { "BACK" },
        ui.orange,
        &ui,
    );
    render_button_card(
        display,
        202,
        154,
        "WK",
        if zh_mode {
            "下一步/首頁"
        } else {
            "NEXT / HOME"
        },
        ui.rose,
        &ui,
    );

    draw_footer_hint(
        display,
        if zh_mode {
            "K1 TOGGLE LED  K0+WK RETURNS TO SETTINGS"
        } else {
            "K1 TOGGLE LED  K0+WK RETURNS TO SETTINGS"
        },
        ui.amber,
        &ui,
    );
}

fn draw_runtime_monitor(display: &mut Display, x: u16, y: u16, led_on: bool, ui: &Palette) {
    display.fill_rect(x, y, 36, 24, color::mix(ui.panel, ui.canvas, 12));
    display.stroke_rect(x, y, 36, 24, 1, ui.white);
    display.fill_rect(x + 4, y + 4, 28, 14, color::mix(ui.cyan, ui.white, 72));
    display.stroke_rect(x + 4, y + 4, 28, 14, 1, ui.indigo);
    display.fill_rect(x + 10, y + 9, 4, 4, ui.text);
    display.fill_rect(x + 22, y + 9, 4, 4, ui.text);
    display.fill_rect(x + 15, y + 18, 8, 2, ui.shadow);
    display.fill_rect(x + 28, y + 4, 4, 4, if led_on { ui.lime } else { ui.rose });
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
