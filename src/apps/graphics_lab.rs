use core::fmt::Write;

use heapless::String;
use libm::{atan2f, cosf, floorf, sinf, sqrtf};

use crate::board::ButtonSnapshot;
use crate::display::{color, palette, Display, ThemeMode};
use crate::touch::TouchState;
use crate::ui::{
    draw_footer_hint, draw_gradient_background, draw_shell_window, draw_title_bar, render_nav_back,
    NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const LAB_W: usize = 60;
const LAB_H: usize = 36;
const LAB_PIXELS: usize = LAB_W * LAB_H;
const LAB_SCALE: u16 = 5;
const LAB_X: u16 = 10;
const LAB_Y: u16 = 24;
const MAX_STARS: usize = 72;
const MODE_COUNT: usize = 6;
const CARD_X: u16 = 22;
const CARD_Y: u16 = 58;
const CARD_W: u16 = 134;
const CARD_H: u16 = 46;
const CARD_GAP_X: u16 = 12;
const CARD_GAP_Y: u16 = 10;
const LAB_FRAME_INTERVAL_MS: u16 = 66;
const RUN_BACK_X: u16 = 8;
const RUN_BACK_Y: u16 = 4;
const RUN_BACK_W: u16 = 52;
const RUN_BACK_H: u16 = 16;
const RUN_MODE_X: u16 = 70;
const RUN_MODE_Y: u16 = 4;
const RUN_MODE_W: u16 = 130;
const RUN_MODE_H: u16 = 16;
const RUN_TIME_X: u16 = 248;
const RUN_TIME_Y: u16 = 4;
const RUN_TIME_W: u16 = 64;
const RUN_TIME_H: u16 = 16;
const RUN_INFO_X: u16 = 10;
const RUN_INFO_Y: u16 = 208;
const RUN_INFO_W: u16 = 300;
const RUN_INFO_H: u16 = 22;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GraphicsLabAction {
    Stay,
    ExitGameCenter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GraphicsLabState {
    Select,
    Run,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LabMode {
    Starfield,
    Plasma,
    Rotozoom,
    Tunnel,
    Wireframe,
    Fire,
}

#[derive(Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    z: f32,
    hue: u8,
}

const BLANK_STAR: Star = Star {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    hue: 0,
};

pub struct GraphicsLabApp {
    state: GraphicsLabState,
    selected_mode: usize,
    info_overlay: bool,
    full_redraw_pending: bool,
    render_pending: bool,
    render_accum_ms: u16,
    ticks_ms: u32,
    seed: u32,
    frame: [u16; LAB_PIXELS],
    fire: [u8; LAB_PIXELS],
    stars: [Star; MAX_STARS],
}

impl GraphicsLabApp {
    pub const fn new() -> Self {
        Self {
            state: GraphicsLabState::Select,
            selected_mode: 0,
            info_overlay: true,
            full_redraw_pending: false,
            render_pending: false,
            render_accum_ms: 0,
            ticks_ms: 0,
            seed: 0x00A1_40EF,
            frame: [0; LAB_PIXELS],
            fire: [0; LAB_PIXELS],
            stars: [BLANK_STAR; MAX_STARS],
        }
    }

    pub fn enter(&mut self) {
        self.state = GraphicsLabState::Select;
        self.info_overlay = true;
        self.ticks_ms = 0;
        self.prime_mode();
        self.full_redraw_pending = true;
        self.render_pending = true;
        self.render_accum_ms = 0;
    }

    pub fn start_showcase(&mut self, mode_index: usize) {
        self.selected_mode = mode_index % MODE_COUNT;
        self.state = GraphicsLabState::Run;
        self.prime_mode();
        self.info_overlay = false;
        self.full_redraw_pending = true;
        self.render_pending = true;
        self.render_accum_ms = 0;
    }

    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> GraphicsLabAction {
        match self.state {
            GraphicsLabState::Select => {
                if input.home_chord()
                    || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    self.state = GraphicsLabState::Select;
                    return GraphicsLabAction::ExitGameCenter;
                }
                self.update_select(input, touch)
            }
            GraphicsLabState::Run => {
                if input.home_chord()
                    || touch_released_in_rect(touch, RUN_BACK_X, RUN_BACK_Y, RUN_BACK_W, RUN_BACK_H)
                {
                    self.state = GraphicsLabState::Select;
                    self.full_redraw_pending = true;
                    self.render_pending = true;
                    return GraphicsLabAction::Stay;
                }
                if input.k0_just_pressed {
                    self.selected_mode = (self.selected_mode + MODE_COUNT - 1) % MODE_COUNT;
                    self.prime_mode();
                    self.render_pending = true;
                    self.full_redraw_pending = true;
                }
                if input.wkup_just_pressed {
                    self.selected_mode = (self.selected_mode + 1) % MODE_COUNT;
                    self.prime_mode();
                    self.render_pending = true;
                    self.full_redraw_pending = true;
                }
                if input.k1_just_pressed {
                    self.info_overlay = !self.info_overlay;
                    self.render_pending = true;
                    self.full_redraw_pending = true;
                }
                self.ticks_ms = self.ticks_ms.saturating_add(dt_ms);
                self.update_dynamic_effects(dt_ms);
                self.queue_runtime_frame(dt_ms);
            }
        }

        GraphicsLabAction::Stay
    }

    pub fn needs_animation(&self) -> bool {
        matches!(self.state, GraphicsLabState::Run) && self.render_pending
    }

    pub fn take_full_redraw_request(&mut self) -> bool {
        let redraw = self.full_redraw_pending;
        self.full_redraw_pending = false;
        redraw
    }

    pub fn take_render_request(&mut self) -> bool {
        let redraw = self.render_pending;
        self.render_pending = false;
        redraw
    }

    pub fn can_partial_render(&self) -> bool {
        matches!(self.state, GraphicsLabState::Run)
    }

    pub fn render(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        match self.state {
            GraphicsLabState::Select => {
                let ui = palette(theme);
                draw_gradient_background(display, theme, 120);
                draw_shell_window(display, ui.lime, &ui);
                draw_title_bar(
                    display,
                    if zh_mode {
                        "圖學實驗室"
                    } else {
                        "GRAPHICS LAB"
                    },
                    if zh_mode {
                        "starfield / plasma / wireframe / fire"
                    } else {
                        "starfield / plasma / wireframe / fire"
                    },
                    ui.lime,
                    &ui,
                );
                render_nav_back(display, zh_mode, ui.white, &ui);
                self.render_select(display, zh_mode, &ui);
            }
            GraphicsLabState::Run => {
                let ui = palette(theme);
                display.fill_rect(0, 0, 320, 240, color::mix(ui.shadow, ui.indigo, 28));
                display.stroke_rect(
                    LAB_X.saturating_sub(2),
                    LAB_Y.saturating_sub(2),
                    LAB_W as u16 * LAB_SCALE + 4,
                    LAB_H as u16 * LAB_SCALE + 4,
                    1,
                    mode_accent(mode_from_index(self.selected_mode), &ui),
                );
                self.render_runtime(display, zh_mode, &ui);
            }
        }
    }

    pub fn render_partial(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        if !self.can_partial_render() {
            self.render(display, theme, zh_mode);
            return;
        }
        let ui = palette(theme);
        self.render_runtime(display, zh_mode, &ui);
    }

    fn update_select(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        if input.k0_just_pressed {
            self.selected_mode = (self.selected_mode + MODE_COUNT - 1) % MODE_COUNT;
            self.full_redraw_pending = true;
        }
        if input.wkup_just_pressed {
            self.selected_mode = (self.selected_mode + 1) % MODE_COUNT;
            self.full_redraw_pending = true;
        }
        if input.k1_just_pressed {
            self.state = GraphicsLabState::Run;
            self.prime_mode();
            self.full_redraw_pending = true;
            return;
        }

        if touch.just_released {
            for index in 0..MODE_COUNT {
                let (x, y) = mode_card_rect(index);
                if touch_released_in_rect(touch, x, y, CARD_W, CARD_H) {
                    if self.selected_mode == index {
                        self.state = GraphicsLabState::Run;
                        self.prime_mode();
                        self.render_pending = true;
                    } else {
                        self.selected_mode = index;
                    }
                    self.full_redraw_pending = true;
                    return;
                }
            }
        }
    }

    fn render_select(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.fill_rect(22, 42, 124, 14, color::mix(ui.panel_alt, ui.lime, 34));
        display.stroke_rect(22, 42, 124, 14, 1, ui.lime);
        display.text(
            28,
            45,
            if zh_mode {
                "選擇模式"
            } else {
                "SELECT MODE"
            },
            ui.text,
            color::mix(ui.panel_alt, ui.lime, 34),
            1,
        );
        display.fill_rect(172, 42, 130, 14, color::mix(ui.panel_alt, ui.cyan, 26));
        display.stroke_rect(172, 42, 130, 14, 1, ui.cyan);
        display.text(
            178,
            45,
            if zh_mode {
                "K1 執行 / 觸控卡片"
            } else {
                "K1 RUN / TAP CARD"
            },
            ui.text_muted,
            color::mix(ui.panel_alt, ui.cyan, 26),
            1,
        );

        for index in 0..MODE_COUNT {
            let mode = mode_from_index(index);
            let (x, y) = mode_card_rect(index);
            let selected = self.selected_mode == index;
            let accent = mode_accent(mode, ui);
            let fill = if selected { ui.panel_alt } else { ui.panel };
            display.panel(
                x,
                y,
                CARD_W,
                CARD_H,
                fill,
                if selected { accent } else { ui.steel },
            );
            display.fill_rect(x + 8, y + 8, 30, 18, color::mix(fill, accent, 28));
            display.stroke_rect(x + 8, y + 8, 30, 18, 1, accent);
            draw_mode_glyph(display, x + 15, y + 12, mode, accent, fill, ui);
            let name = fit_text(display, mode_name(mode, zh_mode), 80);
            let tagline = fit_text(display, mode_tagline(mode, zh_mode), 82);
            display.text(x + 46, y + 8, &name, ui.text, fill, 1);
            display.text(x + 46, y + 20, &tagline, ui.text_muted, fill, 1);
            display.text(
                x + 96,
                y + 32,
                if selected { "RUN" } else { "MODE" },
                color::mix(ui.white, accent, 100),
                fill,
                1,
            );
        }

        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK 切換模式  K1 執行  執行中 K1 可切換說明"
            } else {
                "K0/WK SWITCH MODE  K1 RUN  IN MODE K1 TOGGLE INFO"
            },
            ui.lime,
            ui,
        );
    }

    fn render_runtime(
        &mut self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        self.render_mode_frame();
        display.draw_rgb565_scaled(
            LAB_X,
            LAB_Y,
            LAB_W as u16,
            LAB_H as u16,
            LAB_SCALE,
            &self.frame,
        );

        let mode = mode_from_index(self.selected_mode);
        display.fill_rect(
            RUN_BACK_X,
            RUN_BACK_Y,
            RUN_BACK_W,
            RUN_BACK_H,
            color::mix(ui.panel, ui.shadow, 36),
        );
        display.stroke_rect(RUN_BACK_X, RUN_BACK_Y, RUN_BACK_W, RUN_BACK_H, 1, ui.white);
        display.centered_text(
            RUN_BACK_X + RUN_BACK_W / 2,
            RUN_BACK_Y + 4,
            if zh_mode { "返回" } else { "BACK" },
            ui.text,
            color::mix(ui.panel, ui.shadow, 36),
            1,
        );

        display.fill_rect(
            RUN_MODE_X,
            RUN_MODE_Y,
            RUN_MODE_W,
            RUN_MODE_H,
            color::mix(ui.panel_alt, mode_accent(mode, ui), 22),
        );
        display.stroke_rect(
            RUN_MODE_X,
            RUN_MODE_Y,
            RUN_MODE_W,
            RUN_MODE_H,
            1,
            mode_accent(mode, ui),
        );
        display.text(
            RUN_MODE_X + 8,
            RUN_MODE_Y + 4,
            &fit_text(display, mode_name(mode, zh_mode), RUN_MODE_W - 16),
            ui.text,
            color::mix(ui.panel_alt, mode_accent(mode, ui), 22),
            1,
        );

        let mut tick_line: String<20> = String::new();
        let _ = write!(
            &mut tick_line,
            "{} {:02}.{:01}s",
            if zh_mode { "時間" } else { "TIME" },
            self.ticks_ms / 1000,
            (self.ticks_ms % 1000) / 100
        );
        display.fill_rect(
            RUN_TIME_X,
            RUN_TIME_Y,
            RUN_TIME_W,
            RUN_TIME_H,
            color::mix(ui.panel, ui.shadow, 36),
        );
        display.stroke_rect(RUN_TIME_X, RUN_TIME_Y, RUN_TIME_W, RUN_TIME_H, 1, ui.cyan);
        display.text(
            RUN_TIME_X + 6,
            RUN_TIME_Y + 4,
            &tick_line,
            ui.white,
            color::mix(ui.panel, ui.shadow, 36),
            1,
        );

        if self.info_overlay {
            display.fill_rect(
                RUN_INFO_X,
                RUN_INFO_Y,
                RUN_INFO_W,
                RUN_INFO_H,
                color::mix(ui.panel, ui.shadow, 36),
            );
            display.stroke_rect(RUN_INFO_X, RUN_INFO_Y, RUN_INFO_W, RUN_INFO_H, 1, ui.white);
            let info_line = fit_text(display, mode_tagline(mode, zh_mode), RUN_INFO_W - 20);
            let hint_line = fit_text(display, mode_hint(mode, zh_mode), RUN_INFO_W - 20);
            display.text(
                RUN_INFO_X + 8,
                RUN_INFO_Y + 4,
                &info_line,
                ui.text,
                color::mix(ui.panel, ui.shadow, 36),
                1,
            );
            display.text(
                RUN_INFO_X + 8,
                RUN_INFO_Y + 14,
                &hint_line,
                ui.text_muted,
                color::mix(ui.panel, ui.shadow, 36),
                1,
            );
        }
    }

    fn prime_mode(&mut self) {
        self.ticks_ms = 0;
        self.info_overlay = true;
        self.render_accum_ms = 0;
        self.frame = [0; LAB_PIXELS];
        if matches!(mode_from_index(self.selected_mode), LabMode::Fire) {
            self.fire = [0; LAB_PIXELS];
        }
        if matches!(mode_from_index(self.selected_mode), LabMode::Starfield) {
            for index in 0..MAX_STARS {
                self.reset_star(index, true);
            }
        }
    }

    fn update_dynamic_effects(&mut self, dt_ms: u32) {
        match mode_from_index(self.selected_mode) {
            LabMode::Starfield => self.update_starfield(dt_ms),
            LabMode::Fire => self.update_fire(),
            _ => {}
        }
    }

    fn queue_runtime_frame(&mut self, dt_ms: u32) {
        self.render_accum_ms = self
            .render_accum_ms
            .saturating_add(dt_ms.min(u16::MAX as u32) as u16);
        if self.render_accum_ms >= LAB_FRAME_INTERVAL_MS {
            self.render_accum_ms = self.render_accum_ms.saturating_sub(LAB_FRAME_INTERVAL_MS);
            self.render_pending = true;
        }
    }

    fn render_mode_frame(&mut self) {
        match mode_from_index(self.selected_mode) {
            LabMode::Starfield => self.render_starfield(),
            LabMode::Plasma => self.render_plasma(),
            LabMode::Rotozoom => self.render_rotozoom(),
            LabMode::Tunnel => self.render_tunnel(),
            LabMode::Wireframe => self.render_wireframe(),
            LabMode::Fire => self.render_fire(),
        }
    }

    fn update_starfield(&mut self, dt_ms: u32) {
        let delta = dt_ms as f32 * 0.0016;
        for index in 0..MAX_STARS {
            let star = &mut self.stars[index];
            star.z -= delta;
            if star.z <= 0.18 {
                self.reset_star(index, false);
            }
        }
    }

    fn render_starfield(&mut self) {
        self.clear_frame(color::rgb565(6, 8, 16));
        for index in 0..MAX_STARS {
            let star = self.stars[index];
            let px = LAB_W as f32 * 0.5 + (star.x / star.z) * 8.0;
            let py = LAB_H as f32 * 0.5 + (star.y / star.z) * 6.0;
            let sx = px as i16;
            let sy = py as i16;
            let shade = clamp_byte((255.0 - star.z * 42.0) as i32);
            let color = match star.hue % 3 {
                0 => color::mix(color::WHITE, color::CYAN, shade),
                1 => color::mix(color::WHITE, color::AMBER, shade),
                _ => color::mix(color::WHITE, color::ROSE, shade),
            };
            self.plot(sx, sy, color);
            if star.z < 2.0 {
                self.plot(sx - 1, sy, color::mix(color, color::MIDNIGHT, 90));
            }
        }
    }

    fn render_plasma(&mut self) {
        let t = self.ticks_ms as f32 * 0.003;
        for y in 0..LAB_H {
            for x in 0..LAB_W {
                let xf = x as f32 * 0.34;
                let yf = y as f32 * 0.30;
                let dx = x as f32 - LAB_W as f32 * 0.5;
                let dy = y as f32 - LAB_H as f32 * 0.5;
                let radial = sqrtf(dx * dx + dy * dy) * 0.45;
                let wave = sinf(xf + t)
                    + sinf(yf - t * 0.7)
                    + sinf((xf + yf) * 0.6 + t * 1.2)
                    + sinf(radial - t * 1.4);
                let normalized = ((wave + 4.0) * 31.0).clamp(0.0, 255.0) as u8;
                self.frame[y * LAB_W + x] = palette_plasma(normalized);
            }
        }
    }

    fn render_rotozoom(&mut self) {
        let t = self.ticks_ms as f32 * 0.002;
        let angle = t * 0.8;
        let zoom = 1.1 + sinf(t * 1.4) * 0.35;
        let ca = cosf(angle);
        let sa = sinf(angle);
        for y in 0..LAB_H {
            for x in 0..LAB_W {
                let dx = x as f32 - LAB_W as f32 * 0.5;
                let dy = y as f32 - LAB_H as f32 * 0.5;
                let u = (dx * ca - dy * sa) / zoom;
                let v = (dx * sa + dy * ca) / zoom;
                let check = (((floorf(u * 0.45) as i32) ^ (floorf(v * 0.45) as i32)) & 1) == 0;
                let stripe = (((u + v) * 0.33 + t * 2.8) as i32 & 3) == 0;
                let color = if check {
                    if stripe {
                        color::rgb565(255, 214, 100)
                    } else {
                        color::rgb565(86, 206, 255)
                    }
                } else if stripe {
                    color::rgb565(255, 105, 138)
                } else {
                    color::rgb565(28, 36, 64)
                };
                self.frame[y * LAB_W + x] = color;
            }
        }
    }

    fn render_tunnel(&mut self) {
        let t = self.ticks_ms as f32 * 0.0022;
        let cx = LAB_W as f32 * 0.5;
        let cy = LAB_H as f32 * 0.5;
        for y in 0..LAB_H {
            for x in 0..LAB_W {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let r = sqrtf(dx * dx + dy * dy).max(0.12);
                let a = atan2f(dy, dx);
                let u = a * 5.0 + t * 2.6;
                let v = (11.0 / r) + t * 5.2;
                let stripe = (((floorf(u) as i32) + (floorf(v) as i32)) & 1) == 0;
                let pulse = (((r * 3.8) as i32 + (t * 12.0) as i32) & 3) == 0;
                self.frame[y * LAB_W + x] = if stripe {
                    if pulse {
                        color::rgb565(255, 244, 220)
                    } else {
                        color::rgb565(0, 210, 255)
                    }
                } else if pulse {
                    color::rgb565(255, 92, 140)
                } else {
                    color::rgb565(26, 22, 48)
                };
            }
        }
    }

    fn render_wireframe(&mut self) {
        self.clear_frame(color::rgb565(10, 16, 26));
        let t = self.ticks_ms as f32 * 0.0018;
        let shape = ((self.ticks_ms / 4500) % 3) as usize;
        let (verts, edges) = match shape {
            0 => (&CUBE_VERTICES[..], &CUBE_EDGES[..]),
            1 => (&PYRAMID_VERTICES[..], &PYRAMID_EDGES[..]),
            _ => (&SHIP_VERTICES[..], &SHIP_EDGES[..]),
        };

        let sx = sinf(t);
        let cx = cosf(t);
        let sy = sinf(t * 0.7);
        let cy = cosf(t * 0.7);
        let sz = sinf(t * 0.5);
        let cz = cosf(t * 0.5);

        let mut projected = [(0i16, 0i16); 8];
        for (index, vertex) in verts.iter().enumerate() {
            let (mut x, mut y, mut z) = *vertex;
            let ny = y * cx - z * sx;
            let nz = y * sx + z * cx;
            y = ny;
            z = nz;

            let nx = x * cy + z * sy;
            let nz = -x * sy + z * cy;
            x = nx;
            z = nz;

            let nx = x * cz - y * sz;
            let ny = x * sz + y * cz;
            x = nx;
            y = ny;

            let perspective = 11.0 / (z + 4.5);
            let px = LAB_W as f32 * 0.5 + x * perspective * 6.0;
            let py = LAB_H as f32 * 0.5 + y * perspective * 5.0;
            projected[index] = (px as i16, py as i16);
        }

        for (a, b) in edges.iter().copied() {
            self.line(
                projected[a].0,
                projected[a].1,
                projected[b].0,
                projected[b].1,
                color::rgb565(148, 241, 255),
            );
        }
        for (x, y) in projected.into_iter().take(verts.len()) {
            self.plot(x, y, color::rgb565(255, 214, 84));
        }
    }

    fn update_fire(&mut self) {
        for x in 0..LAB_W {
            let idx = (LAB_H - 1) * LAB_W + x;
            let noise = (self.next_rand() & 0x3F) as u8;
            self.fire[idx] = 180u8.saturating_add(noise);
        }

        for y in 1..LAB_H {
            for x in 0..LAB_W {
                let src = y * LAB_W + x;
                let below = self.fire[src];
                let spread = (self.next_rand() % 3) as i32 - 1;
                let decay = ((self.next_rand() & 0x03) as u8) * 12;
                let dst_x = (x as i32 + spread).clamp(0, LAB_W as i32 - 1) as usize;
                let dst = (y - 1) * LAB_W + dst_x;
                self.fire[dst] = below.saturating_sub(decay);
            }
        }
    }

    fn render_fire(&mut self) {
        for (index, heat) in self.fire.iter().copied().enumerate() {
            self.frame[index] = palette_fire(heat);
        }
    }

    fn clear_frame(&mut self, color: u16) {
        self.frame.fill(color);
    }

    fn plot(&mut self, x: i16, y: i16, color: u16) {
        if x < 0 || y < 0 || x >= LAB_W as i16 || y >= LAB_H as i16 {
            return;
        }
        self.frame[y as usize * LAB_W + x as usize] = color;
    }

    fn line(&mut self, x0: i16, y0: i16, x1: i16, y1: i16, color: u16) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.plot(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err * 2;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn reset_star(&mut self, index: usize, spread_z: bool) {
        let x = ((self.next_rand() % 280) as i32 - 140) as f32 / 24.0;
        let y = ((self.next_rand() % 200) as i32 - 100) as f32 / 24.0;
        let z = if spread_z {
            0.4 + (self.next_rand() % 120) as f32 / 18.0
        } else {
            5.0 + (self.next_rand() % 50) as f32 / 12.0
        };
        self.stars[index] = Star {
            x,
            y,
            z,
            hue: (self.next_rand() & 0xFF) as u8,
        };
    }

    fn next_rand(&mut self) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.seed
    }
}

const CUBE_VERTICES: [(f32, f32, f32); 8] = [
    (-1.0, -1.0, -1.0),
    (1.0, -1.0, -1.0),
    (1.0, 1.0, -1.0),
    (-1.0, 1.0, -1.0),
    (-1.0, -1.0, 1.0),
    (1.0, -1.0, 1.0),
    (1.0, 1.0, 1.0),
    (-1.0, 1.0, 1.0),
];

const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

const PYRAMID_VERTICES: [(f32, f32, f32); 5] = [
    (-1.2, -1.0, -1.2),
    (1.2, -1.0, -1.2),
    (1.2, -1.0, 1.2),
    (-1.2, -1.0, 1.2),
    (0.0, 1.2, 0.0),
];

const PYRAMID_EDGES: [(usize, usize); 8] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
];

const SHIP_VERTICES: [(f32, f32, f32); 6] = [
    (0.0, 0.0, -1.6),
    (-1.2, -0.5, 0.8),
    (1.2, -0.5, 0.8),
    (0.0, 0.8, 0.4),
    (-0.6, -0.2, 1.6),
    (0.6, -0.2, 1.6),
];

const SHIP_EDGES: [(usize, usize); 9] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (1, 3),
    (2, 3),
    (1, 4),
    (2, 5),
    (4, 5),
    (1, 2),
];

fn mode_from_index(index: usize) -> LabMode {
    match index {
        0 => LabMode::Starfield,
        1 => LabMode::Plasma,
        2 => LabMode::Rotozoom,
        3 => LabMode::Tunnel,
        4 => LabMode::Wireframe,
        _ => LabMode::Fire,
    }
}

fn mode_card_rect(index: usize) -> (u16, u16) {
    let column = index % 2;
    let row = index / 2;
    (
        CARD_X + column as u16 * (CARD_W + CARD_GAP_X),
        CARD_Y + row as u16 * (CARD_H + CARD_GAP_Y),
    )
}

fn mode_name(mode: LabMode, zh_mode: bool) -> &'static str {
    match (mode, zh_mode) {
        (LabMode::Starfield, true) => "星空投影",
        (LabMode::Plasma, true) => "電漿波場",
        (LabMode::Rotozoom, true) => "旋轉縮放",
        (LabMode::Tunnel, true) => "極座標隧道",
        (LabMode::Wireframe, true) => "3D 線框",
        (LabMode::Fire, true) => "像素火焰",
        (LabMode::Starfield, false) => "STARFIELD",
        (LabMode::Plasma, false) => "PLASMA",
        (LabMode::Rotozoom, false) => "ROTOZOOM",
        (LabMode::Tunnel, false) => "TUNNEL",
        (LabMode::Wireframe, false) => "WIREFRAME",
        (LabMode::Fire, false) => "FIRE",
    }
}

fn mode_tagline(mode: LabMode, zh_mode: bool) -> &'static str {
    match (mode, zh_mode) {
        (LabMode::Starfield, true) => "透視 / 深度",
        (LabMode::Plasma, true) => "週期 / 調色盤",
        (LabMode::Rotozoom, true) => "仿射 / 採樣",
        (LabMode::Tunnel, true) => "極座標 / 吸入",
        (LabMode::Wireframe, true) => "投影 / 線段",
        (LabMode::Fire, true) => "擴散 / 緩衝",
        (LabMode::Starfield, false) => "DEPTH / PROJECTION",
        (LabMode::Plasma, false) => "WAVE / PALETTE",
        (LabMode::Rotozoom, false) => "AFFINE / SAMPLE",
        (LabMode::Tunnel, false) => "POLAR / FLOW",
        (LabMode::Wireframe, false) => "3D / PROJECTION",
        (LabMode::Fire, false) => "HEAT / DIFFUSION",
    }
}

fn mode_hint(mode: LabMode, zh_mode: bool) -> &'static str {
    match (mode, zh_mode) {
        (LabMode::Starfield, true) => "多層星點往前衝",
        (LabMode::Plasma, true) => "多個正弦場相加",
        (LabMode::Rotozoom, true) => "棋盤圖旋轉縮放",
        (LabMode::Tunnel, true) => "角度與距離映射",
        (LabMode::Wireframe, true) => "旋轉幾何投影",
        (LabMode::Fire, true) => "像素熱度向上擴散",
        (LabMode::Starfield, false) => "LAYERED STARS RUSH FORWARD",
        (LabMode::Plasma, false) => "COMBINED SINE FIELDS",
        (LabMode::Rotozoom, false) => "ROTATED CHECKER SAMPLE",
        (LabMode::Tunnel, false) => "ANGLE + DISTANCE MAPPING",
        (LabMode::Wireframe, false) => "ROTATE AND PROJECT LINES",
        (LabMode::Fire, false) => "PIXEL HEAT DIFFUSES UP",
    }
}

fn mode_accent(mode: LabMode, ui: &crate::display::Palette) -> u16 {
    match mode {
        LabMode::Starfield => ui.cyan,
        LabMode::Plasma => ui.rose,
        LabMode::Rotozoom => ui.amber,
        LabMode::Tunnel => ui.orange,
        LabMode::Wireframe => ui.white,
        LabMode::Fire => ui.lime,
    }
}

fn draw_mode_glyph(
    display: &mut Display,
    x: u16,
    y: u16,
    mode: LabMode,
    accent: u16,
    bg: u16,
    ui: &crate::display::Palette,
) {
    match mode {
        LabMode::Starfield => {
            display.fill_rect(x + 2, y + 7, 2, 2, ui.white);
            display.fill_rect(x + 10, y + 3, 2, 2, accent);
            display.fill_rect(x + 14, y + 10, 2, 2, color::mix(accent, ui.white, 80));
        }
        LabMode::Plasma => {
            display.fill_rect(x + 1, y + 4, 16, 8, color::mix(bg, accent, 18));
            display.fill_rect(x + 3, y + 6, 3, 2, ui.white);
            display.fill_rect(x + 8, y + 4, 4, 3, accent);
            display.fill_rect(x + 12, y + 8, 3, 2, ui.rose);
        }
        LabMode::Rotozoom => {
            display.fill_rect(x + 2, y + 2, 12, 12, color::mix(bg, accent, 22));
            display.stroke_rect(x + 2, y + 2, 12, 12, 1, accent);
            display.fill_rect(x + 4, y + 4, 3, 3, ui.white);
            display.fill_rect(x + 9, y + 9, 3, 3, ui.amber);
        }
        LabMode::Tunnel => {
            display.stroke_rect(x + 3, y + 3, 12, 10, 1, accent);
            display.stroke_rect(x + 5, y + 5, 8, 6, 1, ui.white);
            display.fill_rect(x + 8, y + 7, 2, 2, ui.rose);
        }
        LabMode::Wireframe => {
            display.fill_rect(x + 5, y + 2, 6, 1, accent);
            display.fill_rect(x + 3, y + 5, 10, 1, accent);
            display.fill_rect(x + 5, y + 10, 6, 1, accent);
            display.fill_rect(x + 4, y + 3, 1, 7, accent);
            display.fill_rect(x + 11, y + 3, 1, 7, accent);
        }
        LabMode::Fire => {
            display.fill_rect(x + 7, y + 2, 4, 4, ui.amber);
            display.fill_rect(x + 5, y + 5, 8, 5, ui.orange);
            display.fill_rect(x + 6, y + 8, 6, 4, ui.rose);
        }
    }
}

fn palette_plasma(v: u8) -> u16 {
    let a = color::mix(color::rgb565(18, 16, 50), color::rgb565(90, 42, 214), v);
    let b = color::mix(color::rgb565(44, 148, 255), color::rgb565(255, 200, 92), v);
    color::mix(a, b, v / 2)
}

fn palette_fire(v: u8) -> u16 {
    if v < 40 {
        color::mix(
            color::rgb565(0, 0, 0),
            color::rgb565(64, 8, 2),
            v.saturating_mul(4),
        )
    } else if v < 96 {
        color::mix(
            color::rgb565(64, 8, 2),
            color::rgb565(180, 48, 10),
            v.saturating_sub(40).saturating_mul(4),
        )
    } else if v < 180 {
        color::mix(
            color::rgb565(180, 48, 10),
            color::rgb565(255, 170, 30),
            v.saturating_sub(96).saturating_mul(3),
        )
    } else {
        color::mix(
            color::rgb565(255, 170, 30),
            color::rgb565(255, 248, 210),
            v.saturating_sub(180).saturating_mul(3),
        )
    }
}

fn clamp_byte(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

fn fit_text(display: &Display, text: &str, max_width: u16) -> String<48> {
    let mut exact = String::<48>::new();
    let _ = exact.push_str(text);
    if display.measure_text(&exact, 1) <= max_width {
        return exact;
    }

    let mut fitted = String::<48>::new();
    for ch in text.chars() {
        let mut candidate = fitted.clone();
        if candidate.push(ch).is_err() {
            break;
        }
        let _ = candidate.push_str("..");
        if display.measure_text(&candidate, 1) > max_width {
            break;
        }
        let _ = fitted.push(ch);
    }
    let _ = fitted.push_str("..");
    fitted
}
