use crate::display::{palette, Display, ThemeMode};
use crate::system_info;

use super::draw_gradient_background;

pub fn render_safe_mode(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    selected_index: usize,
    touch_ready: bool,
) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 132);
    display.panel(16, 12, 288, 34, ui.panel, ui.rose);
    display.text(
        28,
        20,
        if zh_mode { "安全模式" } else { "SAFE MODE" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        152,
        22,
        if zh_mode {
            "最小化開機與修復入口"
        } else {
            "MINIMAL BOOT + RECOVERY"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    display.panel(18, 56, 284, 42, ui.panel_alt, ui.orange);
    display.text(
        28,
        66,
        system_info::safe_mode_hint(zh_mode),
        ui.text,
        ui.panel_alt,
        1,
    );
    display.text(
        28,
        82,
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
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    let rows = [
        (
            if zh_mode {
                "進入首頁"
            } else {
                "CONTINUE TO HOME"
            },
            if zh_mode {
                "離開安全模式，正常操作系統"
            } else {
                "LEAVE SAFE MODE AND OPEN THE DESKTOP"
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
                "重新建立可用的觸控校正"
            } else {
                "REBUILD A CLEAN TOUCH CALIBRATION"
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
                "檢查儲存區、資產與恢復工具"
            } else {
                "INSPECT STORAGE, ASSETS, AND RECOVERY TOOLS"
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
        display.text(28, y + 6, title, ui.text, fill, 1);
        display.text(28, y + 16, subtitle, ui.text_muted, fill, 1);
    }

    display.panel(18, 220, 284, 14, ui.panel_alt, ui.white);
    display.text(
        28,
        223,
        if zh_mode {
            "K0/WK 選擇，K1 執行"
        } else {
            "K0/WK SELECT, K1 OPEN"
        },
        ui.text,
        ui.panel_alt,
        1,
    );
}
