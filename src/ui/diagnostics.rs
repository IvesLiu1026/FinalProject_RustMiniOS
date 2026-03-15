use core::fmt::Write;

use heapless::String;

use crate::app_registry::descriptor;
use crate::board::Board;
use crate::display::{palette, Display, ThemeMode};
use crate::dungeon::RenderStrategy;
use crate::storage::StorageStatus;
use crate::system_info;

use super::{
    draw_gradient_background, render_nav_back, DiagnosticsNotice, DIAG_ACTION_H, DIAG_ACTION_W,
    DIAG_ACTION_Y, DIAG_CLEAR_X, DIAG_RESET_X,
};

pub fn render_diagnostics(
    display: &mut Display,
    board: &Board,
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
    display.panel(16, 12, 288, 34, ui.panel, ui.white);
    render_nav_back(display, zh_mode, ui.cyan, &ui);
    display.text(
        74,
        20,
        if zh_mode {
            "系統診斷"
        } else {
            "DIAGNOSTICS"
        },
        ui.text,
        ui.panel,
        2,
    );

    display.panel(18, 56, 284, 46, ui.panel_alt, ui.cyan);
    let mut fps_line: String<24> = String::new();
    let _ = write!(&mut fps_line, "{} FPS", fps);
    display.text(
        28,
        66,
        if zh_mode {
            "返回頁面"
        } else {
            "RETURN TARGET"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );
    display.text(126, 66, active_screen, ui.text, ui.panel_alt, 1);
    display.text(
        28,
        80,
        if zh_mode {
            "即時幀率"
        } else {
            "FRAME RATE"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );
    display.text(126, 80, &fps_line, ui.amber, ui.panel_alt, 1);
    let mut build_line: String<32> = String::new();
    let _ = write!(
        &mut build_line,
        "v{} {} {}",
        system_info::app_version(),
        system_info::git_sha(),
        system_info::build_profile()
    );
    display.text(
        28,
        92,
        if zh_mode {
            "建置資訊"
        } else {
            "BUILD INFO"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );
    display.text(126, 92, &build_line, ui.text, ui.panel_alt, 1);

    display.panel(18, 108, 136, 80, ui.panel, ui.orange);
    display.text(
        28,
        118,
        if zh_mode { "系統狀態" } else { "SYSTEM" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        28,
        140,
        if board.led_on() {
            if zh_mode {
                "LED: 開啟"
            } else {
                "LED: ON"
            }
        } else if zh_mode {
            "LED: 關閉"
        } else {
            "LED: OFF"
        },
        if board.led_on() {
            ui.lime
        } else {
            ui.text_muted
        },
        ui.panel,
        1,
    );
    display.text(
        28,
        154,
        if touch_ready {
            if zh_mode {
                "觸控: 已校正"
            } else {
                "TOUCH: CALIBRATED"
            }
        } else if zh_mode {
            "觸控: 未就緒"
        } else {
            "TOUCH: NOT READY"
        },
        if touch_ready { ui.cyan } else { ui.rose },
        ui.panel,
        1,
    );
    display.text(28, 168, render_strategy.label(), ui.text_muted, ui.panel, 1);
    display.text(
        28,
        180,
        if safe_boot {
            if zh_mode {
                "本次: 安全模式"
            } else {
                "BOOT: SAFE MODE"
            }
        } else if zh_mode {
            "本次: 正常開機"
        } else {
            "BOOT: NORMAL"
        },
        if safe_boot { ui.amber } else { ui.text_muted },
        ui.panel,
        1,
    );

    display.panel(166, 108, 136, 80, ui.panel, ui.lime);
    display.text(
        176,
        118,
        if zh_mode { "儲存狀態" } else { "STORAGE" },
        ui.text,
        ui.panel,
        2,
    );
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
    display.text(176, 136, status_text, ui.text_muted, ui.panel, 1);

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
    display.text(176, 148, &version_line, ui.text_muted, ui.panel, 1);

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
    display.text(176, 160, &save_line, ui.text_muted, ui.panel, 1);

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
    display.text(176, 172, &media_line, ui.text_muted, ui.panel, 1);

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
    display.text(176, 184, &recent_line, ui.text_muted, ui.panel, 1);

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

    display.panel(18, 216, 284, 22, ui.panel_alt, ui.white);
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
    display.text(28, 222, notice_title, ui.text, ui.panel_alt, 1);
    display.text(28, 232, notice_body, ui.text_muted, ui.panel_alt, 1);
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
    display.text(x + 8, y + 5, title, ui.text, fill, 1);
    display.text(x + 8, y + 14, subtitle, ui.text_muted, fill, 1);
}
