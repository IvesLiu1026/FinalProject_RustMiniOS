use crate::display::{palette, Display, ThemeMode};
use crate::dungeon::RenderStrategy;

use super::{draw_gradient_background, render_nav_back};

pub fn render_settings(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    render_strategy: RenderStrategy,
    selected_index: usize,
) {
    const ROW_STEP: u16 = 25;

    let ui = palette(theme);
    draw_gradient_background(display, theme, 84);
    display.panel(16, 12, 288, 34, ui.panel, ui.orange);
    render_nav_back(display, zh_mode, ui.white, &ui);
    display.text(
        74,
        20,
        if zh_mode { "系統設定" } else { "SETTINGS" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        154,
        22,
        if zh_mode {
            "K0/WK 移動  K1 執行"
        } else {
            "K0/WK MOVE  K1 APPLY"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    let rows = [
        (
            if zh_mode {
                "主題模式"
            } else {
                "THEME MODE"
            },
            match theme {
                ThemeMode::Dark => {
                    if zh_mode {
                        "深色工作桌面"
                    } else {
                        "DARK DESKTOP"
                    }
                }
                ThemeMode::Light => {
                    if zh_mode {
                        "淺色工作桌面"
                    } else {
                        "LIGHT DESKTOP"
                    }
                }
            },
            ui.cyan,
        ),
        (
            if zh_mode { "語言" } else { "LANGUAGE" },
            if zh_mode {
                "繁體中文介面"
            } else {
                "ENGLISH UI"
            },
            ui.rose,
        ),
        ("RENDER STRATEGY", render_strategy.detail(), ui.lime),
        (
            if zh_mode { "控制室" } else { "CONTROL ROOM" },
            if zh_mode {
                "檢查板子狀態與 LED"
            } else {
                "BOARD STATUS + LED"
            },
            ui.amber,
        ),
        (
            if zh_mode { "觸控校正" } else { "TOUCH LAB" },
            if zh_mode {
                "重新校正觸控"
            } else {
                "RECALIBRATE TOUCH"
            },
            ui.orange,
        ),
        (
            if zh_mode {
                "系統診斷"
            } else {
                "DIAGNOSTICS"
            },
            if zh_mode {
                "檢查 FPS、資產與執行狀態"
            } else {
                "FPS + ASSETS + STATUS"
            },
            ui.white,
        ),
        (
            if zh_mode { "關於系統" } else { "ABOUT" },
            if zh_mode {
                "版本、建置與安全模式提示"
            } else {
                "VERSION, BUILD, AND SAFE MODE HINTS"
            },
            ui.cyan,
        ),
    ];

    for (index, (title, subtitle, accent)) in rows.iter().enumerate() {
        let y = 52 + index as u16 * ROW_STEP;
        let selected = selected_index == index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { *accent } else { ui.steel };
        display.panel(20, y, 280, 24, fill, border);
        display.text(28, y + 5, title, ui.text, fill, 1);
        display.text(138, y + 8, subtitle, ui.text_muted, fill, 1);
        if index == 2 {
            display.text(28, y + 14, render_strategy.label(), ui.text_muted, fill, 1);
        }
    }

    display.panel(22, 226, 276, 12, ui.panel, ui.amber);
    display.text(
        28,
        228,
        if zh_mode {
            "K1 執行選項  K0+WK 回首頁"
        } else {
            "K1 APPLY ITEM  K0+WK RETURN HOME"
        },
        ui.text,
        ui.panel,
        1,
    );
}
