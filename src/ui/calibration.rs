use core::fmt::Write;

use heapless::String;

use crate::display::{palette, Display, ThemeMode};

use super::{draw_gradient_background, render_nav_back};

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
    display.panel(16, 12, 288, 34, ui.panel, ui.orange);
    render_nav_back(display, zh_mode, ui.white, &ui);
    display.text(
        74,
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
