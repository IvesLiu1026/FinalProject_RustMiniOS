use crate::board::ButtonSnapshot;
use crate::display::{palette, Display, ThemeMode};
use crate::touch::TouchState;
use crate::ui::{
    draw_gradient_background, render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const GRID_X: u16 = 42;
const GRID_Y: u16 = 70;
const GRID_COLS: usize = 3;
const GRID_ROWS: usize = 3;
const GRID_CELL_W: u16 = 72;
const GRID_CELL_H: u16 = 42;
const GRID_GAP: u16 = 10;
const ROUND_TIME_MS: u16 = 20_000;

pub enum TapRushAction {
    Stay,
    ExitGameCenter,
}

pub struct TapRushApp {
    running: bool,
    score: u16,
    best_score: u16,
    time_left_ms: u16,
    hot_index: usize,
    seed: u32,
    redraw_pending: bool,
}

impl TapRushApp {
    pub const fn new() -> Self {
        Self {
            running: false,
            score: 0,
            best_score: 0,
            time_left_ms: ROUND_TIME_MS,
            hot_index: 0,
            seed: 0x00C0_FFEE,
            redraw_pending: false,
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> TapRushAction {
        if input.k0_just_pressed
            || input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            return TapRushAction::ExitGameCenter;
        }

        if !self.running {
            if input.k1_just_pressed || touch_released_in_rect(touch, 100, 194, 120, 22) {
                self.start_round();
            }
            return TapRushAction::Stay;
        }

        self.time_left_ms = self.time_left_ms.saturating_sub(dt_ms as u16);
        self.redraw_pending = true;
        if self.time_left_ms == 0 {
            self.running = false;
            self.best_score = self.best_score.max(self.score);
            return TapRushAction::Stay;
        }

        if touch.just_released {
            let (x, y) = cell_origin(self.hot_index);
            if touch_released_in_rect(touch, x, y, GRID_CELL_W, GRID_CELL_H) {
                self.score = self.score.saturating_add(1);
                self.advance_hot_target();
                self.redraw_pending = true;
            }
        }

        TapRushAction::Stay
    }

    pub fn needs_animation(&self) -> bool {
        self.running
    }

    pub fn take_redraw_request(&mut self) -> bool {
        let redraw = self.redraw_pending;
        self.redraw_pending = false;
        redraw
    }

    pub fn best_score(&self) -> u16 {
        self.best_score
    }

    pub fn set_best_score(&mut self, best_score: u16) {
        self.best_score = best_score;
        self.redraw_pending = true;
    }

    pub fn render(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 96);

        display.panel(14, 10, 292, 34, ui.panel, ui.amber);
        render_nav_back(display, zh_mode, ui.white, &ui);
        display.text(
            74,
            18,
            if zh_mode { "點擊衝刺" } else { "TAP RUSH" },
            ui.text,
            ui.panel,
            2,
        );
        display.text(
            166,
            20,
            if zh_mode {
                "20 秒內盡量點亮目標"
            } else {
                "HIT THE HOT TILE AS MANY TIMES AS YOU CAN"
            },
            ui.text_muted,
            ui.panel,
            1,
        );

        let mut score_text = heapless::String::<24>::new();
        let _ = core::fmt::write(
            &mut score_text,
            format_args!("{} {}", if zh_mode { "分數" } else { "SCORE" }, self.score),
        );
        let mut time_text = heapless::String::<24>::new();
        let _ = core::fmt::write(
            &mut time_text,
            format_args!(
                "{} {}.{:01}",
                if zh_mode { "剩餘" } else { "TIME" },
                self.time_left_ms / 1000,
                (self.time_left_ms % 1000) / 100
            ),
        );

        display.panel(24, 52, 124, 16, ui.panel, ui.cyan);
        display.text(34, 56, &score_text, ui.text, ui.panel, 1);
        display.panel(172, 52, 124, 16, ui.panel, ui.rose);
        display.text(182, 56, &time_text, ui.text, ui.panel, 1);
        let mut best_text = heapless::String::<24>::new();
        let _ = core::fmt::write(
            &mut best_text,
            format_args!(
                "{} {}",
                if zh_mode { "最佳" } else { "BEST" },
                self.best_score
            ),
        );

        for index in 0..(GRID_COLS * GRID_ROWS) {
            let (x, y) = cell_origin(index);
            let hot = self.running && index == self.hot_index;
            let accent = if hot { ui.amber } else { ui.steel };
            let fill = if hot { ui.panel_alt } else { ui.panel };
            display.panel(x, y, GRID_CELL_W, GRID_CELL_H, fill, accent);
            display.centered_text(
                x + GRID_CELL_W / 2,
                y + 14,
                if hot {
                    if zh_mode {
                        "點我"
                    } else {
                        "TAP"
                    }
                } else if zh_mode {
                    "待命"
                } else {
                    "IDLE"
                },
                if hot { ui.amber } else { ui.text_muted },
                fill,
                1,
            );
        }

        if !self.running {
            display.panel(74, 188, 172, 34, ui.panel_alt, ui.amber);
            display.centered_text(
                160,
                198,
                if self.score == 0 {
                    if zh_mode {
                        "準備開始"
                    } else {
                        "READY TO START"
                    }
                } else if zh_mode {
                    "本輪結束"
                } else {
                    "ROUND COMPLETE"
                },
                ui.text,
                ui.panel_alt,
                2,
            );
            display.panel(100, 194, 120, 22, ui.panel, ui.white);
            display.centered_text(
                160,
                200,
                if zh_mode {
                    "點一下重開"
                } else {
                    "TAP TO RESTART"
                },
                ui.text,
                ui.panel,
                1,
            );
            display.centered_text(160, 214, &best_text, ui.text_muted, ui.panel_alt, 1);
        }

        display.panel(18, 226, 284, 12, ui.panel, ui.white);
        display.text(
            28,
            228,
            if zh_mode {
                "K1 開始  觸控擊中目標  K0 返回遊戲中心"
            } else {
                "K1 START  TAP HOT TILE  K0 BACK TO GAME CENTER"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
    }

    fn start_round(&mut self) {
        self.running = true;
        self.score = 0;
        self.time_left_ms = ROUND_TIME_MS;
        self.advance_hot_target();
        self.redraw_pending = true;
    }

    fn advance_hot_target(&mut self) {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let mut next = (self.seed as usize) % (GRID_COLS * GRID_ROWS);
        if next == self.hot_index {
            next = (next + 1) % (GRID_COLS * GRID_ROWS);
        }
        self.hot_index = next;
    }
}

fn cell_origin(index: usize) -> (u16, u16) {
    let col = index % GRID_COLS;
    let row = index / GRID_COLS;
    (
        GRID_X + col as u16 * (GRID_CELL_W + GRID_GAP),
        GRID_Y + row as u16 * (GRID_CELL_H + GRID_GAP),
    )
}
