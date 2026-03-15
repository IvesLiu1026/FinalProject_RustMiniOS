use crate::board;

const SCREEN_WIDTH: u16 = 320;
const SCREEN_HEIGHT: u16 = 240;
const TOUCH_THRESHOLD: u16 = 128;
const TOUCH_PLAUSIBLE_MIN: u16 = 120;
const TOUCH_PLAUSIBLE_MAX: u16 = 4080;
const TOUCH_X_COMMAND: u8 = 0xD0;
const TOUCH_Y_COMMAND: u8 = 0x90;

#[derive(Clone, Copy)]
pub struct TouchCalibration {
    pub x_min: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_max: u16,
    pub swap_xy: bool,
    pub invert_x: bool,
    pub invert_y: bool,
    pub valid: bool,
    pub affine: bool,
    pub ax: f32,
    pub bx: f32,
    pub cx: f32,
    pub ay: f32,
    pub by: f32,
    pub cy: f32,
}

impl Default for TouchCalibration {
    fn default() -> Self {
        Self {
            x_min: 260,
            x_max: 3900,
            y_min: 300,
            y_max: 3900,
            swap_xy: true,
            invert_x: true,
            invert_y: false,
            valid: true,
            affine: false,
            ax: 0.0,
            bx: 0.0,
            cx: 0.0,
            ay: 0.0,
            by: 0.0,
            cy: 0.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TouchState {
    pub active: bool,
    pub just_pressed: bool,
    pub just_released: bool,
    pub moved: bool,
    pub dragging: bool,
    pub x: u16,
    pub y: u16,
    pub raw_x: u16,
    pub raw_y: u16,
    pub start_x: u16,
    pub start_y: u16,
    pub release_x: u16,
    pub release_y: u16,
    pub held_ms: u16,
}

pub struct Touch {
    state: TouchState,
    calibration: TouchCalibration,
}

impl Touch {
    pub fn new() -> Self {
        Self {
            state: TouchState::default(),
            calibration: TouchCalibration::default(),
        }
    }

    pub fn update(&mut self, elapsed_ms: u16) -> TouchState {
        let mut raw_x = 0u16;
        let mut raw_y = 0u16;
        let active = self.sample_raw(&mut raw_x, &mut raw_y);

        self.state.just_pressed = active && !self.state.active;
        self.state.just_released = !active && self.state.active;
        self.state.moved = false;

        if active {
            let (mapped_x, mapped_y) = if self.calibration.valid && self.calibration.affine {
                apply_affine(raw_x, raw_y, &self.calibration)
            } else {
                let mut work_x = raw_x;
                let mut work_y = raw_y;
                if self.calibration.swap_xy {
                    core::mem::swap(&mut work_x, &mut work_y);
                }

                (
                    apply_axis(
                        work_x,
                        self.calibration.x_min,
                        self.calibration.x_max,
                        SCREEN_WIDTH - 1,
                        self.calibration.invert_x,
                    ),
                    apply_axis(
                        work_y,
                        self.calibration.y_min,
                        self.calibration.y_max,
                        SCREEN_HEIGHT - 1,
                        self.calibration.invert_y,
                    ),
                )
            };

            if self.state.just_pressed {
                self.state.start_x = mapped_x;
                self.state.start_y = mapped_y;
                self.state.release_x = mapped_x;
                self.state.release_y = mapped_y;
                self.state.held_ms = 0;
                self.state.dragging = false;
            } else {
                self.state.held_ms = self.state.held_ms.saturating_add(elapsed_ms);
            }

            let filtered_x = if self.state.just_pressed {
                mapped_x
            } else {
                ((self.state.x as u32 * 3 + mapped_x as u32 + 2) / 4) as u16
            };
            let filtered_y = if self.state.just_pressed {
                mapped_y
            } else {
                ((self.state.y as u32 * 3 + mapped_y as u32 + 2) / 4) as u16
            };

            let dx = (filtered_x as i32 - self.state.x as i32).abs();
            let dy = (filtered_y as i32 - self.state.y as i32).abs();
            if dx > 2 || dy > 2 {
                self.state.moved = true;
                let total_dx = (filtered_x as i32 - self.state.start_x as i32).abs();
                let total_dy = (filtered_y as i32 - self.state.start_y as i32).abs();
                if total_dx > 12 || total_dy > 12 {
                    self.state.dragging = true;
                }
            }

            self.state.raw_x = raw_x;
            self.state.raw_y = raw_y;
            self.state.x = filtered_x;
            self.state.y = filtered_y;
            self.state.release_x = filtered_x;
            self.state.release_y = filtered_y;
        } else if !self.state.just_released {
            self.state.held_ms = 0;
            self.state.dragging = false;
        }

        self.state.active = active;
        self.state
    }

    pub fn state(&self) -> TouchState {
        self.state
    }

    pub fn set_calibration(&mut self, calibration: TouchCalibration) {
        self.calibration = calibration;
    }

    fn sample_raw(&self, raw_x: &mut u16, raw_y: &mut u16) -> bool {
        let irq_active = board::touch_irq_active();
        if !irq_active {
            return false;
        }
        let sample_x = average_samples(TOUCH_X_COMMAND);
        let sample_y = average_samples(TOUCH_Y_COMMAND);

        if !samples_plausible(sample_x, sample_y) {
            return false;
        }

        if sample_x < TOUCH_THRESHOLD || sample_y < TOUCH_THRESHOLD {
            return false;
        }

        *raw_x = sample_x;
        *raw_y = sample_y;
        true
    }
}

fn read_channel(command: u8) -> u16 {
    board::touch_select(true);
    board::touch_transfer8(command);
    let _ = ((board::touch_transfer8(0) as u16) << 8) | board::touch_transfer8(0) as u16;
    board::touch_select(false);

    board::touch_select(true);
    board::touch_transfer8(command);
    let value = ((board::touch_transfer8(0) as u16) << 8) | board::touch_transfer8(0) as u16;
    board::touch_select(false);

    (value >> 3) & 0x0FFF
}

fn average_samples(command: u8) -> u16 {
    let mut total = 0u32;
    let mut min_value = u16::MAX;
    let mut max_value = 0u16;

    for _ in 0..5 {
        let sample = read_channel(command);
        min_value = min_value.min(sample);
        max_value = max_value.max(sample);
        total += sample as u32;
    }

    total -= min_value as u32;
    total -= max_value as u32;
    (total / 3) as u16
}

fn apply_axis(raw: u16, min_value: u16, max_value: u16, out_max: u16, invert: bool) -> u16 {
    if max_value <= min_value {
        return 0;
    }

    let raw = raw.clamp(min_value, max_value);
    let numerator = ((raw - min_value) as u32 * out_max as u32) / (max_value - min_value) as u32;
    let mut mapped = numerator.min(out_max as u32) as u16;

    if invert {
        mapped = out_max - mapped;
    }

    mapped
}

fn samples_plausible(sample_x: u16, sample_y: u16) -> bool {
    (TOUCH_PLAUSIBLE_MIN..=TOUCH_PLAUSIBLE_MAX).contains(&sample_x)
        && (TOUCH_PLAUSIBLE_MIN..=TOUCH_PLAUSIBLE_MAX).contains(&sample_y)
}

fn apply_affine(raw_x: u16, raw_y: u16, calibration: &TouchCalibration) -> (u16, u16) {
    let x = calibration.ax * raw_x as f32 + calibration.bx * raw_y as f32 + calibration.cx;
    let y = calibration.ay * raw_x as f32 + calibration.by * raw_y as f32 + calibration.cy;
    (
        clamp_screen(x, SCREEN_WIDTH - 1),
        clamp_screen(y, SCREEN_HEIGHT - 1),
    )
}

fn clamp_screen(value: f32, max_value: u16) -> u16 {
    if value <= 0.0 {
        0
    } else if value >= max_value as f32 {
        max_value
    } else {
        (value + 0.5) as u16
    }
}
