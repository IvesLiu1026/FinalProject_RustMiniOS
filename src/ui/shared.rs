use crate::display::{color, palette, Display, Palette, ThemeMode, SCREEN_WIDTH};

use super::{NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y};

pub fn draw_gradient_background(display: &mut Display, theme: ThemeMode, shift: u8) {
    let ui = palette(theme);
    for band in 0..12u16 {
        let tint = (band * 20) as u8;
        let top = color::mix(ui.canvas, ui.indigo, tint.wrapping_add(shift));
        let bottom = color::mix(ui.panel, ui.sky, tint);
        let fill = color::mix(top, bottom, 120);
        display.fill_rect(0, band * 20, SCREEN_WIDTH, 20, fill);
    }
}

pub fn render_nav_back(display: &mut Display, zh_mode: bool, accent: u16, ui: &Palette) {
    display.panel(
        NAV_BACK_X,
        NAV_BACK_Y,
        NAV_BACK_W,
        NAV_BACK_H,
        ui.panel_alt,
        accent,
    );
    display.centered_text(
        NAV_BACK_X + NAV_BACK_W / 2,
        NAV_BACK_Y + 4,
        if zh_mode { "返回" } else { "BACK" },
        ui.text,
        ui.panel_alt,
        1,
    );
}
