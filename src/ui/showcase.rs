use crate::display::{color, palette, Display, ThemeMode};

use super::shared::{SHELL_WINDOW_H, SHELL_WINDOW_W, SHELL_WINDOW_X, SHELL_WINDOW_Y};

pub fn render_showcase_overlay(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    scene_title: &str,
    scene_subtitle: &str,
    scene_index: usize,
    scene_count: usize,
    paused: bool,
    remaining_sec: u8,
    progress_pct: u8,
) {
    let ui = palette(theme);
    let accent = if paused { ui.rose } else { ui.amber };
    let title = fit_showcase_text(display, scene_title, 130);
    let subtitle = fit_showcase_text(display, scene_subtitle, 130);
    let bar_y = SHELL_WINDOW_Y + SHELL_WINDOW_H - 28;
    let bar_fill = color::mix(ui.panel_alt, accent, 20);
    display.fill_rect(SHELL_WINDOW_X + 8, bar_y, SHELL_WINDOW_W - 16, 24, bar_fill);
    display.stroke_rect(
        SHELL_WINDOW_X + 8,
        bar_y,
        SHELL_WINDOW_W - 16,
        24,
        1,
        accent,
    );

    display.fill_rect(
        SHELL_WINDOW_X + 12,
        bar_y + 2,
        54,
        12,
        color::mix(bar_fill, accent, 26),
    );
    display.stroke_rect(SHELL_WINDOW_X + 12, bar_y + 2, 54, 12, 1, accent);
    display.text(
        SHELL_WINDOW_X + 18,
        bar_y + 4,
        if zh_mode { "展示模式" } else { "SHOWCASE" },
        ui.text,
        color::mix(bar_fill, accent, 26),
        1,
    );

    display.text(SHELL_WINDOW_X + 74, bar_y + 4, &title, ui.text, bar_fill, 1);
    display.text(
        SHELL_WINDOW_X + 74,
        bar_y + 14,
        &subtitle,
        ui.text_muted,
        bar_fill,
        1,
    );

    let status_fill = color::mix(ui.panel, accent, 20);
    display.fill_rect(SHELL_WINDOW_X + 214, bar_y + 2, 40, 12, status_fill);
    display.stroke_rect(SHELL_WINDOW_X + 214, bar_y + 2, 40, 12, 1, accent);
    display.centered_text(
        SHELL_WINDOW_X + 234,
        bar_y + 4,
        if paused {
            if zh_mode {
                "暫停"
            } else {
                "PAUSE"
            }
        } else if zh_mode {
            "自動"
        } else {
            "AUTO"
        },
        ui.text,
        status_fill,
        1,
    );

    let mut index_text = heapless::String::<20>::new();
    let _ = core::fmt::Write::write_fmt(
        &mut index_text,
        format_args!("{}/{}  {:02}s", scene_index + 1, scene_count, remaining_sec),
    );
    display.text(
        SHELL_WINDOW_X + 260,
        bar_y + 4,
        &index_text,
        ui.white,
        bar_fill,
        1,
    );

    let progress_fill = ((SHELL_WINDOW_W - 96) as u32 * progress_pct.min(100) as u32 / 100) as u16;
    display.fill_rect(
        SHELL_WINDOW_X + 12,
        bar_y + 19,
        SHELL_WINDOW_W - 96,
        3,
        color::mix(ui.panel, accent, 18),
    );
    if progress_fill > 0 {
        display.fill_rect(SHELL_WINDOW_X + 12, bar_y + 19, progress_fill, 3, accent);
    }
    display.stroke_rect(
        SHELL_WINDOW_X + 12,
        bar_y + 19,
        SHELL_WINDOW_W - 96,
        3,
        1,
        color::mix(accent, ui.white, 24),
    );
}

fn fit_showcase_text(display: &Display, text: &str, max_width: u16) -> heapless::String<48> {
    let mut out = heapless::String::<48>::new();
    if display.measure_text(text, 1) <= max_width {
        let _ = out.push_str(text);
        return out;
    }

    for ch in text.chars() {
        let mut candidate = out.clone();
        let _ = candidate.push(ch);
        let _ = candidate.push_str("..");
        if display.measure_text(&candidate, 1) > max_width {
            break;
        }
        let _ = out.push(ch);
    }
    let _ = out.push_str("..");
    out
}
