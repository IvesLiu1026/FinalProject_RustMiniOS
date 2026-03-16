use core::fmt::Write;

use heapless::String;

use crate::app_registry::descriptor;
use crate::board::{self, Board};
use crate::display::{color, palette, Display, ThemeMode};
use crate::dungeon::RenderStrategy;
use crate::storage::StorageStatus;
use crate::system_info;

use super::{
    draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width, render_nav_back, DiagnosticsNotice, DIAG_ACTION_H, DIAG_ACTION_W,
    DIAG_ACTION_Y, DIAG_CLEAR_X, DIAG_RESET_X,
};

pub fn render_diagnostics(
    display: &mut Display,
    _board: &Board,
    theme: ThemeMode,
    zh_mode: bool,
    active_screen: &str,
    fps: u16,
    touch_ready: bool,
    render_strategy: RenderStrategy,
    storage_status: StorageStatus,
    safe_boot: bool,
    selected_action: usize,
    action_armed: bool,
    notice: Option<DiagnosticsNotice>,
) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 96);
    draw_shell_window(display, ui.white, &ui);
    draw_title_bar(
        display,
        if zh_mode {
            "系統診斷"
        } else {
            "DIAGNOSTICS"
        },
        if zh_mode {
            "build / storage / runtime / recovery"
        } else {
            "build / storage / runtime / recovery"
        },
        ui.white,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.cyan, &ui);
    let mut fps_line: String<24> = String::new();
    let _ = write!(&mut fps_line, "{} FPS", fps);
    let mut build_line: String<32> = String::new();
    let _ = write!(
        &mut build_line,
        "v{} {}",
        system_info::app_version(),
        system_info::git_sha()
    );
    draw_info_strip(
        display,
        18,
        52,
        92,
        if zh_mode { "畫面" } else { "RETURN" },
        active_screen,
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        114,
        52,
        92,
        if zh_mode { "幀率" } else { "FPS" },
        &fps_line,
        ui.amber,
        &ui,
    );
    draw_info_strip(
        display,
        210,
        52,
        92,
        if zh_mode { "建置" } else { "BUILD" },
        &build_line,
        ui.white,
        &ui,
    );

    display.panel(18, 74, 136, 108, ui.panel, ui.orange);
    let system_title =
        fit_text_to_width(display, if zh_mode { "系統狀態" } else { "SYSTEM" }, 74, 1);
    display.text(28, 86, &system_title, ui.text, ui.panel, 1);
    draw_runtime_icon(display, 116, 82, &ui);
    draw_info_strip(
        display,
        28,
        106,
        116,
        if zh_mode { "按鍵中斷" } else { "BTN IRQ" },
        &button_irq_line(),
        if board::button_exti_pending_mask() != 0 {
            ui.amber
        } else {
            ui.lime
        },
        &ui,
    );
    draw_info_strip(
        display,
        28,
        124,
        116,
        if zh_mode { "觸控中斷" } else { "TOUCH IRQ" },
        &touch_irq_line(zh_mode, touch_ready),
        if touch_ready { ui.cyan } else { ui.rose },
        &ui,
    );
    draw_info_strip(
        display,
        28,
        142,
        116,
        if zh_mode { "渲染" } else { "RENDER" },
        render_strategy.label(),
        ui.lime,
        &ui,
    );
    draw_info_strip(
        display,
        28,
        160,
        116,
        if zh_mode { "開機" } else { "BOOT" },
        if safe_boot {
            if zh_mode {
                "安全模式"
            } else {
                "SAFE"
            }
        } else if zh_mode {
            "正常"
        } else {
            "NORMAL"
        },
        if safe_boot { ui.amber } else { ui.cyan },
        &ui,
    );

    display.panel(166, 74, 136, 108, ui.panel, ui.lime);
    let storage_title =
        fit_text_to_width(display, if zh_mode { "儲存狀態" } else { "STORAGE" }, 74, 1);
    display.text(176, 86, &storage_title, ui.text, ui.panel, 1);
    draw_storage_icon(display, 262, 82, &ui);
    let status_text = if storage_status.valid_record {
        if zh_mode {
            "紀錄正常"
        } else {
            "RECORD OK"
        }
    } else if storage_status.found_magic {
        if zh_mode {
            "資料損毀"
        } else {
            "CORRUPT DATA"
        }
    } else if zh_mode {
        "尚未建立"
    } else {
        "EMPTY"
    };
    draw_info_strip(
        display,
        176,
        106,
        116,
        if zh_mode { "紀錄" } else { "STATUS" },
        status_text,
        if storage_status.valid_record {
            ui.lime
        } else {
            ui.rose
        },
        &ui,
    );

    let mut version_line: String<28> = String::new();
    if storage_status.found_magic {
        let _ = write!(
            &mut version_line,
            "V{} {} {}B",
            storage_status.version,
            if storage_status.checksum_ok {
                "CK"
            } else {
                "--"
            },
            storage_status.record_bytes
        );
    } else {
        let _ = write!(&mut version_line, "V-- -- --B");
    }
    draw_info_strip(
        display,
        176,
        124,
        116,
        if zh_mode { "格式" } else { "FORMAT" },
        &version_line,
        ui.white,
        &ui,
    );

    let mut save_line: String<28> = String::new();
    let _ = write!(
        &mut save_line,
        "{} {} P{}",
        if zh_mode { "存檔" } else { "SAVES" },
        if storage_status.has_app_saves {
            if zh_mode {
                "有"
            } else {
                "YES"
            }
        } else if zh_mode {
            "空"
        } else {
            "NO"
        },
        storage_status.paint_pixels_used
    );
    draw_info_strip(
        display,
        176,
        142,
        116,
        if zh_mode { "存檔" } else { "SAVES" },
        &save_line,
        ui.amber,
        &ui,
    );

    let mut media_line: String<28> = String::new();
    let _ = write!(
        &mut media_line,
        "{}{} {}{} {}",
        if zh_mode { "圖" } else { "S" },
        system_info::still_count(),
        if zh_mode { "動" } else { "M" },
        system_info::motion_clip_count(),
        if system_info::album_backend() == "mac-companion" {
            "MAC"
        } else {
            "ROM"
        }
    );
    draw_info_strip(
        display,
        176,
        160,
        116,
        if zh_mode { "媒體" } else { "MEDIA" },
        &media_line,
        ui.cyan,
        &ui,
    );

    let mut recent_line: String<28> = String::new();
    let recent_label = storage_status
        .recent_app
        .map(|app_id| descriptor(app_id).title(zh_mode))
        .unwrap_or(if zh_mode { "無" } else { "NONE" });
    let _ = write!(
        &mut recent_line,
        "{} {}",
        if zh_mode { "最近" } else { "LAST" },
        recent_label
    );
    draw_info_strip(
        display,
        176,
        178,
        116,
        if zh_mode { "最近" } else { "LAST" },
        recent_label,
        ui.rose,
        &ui,
    );

    display.fill_rect(18, 176, 132, 8, color::mix(ui.panel_alt, ui.white, 10));
    display.stroke_rect(18, 176, 132, 8, 1, ui.steel);
    let recovery_header = fit_text_to_width(
        display,
        if zh_mode {
            "恢復動作 / 需二次確認"
        } else {
            "RECOVERY / DOUBLE CONFIRM"
        },
        120,
        1,
    );
    display.text(
        24,
        177,
        &recovery_header,
        ui.text_muted,
        color::mix(ui.panel_alt, ui.white, 10),
        1,
    );

    let selected_clear = selected_action == 0;
    let selected_reset = selected_action == 1;
    render_diag_action_button(
        display,
        DIAG_CLEAR_X,
        DIAG_ACTION_Y,
        DIAG_ACTION_W,
        DIAG_ACTION_H,
        if selected_clear { ui.cyan } else { ui.steel },
        selected_clear && action_armed,
        if zh_mode {
            "清除存檔"
        } else {
            "CLEAR SAVES"
        },
        if zh_mode {
            "保留系統設定"
        } else {
            "KEEP SYSTEM SETTINGS"
        },
        &ui,
    );
    render_diag_action_button(
        display,
        DIAG_RESET_X,
        DIAG_ACTION_Y,
        DIAG_ACTION_W,
        DIAG_ACTION_H,
        if selected_reset { ui.rose } else { ui.steel },
        selected_reset && action_armed,
        if zh_mode {
            "恢復出廠"
        } else {
            "FACTORY RESET"
        },
        if zh_mode {
            "清除全部，回到校正"
        } else {
            "WIPE ALL AND RE-CALIBRATE"
        },
        &ui,
    );

    let (notice_title, notice_body) = match notice {
        Some(DiagnosticsNotice::ClearReady) => (
            if zh_mode {
                "再次確認清除存檔"
            } else {
                "PRESS AGAIN TO CLEAR SAVES"
            },
            if zh_mode {
                "相簿、畫布與分數會被清空"
            } else {
                "ALBUM, CANVAS, AND SCORES WILL BE CLEARED"
            },
        ),
        Some(DiagnosticsNotice::Cleared) => (
            if zh_mode {
                "已清除 app 存檔"
            } else {
                "APP SAVES CLEARED"
            },
            if zh_mode {
                "主題、語言與觸控校正已保留"
            } else {
                "THEME, LANGUAGE, AND TOUCH CAL ARE PRESERVED"
            },
        ),
        Some(DiagnosticsNotice::ClearFailed) => (
            if zh_mode {
                "清除存檔失敗"
            } else {
                "CLEAR FAILED"
            },
            if zh_mode {
                "請重試，或重新上電後再檢查"
            } else {
                "RETRY OR POWER-CYCLE BEFORE CHECKING AGAIN"
            },
        ),
        Some(DiagnosticsNotice::ResetReady) => (
            if zh_mode {
                "再次確認恢復出廠"
            } else {
                "PRESS AGAIN FOR FACTORY RESET"
            },
            if zh_mode {
                "將清除所有設定與觸控校正"
            } else {
                "ALL SETTINGS AND TOUCH CAL DATA WILL BE ERASED"
            },
        ),
        Some(DiagnosticsNotice::ResetFailed) => (
            if zh_mode {
                "恢復出廠失敗"
            } else {
                "FACTORY RESET FAILED"
            },
            if zh_mode {
                "儲存區擦除沒有成功"
            } else {
                "THE STORAGE SECTOR COULD NOT BE ERASED"
            },
        ),
        None => (
            if zh_mode {
                "K0/WK 選操作，K1 執行，需二次確認"
            } else {
                "K0/WK SELECT, K1 APPLY, SECOND PRESS TO CONFIRM"
            },
            if zh_mode {
                "這裡也可查看最近 app 與存檔是否存在"
            } else {
                "CHECK RECENT APP AND SAVE PRESENCE HERE AS WELL"
            },
        ),
    };
    display.fill_rect(18, 206, 284, 22, ui.panel_alt);
    display.stroke_rect(18, 206, 284, 22, 1, ui.steel);
    let notice_title = fit_text_to_width(display, notice_title, 210, 1);
    let notice_body = fit_text_to_width(display, notice_body, 210, 1);
    display.text(28, 210, &notice_title, ui.text, ui.panel_alt, 1);
    display.text(28, 220, &notice_body, ui.text_muted, ui.panel_alt, 1);
    display.fill_rect(
        248,
        208,
        44,
        10,
        color::mix(ui.panel, if safe_boot { ui.amber } else { ui.cyan }, 20),
    );
    display.stroke_rect(
        248,
        208,
        44,
        10,
        1,
        if safe_boot { ui.amber } else { ui.cyan },
    );
    display.centered_text(
        270,
        211,
        if safe_boot {
            if zh_mode {
                "SAFE"
            } else {
                "SAFE"
            }
        } else if zh_mode {
            "LIVE"
        } else {
            "LIVE"
        },
        ui.text,
        color::mix(ui.panel, if safe_boot { ui.amber } else { ui.cyan }, 20),
        1,
    );
}

fn render_diag_action_button(
    display: &mut Display,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    accent: u16,
    armed: bool,
    title: &str,
    subtitle: &str,
    ui: &crate::display::Palette,
) {
    let fill = if armed { ui.panel } else { ui.panel_alt };
    display.panel(x, y, w, h, fill, accent);
    display.fill_rect(x + 8, y + 5, 10, 10, color::mix(fill, accent, 24));
    display.stroke_rect(x + 8, y + 5, 10, 10, 1, accent);
    display.fill_rect(x + 11, y + 8, 4, 4, accent);
    let title = fit_text_to_width(display, title, w.saturating_sub(32), 1);
    let subtitle = fit_text_to_width(display, subtitle, w.saturating_sub(32), 1);
    display.text(x + 24, y + 5, &title, ui.text, fill, 1);
    display.text(x + 24, y + 13, &subtitle, ui.text_muted, fill, 1);
}

fn draw_runtime_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 24, 16, color::mix(ui.panel_alt, ui.orange, 18));
    display.stroke_rect(x, y, 24, 16, 1, ui.orange);
    display.fill_rect(x + 3, y + 3, 14, 8, ui.text);
    display.fill_rect(x + 5, y + 5, 10, 4, color::mix(ui.cyan, ui.white, 76));
    display.fill_rect(x + 18, y + 4, 3, 3, ui.lime);
    display.fill_rect(x + 18, y + 9, 3, 3, ui.amber);
}

fn draw_storage_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 24, 16, color::mix(ui.panel_alt, ui.lime, 18));
    display.stroke_rect(x, y, 24, 16, 1, ui.lime);
    display.fill_rect(x + 4, y + 3, 10, 10, ui.white);
    display.stroke_rect(x + 4, y + 3, 10, 10, 1, ui.cyan);
    display.fill_rect(x + 6, y + 5, 6, 3, color::mix(ui.cyan, ui.white, 76));
    display.fill_rect(x + 15, y + 4, 5, 8, ui.text);
    display.fill_rect(x + 16, y + 6, 3, 4, color::mix(ui.panel_alt, ui.rose, 20));
}

fn touch_irq_line(zh_mode: bool, touch_ready: bool) -> String<24> {
    let mut line: String<24> = String::new();
    let readiness = if touch_ready {
        if zh_mode {
            "就緒"
        } else {
            "READY"
        }
    } else if zh_mode {
        "未校正"
    } else {
        "PEND"
    };
    let pending = if board::touch_exti_pending() { "*" } else { "" };
    let _ = write!(
        &mut line,
        "{} E{}{}",
        readiness,
        board::touch_exti_event_count(),
        pending
    );
    line
}

fn button_irq_line() -> String<24> {
    let (k1, k0, wkup) = board::button_exti_counts();
    let mut line: String<24> = String::new();
    let pending = if board::button_exti_pending_mask() != 0 {
        "*"
    } else {
        ""
    };
    let _ = write!(&mut line, "1:{} 0:{} W:{}{}", k1, k0, wkup, pending);
    line
}
