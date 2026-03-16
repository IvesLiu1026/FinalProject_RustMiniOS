use crate::display::{color, palette, Display, ThemeMode};
use crate::system_info;

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width,
};

pub fn render_safe_mode(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    selected_index: usize,
    touch_ready: bool,
) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 132);
    draw_shell_window(display, ui.rose, &ui);
    draw_title_bar(
        display,
        if zh_mode { "安全模式" } else { "SAFE MODE" },
        if zh_mode {
            "minimal boot + recovery"
        } else {
            "minimal boot + recovery"
        },
        ui.rose,
        &ui,
    );

    draw_info_strip(
        display,
        18,
        46,
        132,
        if zh_mode { "模式" } else { "BOOT" },
        "SAFE",
        ui.rose,
        &ui,
    );
    draw_info_strip(
        display,
        164,
        46,
        138,
        if zh_mode { "觸控" } else { "TOUCH" },
        if touch_ready { "READY" } else { "BYPASS" },
        if touch_ready { ui.cyan } else { ui.amber },
        &ui,
    );

    display.panel(18, 62, 284, 38, ui.panel_alt, ui.orange);
    draw_safe_mode_badge(display, 248, 70, &ui);
    let hint_line = fit_text_to_width(display, system_info::safe_mode_hint(zh_mode), 198, 1);
    display.text(28, 72, &hint_line, ui.text, ui.panel_alt, 1);
    let detail_line = fit_text_to_width(
        display,
        if touch_ready {
            if zh_mode {
                "目前已有觸控校正，可直接進桌面"
            } else {
                "TOUCH CAL IS AVAILABLE, HOME CAN BOOT DIRECTLY"
            }
        } else if zh_mode {
            "目前未使用既有校正，可先進校正或診斷"
        } else {
            "PERSISTED CALIBRATION WAS BYPASSED, USE CAL OR DIAGNOSTICS"
        },
        198,
        1,
    );
    display.text(28, 86, &detail_line, ui.text_muted, ui.panel_alt, 1);

    let rows = [
        (
            if zh_mode {
                "進入首頁"
            } else {
                "CONTINUE TO HOME"
            },
            if zh_mode {
                "離開安全模式"
            } else {
                "LEAVE SAFE MODE"
            },
            ui.cyan,
        ),
        (
            if zh_mode {
                "觸控校正"
            } else {
                "TOUCH CALIBRATION"
            },
            if zh_mode {
                "重新建立校正"
            } else {
                "REBUILD CAL"
            },
            ui.amber,
        ),
        (
            if zh_mode {
                "系統診斷"
            } else {
                "DIAGNOSTICS"
            },
            if zh_mode {
                "檢查儲存區"
            } else {
                "CHECK STORAGE"
            },
            ui.lime,
        ),
    ];

    for (index, (title, subtitle, accent)) in rows.iter().enumerate() {
        let y = 110 + index as u16 * 34;
        let selected = selected_index == index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { *accent } else { ui.steel };
        display.panel(18, y, 284, 28, fill, border);
        display.fill_rect(28, y + 7, 12, 12, color::mix(fill, *accent, 24));
        display.stroke_rect(28, y + 7, 12, 12, 1, *accent);
        display.fill_rect(32, y + 11, 4, 4, *accent);
        let title = fit_text_to_width(display, title, 190, 1);
        let subtitle = fit_text_to_width(display, subtitle, 190, 1);
        display.text(46, y + 7, &title, ui.text, fill, 1);
        display.text(46, y + 17, &subtitle, ui.text_muted, fill, 1);
        if selected {
            display.text(274, y + 10, ">", *accent, fill, 1);
        }
    }

    draw_footer_hint(
        display,
        if zh_mode {
            "K0/WK SELECT  K1 OPEN  HOLD K1 DURING BOOT TO RETURN"
        } else {
            "K0/WK SELECT  K1 OPEN  HOLD K1 DURING BOOT TO RETURN"
        },
        ui.white,
        &ui,
    );
}

fn draw_safe_mode_badge(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 28, 18, color::mix(ui.panel, ui.rose, 20));
    display.stroke_rect(x, y, 28, 18, 1, ui.rose);
    display.fill_rect(x + 6, y + 4, 16, 10, ui.text);
    display.fill_rect(x + 9, y + 7, 10, 4, color::mix(ui.panel_alt, ui.amber, 22));
    display.fill_rect(x + 11, y + 3, 6, 2, ui.amber);
}
