use core::fmt::Write;

use heapless::String;

use crate::display::{color, palette, Display, ThemeMode};

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width, render_nav_back,
};

pub fn render_touch_calibration(display: &mut Display, step: u8, theme: ThemeMode, zh_mode: bool) {
    const TARGET_X: [u16; 5] = [28, 292, 160, 292, 28];
    const TARGET_Y: [u16; 5] = [40, 40, 122, 210, 210];
    const TARGET_LABELS_EN: [&str; 5] = [
        "TOP LEFT",
        "TOP RIGHT",
        "CENTER",
        "BOTTOM RIGHT",
        "BOTTOM LEFT",
    ];
    const TARGET_LABELS_ZH: [&str; 5] = ["左上", "右上", "中央", "右下", "左下"];

    let ui = palette(theme);
    draw_gradient_background(display, theme, 120);
    draw_shell_window(display, ui.orange, &ui);
    draw_title_bar(
        display,
        if zh_mode {
            "觸控校正精靈"
        } else {
            "TOUCH SETUP WIZARD"
        },
        if zh_mode {
            "five-point setup utility"
        } else {
            "five-point setup utility"
        },
        ui.orange,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.white, &ui);

    draw_info_strip(
        display,
        18,
        46,
        132,
        if zh_mode { "流程" } else { "SETUP" },
        "5 POINT",
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        164,
        46,
        138,
        if zh_mode { "狀態" } else { "STATE" },
        if zh_mode { "等待點擊" } else { "WAIT TAP" },
        ui.amber,
        &ui,
    );

    display.panel(18, 62, 284, 38, ui.panel_alt, ui.cyan);
    draw_wizard_chip(display, 250, 70, &ui);
    let intro_title = fit_text_to_width(
        display,
        if zh_mode {
            "請依序點擊五個準星"
        } else {
            "TAP THE FIVE TARGETS IN ORDER"
        },
        200,
        1,
    );
    let intro_body = fit_text_to_width(
        display,
        if zh_mode {
            "完成後會保存，下次開機可直接進桌面"
        } else {
            "CALIBRATION SAVES AFTER FINISH"
        },
        210,
        1,
    );
    display.text(28, 72, &intro_title, ui.text, ui.panel_alt, 1);
    display.text(28, 84, &intro_body, ui.text_muted, ui.panel_alt, 1);

    let safe_step = (step.min(4)) as usize;
    let mut line: String<24> = String::new();
    let _ = write!(
        &mut line,
        "{} {}/5",
        if zh_mode { "步驟" } else { "STEP" },
        safe_step + 1
    );
    display.panel(108, 110, 104, 22, ui.panel, ui.cyan);
    display.centered_text(160, 117, &line, ui.text, ui.panel, 1);
    display.panel(88, 138, 144, 20, ui.panel, ui.rose);
    display.centered_text(
        160,
        144,
        if zh_mode {
            TARGET_LABELS_ZH[safe_step]
        } else {
            TARGET_LABELS_EN[safe_step]
        },
        ui.text,
        ui.panel,
        1,
    );

    display.panel(54, 164, 212, 50, ui.panel, ui.orange);
    display.text(
        66,
        176,
        if zh_mode {
            "校正區域"
        } else {
            "CALIBRATION FIELD"
        },
        ui.text,
        ui.panel,
        1,
    );
    let tx = TARGET_X[safe_step];
    let ty = TARGET_Y[safe_step];
    display.fill_rect(tx.saturating_sub(20), ty, 40, 2, ui.rose);
    display.fill_rect(tx, ty.saturating_sub(20), 2, 40, ui.rose);
    display.fill_rect(
        tx.saturating_sub(6),
        ty.saturating_sub(6),
        12,
        12,
        color::mix(ui.panel_alt, ui.rose, 24),
    );
    display.stroke_rect(
        tx.saturating_sub(6),
        ty.saturating_sub(6),
        12,
        12,
        1,
        ui.rose,
    );

    for index in 0..5usize {
        let dot_x = 118 + index as u16 * 18;
        let fill = if index <= safe_step {
            ui.cyan
        } else {
            ui.steel
        };
        display.fill_rect(dot_x, 198, 8, 8, fill);
        display.stroke_rect(dot_x, 198, 8, 8, 1, ui.white);
    }

    draw_footer_hint(
        display,
        if zh_mode {
            "點準星完成校正  若已有資料可按 K0 返回"
        } else {
            "TAP TARGET  K0 RETURNS IF CALIBRATION EXISTS"
        },
        ui.white,
        &ui,
    );
}

fn draw_wizard_chip(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 24, 18, color::mix(ui.panel_alt, ui.orange, 18));
    display.stroke_rect(x, y, 24, 18, 1, ui.orange);
    display.fill_rect(x + 4, y + 4, 16, 10, ui.text);
    display.fill_rect(x + 6, y + 6, 12, 6, color::mix(ui.panel_alt, ui.white, 18));
    display.fill_rect(x + 10, y + 2, 4, 2, ui.amber);
}
