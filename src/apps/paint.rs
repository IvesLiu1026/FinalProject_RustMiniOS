use crate::board::ButtonSnapshot;
use crate::display::{color, palette, Display, ThemeMode};
use crate::touch::TouchState;
use crate::ui::{
    draw_gradient_background, render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::{touch_active_in_rect, touch_released_in_rect};

const CANVAS_COLS: usize = 24;
const CANVAS_ROWS: usize = 20;
const CANVAS_CELL: u16 = 8;
const CANVAS_X: u16 = 18;
const CANVAS_Y: u16 = 62;
const CANVAS_W: u16 = CANVAS_COLS as u16 * CANVAS_CELL;
const CANVAS_H: u16 = CANVAS_ROWS as u16 * CANVAS_CELL;
pub const PAINT_PIXEL_COUNT: usize = CANVAS_COLS * CANVAS_ROWS;

const SWATCH_X: u16 = 226;
const SWATCH_Y: u16 = 70;
const SWATCH_W: u16 = 32;
const SWATCH_H: u16 = 20;
const SWATCH_GAP: u16 = 8;

const PAINT_COLORS: [u16; 8] = [
    color::WHITE,
    color::CYAN,
    color::ORANGE,
    color::ROSE,
    color::LIME,
    color::AMBER,
    color::INDIGO,
    color::MIDNIGHT,
];

pub enum PaintAction {
    Stay,
    ExitHome,
}

#[derive(Clone, Copy)]
pub struct PaintState {
    pub pixels: [u8; PAINT_PIXEL_COUNT],
    pub selected_color: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaintRedraw {
    Full,
    Palette,
    ClearCanvas,
    Cell { col: usize, row: usize },
}

pub struct PaintApp {
    pixels: [u8; PAINT_PIXEL_COUNT],
    selected_color: usize,
    redraw_pending: Option<PaintRedraw>,
}

impl PaintApp {
    pub const fn new() -> Self {
        Self {
            pixels: [0; PAINT_PIXEL_COUNT],
            selected_color: 1,
            redraw_pending: None,
        }
    }

    pub fn update(&mut self, input: &ButtonSnapshot, touch: &TouchState) -> PaintAction {
        if input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            return PaintAction::ExitHome;
        }

        if input.k0_just_pressed {
            self.selected_color =
                (self.selected_color + PAINT_COLORS.len() - 1) % PAINT_COLORS.len();
            self.request_redraw(PaintRedraw::Palette);
        }
        if input.wkup_just_pressed {
            self.selected_color = (self.selected_color + 1) % PAINT_COLORS.len();
            self.request_redraw(PaintRedraw::Palette);
        }
        if input.k1_just_pressed {
            self.clear();
        }

        if touch_released_in_rect(touch, 220, 218, 82, 14) {
            self.clear();
        }

        for index in 0..PAINT_COLORS.len() {
            let (x, y) = swatch_origin(index);
            if touch_released_in_rect(touch, x, y, SWATCH_W, SWATCH_H) {
                self.selected_color = index;
                self.request_redraw(PaintRedraw::Palette);
            }
        }

        if touch_active_in_rect(touch, CANVAS_X, CANVAS_Y, CANVAS_W, CANVAS_H) {
            let col = ((touch.x - CANVAS_X) / CANVAS_CELL).min(CANVAS_COLS as u16 - 1) as usize;
            let row = ((touch.y - CANVAS_Y) / CANVAS_CELL).min(CANVAS_ROWS as u16 - 1) as usize;
            let pixel = &mut self.pixels[row * CANVAS_COLS + col];
            if *pixel as usize != self.selected_color {
                *pixel = self.selected_color as u8;
                self.request_redraw(PaintRedraw::Cell { col, row });
            }
        }

        PaintAction::Stay
    }

    pub fn take_redraw_request(&mut self) -> Option<PaintRedraw> {
        let redraw = self.redraw_pending;
        self.redraw_pending = None;
        redraw
    }

    pub fn snapshot(&self) -> PaintState {
        PaintState {
            pixels: self.pixels,
            selected_color: self.selected_color.min(u8::MAX as usize) as u8,
        }
    }

    pub fn restore(&mut self, state: PaintState) {
        self.pixels = state.pixels;
        for pixel in &mut self.pixels {
            *pixel = (*pixel).min((PAINT_COLORS.len() - 1) as u8);
        }
        self.selected_color = (state.selected_color as usize).min(PAINT_COLORS.len() - 1);
        self.redraw_pending = Some(PaintRedraw::Full);
    }

    pub fn render(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 74);

        display.panel(14, 10, 292, 34, ui.panel, ui.lime);
        render_nav_back(display, zh_mode, ui.orange, &ui);
        display.text(
            74,
            18,
            if zh_mode {
                "像素畫板"
            } else {
                "PIXEL PAINT"
            },
            ui.text,
            ui.panel,
            2,
        );
        display.text(
            150,
            20,
            if zh_mode {
                "低解析畫布，剛好很復古"
            } else {
                "LOW-RES CANVAS WITH RETRO CHARM"
            },
            ui.text_muted,
            ui.panel,
            1,
        );

        display.panel(
            CANVAS_X - 4,
            CANVAS_Y - 4,
            CANVAS_W + 8,
            CANVAS_H + 8,
            ui.panel,
            ui.cyan,
        );
        for row in 0..CANVAS_ROWS {
            for col in 0..CANVAS_COLS {
                let color = PAINT_COLORS[self.pixels[row * CANVAS_COLS + col] as usize];
                let x = CANVAS_X + col as u16 * CANVAS_CELL;
                let y = CANVAS_Y + row as u16 * CANVAS_CELL;
                display.fill_rect(x, y, CANVAS_CELL, CANVAS_CELL, color);
            }
        }

        display.panel(218, 56, 86, 154, ui.panel, ui.orange);
        display.text(
            228,
            64,
            if zh_mode { "色盤" } else { "PALETTE" },
            ui.text,
            ui.panel,
            2,
        );

        for index in 0..PAINT_COLORS.len() {
            let (x, y) = swatch_origin(index);
            let accent = if self.selected_color == index {
                ui.white
            } else {
                ui.steel
            };
            display.panel(x, y, SWATCH_W, SWATCH_H, ui.panel_alt, accent);
            display.fill_rect(
                x + 5,
                y + 4,
                SWATCH_W - 10,
                SWATCH_H - 8,
                PAINT_COLORS[index],
            );
        }

        display.panel(220, 218, 82, 14, ui.panel_alt, ui.rose);
        display.centered_text(
            261,
            222,
            if zh_mode { "清空畫布" } else { "CLEAR" },
            ui.text,
            ui.panel_alt,
            1,
        );

        display.panel(14, 222, 196, 16, ui.panel, ui.white);
        display.text(
            22,
            226,
            if zh_mode {
                "拖曳作畫  K0/WK 換色  K1 清空"
            } else {
                "DRAG TO DRAW  K0/WK COLOR  K1 CLEAR"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
    }

    pub fn render_partial(
        &self,
        display: &mut Display,
        theme: ThemeMode,
        zh_mode: bool,
        redraw: PaintRedraw,
    ) {
        match redraw {
            PaintRedraw::Full => self.render(display, theme, zh_mode),
            PaintRedraw::Palette => self.render_palette(display, theme, zh_mode),
            PaintRedraw::ClearCanvas => self.render_canvas(display, theme),
            PaintRedraw::Cell { col, row } => self.render_canvas_cell(display, col, row),
        }
    }

    fn clear(&mut self) {
        self.pixels = [0; PAINT_PIXEL_COUNT];
        self.request_redraw(PaintRedraw::ClearCanvas);
    }

    fn render_canvas(&self, display: &mut Display, theme: ThemeMode) {
        let ui = palette(theme);
        display.panel(
            CANVAS_X - 4,
            CANVAS_Y - 4,
            CANVAS_W + 8,
            CANVAS_H + 8,
            ui.panel,
            ui.cyan,
        );
        for row in 0..CANVAS_ROWS {
            for col in 0..CANVAS_COLS {
                self.render_canvas_cell(display, col, row);
            }
        }
    }

    fn render_canvas_cell(&self, display: &mut Display, col: usize, row: usize) {
        let color = PAINT_COLORS[self.pixels[row * CANVAS_COLS + col] as usize];
        let x = CANVAS_X + col as u16 * CANVAS_CELL;
        let y = CANVAS_Y + row as u16 * CANVAS_CELL;
        display.fill_rect(x, y, CANVAS_CELL, CANVAS_CELL, color);
    }

    fn render_palette(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        display.panel(218, 56, 86, 154, ui.panel, ui.orange);
        display.text(
            228,
            64,
            if zh_mode { "色盤" } else { "PALETTE" },
            ui.text,
            ui.panel,
            2,
        );

        for index in 0..PAINT_COLORS.len() {
            let (x, y) = swatch_origin(index);
            let accent = if self.selected_color == index {
                ui.white
            } else {
                ui.steel
            };
            display.panel(x, y, SWATCH_W, SWATCH_H, ui.panel_alt, accent);
            display.fill_rect(
                x + 5,
                y + 4,
                SWATCH_W - 10,
                SWATCH_H - 8,
                PAINT_COLORS[index],
            );
        }

        display.panel(220, 218, 82, 14, ui.panel_alt, ui.rose);
        display.centered_text(
            261,
            222,
            if zh_mode { "清空畫布" } else { "CLEAR" },
            ui.text,
            ui.panel_alt,
            1,
        );
    }

    fn request_redraw(&mut self, redraw: PaintRedraw) {
        self.redraw_pending = Some(match (self.redraw_pending, redraw) {
            (Some(PaintRedraw::Full), _) => PaintRedraw::Full,
            (Some(PaintRedraw::ClearCanvas), PaintRedraw::Cell { .. }) => PaintRedraw::ClearCanvas,
            (Some(PaintRedraw::ClearCanvas), PaintRedraw::Palette) => PaintRedraw::Full,
            (_, value) => value,
        });
    }
}

fn swatch_origin(index: usize) -> (u16, u16) {
    let column = index % 2;
    let row = index / 2;
    (
        SWATCH_X + column as u16 * (SWATCH_W + SWATCH_GAP),
        SWATCH_Y + row as u16 * (SWATCH_H + SWATCH_GAP),
    )
}
