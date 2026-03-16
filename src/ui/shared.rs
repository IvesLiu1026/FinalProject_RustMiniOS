use heapless::String;

use crate::app_registry::AppIcon;
use crate::display::{color, palette, Display, Palette, ThemeMode, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y};

pub const SHELL_WINDOW_X: u16 = 10;
pub const SHELL_WINDOW_Y: u16 = 10;
pub const SHELL_WINDOW_W: u16 = 300;
pub const SHELL_WINDOW_H: u16 = 220;
pub const SHELL_CONTENT_X: u16 = 18;

pub fn draw_gradient_background(display: &mut Display, theme: ThemeMode, shift: u8) {
    let ui = palette(theme);
    for band in 0..15u16 {
        let tint = shift.wrapping_add((band * 8) as u8);
        let fill = match theme {
            ThemeMode::Light => color::mix(ui.canvas, ui.sky, tint / 3),
            ThemeMode::Dark => color::mix(ui.canvas, ui.indigo, tint / 2),
        };
        display.fill_rect(0, band * 16, SCREEN_WIDTH, 16, fill);
    }

    let line = match theme {
        ThemeMode::Light => color::mix(ui.panel, ui.white, 110),
        ThemeMode::Dark => color::mix(ui.panel, ui.text_muted, 50),
    };
    for y in (0..SCREEN_HEIGHT).step_by(12) {
        display.fill_rect(0, y, SCREEN_WIDTH, 1, line);
    }
    for x in (6..SCREEN_WIDTH).step_by(32) {
        display.fill_rect(x, 0, 1, SCREEN_HEIGHT, color::mix(ui.shadow, ui.canvas, 80));
    }
}

pub fn draw_shell_window(display: &mut Display, accent: u16, ui: &Palette) {
    let outer = color::mix(ui.canvas, ui.floor, 52);
    let bezel = color::mix(ui.panel, ui.canvas, 36);
    display.fill_rect(
        SHELL_WINDOW_X.saturating_sub(4),
        SHELL_WINDOW_Y.saturating_sub(4),
        SHELL_WINDOW_W + 8,
        SHELL_WINDOW_H + 8,
        outer,
    );
    display.stroke_rect(
        SHELL_WINDOW_X.saturating_sub(4),
        SHELL_WINDOW_Y.saturating_sub(4),
        SHELL_WINDOW_W + 8,
        SHELL_WINDOW_H + 8,
        2,
        color::mix(accent, ui.white, 96),
    );
    display.fill_rect(
        SHELL_WINDOW_X,
        SHELL_WINDOW_Y,
        SHELL_WINDOW_W,
        SHELL_WINDOW_H,
        bezel,
    );
    display.stroke_rect(
        SHELL_WINDOW_X,
        SHELL_WINDOW_Y,
        SHELL_WINDOW_W,
        SHELL_WINDOW_H,
        2,
        color::mix(ui.white, accent, 72),
    );
    display.stroke_rect(
        SHELL_WINDOW_X + 2,
        SHELL_WINDOW_Y + 2,
        SHELL_WINDOW_W - 4,
        SHELL_WINDOW_H - 4,
        1,
        ui.shadow,
    );
    display.fill_rect(
        SHELL_WINDOW_X + 4,
        SHELL_WINDOW_Y + 24,
        SHELL_WINDOW_W - 8,
        SHELL_WINDOW_H - 28,
        ui.panel,
    );
}

pub fn draw_title_bar(
    display: &mut Display,
    title: &str,
    subtitle: &str,
    accent: u16,
    ui: &Palette,
) {
    const SUBTITLE_CAP_W: u16 = 92;
    let bar_fill = color::mix(ui.indigo, accent, 34);
    let bar_x = SHELL_WINDOW_X + 4;
    let bar_y = SHELL_WINDOW_Y + 4;
    let bar_w = SHELL_WINDOW_W - 8;
    let bar_right = bar_x + bar_w;
    display.fill_rect(bar_x, bar_y, bar_w, 18, bar_fill);
    display.stroke_rect(bar_x, bar_y, bar_w, 18, 1, color::mix(ui.white, accent, 80));
    display.fill_rect(bar_x + 4, bar_y + 4, 3, 3, ui.white);
    display.fill_rect(
        bar_x + 10,
        bar_y + 4,
        3,
        3,
        color::mix(ui.white, accent, 40),
    );
    display.fill_rect(
        bar_x + 16,
        bar_y + 4,
        3,
        3,
        color::mix(ui.shadow, accent, 18),
    );

    let title_x = NAV_BACK_X + NAV_BACK_W + 10;
    let subtitle_box_x = bar_right.saturating_sub(SUBTITLE_CAP_W).saturating_sub(6);
    let title_max_width = subtitle_box_x.saturating_sub(title_x).saturating_sub(8);
    let title_text = fit_text_to_width(display, title, title_max_width, 1);
    display.text(
        title_x,
        SHELL_WINDOW_Y + 9,
        &title_text,
        ui.white,
        bar_fill,
        1,
    );
    let subtitle_fill = color::mix(ui.panel_alt, accent, 18);
    display.fill_rect(subtitle_box_x, bar_y + 2, SUBTITLE_CAP_W, 14, subtitle_fill);
    display.stroke_rect(subtitle_box_x, bar_y + 2, SUBTITLE_CAP_W, 14, 1, accent);
    let subtitle_text = fit_text_to_width(display, subtitle, SUBTITLE_CAP_W.saturating_sub(12), 1);
    display.text(
        subtitle_box_x + 6,
        SHELL_WINDOW_Y + 9,
        &subtitle_text,
        color::mix(ui.text_muted, ui.white, 90),
        subtitle_fill,
        1,
    );
}

pub fn draw_info_strip(
    display: &mut Display,
    x: u16,
    y: u16,
    width: u16,
    title: &str,
    value: &str,
    accent: u16,
    ui: &Palette,
) {
    let fill = color::mix(ui.panel_alt, accent, 36);
    let value_color = if ui.text == color::INK {
        color::mix(ui.text, accent, 52)
    } else {
        color::mix(ui.white, accent, 72)
    };
    display.fill_rect(x, y, width, 14, fill);
    display.stroke_rect(x, y, width, 14, 1, accent);
    let reserve_title = display.measure_text(title, 1).min(width / 2);
    let value_max = width
        .saturating_sub(12)
        .saturating_sub(reserve_title)
        .saturating_sub(6);
    let value_text = fit_text_to_width(display, value, value_max.max(18), 1);
    let value_width = display.measure_text(&value_text, 1);
    let title_max = width
        .saturating_sub(12)
        .saturating_sub(value_width)
        .saturating_sub(6);
    let title_text = fit_text_to_width(display, title, title_max.max(18), 1);
    display.text(x + 6, y + 3, &title_text, ui.text, fill, 1);
    let value_x = x + width.saturating_sub(value_width).saturating_sub(6);
    display.text(value_x, y + 3, &value_text, value_color, fill, 1);
}

pub fn draw_footer_hint(display: &mut Display, text: &str, accent: u16, ui: &Palette) {
    let y = SHELL_WINDOW_Y + SHELL_WINDOW_H - 20;
    let inner_x = SHELL_WINDOW_X + 8;
    let inner_w = SHELL_WINDOW_W - 16;
    let fitted = fit_text_to_width(display, text, inner_w.saturating_sub(12), 1);
    display.fill_rect(inner_x, y, inner_w, 14, ui.panel_alt);
    display.stroke_rect(inner_x, y, inner_w, 14, 1, accent);
    display.text(inner_x + 6, y + 3, &fitted, ui.text, ui.panel_alt, 1);
}

pub fn draw_scrollbar(
    display: &mut Display,
    x: u16,
    y: u16,
    height: u16,
    top_row: usize,
    visible_rows: usize,
    total_rows: usize,
    accent: u16,
    ui: &Palette,
) {
    display.fill_rect(x, y, 8, height, color::mix(ui.panel, ui.shadow, 70));
    display.stroke_rect(x, y, 8, height, 1, ui.steel);
    if total_rows == 0 || visible_rows >= total_rows {
        display.fill_rect(x + 1, y + 1, 6, height.saturating_sub(2), accent);
        return;
    }

    let thumb_h = ((height as usize * visible_rows) / total_rows)
        .max(18)
        .min(height as usize) as u16;
    let max_offset = height.saturating_sub(thumb_h);
    let max_scroll = total_rows.saturating_sub(visible_rows).max(1);
    let thumb_y = y + ((max_offset as usize * top_row.min(max_scroll)) / max_scroll) as u16;
    display.fill_rect(x + 1, thumb_y + 1, 6, thumb_h.saturating_sub(2), accent);
}

pub fn draw_desktop_shortcut(
    display: &mut Display,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    icon: AppIcon,
    label: &str,
    accent: u16,
    selected: bool,
    ui: &Palette,
) {
    let badge_fill = color::mix(ui.white, accent, 18);
    let badge_border = color::mix(accent, ui.shadow, 24);
    let shadow = color::mix(ui.shadow, accent, 24);
    let label_fill = if selected {
        color::rgb565(0, 0, 132)
    } else {
        ui.canvas
    };
    let label_text = if selected { ui.white } else { ui.text };

    display.fill_rect(x + 13, y + 13, 34, 26, shadow);
    display.fill_rect(x + 10, y + 10, 34, 26, badge_fill);
    display.stroke_rect(x + 10, y + 10, 34, 26, 1, badge_border);
    display.stroke_rect(x + 11, y + 11, 32, 24, 1, ui.white);
    draw_app_icon(display, x + 16, y + 13, icon, accent, badge_fill, ui);
    display.fill_rect(x + 12, y + 28, 6, 6, ui.white);
    display.stroke_rect(x + 12, y + 28, 6, 6, 1, badge_border);
    display.fill_rect(x + 14, y + 30, 2, 2, accent);

    display.fill_rect(x + 2, y + height - 14, width - 4, 12, label_fill);
    if selected {
        display.stroke_rect(x + 2, y + height - 14, width - 4, 12, 1, ui.white);
    }
    let label_width = display.measure_text(label, 1);
    let label_x = x + width.saturating_sub(label_width) / 2;
    display.text(label_x, y + height - 11, label, label_text, label_fill, 1);

    if selected {
        display.stroke_rect(x + 8, y + 8, 38, 30, 1, ui.white);
    }
}

pub fn draw_app_icon(
    display: &mut Display,
    x: u16,
    y: u16,
    icon: AppIcon,
    accent: u16,
    bg: u16,
    ui: &Palette,
) {
    let paper = color::mix(ui.white, bg, 20);
    match icon {
        AppIcon::Album => {
            let folder = color::mix(ui.amber, paper, 32);
            display.fill_rect(x + 2, y + 5, 15, 8, folder);
            display.stroke_rect(x + 2, y + 5, 15, 8, 1, accent);
            display.fill_rect(x + 4, y + 3, 5, 3, color::mix(folder, ui.white, 40));
            display.fill_rect(x + 6, y + 7, 8, 4, color::mix(ui.cyan, ui.white, 88));
            display.fill_rect(x + 7, y + 8, 2, 2, ui.amber);
            display.fill_rect(x + 11, y + 8, 2, 2, ui.lime);
            display.fill_rect(x + 5, y + 12, 9, 1, ui.rose);
            display.fill_rect(x + 13, y + 4, 1, 3, ui.white);
            display.fill_rect(x + 12, y + 5, 3, 1, ui.white);
        }
        AppIcon::GameCenter => {
            let shell = color::mix(paper, ui.white, 28);
            display.fill_rect(x + 2, y + 6, 14, 7, shell);
            display.stroke_rect(x + 2, y + 6, 14, 7, 1, accent);
            display.fill_rect(x + 6, y + 4, 6, 3, shell);
            display.stroke_rect(x + 6, y + 4, 6, 3, 1, accent);
            display.fill_rect(x + 5, y + 8, 2, 3, ui.text);
            display.fill_rect(x + 4, y + 9, 4, 1, ui.text);
            display.fill_rect(x + 11, y + 8, 2, 2, ui.rose);
            display.fill_rect(x + 13, y + 10, 2, 2, ui.cyan);
            display.fill_rect(x + 6, y + 14, 6, 1, ui.white);
            display.fill_rect(x + 7, y + 13, 1, 1, ui.text);
            display.fill_rect(x + 10, y + 13, 1, 1, ui.text);
        }
        AppIcon::Paint => {
            display.fill_rect(x + 3, y + 2, 12, 8, color::mix(paper, ui.white, 26));
            display.stroke_rect(x + 3, y + 2, 12, 8, 1, accent);
            display.fill_rect(x + 5, y + 4, 2, 2, ui.rose);
            display.fill_rect(x + 8, y + 3, 2, 2, ui.amber);
            display.fill_rect(x + 11, y + 5, 2, 2, ui.cyan);
            display.fill_rect(x + 6, y + 7, 5, 1, ui.lime);
            display.fill_rect(x + 1, y + 13, 9, 1, accent);
            display.fill_rect(x + 10, y + 11, 4, 4, ui.amber);
            display.fill_rect(x + 13, y + 12, 2, 2, ui.orange);
            display.fill_rect(x + 12, y + 14, 1, 2, ui.orange);
        }
        AppIcon::Settings => {
            display.fill_rect(x + 2, y + 2, 14, 11, color::mix(paper, ui.white, 26));
            display.stroke_rect(x + 2, y + 2, 14, 11, 1, accent);
            display.fill_rect(x + 4, y + 5, 10, 1, ui.text);
            display.fill_rect(x + 4, y + 8, 10, 1, ui.text);
            display.fill_rect(x + 6, y + 4, 2, 3, ui.cyan);
            display.fill_rect(x + 10, y + 7, 2, 3, ui.amber);
            display.fill_rect(x + 13, y + 2, 2, 2, ui.rose);
            display.fill_rect(x + 5, y + 14, 8, 1, ui.white);
            display.fill_rect(x + 8, y + 13, 2, 3, ui.steel);
        }
        AppIcon::Dungeon => {
            display.fill_rect(x + 3, y + 2, 11, 12, color::mix(paper, ui.white, 20));
            display.stroke_rect(x + 3, y + 2, 11, 12, 1, accent);
            display.fill_rect(x + 5, y + 6, 6, 8, accent);
            display.fill_rect(x + 7, y + 9, 2, 2, ui.white);
            display.fill_rect(x + 4, y + 4, 2, 2, ui.amber);
        }
        AppIcon::Hunter => {
            display.fill_rect(x + 7, y + 1, 2, 12, accent);
            display.fill_rect(x + 1, y + 6, 14, 2, accent);
            display.stroke_rect(x + 4, y + 3, 8, 8, 1, ui.white);
            display.fill_rect(x + 7, y + 6, 2, 2, ui.amber);
            display.fill_rect(x + 12, y + 2, 2, 2, ui.rose);
        }
        AppIcon::TapRush => {
            display.fill_rect(x + 3, y + 3, 9, 3, paper);
            display.fill_rect(x + 5, y + 6, 7, 2, paper);
            display.fill_rect(x + 2, y + 5, 5, 1, accent);
            display.fill_rect(x + 7, y + 8, 4, 1, accent);
            display.fill_rect(x + 3, y + 10, 4, 1, ui.amber);
            display.fill_rect(x + 9, y + 11, 3, 1, ui.white);
        }
        AppIcon::Racer => {
            display.fill_rect(x + 2, y + 8, 14, 5, color::mix(paper, accent, 24));
            display.stroke_rect(x + 2, y + 8, 14, 5, 1, accent);
            display.fill_rect(x + 5, y + 5, 8, 4, ui.white);
            display.fill_rect(x + 3, y + 13, 3, 2, ui.shadow);
            display.fill_rect(x + 12, y + 13, 3, 2, ui.shadow);
            display.fill_rect(x + 4, y + 9, 2, 1, ui.amber);
            display.fill_rect(x + 12, y + 9, 2, 1, ui.rose);
            display.fill_rect(x + 8, y + 3, 1, 2, ui.white);
        }
        AppIcon::Lab => {
            display.fill_rect(x + 2, y + 3, 14, 11, color::mix(paper, ui.white, 26));
            display.stroke_rect(x + 2, y + 3, 14, 11, 1, accent);
            display.fill_rect(x + 4, y + 5, 3, 3, ui.cyan);
            display.fill_rect(x + 8, y + 4, 3, 3, ui.rose);
            display.fill_rect(x + 11, y + 8, 3, 3, ui.amber);
            display.fill_rect(x + 6, y + 10, 4, 2, ui.lime);
            display.fill_rect(x + 6, y + 1, 1, 2, ui.white);
            display.fill_rect(x + 11, y + 1, 1, 2, ui.white);
        }
    }
}

pub fn render_nav_back(display: &mut Display, zh_mode: bool, accent: u16, ui: &Palette) {
    let fill = color::mix(ui.panel_alt, accent, 42);
    display.fill_rect(NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H, fill);
    display.stroke_rect(NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H, 1, accent);
    display.fill_rect(NAV_BACK_X + 6, NAV_BACK_Y + 7, 8, 2, ui.text);
    display.fill_rect(NAV_BACK_X + 6, NAV_BACK_Y + 7, 2, 2, ui.text);
    display.fill_rect(NAV_BACK_X + 8, NAV_BACK_Y + 5, 2, 2, ui.text);
    display.fill_rect(NAV_BACK_X + 8, NAV_BACK_Y + 9, 2, 2, ui.text);
    display.text(
        NAV_BACK_X + 18,
        NAV_BACK_Y + 4,
        if zh_mode { "返回" } else { "BACK" },
        ui.text,
        fill,
        1,
    );
}

pub fn fit_text_to_width(display: &Display, text: &str, max_width: u16, scale: u16) -> String<80> {
    let mut exact = String::<80>::new();
    let _ = exact.push_str(text);
    if display.measure_text(&exact, scale) <= max_width {
        return exact;
    }

    let mut fitted = String::<80>::new();
    let ellipsis = "..";
    for ch in text.chars() {
        let mut candidate = fitted.clone();
        if candidate.push(ch).is_err() {
            break;
        }
        let _ = candidate.push_str(ellipsis);
        if display.measure_text(&candidate, scale) > max_width {
            break;
        }
        let _ = fitted.push(ch);
    }
    let _ = fitted.push_str(ellipsis);
    fitted
}

pub fn theme_mode_label(theme: ThemeMode, zh_mode: bool) -> &'static str {
    match (theme, zh_mode) {
        (ThemeMode::Light, true) => "Classic Day",
        (ThemeMode::Dark, true) => "After Hours",
        (ThemeMode::Light, false) => "CLASSIC DAY",
        (ThemeMode::Dark, false) => "AFTER HOURS",
    }
}
