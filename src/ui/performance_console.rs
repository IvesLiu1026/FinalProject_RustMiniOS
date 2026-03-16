use core::fmt::Write;

use heapless::String;

use crate::display::{color, palette, Display, ThemeMode};
use crate::dungeon::RenderStrategy;
use crate::system_info;

use super::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    fit_text_to_width, render_nav_back,
};

pub const PERF_BENCH_X: u16 = 220;
pub const PERF_BENCH_Y: u16 = 184;
pub const PERF_BENCH_W: u16 = 72;
pub const PERF_BENCH_H: u16 = 28;

pub fn render_performance_console(
    display: &mut Display,
    theme: ThemeMode,
    zh_mode: bool,
    current_screen: &str,
    focus_title: &str,
    focus_subtitle: &str,
    recent_app: &str,
    pipeline: &str,
    cadence: &str,
    fps: u16,
    render_strategy: RenderStrategy,
) {
    let ui = palette(theme);
    let strong_text = if matches!(theme, ThemeMode::Light) {
        ui.text
    } else {
        ui.white
    };
    let subtle_text = if matches!(theme, ThemeMode::Light) {
        color::mix(ui.text, ui.shadow, 70)
    } else {
        ui.text_muted
    };
    draw_gradient_background(display, theme, 104);
    draw_shell_window(display, ui.cyan, &ui);
    draw_title_bar(
        display,
        if zh_mode {
            "效能儀表"
        } else {
            "PERFORMANCE CONSOLE"
        },
        if zh_mode {
            "fps / render path / flash / ram"
        } else {
            "fps / render path / flash / ram"
        },
        ui.cyan,
        &ui,
    );
    render_nav_back(display, zh_mode, ui.white, &ui);

    let mut fps_line: String<24> = String::new();
    let _ = write!(&mut fps_line, "{} FPS", fps);
    let mut frame_line: String<24> = String::new();
    if fps == 0 {
        let _ = frame_line.push_str("-- ms");
    } else {
        let _ = write!(&mut frame_line, "{} ms", (1000u32 / fps.max(1) as u32));
    }
    let mut build_line: String<32> = String::new();
    let _ = write!(
        &mut build_line,
        "v{} {}",
        system_info::app_version(),
        system_info::git_sha()
    );
    let focus_title = fit_console_text(display, focus_title, 74);
    let pipeline = fit_console_text(display, pipeline, 74);
    let cadence = fit_console_text(display, cadence, 74);
    let focus_subtitle = fit_console_text(display, focus_subtitle, 176);
    let current_screen = fit_console_text(display, current_screen, 44);
    let build_line = fit_console_text(display, &build_line, 54);
    draw_info_strip(
        display,
        18,
        52,
        92,
        if zh_mode { "監看" } else { "FOCUS" },
        &current_screen,
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

    display.panel(18, 74, 136, 96, ui.panel, ui.orange);
    let render_title = fit_text_to_width(
        display,
        if zh_mode {
            "渲染路徑"
        } else {
            "RENDER PATH"
        },
        74,
        1,
    );
    display.text(28, 86, &render_title, strong_text, ui.panel, 1);
    draw_scope_icon(display, 116, 82, &ui);
    draw_info_strip(
        display,
        28,
        106,
        116,
        if zh_mode { "焦點" } else { "TARGET" },
        &focus_title,
        ui.cyan,
        &ui,
    );
    draw_info_strip(
        display,
        28,
        124,
        116,
        if zh_mode { "管線" } else { "PIPELINE" },
        &pipeline,
        ui.lime,
        &ui,
    );
    draw_info_strip(
        display,
        28,
        142,
        116,
        if zh_mode { "節奏" } else { "CADENCE" },
        &cadence,
        ui.amber,
        &ui,
    );

    display.panel(166, 74, 136, 96, ui.panel, ui.lime);
    let memory_title = fit_text_to_width(
        display,
        if zh_mode { "記憶體用量" } else { "MEMORY" },
        74,
        1,
    );
    display.text(176, 86, &memory_title, strong_text, ui.panel, 1);
    draw_memory_icon(display, 264, 82, &ui);

    let mut flash_line: String<24> = String::new();
    let _ = write!(
        &mut flash_line,
        "{}/{}",
        compact_kb(system_info::flash_used_bytes()),
        compact_kb(system_info::flash_capacity_bytes())
    );
    draw_info_strip(
        display,
        176,
        106,
        116,
        if zh_mode { "程式" } else { "FLASH" },
        &flash_line,
        ui.amber,
        &ui,
    );

    let mut bss_line: String<24> = String::new();
    let _ = write!(
        &mut bss_line,
        "{}/{}",
        compact_kb(system_info::bss_bytes()),
        compact_kb(system_info::ram_capacity_bytes())
    );
    draw_info_strip(
        display,
        176,
        124,
        116,
        if zh_mode { "靜態" } else { "BSS" },
        &bss_line,
        ui.rose,
        &ui,
    );

    let mut data_line: String<24> = String::new();
    let _ = write!(&mut data_line, "{}B", system_info::data_bytes());
    draw_info_strip(
        display,
        176,
        142,
        116,
        if zh_mode { "資料" } else { "DATA" },
        &data_line,
        ui.white,
        &ui,
    );

    display.panel(18, 176, 284, 46, ui.panel_alt, ui.cyan);
    let snapshot_title = fit_text_to_width(
        display,
        if zh_mode {
            "系統快照"
        } else {
            "SYSTEM SNAPSHOT"
        },
        120,
        1,
    );
    display.text(28, 188, &snapshot_title, strong_text, ui.panel_alt, 1);
    display.fill_rect(176, 182, 34, 12, color::mix(ui.panel, ui.cyan, 18));
    display.stroke_rect(176, 182, 34, 12, 1, ui.cyan);
    display.centered_text(
        193,
        185,
        if zh_mode { "即時" } else { "LIVE" },
        strong_text,
        color::mix(ui.panel, ui.cyan, 18),
        1,
    );
    display.text(28, 202, &focus_subtitle, subtle_text, ui.panel_alt, 1);

    let mut recent_line: String<32> = String::new();
    let _ = write!(
        &mut recent_line,
        "{} {} / {}",
        if zh_mode { "最近" } else { "LAST" },
        recent_app,
        render_strategy.label()
    );
    let recent_line = fit_console_text(display, &recent_line, 176);
    display.text(28, 213, &recent_line, strong_text, ui.panel_alt, 1);

    display.fill_rect(
        PERF_BENCH_X,
        PERF_BENCH_Y,
        PERF_BENCH_W,
        PERF_BENCH_H,
        color::mix(ui.panel, ui.shadow, 30),
    );
    display.stroke_rect(
        PERF_BENCH_X,
        PERF_BENCH_Y,
        PERF_BENCH_W,
        PERF_BENCH_H,
        1,
        ui.cyan,
    );
    let frame_width = display.measure_text(&frame_line, 1);
    let frame_x = (PERF_BENCH_X + PERF_BENCH_W / 2).saturating_sub(frame_width / 2);
    display.text(
        frame_x,
        191,
        &frame_line,
        strong_text,
        color::mix(ui.panel, ui.shadow, 30),
        1,
    );
    display.text(
        PERF_BENCH_X + 10,
        202,
        if zh_mode { "K1 測試" } else { "K1 BENCH" },
        strong_text,
        color::mix(ui.panel, ui.shadow, 30),
        1,
    );
    let bench_fill = ((fps.min(30) as u32 * (PERF_BENCH_W - 12) as u32) / 30) as u16;
    display.fill_rect(
        PERF_BENCH_X + 6,
        PERF_BENCH_Y + 22,
        PERF_BENCH_W - 12,
        3,
        color::mix(ui.panel_alt, ui.cyan, 14),
    );
    if bench_fill > 0 {
        display.fill_rect(PERF_BENCH_X + 6, PERF_BENCH_Y + 22, bench_fill, 3, ui.cyan);
    }
    display.stroke_rect(
        PERF_BENCH_X + 6,
        PERF_BENCH_Y + 22,
        PERF_BENCH_W - 12,
        3,
        1,
        color::mix(ui.cyan, ui.white, 24),
    );

    draw_footer_hint(
        display,
        if zh_mode {
            "按 K1 或點 BENCH 執行測試"
        } else {
            "K1 OR TAP BENCH BOX TO RUN TESTS"
        },
        ui.cyan,
        &ui,
    );
}

fn compact_kb(bytes: usize) -> String<12> {
    let mut out: String<12> = String::new();
    if bytes >= 1024 {
        let whole = bytes / 1024;
        let frac = ((bytes % 1024) * 10) / 1024;
        let _ = write!(&mut out, "{}.{}K", whole, frac);
    } else {
        let _ = write!(&mut out, "{}B", bytes);
    }
    out
}

fn fit_console_text(display: &Display, text: &str, max_width: u16) -> String<32> {
    let mut out: String<32> = String::new();
    if display.measure_text(text, 1) <= max_width {
        let _ = out.push_str(text);
        return out;
    }

    for ch in text.chars() {
        let mut candidate: String<32> = String::new();
        let _ = candidate.push_str(&out);
        let _ = candidate.push(ch);
        let _ = candidate.push_str("...");
        if display.measure_text(&candidate, 1) > max_width {
            break;
        }
        let _ = out.push(ch);
    }
    let _ = out.push_str("...");
    out
}

fn draw_scope_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 24, 18, color::mix(ui.panel_alt, ui.cyan, 20));
    display.stroke_rect(x, y, 24, 18, 1, ui.cyan);
    display.fill_rect(x + 10, y + 3, 4, 10, ui.white);
    display.fill_rect(x + 7, y + 6, 10, 4, ui.white);
    display.fill_rect(x + 4, y + 14, 16, 1, ui.steel);
}

fn draw_memory_icon(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 22, 18, color::mix(ui.panel_alt, ui.lime, 18));
    display.stroke_rect(x, y, 22, 18, 1, ui.lime);
    display.fill_rect(x + 5, y + 4, 12, 10, ui.text);
    display.fill_rect(x + 8, y + 7, 6, 4, ui.white);
    let mut pin_x = x + 3;
    while pin_x <= x + 17 {
        display.fill_rect(pin_x, y + 1, 1, 2, ui.amber);
        display.fill_rect(pin_x, y + 15, 1, 2, ui.amber);
        pin_x += 3;
    }
}
