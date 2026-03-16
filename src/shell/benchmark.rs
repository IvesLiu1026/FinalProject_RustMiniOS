use core::fmt::Write;

use heapless::String;

use super::update::touch_started_in_rect;
use super::*;
use crate::system_info;
use crate::ui::{
    draw_footer_hint, draw_gradient_background, draw_shell_window, draw_title_bar, render_nav_back,
};

const BENCH_MENU_START_X: u16 = 90;
const BENCH_MENU_START_Y: u16 = 190;
const BENCH_MENU_START_W: u16 = 140;
const BENCH_MENU_START_H: u16 = 24;
const BENCH_RESULT_RERUN_X: u16 = 48;
const BENCH_RESULT_RERUN_Y: u16 = 192;
const BENCH_RESULT_RERUN_W: u16 = 100;
const BENCH_RESULT_BACK_X: u16 = 172;
const BENCH_RESULT_BACK_Y: u16 = 192;
const BENCH_RESULT_BACK_W: u16 = 100;
const BENCH_BUTTON_H: u16 = 24;
const BENCH_RGB_W: usize = 60;
const BENCH_RGB_H: usize = 36;
const BENCH_RGB_PIXELS: usize = BENCH_RGB_W * BENCH_RGB_H;
const BENCH_RGB_SCALE: u16 = 5;
const BENCH_RGB_X: u16 = 10;
const BENCH_RGB_Y: u16 = 30;

static mut BENCH_RGB_BUFFER: [u16; BENCH_RGB_PIXELS] = [0; BENCH_RGB_PIXELS];

impl BenchmarkCase {
    fn title(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::UiFill, true) => "UI Fill",
            (Self::RgbBlit, true) => "RGB Blit",
            (Self::PseudoRacer, true) => "Pseudo Racer",
            (Self::GraphicsLab, true) => "Graphics Lab",
            (Self::UiFill, false) => "UI FILL",
            (Self::RgbBlit, false) => "RGB BLIT",
            (Self::PseudoRacer, false) => "PSEUDO RACER",
            (Self::GraphicsLab, false) => "GRAPHICS LAB",
        }
    }

    fn subtitle(self, zh_mode: bool) -> &'static str {
        match (self, zh_mode) {
            (Self::UiFill, true) => "panel / rect / text throughput",
            (Self::RgbBlit, true) => "scaled rgb565 viewport transfer",
            (Self::PseudoRacer, true) => "buffered road sample scene",
            (Self::GraphicsLab, true) => "reduced framebuffer math demo",
            (Self::UiFill, false) => "panel / rect / text throughput",
            (Self::RgbBlit, false) => "scaled rgb565 viewport transfer",
            (Self::PseudoRacer, false) => "buffered road sample scene",
            (Self::GraphicsLab, false) => "reduced framebuffer math demo",
        }
    }

    const fn duration_ms(self) -> u32 {
        match self {
            Self::UiFill => 2200,
            Self::RgbBlit => 2200,
            Self::PseudoRacer => 2800,
            Self::GraphicsLab => 2800,
        }
    }
}

impl MiniOs {
    pub(super) fn enter_benchmark_mode(&mut self) {
        self.benchmark_mode = BenchmarkMode {
            state: BenchmarkState::Menu,
            case_index: 0,
            case_elapsed_ms: 0,
            fps_sum: 0,
            fps_samples: 0,
            min_fps: u16::MAX,
            stage_full_redraw: true,
            rgb_phase: 0,
            results: [EMPTY_BENCH_RESULT; BENCH_COUNT],
        };
        self.switch_screen(Screen::Benchmark);
    }

    pub(super) fn update_benchmark(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> bool {
        if input.home_chord()
            || touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            self.switch_screen(Screen::PerformanceConsole);
            return true;
        }

        match self.benchmark_mode.state {
            BenchmarkState::Menu => {
                if input.k1_just_pressed
                    || touch_started_in_rect(
                        touch,
                        BENCH_MENU_START_X,
                        BENCH_MENU_START_Y,
                        BENCH_MENU_START_W,
                        BENCH_MENU_START_H,
                    )
                {
                    self.start_benchmark_run();
                    return true;
                }
                false
            }
            BenchmarkState::Running => self.update_benchmark_running(dt_ms),
            BenchmarkState::Results => {
                if input.k1_just_pressed
                    || touch_started_in_rect(
                        touch,
                        BENCH_RESULT_RERUN_X,
                        BENCH_RESULT_RERUN_Y,
                        BENCH_RESULT_RERUN_W,
                        BENCH_BUTTON_H,
                    )
                {
                    self.start_benchmark_run();
                    return true;
                }
                if input.wkup_just_pressed
                    || touch_started_in_rect(
                        touch,
                        BENCH_RESULT_BACK_X,
                        BENCH_RESULT_BACK_Y,
                        BENCH_RESULT_BACK_W,
                        BENCH_BUTTON_H,
                    )
                {
                    self.switch_screen(Screen::PerformanceConsole);
                    return true;
                }
                false
            }
        }
    }

    pub(super) fn render_benchmark(&mut self, display: &mut Display, full_refresh: bool) {
        match self.benchmark_mode.state {
            BenchmarkState::Menu => self.render_benchmark_menu(display),
            BenchmarkState::Running => self.render_benchmark_running(display, full_refresh),
            BenchmarkState::Results => self.render_benchmark_results(display),
        }
    }

    fn start_benchmark_run(&mut self) {
        self.benchmark_mode.state = BenchmarkState::Running;
        self.benchmark_mode.case_index = 0;
        self.benchmark_mode.case_elapsed_ms = 0;
        self.benchmark_mode.fps_sum = 0;
        self.benchmark_mode.fps_samples = 0;
        self.benchmark_mode.min_fps = u16::MAX;
        self.benchmark_mode.stage_full_redraw = true;
        self.benchmark_mode.rgb_phase = 0;
        self.benchmark_mode.results = [EMPTY_BENCH_RESULT; BENCH_COUNT];
        self.start_benchmark_case(0);
        self.force_full_redraw = true;
    }

    fn start_benchmark_case(&mut self, index: usize) {
        self.benchmark_mode.case_index = index.min(BENCH_COUNT - 1);
        self.benchmark_mode.case_elapsed_ms = 0;
        self.benchmark_mode.fps_sum = 0;
        self.benchmark_mode.fps_samples = 0;
        self.benchmark_mode.min_fps = u16::MAX;
        self.benchmark_mode.stage_full_redraw = true;
        self.benchmark_mode.rgb_phase = 0;

        match BENCH_CASES[self.benchmark_mode.case_index] {
            BenchmarkCase::UiFill | BenchmarkCase::RgbBlit => {}
            BenchmarkCase::PseudoRacer => {
                self.performance_focus_app = Some(AppId::PseudoRacer);
                self.pseudo_racer.start_showcase(1);
            }
            BenchmarkCase::GraphicsLab => {
                self.performance_focus_app = Some(AppId::GraphicsLab);
                self.graphics_lab.start_showcase(2);
            }
        }
        self.force_full_redraw = true;
    }

    fn update_benchmark_running(&mut self, dt_ms: u32) -> bool {
        match BENCH_CASES[self.benchmark_mode.case_index] {
            BenchmarkCase::UiFill => {
                self.benchmark_mode.rgb_phase = self
                    .benchmark_mode
                    .rgb_phase
                    .wrapping_add((dt_ms / 8).max(1) as u16);
            }
            BenchmarkCase::RgbBlit => {
                self.benchmark_mode.rgb_phase = self
                    .benchmark_mode
                    .rgb_phase
                    .wrapping_add((dt_ms / 6).max(1) as u16);
            }
            BenchmarkCase::PseudoRacer => {
                let _ = self.pseudo_racer.update(
                    &ButtonSnapshot::default(),
                    &TouchState::default(),
                    dt_ms,
                );
                if self.pseudo_racer.take_persist_request() {
                    self.save_storage();
                }
                if self.pseudo_racer.take_full_redraw_request() {
                    self.benchmark_mode.stage_full_redraw = true;
                }
            }
            BenchmarkCase::GraphicsLab => {
                let _ = self.graphics_lab.update(
                    &ButtonSnapshot::default(),
                    &TouchState::default(),
                    dt_ms,
                );
                if self.graphics_lab.take_full_redraw_request() {
                    self.benchmark_mode.stage_full_redraw = true;
                }
            }
        }

        if self.fps_estimate > 0 {
            self.benchmark_mode.fps_sum = self
                .benchmark_mode
                .fps_sum
                .saturating_add(self.fps_estimate as u32);
            self.benchmark_mode.fps_samples = self.benchmark_mode.fps_samples.saturating_add(1);
            self.benchmark_mode.min_fps = self.benchmark_mode.min_fps.min(self.fps_estimate);
        }

        self.benchmark_mode.case_elapsed_ms =
            self.benchmark_mode.case_elapsed_ms.saturating_add(dt_ms);
        if self.benchmark_mode.case_elapsed_ms
            >= BENCH_CASES[self.benchmark_mode.case_index].duration_ms()
        {
            let samples = self.benchmark_mode.fps_samples.max(1) as u32;
            let avg_fps = (self.benchmark_mode.fps_sum / samples) as u16;
            self.benchmark_mode.results[self.benchmark_mode.case_index] = BenchmarkResult {
                avg_fps,
                min_fps: if self.benchmark_mode.min_fps == u16::MAX {
                    0
                } else {
                    self.benchmark_mode.min_fps
                },
                duration_ms: self.benchmark_mode.case_elapsed_ms,
            };

            let next = self.benchmark_mode.case_index + 1;
            if next >= BENCH_COUNT {
                self.benchmark_mode.state = BenchmarkState::Results;
                self.force_full_redraw = true;
            } else {
                self.start_benchmark_case(next);
            }
            return true;
        }

        true
    }

    fn render_benchmark_menu(&self, display: &mut Display) {
        let zh = self.language.is_zh();
        let ui = palette(self.theme);
        let strong_text = benchmark_strong_text(self.theme, &ui);
        draw_gradient_background(display, self.theme, 88);
        draw_shell_window(display, ui.amber, &ui);
        draw_title_bar(
            display,
            if zh { "效能測試" } else { "BENCHMARK" },
            if zh {
                "ui fill / rgb blit / racer / graphics"
            } else {
                "ui fill / rgb blit / racer / graphics"
            },
            ui.amber,
            &ui,
        );
        render_nav_back(display, zh, ui.white, &ui);

        display.panel(18, 56, 284, 24, ui.panel_alt, ui.amber);
        display.text(
            28,
            64,
            if zh {
                "TIMED SWEEP 4 CASES  •  LIVE FPS SAMPLING  •  LCD PATH CHECK"
            } else {
                "TIMED SWEEP 4 CASES  •  LIVE FPS SAMPLING  •  LCD PATH CHECK"
            },
            strong_text,
            ui.panel_alt,
            1,
        );

        for (index, case) in BENCH_CASES.iter().enumerate() {
            let x = if index & 1 == 0 { 18 } else { 162 };
            let y = 88 + (index as u16 / 2) * 48;
            let accent = benchmark_case_accent(*case, &ui);
            let title = fit_benchmark_text(display, case.title(zh), 76);
            let subtitle = fit_benchmark_text(display, case.subtitle(zh), 120);
            let mut duration_line: String<16> = String::new();
            let _ = write!(&mut duration_line, "{} ms", case.duration_ms());
            display.panel(x, y, 140, 44, ui.panel, accent);
            display.text(x + 10, y + 8, &title, strong_text, ui.panel, 1);
            display.fill_rect(x + 94, y + 6, 36, 12, color::mix(ui.panel_alt, accent, 28));
            display.stroke_rect(x + 94, y + 6, 36, 12, 1, accent);
            display.centered_text(
                x + 112,
                y + 10,
                &duration_line,
                strong_text,
                color::mix(ui.panel_alt, accent, 28),
                1,
            );
            display.text(x + 10, y + 22, &subtitle, ui.text_muted, ui.panel, 1);
            display.fill_rect(x + 10, y + 34, 82, 4, color::mix(ui.panel_alt, accent, 18));
            display.fill_rect(x + 10, y + 34, 36 + (index as u16 * 12), 4, accent);
            display.text(
                x + 98,
                y + 31,
                benchmark_case_tag(*case, zh),
                strong_text,
                ui.panel,
                1,
            );
        }

        display.fill_rect(
            BENCH_MENU_START_X,
            BENCH_MENU_START_Y,
            BENCH_MENU_START_W,
            BENCH_MENU_START_H,
            color::mix(ui.panel_alt, ui.cyan, 34),
        );
        display.stroke_rect(
            BENCH_MENU_START_X,
            BENCH_MENU_START_Y,
            BENCH_MENU_START_W,
            BENCH_MENU_START_H,
            1,
            ui.cyan,
        );
        display.centered_text(
            BENCH_MENU_START_X + BENCH_MENU_START_W / 2,
            BENCH_MENU_START_Y + 8,
            if zh {
                "K1 啟動測試"
            } else {
                "K1 RUN BENCH"
            },
            ui.text,
            color::mix(ui.panel_alt, ui.cyan, 34),
            1,
        );

        draw_footer_hint(
            display,
            if zh {
                "RUN FOUR TESTS AND CAPTURE AVG / MIN FPS"
            } else {
                "RUN FOUR TESTS AND CAPTURE AVG / MIN FPS"
            },
            ui.amber,
            &ui,
        );
    }

    fn render_benchmark_running(&mut self, display: &mut Display, full_refresh: bool) {
        let case = BENCH_CASES[self.benchmark_mode.case_index];
        match case {
            BenchmarkCase::UiFill => self.render_benchmark_ui_fill(display, full_refresh),
            BenchmarkCase::RgbBlit => self.render_benchmark_rgb_blit(display, full_refresh),
            BenchmarkCase::PseudoRacer => {
                if full_refresh || self.benchmark_mode.stage_full_redraw {
                    self.pseudo_racer
                        .render(display, self.theme, self.language.is_zh());
                } else {
                    self.pseudo_racer
                        .render_partial(display, self.theme, self.language.is_zh());
                }
            }
            BenchmarkCase::GraphicsLab => {
                if full_refresh || self.benchmark_mode.stage_full_redraw {
                    self.graphics_lab
                        .render(display, self.theme, self.language.is_zh());
                } else {
                    self.graphics_lab
                        .render_partial(display, self.theme, self.language.is_zh());
                }
            }
        }
        self.benchmark_mode.stage_full_redraw = false;
        self.render_benchmark_overlay(display, case);
    }

    fn render_benchmark_results(&self, display: &mut Display) {
        let zh = self.language.is_zh();
        let ui = palette(self.theme);
        let strong_text = benchmark_strong_text(self.theme, &ui);
        let (overall_avg, overall_min, best_case) = benchmark_summary(&self.benchmark_mode.results);
        let profile = benchmark_profile_label(overall_avg, zh);
        let profile_accent = benchmark_profile_accent(overall_avg, &ui);
        let best_title = fit_benchmark_text(display, BENCH_CASES[best_case].title(zh), 72);
        let score = benchmark_score(&self.benchmark_mode.results);
        let grade = benchmark_grade(score, zh);
        let mut avg_line: String<20> = String::new();
        let _ = write!(&mut avg_line, "{} FPS", overall_avg);
        let mut worst_line: String<20> = String::new();
        let _ = write!(&mut worst_line, "MIN {}", overall_min);
        let mut score_line: String<20> = String::new();
        let _ = write!(&mut score_line, "{}", score);
        let mut memory_line: String<32> = String::new();
        let _ = write!(
            &mut memory_line,
            "FLASH {}  BSS {}",
            benchmark_compact_kb(system_info::flash_used_bytes()),
            benchmark_compact_kb(system_info::bss_bytes())
        );

        draw_gradient_background(display, self.theme, 88);
        draw_shell_window(display, profile_accent, &ui);
        draw_title_bar(
            display,
            if zh { "測試結果" } else { "BENCH RESULTS" },
            if zh {
                "avg fps / min fps / timed samples"
            } else {
                "avg fps / min fps / timed samples"
            },
            profile_accent,
            &ui,
        );
        render_nav_back(display, zh, ui.white, &ui);

        display.panel(18, 56, 284, 40, ui.panel_alt, profile_accent);
        display.text(
            28,
            63,
            if zh { "整體平均" } else { "OVERALL AVG" },
            strong_text,
            ui.panel_alt,
            1,
        );
        display.text(28, 73, &avg_line, ui.white, ui.panel_alt, 2);
        display.text(
            116,
            63,
            if zh { "分數" } else { "SCORE" },
            strong_text,
            ui.panel_alt,
            1,
        );
        display.text(116, 73, &score_line, ui.white, ui.panel_alt, 2);
        display.text(
            188,
            63,
            if zh { "最佳" } else { "BEST" },
            strong_text,
            ui.panel_alt,
            1,
        );
        display.text(188, 73, &best_title, ui.white, ui.panel_alt, 1);
        display.fill_rect(256, 60, 34, 18, color::mix(ui.panel, profile_accent, 24));
        display.stroke_rect(256, 60, 34, 18, 1, profile_accent);
        display.centered_text(
            273,
            66,
            grade,
            ui.white,
            color::mix(ui.panel, profile_accent, 24),
            1,
        );
        display.text(
            256,
            81,
            if zh { "輪廓" } else { "PROFILE" },
            ui.text_muted,
            ui.panel_alt,
            1,
        );
        display.text(208, 81, profile, ui.white, ui.panel_alt, 1);
        display.text(28, 86, &worst_line, ui.text_muted, ui.panel_alt, 1);

        display.fill_rect(18, 102, 284, 12, color::mix(ui.panel, profile_accent, 18));
        display.stroke_rect(18, 102, 284, 12, 1, ui.steel);
        let memory_line = fit_benchmark_text(display, &memory_line, 240);
        display.text(
            26,
            105,
            &memory_line,
            strong_text,
            color::mix(ui.panel, profile_accent, 18),
            1,
        );
        display.text(
            240,
            105,
            if zh { "TARGET 15 FPS" } else { "TARGET 15 FPS" },
            ui.text_muted,
            color::mix(ui.panel, profile_accent, 18),
            1,
        );

        for (index, case) in BENCH_CASES.iter().enumerate() {
            let x = if index & 1 == 0 { 18 } else { 162 };
            let y = 120 + (index as u16 / 2) * 34;
            let result = self.benchmark_mode.results[index];
            let accent = benchmark_case_accent(*case, &ui);
            let fill = color::mix(ui.panel_alt, accent, 12);
            let title = fit_benchmark_text(display, case.title(zh), 70);
            let case_grade = benchmark_case_grade(result, zh);
            let mut duration_line: String<16> = String::new();
            let _ = write!(&mut duration_line, "{}ms", result.duration_ms);
            let mut avg_line: String<16> = String::new();
            let _ = write!(&mut avg_line, "AVG {}", result.avg_fps);
            let mut min_line: String<16> = String::new();
            let _ = write!(&mut min_line, "MIN {}", result.min_fps);
            let bar_w = ((result.avg_fps.min(30) as u32 * 54) / 30) as u16;

            display.fill_rect(x, y, 140, 30, fill);
            display.stroke_rect(x, y, 140, 30, 1, accent);
            display.text(x + 8, y + 6, &title, strong_text, fill, 1);
            display.fill_rect(x + 92, y + 4, 38, 9, color::mix(ui.panel, accent, 18));
            display.stroke_rect(x + 92, y + 4, 38, 9, 1, accent);
            display.centered_text(
                x + 111,
                y + 6,
                &duration_line,
                strong_text,
                color::mix(ui.panel, accent, 18),
                1,
            );
            display.fill_rect(x + 92, y + 16, 38, 9, color::mix(ui.panel_alt, accent, 22));
            display.stroke_rect(x + 92, y + 16, 38, 9, 1, color::mix(accent, ui.white, 24));
            display.centered_text(
                x + 111,
                y + 18,
                case_grade,
                ui.white,
                color::mix(ui.panel_alt, accent, 22),
                1,
            );
            display.fill_rect(x + 8, y + 18, 54, 5, color::mix(ui.panel, accent, 18));
            if bar_w > 0 {
                display.fill_rect(x + 8, y + 18, bar_w, 5, accent);
            }
            display.stroke_rect(x + 8, y + 18, 54, 5, 1, color::mix(accent, ui.white, 40));
            display.text(x + 68, y + 17, &avg_line, ui.white, fill, 1);
            display.text(x + 68, y + 25, &min_line, ui.text_muted, fill, 1);
        }

        display.fill_rect(
            BENCH_RESULT_RERUN_X,
            BENCH_RESULT_RERUN_Y,
            BENCH_RESULT_RERUN_W,
            BENCH_BUTTON_H,
            color::mix(ui.panel_alt, ui.cyan, 32),
        );
        display.stroke_rect(
            BENCH_RESULT_RERUN_X,
            BENCH_RESULT_RERUN_Y,
            BENCH_RESULT_RERUN_W,
            BENCH_BUTTON_H,
            1,
            ui.cyan,
        );
        display.centered_text(
            BENCH_RESULT_RERUN_X + BENCH_RESULT_RERUN_W / 2,
            BENCH_RESULT_RERUN_Y + 8,
            if zh { "K1 重跑" } else { "K1 RERUN" },
            ui.text,
            color::mix(ui.panel_alt, ui.cyan, 32),
            1,
        );

        display.fill_rect(
            BENCH_RESULT_BACK_X,
            BENCH_RESULT_BACK_Y,
            BENCH_RESULT_BACK_W,
            BENCH_BUTTON_H,
            color::mix(ui.panel_alt, profile_accent, 18),
        );
        display.stroke_rect(
            BENCH_RESULT_BACK_X,
            BENCH_RESULT_BACK_Y,
            BENCH_RESULT_BACK_W,
            BENCH_BUTTON_H,
            1,
            profile_accent,
        );
        display.centered_text(
            BENCH_RESULT_BACK_X + BENCH_RESULT_BACK_W / 2,
            BENCH_RESULT_BACK_Y + 8,
            if zh { "WK 返回" } else { "WK RETURN" },
            ui.text,
            color::mix(ui.panel_alt, profile_accent, 18),
            1,
        );

        let footer_fill = color::mix(ui.panel, profile_accent, 12);
        display.fill_rect(18, 220, 284, 8, footer_fill);
        display.stroke_rect(18, 220, 284, 8, 1, ui.steel);
        display.text(
            24,
            221,
            if zh {
                "SCORE + GRADE SUMMARIZE LCD UI / BLIT / RACER / LAB LOAD"
            } else {
                "SCORE + GRADE SUMMARIZE LCD UI / BLIT / RACER / LAB LOAD"
            },
            ui.text_muted,
            footer_fill,
            1,
        );
    }

    fn render_benchmark_ui_fill(&self, display: &mut Display, full_refresh: bool) {
        let ui = palette(self.theme);
        if full_refresh {
            display.fill_rect(0, 0, 320, 240, color::mix(ui.canvas, ui.floor, 46));
        }
        let phase = self.benchmark_mode.rgb_phase as u16;
        for band in 0..8u16 {
            let y = 18 + band * 24;
            let width = 180 + ((phase + band * 7) % 104);
            let accent = match band & 3 {
                0 => ui.cyan,
                1 => ui.orange,
                2 => ui.rose,
                _ => ui.lime,
            };
            display.fill_rect(20, y, width, 16, color::mix(ui.panel_alt, accent, 28));
            display.stroke_rect(20, y, width, 16, 1, accent);
            display.fill_rect(24 + (phase + band * 9) % 42, y + 4, 10, 8, ui.white);
        }
        for box_idx in 0..6u16 {
            let x = 18 + (box_idx % 3) * 96;
            let y = 34 + (box_idx / 3) * 90;
            let shift = (phase / 3 + box_idx * 6) % 18;
            display.fill_rect(x, y + shift, 84, 36, color::mix(ui.panel, ui.indigo, 24));
            display.stroke_rect(x, y + shift, 84, 36, 1, ui.steel);
            display.fill_rect(x + 8, y + shift + 8, 44, 8, ui.cyan);
            display.fill_rect(x + 8, y + shift + 20, 60, 6, ui.text_muted);
        }
    }

    fn render_benchmark_rgb_blit(&mut self, display: &mut Display, full_refresh: bool) {
        let ui = palette(self.theme);
        if full_refresh {
            display.fill_rect(0, 0, 320, 240, color::mix(ui.shadow, ui.indigo, 20));
        }
        unsafe {
            let buffer = &mut *core::ptr::addr_of_mut!(BENCH_RGB_BUFFER);
            let phase = self.benchmark_mode.rgb_phase as usize;
            for y in 0..BENCH_RGB_H {
                for x in 0..BENCH_RGB_W {
                    let idx = y * BENCH_RGB_W + x;
                    let checker = ((x / 6 + y / 6 + phase / 3) & 1) == 0;
                    let pulse = ((x * 3 + y * 5 + phase) & 31) as u8;
                    buffer[idx] = if checker {
                        color::rgb565(30 + pulse * 4, 90 + pulse * 3, 180)
                    } else {
                        color::rgb565(180, 44 + pulse * 2, 40 + pulse * 3)
                    };
                }
            }
            display.draw_rgb565_scaled(
                BENCH_RGB_X,
                BENCH_RGB_Y,
                BENCH_RGB_W as u16,
                BENCH_RGB_H as u16,
                BENCH_RGB_SCALE,
                buffer,
            );
        }
        display.stroke_rect(
            BENCH_RGB_X - 2,
            BENCH_RGB_Y - 2,
            BENCH_RGB_W as u16 * BENCH_RGB_SCALE + 4,
            BENCH_RGB_H as u16 * BENCH_RGB_SCALE + 4,
            1,
            ui.amber,
        );
    }

    fn render_benchmark_overlay(&self, display: &mut Display, case: BenchmarkCase) {
        let zh = self.language.is_zh();
        let ui = palette(self.theme);
        let strong_text = benchmark_strong_text(self.theme, &ui);
        let overlay_fill = color::mix(ui.panel, ui.shadow, 26);
        let y = 206;
        display.fill_rect(8, y, 304, 26, overlay_fill);
        display.stroke_rect(8, y, 304, 26, 1, ui.steel);

        let mut progress_line: String<48> = String::new();
        let duration = case.duration_ms().max(1);
        let progress = (self.benchmark_mode.case_elapsed_ms.min(duration) * 100) / duration;
        let _ = write!(
            &mut progress_line,
            "{} {}/{}  {}%",
            case.title(zh),
            self.benchmark_mode.case_index + 1,
            BENCH_COUNT,
            progress
        );
        let progress_line = fit_benchmark_text(display, &progress_line, 146);
        display.text(14, y + 4, &progress_line, strong_text, overlay_fill, 1);

        let avg = if self.benchmark_mode.fps_samples == 0 {
            0
        } else {
            (self.benchmark_mode.fps_sum / self.benchmark_mode.fps_samples as u32) as u16
        };
        let min = if self.benchmark_mode.min_fps == u16::MAX {
            0
        } else {
            self.benchmark_mode.min_fps
        };
        let mut stat_line: String<32> = String::new();
        let _ = write!(&mut stat_line, "AVG {} / MIN {}", avg, min);
        let stat_width = display.measure_text(&stat_line, 1);
        let stat_x = 306u16.saturating_sub(stat_width);
        display.text(stat_x, y + 4, &stat_line, ui.white, overlay_fill, 1);

        display.fill_rect(14, y + 16, 170, 4, color::mix(ui.panel_alt, ui.white, 18));
        let progress_fill = ((self.benchmark_mode.case_elapsed_ms.min(duration) as u32 * 170)
            / duration as u32) as u16;
        if progress_fill > 0 {
            display.fill_rect(
                14,
                y + 16,
                progress_fill,
                4,
                benchmark_case_accent(case, &ui),
            );
        }
        display.stroke_rect(14, y + 16, 170, 4, 1, color::mix(ui.steel, ui.white, 24));
        display.text(
            194,
            y + 15,
            benchmark_case_tag(case, zh),
            strong_text,
            overlay_fill,
            1,
        );
    }
}

fn benchmark_case_accent(case: BenchmarkCase, ui: &crate::display::Palette) -> u16 {
    match case {
        BenchmarkCase::UiFill => ui.cyan,
        BenchmarkCase::RgbBlit => ui.amber,
        BenchmarkCase::PseudoRacer => ui.orange,
        BenchmarkCase::GraphicsLab => ui.lime,
    }
}

fn benchmark_case_tag(case: BenchmarkCase, zh: bool) -> &'static str {
    match (case, zh) {
        (BenchmarkCase::UiFill, true) => "UI PATH",
        (BenchmarkCase::RgbBlit, true) => "BLIT",
        (BenchmarkCase::PseudoRacer, true) => "VIEWPORT",
        (BenchmarkCase::GraphicsLab, true) => "FRAMEBUF",
        (BenchmarkCase::UiFill, false) => "UI PATH",
        (BenchmarkCase::RgbBlit, false) => "BLIT",
        (BenchmarkCase::PseudoRacer, false) => "VIEWPORT",
        (BenchmarkCase::GraphicsLab, false) => "FRAMEBUF",
    }
}

fn benchmark_strong_text(theme: crate::display::ThemeMode, ui: &crate::display::Palette) -> u16 {
    if matches!(theme, crate::display::ThemeMode::Light) {
        ui.text
    } else {
        ui.white
    }
}

fn benchmark_summary(results: &[BenchmarkResult; BENCH_COUNT]) -> (u16, u16, usize) {
    let mut total = 0u32;
    let mut count = 0u32;
    let mut overall_min = u16::MAX;
    let mut best_index = 0usize;
    let mut best_avg = 0u16;

    for (index, result) in results.iter().enumerate() {
        if result.avg_fps > 0 {
            total = total.saturating_add(result.avg_fps as u32);
            count = count.saturating_add(1);
            if result.avg_fps >= best_avg {
                best_avg = result.avg_fps;
                best_index = index;
            }
        }
        if result.min_fps > 0 {
            overall_min = overall_min.min(result.min_fps);
        }
    }

    (
        if count == 0 {
            0
        } else {
            (total / count) as u16
        },
        if overall_min == u16::MAX {
            0
        } else {
            overall_min
        },
        best_index,
    )
}

fn benchmark_profile_label(avg_fps: u16, zh: bool) -> &'static str {
    match (avg_fps, zh) {
        (24..=u16::MAX, true) => "SMOOTH",
        (18..=23, true) => "STABLE",
        (12..=17, true) => "HEAVY",
        (_, true) => "STRESS",
        (24..=u16::MAX, false) => "SMOOTH",
        (18..=23, false) => "STABLE",
        (12..=17, false) => "HEAVY",
        (_, false) => "STRESS",
    }
}

fn benchmark_profile_accent(avg_fps: u16, ui: &crate::display::Palette) -> u16 {
    match avg_fps {
        24..=u16::MAX => ui.lime,
        18..=23 => ui.cyan,
        12..=17 => ui.amber,
        _ => ui.rose,
    }
}

fn benchmark_score(results: &[BenchmarkResult; BENCH_COUNT]) -> u16 {
    let (avg, min, _) = benchmark_summary(results);
    let score = avg as u32 * 32 + min as u32 * 12;
    score.min(999) as u16
}

fn benchmark_grade(score: u16, zh: bool) -> &'static str {
    match (score, zh) {
        (900..=u16::MAX, true) => "S",
        (760..=899, true) => "A",
        (620..=759, true) => "B",
        (480..=619, true) => "C",
        (_, true) => "D",
        (900..=u16::MAX, false) => "S",
        (760..=899, false) => "A",
        (620..=759, false) => "B",
        (480..=619, false) => "C",
        (_, false) => "D",
    }
}

fn benchmark_case_grade(result: BenchmarkResult, zh: bool) -> &'static str {
    let score = result.avg_fps as u32 * 28 + result.min_fps as u32 * 10;
    match (score, zh) {
        (760..=u32::MAX, true) => "A",
        (560..=759, true) => "B",
        (380..=559, true) => "C",
        (_, true) => "D",
        (760..=u32::MAX, false) => "A",
        (560..=759, false) => "B",
        (380..=559, false) => "C",
        (_, false) => "D",
    }
}

fn fit_benchmark_text(display: &Display, text: &str, max_width: u16) -> String<48> {
    let mut out: String<48> = String::new();
    if display.measure_text(text, 1) <= max_width {
        let _ = out.push_str(text);
        return out;
    }

    for ch in text.chars() {
        let mut candidate: String<48> = String::new();
        let _ = candidate.push_str(&out);
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

fn benchmark_compact_kb(bytes: usize) -> String<12> {
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
