#![no_std]
#![no_main]

mod assets;
mod board;
mod display;
mod dungeon;
mod font;
mod font_zh;
mod touch;
mod ui;

use board::{delay_ms, millis, Board, ButtonSnapshot};
use cortex_m_rt::{entry, exception};
use display::{color, palette, Display, ThemeMode, SCREEN_WIDTH};
use dungeon::{DungeonAction, DungeonApp, RenderStrategy};
use panic_halt as _;
use touch::{Touch, TouchCalibration, TouchState};
use ui::{
    render_control_room, render_home, render_map_select, render_settings, render_touch_calibration,
};

unsafe extern "C" {
    fn stm32f4_Hardware_Init();
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    MapSelect,
    Settings,
    TouchCalibrate,
    ControlRoom,
    DungeonCore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    ZhTw,
}

impl Language {
    fn toggle(&mut self) {
        *self = match self {
            Self::English => Self::ZhTw,
            Self::ZhTw => Self::English,
        };
    }

    fn is_zh(self) -> bool {
        matches!(self, Self::ZhTw)
    }
}

struct MiniOs {
    screen: Screen,
    home_index: usize,
    map_index: usize,
    dungeon: DungeonApp,
    theme: ThemeMode,
    language: Language,
    render_strategy: RenderStrategy,
    last_uptime_second: u32,
    fps_estimate: u16,
    force_full_redraw: bool,
    calibration_step: u8,
    calibration_raw_x: [u16; 5],
    calibration_raw_y: [u16; 5],
    touch_ready: bool,
}

impl MiniOs {
    const fn new() -> Self {
        Self {
            screen: Screen::TouchCalibrate,
            home_index: 0,
            map_index: 0,
            dungeon: DungeonApp::new(),
            theme: ThemeMode::Dark,
            language: Language::English,
            render_strategy: RenderStrategy::Balanced,
            last_uptime_second: 0,
            fps_estimate: 0,
            force_full_redraw: true,
            calibration_step: 0,
            calibration_raw_x: [0; 5],
            calibration_raw_y: [0; 5],
            touch_ready: false,
        }
    }

    fn update(
        &mut self,
        board: &mut Board,
        input: &ButtonSnapshot,
        touch: &TouchState,
        touch_driver: &mut Touch,
        dt_ms: u32,
    ) -> bool {
        let mut dirty = false;
        match self.screen {
            Screen::Home => {
                if input.k0_just_pressed {
                    self.home_index = self.home_index.wrapping_add(3) % 4;
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.home_index = (self.home_index + 1) % 4;
                    dirty = true;
                }
                if input.k1_just_pressed {
                    self.open_selected_screen();
                    dirty = true;
                }

                if touch.just_released {
                    for index in 0..4usize {
                        let y = 64 + index as u16 * 39;
                        if touch_started_in_rect(touch, 20, y, 280, 35) {
                            if self.home_index == index {
                                self.open_selected_screen();
                            } else {
                                self.home_index = index;
                            }
                            dirty = true;
                            break;
                        }
                    }
                }
            }
            Screen::MapSelect => {
                if input.k0_just_pressed {
                    self.map_index = self.map_index.wrapping_add(DungeonApp::map_count() - 1)
                        % DungeonApp::map_count();
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.map_index = (self.map_index + 1) % DungeonApp::map_count();
                    dirty = true;
                }
                if input.k1_just_pressed {
                    self.launch_map();
                    return true;
                }
                if input.home_chord() {
                    self.screen = Screen::Home;
                    self.force_full_redraw = true;
                    return true;
                }
                if touch.just_released {
                    for index in 0..DungeonApp::map_count() {
                        let y = 72 + index as u16 * 44;
                        if touch_started_in_rect(touch, 20, y, 280, 36) {
                            if self.map_index == index {
                                self.launch_map();
                            } else {
                                self.map_index = index;
                            }
                            return true;
                        }
                    }
                    if touch_started_in_rect(touch, 22, 208, 276, 20) {
                        self.screen = Screen::Home;
                        self.force_full_redraw = true;
                        return true;
                    }
                }
            }
            Screen::Settings => {
                if input.k0_just_pressed || input.home_chord() {
                    self.screen = Screen::Home;
                    self.force_full_redraw = true;
                    return true;
                }
                if input.wkup_just_pressed {
                    self.toggle_theme();
                    dirty = true;
                }
                if input.k1_just_pressed {
                    self.language.toggle();
                    dirty = true;
                }
                if touch.just_released {
                    if touch_started_in_rect(touch, 22, 60, 276, 50) {
                        self.toggle_theme();
                        dirty = true;
                    } else if touch_started_in_rect(touch, 22, 118, 276, 50) {
                        self.language.toggle();
                        dirty = true;
                    } else if touch_started_in_rect(touch, 22, 176, 276, 34) {
                        self.render_strategy = self.render_strategy.next();
                        dirty = true;
                    } else if touch_started_in_rect(touch, 22, 216, 276, 14) {
                        self.screen = Screen::Home;
                        self.force_full_redraw = true;
                        return true;
                    }
                }
            }
            Screen::TouchCalibrate => {
                if self.touch_ready && (input.k0_just_pressed || input.home_chord()) {
                    self.screen = Screen::Home;
                    self.force_full_redraw = true;
                    return true;
                }
                if touch.just_released {
                    dirty = true;
                    let index = self.calibration_step as usize;
                    if index < 5 {
                        self.calibration_raw_x[index] = touch.raw_x;
                        self.calibration_raw_y[index] = touch.raw_y;
                        self.calibration_step = self.calibration_step.saturating_add(1);
                    }

                    if self.calibration_step >= 5 {
                        if self.commit_touch_calibration(touch_driver) {
                            self.screen = Screen::Home;
                            self.touch_ready = true;
                        } else {
                            self.calibration_step = 0;
                            self.calibration_raw_x = [0; 5];
                            self.calibration_raw_y = [0; 5];
                        }
                        self.force_full_redraw = true;
                    }
                }
            }
            Screen::ControlRoom => {
                if input.k1_just_pressed {
                    board.toggle_led();
                    dirty = true;
                }
                if touch.just_released {
                    if touch_started_in_rect(touch, 18, 56, 284, 70)
                        || touch_started_in_rect(touch, 20, 138, 85, 52)
                    {
                        board.toggle_led();
                        dirty = true;
                    } else if touch_started_in_rect(touch, 18, 206, 284, 24) {
                        self.screen = Screen::Home;
                        self.force_full_redraw = true;
                        return true;
                    }
                }
                if input.home_chord() {
                    self.screen = Screen::Home;
                    self.force_full_redraw = true;
                    return true;
                }
                let uptime_second = millis() / 1000;
                if uptime_second != self.last_uptime_second {
                    self.last_uptime_second = uptime_second;
                    dirty = true;
                }
            }
            Screen::DungeonCore => {
                match self.dungeon.update(input, touch, dt_ms) {
                    DungeonAction::ExitHome => {
                        self.screen = Screen::Home;
                        self.force_full_redraw = true;
                        return true;
                    }
                    DungeonAction::OpenMapSelect => {
                        self.screen = Screen::MapSelect;
                        self.force_full_redraw = true;
                        return true;
                    }
                    DungeonAction::Stay => {}
                }
                dirty = self.dungeon.needs_animation()
                    || self.dungeon.take_redraw_request()
                    || input.k0_just_pressed
                    || input.k1_just_pressed
                    || input.wkup_just_pressed
                    || input.home_chord()
                    || touch.just_pressed
                    || touch.just_released;
            }
        }
        dirty
    }

    fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.force_full_redraw = true;
    }

    fn open_selected_screen(&mut self) {
        match self.home_index {
            0 => {
                self.screen = Screen::MapSelect;
                self.last_uptime_second = millis() / 1000;
                self.force_full_redraw = true;
            }
            1 => {
                self.screen = Screen::Settings;
                self.last_uptime_second = millis() / 1000;
                self.force_full_redraw = true;
            }
            2 => {
                self.screen = Screen::ControlRoom;
                self.last_uptime_second = millis() / 1000;
                self.force_full_redraw = true;
            }
            _ => self.enter_touch_calibration(),
        }
    }

    fn launch_map(&mut self) {
        self.dungeon.set_map(self.map_index);
        self.screen = Screen::DungeonCore;
        self.force_full_redraw = true;
    }

    fn enter_touch_calibration(&mut self) {
        self.screen = Screen::TouchCalibrate;
        self.calibration_step = 0;
        self.calibration_raw_x = [0; 5];
        self.calibration_raw_y = [0; 5];
        self.force_full_redraw = true;
    }

    fn commit_touch_calibration(&mut self, touch_driver: &mut Touch) -> bool {
        let tl = (self.calibration_raw_x[0], self.calibration_raw_y[0]);
        let tr = (self.calibration_raw_x[1], self.calibration_raw_y[1]);
        let center = (self.calibration_raw_x[2], self.calibration_raw_y[2]);
        let br = (self.calibration_raw_x[3], self.calibration_raw_y[3]);
        let bl = (self.calibration_raw_x[4], self.calibration_raw_y[4]);

        let x_span = abs_diff_u16(tl.0, tr.0)
            .max(abs_diff_u16(bl.0, br.0))
            .max(abs_diff_u16(tl.0, bl.0));
        let y_span = abs_diff_u16(tl.1, tr.1)
            .max(abs_diff_u16(bl.1, br.1))
            .max(abs_diff_u16(tl.1, bl.1));
        if x_span < 300 || y_span < 300 {
            return false;
        }

        let targets = [
            (28.0f32, 40.0f32),
            (292.0f32, 40.0f32),
            (160.0f32, 122.0f32),
            (292.0f32, 210.0f32),
            (28.0f32, 210.0f32),
        ];

        let calibration_points = [
            (tl.0 as f32, tl.1 as f32, targets[0].0, targets[0].1),
            (tr.0 as f32, tr.1 as f32, targets[1].0, targets[1].1),
            (center.0 as f32, center.1 as f32, targets[2].0, targets[2].1),
            (br.0 as f32, br.1 as f32, targets[3].0, targets[3].1),
            (bl.0 as f32, bl.1 as f32, targets[4].0, targets[4].1),
        ];

        let (ax, bx, cx) = match solve_affine_least_squares(&calibration_points, true) {
            Some(v) => v,
            None => return false,
        };
        let (ay, by, cy) = match solve_affine_least_squares(&calibration_points, false) {
            Some(v) => v,
            None => return false,
        };

        let mut worst_error = 0.0f32;
        for (raw_x, raw_y, target_x, target_y) in calibration_points {
            let px = ax * raw_x + bx * raw_y + cx;
            let py = ay * raw_x + by * raw_y + cy;
            let ex = (px - target_x).abs();
            let ey = (py - target_y).abs();
            worst_error = worst_error.max(ex.max(ey));
        }
        if worst_error > 24.0 {
            return false;
        }

        let calibration = TouchCalibration {
            x_min: 0,
            x_max: 4095,
            y_min: 0,
            y_max: 4095,
            swap_xy: false,
            invert_x: false,
            invert_y: false,
            valid: true,
            affine: true,
            ax,
            bx,
            cx,
            ay,
            by,
            cy,
        };

        touch_driver.set_calibration(calibration);
        true
    }

    fn take_full_redraw(&mut self) -> bool {
        let value = self.force_full_redraw;
        self.force_full_redraw = false;
        value
    }

    fn render(
        &mut self,
        display: &mut Display,
        board: &Board,
        touch: &TouchState,
        full_refresh: bool,
    ) {
        match self.screen {
            Screen::Home => render_home(display, self.home_index, self.theme, self.language.is_zh()),
            Screen::MapSelect => {
                render_map_select(display, self.map_index, self.theme, self.language.is_zh())
            }
            Screen::Settings => {
                render_settings(
                    display,
                    self.theme,
                    self.language.is_zh(),
                    self.render_strategy,
                )
            }
            Screen::TouchCalibrate => {
                render_touch_calibration(display, self.calibration_step, self.theme, self.language.is_zh())
            }
            Screen::ControlRoom => {
                render_control_room(display, board, self.theme, self.language.is_zh())
            }
            Screen::DungeonCore => self.dungeon.render(
                display,
                touch,
                full_refresh,
                self.theme,
                self.language.is_zh(),
                self.fps_estimate,
                self.render_strategy,
            ),
        }
    }
}

#[entry]
fn main() -> ! {
    enable_fpu();
    unsafe {
        stm32f4_Hardware_Init();
    }

    let mut board = Board::init();
    let mut touch = Touch::new();
    let mut display = Display::init();
    let mut os = MiniOs::new();

    boot_sequence(&mut display);
    board.set_led(false);
    let touch_state = touch.state();
    os.render(&mut display, &board, &touch_state, true);

    let mut last_frame = millis();

    loop {
        let now = millis();
        let dt = now.wrapping_sub(last_frame);
        if dt < 16 {
            cortex_m::asm::wfi();
            continue;
        }

        last_frame = now;
        let sim_dt = dt.min(25);
        let instant_fps = (1000u32 / dt.max(1)).min(99) as u16;
        os.fps_estimate = if os.fps_estimate == 0 {
            instant_fps
        } else {
            (((os.fps_estimate as u32 * 7) + instant_fps as u32) / 8) as u16
        };
        let buttons = board.poll_buttons();
        let touch_state = touch.update(sim_dt as u16);
        let dirty = os.update(&mut board, &buttons, &touch_state, &mut touch, sim_dt);
        let full_refresh = os.take_full_redraw();
        if dirty || full_refresh {
            os.render(&mut display, &board, &touch_state, full_refresh);
        }
    }
}

#[exception]
fn SysTick() {
    board::systick();
}

fn enable_fpu() {
    unsafe {
        let cpacr = 0xE000_ED88 as *mut u32;
        let current = core::ptr::read_volatile(cpacr);
        core::ptr::write_volatile(cpacr, current | (0b1111 << 20));
    }
}

fn boot_sequence(display: &mut Display) {
    let ui = palette(ThemeMode::Dark);
    for band in 0..12u16 {
        let tint = (band * 18) as u8;
        let fill = color::mix(ui.canvas, ui.indigo, tint);
        display.fill_rect(0, band * 20, SCREEN_WIDTH, 20, fill);
    }

    display.panel(18, 28, 284, 66, ui.panel, ui.cyan);
    display.centered_text(160, 42, "FINAL PROJECT", ui.text, ui.panel, 2);
    display.centered_text(160, 62, "RUST MINI OS", ui.white, ui.panel, 3);

    display.panel(34, 120, 252, 78, ui.panel_alt, ui.orange);
    display.centered_text(160, 136, "TACTILE DUNGEON CONSOLE", ui.text, ui.panel_alt, 2);
    display.centered_text(160, 162, "BOOTING GRAPHICS CORE", ui.text_muted, ui.panel_alt, 1);

    for step in 0..18u16 {
        let fill = 12 + step * 13;
        display.fill_rect(48, 208, fill, 10, ui.cyan);
        display.fill_rect(48 + fill, 208, 224 - fill, 10, ui.panel);
        delay_ms(35);
    }
    delay_ms(160);
}

fn touch_started_in_rect(touch: &TouchState, x: u16, y: u16, width: u16, height: u16) -> bool {
    if touch.dragging {
        return false;
    }

    let tap_x = ((touch.start_x as u32 + touch.release_x as u32) / 2) as u16;
    let tap_y = ((touch.start_y as u32 + touch.release_y as u32) / 2) as u16;
    let slop = 10u16;
    let left = x.saturating_sub(slop);
    let top = y.saturating_sub(slop);
    let right = x.saturating_add(width).saturating_add(slop);
    let bottom = y.saturating_add(height).saturating_add(slop);

    tap_x >= left && tap_x < right && tap_y >= top && tap_y < bottom
}

fn abs_diff_u16(a: u16, b: u16) -> u16 {
    a.abs_diff(b)
}

fn solve_affine_least_squares(
    points: &[(f32, f32, f32, f32)],
    solve_x: bool,
) -> Option<(f32, f32, f32)> {
    let mut s_xx = 0.0f32;
    let mut s_xy = 0.0f32;
    let mut s_yy = 0.0f32;
    let mut s_x = 0.0f32;
    let mut s_y = 0.0f32;
    let mut s_u = 0.0f32;
    let mut s_xu = 0.0f32;
    let mut s_yu = 0.0f32;
    let n = points.len() as f32;

    for &(raw_x, raw_y, target_x, target_y) in points {
        let u = if solve_x { target_x } else { target_y };
        s_xx += raw_x * raw_x;
        s_xy += raw_x * raw_y;
        s_yy += raw_y * raw_y;
        s_x += raw_x;
        s_y += raw_y;
        s_u += u;
        s_xu += raw_x * u;
        s_yu += raw_y * u;
    }

    let det = det3(
        s_xx, s_xy, s_x,
        s_xy, s_yy, s_y,
        s_x,  s_y,  n,
    );
    if det.abs() < 1.0e-6 {
        return None;
    }

    let det_a = det3(
        s_xu, s_xy, s_x,
        s_yu, s_yy, s_y,
        s_u,  s_y,  n,
    );
    let det_b = det3(
        s_xx, s_xu, s_x,
        s_xy, s_yu, s_y,
        s_x,  s_u,  n,
    );
    let det_c = det3(
        s_xx, s_xy, s_xu,
        s_xy, s_yy, s_yu,
        s_x,  s_y,  s_u,
    );

    Some((det_a / det, det_b / det, det_c / det))
}

fn det3(
    a11: f32,
    a12: f32,
    a13: f32,
    a21: f32,
    a22: f32,
    a23: f32,
    a31: f32,
    a32: f32,
    a33: f32,
) -> f32 {
    a11 * (a22 * a33 - a23 * a32)
        - a12 * (a21 * a33 - a23 * a31)
        + a13 * (a21 * a32 - a22 * a31)
}
