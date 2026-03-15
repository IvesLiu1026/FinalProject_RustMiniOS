use crate::app_registry::{descriptor, home_apps};
use crate::display::{palette, Display, ThemeMode};

use super::draw_gradient_background;

pub fn render_home(display: &mut Display, home_index: usize, theme: ThemeMode, zh_mode: bool) {
    let ui = palette(theme);
    draw_gradient_background(display, theme, 12);

    display.panel(16, 12, 288, 48, ui.panel, ui.cyan);
    display.text(26, 22, "RUST MINI OS", ui.text, ui.panel, 3);
    display.text(
        28,
        44,
        if zh_mode {
            "相簿、遊戲、畫板與系統桌面"
        } else {
            "ALBUM + GAMES + PAINT DESKTOP"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    for (index, app_id) in home_apps().iter().copied().enumerate() {
        let app = descriptor(app_id);
        let y = 64 + index as u16 * 39;
        let selected = index == home_index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected {
            app.accent.resolve(&ui)
        } else {
            ui.steel
        };
        display.panel(20, y, 280, 35, fill, border);
        display.text(28, y + 5, app.title(zh_mode), ui.text, fill, 2);
        display.text(30, y + 23, app.subtitle(zh_mode), ui.text_muted, fill, 1);
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
