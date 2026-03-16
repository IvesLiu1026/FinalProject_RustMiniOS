use core::fmt::Write;

use heapless::String;
use libm::{fabsf, floorf, sinf};

use crate::board::ButtonSnapshot;
use crate::display::{color, palette, Display, ThemeMode};
use crate::storage::PersistedPseudoRacerData;
use crate::touch::TouchState;
use crate::ui::{
    draw_footer_hint, draw_gradient_background, draw_shell_window, draw_title_bar, render_nav_back,
    NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::{touch_active_in_rect, touch_released_in_rect};

const VIEW_X: u16 = 18;
const VIEW_Y: u16 = 46;
const VIEW_W: u16 = 284;
const VIEW_H: u16 = 148;
const ROAD_BUF_W: usize = 71;
const ROAD_BUF_H: usize = 37;
const ROAD_BUF_PIXELS: usize = ROAD_BUF_W * ROAD_BUF_H;
const ROAD_BUF_SCALE: u16 = 4;

const TRACK_CARD_X: u16 = 28;
const TRACK_CARD_Y: u16 = 64;
const TRACK_CARD_W: u16 = 264;
const TRACK_CARD_H: u16 = 34;
const TRACK_CARD_GAP: u16 = 10;

const TRACK_COUNT: usize = 3;
const MAX_OBJECTS: usize = 10;
const CHECKPOINT_COUNT: usize = 3;
const VISIBLE_DISTANCE: f32 = 320.0;
const PLAYER_MIN_X: f32 = -0.95;
const PLAYER_MAX_X: f32 = 0.95;
const CRUISE_SPEED: f32 = 42.0;
const MAX_SPEED: f32 = 64.0;
const BRAKE_SPEED: f32 = 18.0;
const TRACK_FAR_CLAMP: f32 = 6.0;
const OFFROAD_THRESHOLD: f32 = 0.76;
const OFFROAD_DANGER_THRESHOLD: f32 = 0.90;
const BOOST_FLASH_MS: u16 = 280;
const RACER_FRAME_INTERVAL_MS: u16 = 50;

static mut ROAD_VIEW_BUFFER: [u16; ROAD_BUF_PIXELS] = [0; ROAD_BUF_PIXELS];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PseudoRacerAction {
    Stay,
    ExitGameCenter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RacerState {
    TrackSelect,
    Countdown,
    Racing,
    Finish,
}

#[derive(Clone, Copy)]
struct TrackSegment {
    length: u16,
    curve: i16,
    hill: i16,
    width: u16,
}

#[derive(Clone, Copy)]
struct RoadObjectSeed {
    distance: u16,
    lane: i8,
    kind: RoadObjectKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RoadObjectKind {
    Traffic,
    Truck,
    Cone,
    Barrier,
}

#[derive(Clone, Copy)]
struct RoadObject {
    distance: f32,
    lane: f32,
    kind: RoadObjectKind,
    active: bool,
}

#[derive(Clone, Copy)]
struct TrackDefinition {
    name_en: &'static str,
    name_zh: &'static str,
    blurb_en: &'static str,
    blurb_zh: &'static str,
    sky: u16,
    horizon: u16,
    ground_a: u16,
    ground_b: u16,
    road_a: u16,
    road_b: u16,
    stripe_a: u16,
    stripe_b: u16,
    checkpoint_time_ms: u32,
    segments: &'static [TrackSegment],
    objects: &'static [RoadObjectSeed],
}

#[derive(Clone, Copy)]
struct RoadState {
    offset: f32,
    hill: f32,
    width: f32,
    curve: f32,
}

const BLANK_OBJECT: RoadObject = RoadObject {
    distance: 0.0,
    lane: 0.0,
    kind: RoadObjectKind::Cone,
    active: false,
};

const SEASIDE_SEGMENTS: [TrackSegment; 8] = [
    TrackSegment {
        length: 120,
        curve: 0,
        hill: 3,
        width: 100,
    },
    TrackSegment {
        length: 110,
        curve: 10,
        hill: 6,
        width: 100,
    },
    TrackSegment {
        length: 90,
        curve: 18,
        hill: -3,
        width: 96,
    },
    TrackSegment {
        length: 120,
        curve: -12,
        hill: -8,
        width: 100,
    },
    TrackSegment {
        length: 100,
        curve: 0,
        hill: 9,
        width: 104,
    },
    TrackSegment {
        length: 110,
        curve: 14,
        hill: 2,
        width: 98,
    },
    TrackSegment {
        length: 90,
        curve: -16,
        hill: -6,
        width: 96,
    },
    TrackSegment {
        length: 140,
        curve: 4,
        hill: 0,
        width: 100,
    },
];

const SUNSET_SEGMENTS: [TrackSegment; 8] = [
    TrackSegment {
        length: 90,
        curve: 8,
        hill: 4,
        width: 98,
    },
    TrackSegment {
        length: 90,
        curve: 18,
        hill: 10,
        width: 96,
    },
    TrackSegment {
        length: 110,
        curve: -22,
        hill: -10,
        width: 94,
    },
    TrackSegment {
        length: 120,
        curve: 12,
        hill: 12,
        width: 102,
    },
    TrackSegment {
        length: 90,
        curve: 0,
        hill: 6,
        width: 104,
    },
    TrackSegment {
        length: 100,
        curve: -18,
        hill: -6,
        width: 94,
    },
    TrackSegment {
        length: 100,
        curve: 20,
        hill: -8,
        width: 96,
    },
    TrackSegment {
        length: 140,
        curve: -8,
        hill: 0,
        width: 100,
    },
];

const NIGHT_SEGMENTS: [TrackSegment; 9] = [
    TrackSegment {
        length: 80,
        curve: 0,
        hill: 2,
        width: 100,
    },
    TrackSegment {
        length: 90,
        curve: 24,
        hill: 5,
        width: 94,
    },
    TrackSegment {
        length: 80,
        curve: -24,
        hill: -6,
        width: 92,
    },
    TrackSegment {
        length: 100,
        curve: 16,
        hill: 8,
        width: 94,
    },
    TrackSegment {
        length: 80,
        curve: -18,
        hill: -2,
        width: 92,
    },
    TrackSegment {
        length: 100,
        curve: 22,
        hill: -10,
        width: 94,
    },
    TrackSegment {
        length: 90,
        curve: -20,
        hill: 12,
        width: 94,
    },
    TrackSegment {
        length: 100,
        curve: 8,
        hill: -4,
        width: 96,
    },
    TrackSegment {
        length: 120,
        curve: 0,
        hill: 0,
        width: 100,
    },
];

const SEASIDE_OBJECTS: [RoadObjectSeed; 8] = [
    RoadObjectSeed {
        distance: 110,
        lane: -1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 168,
        lane: 1,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 244,
        lane: 0,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 318,
        lane: -1,
        kind: RoadObjectKind::Barrier,
    },
    RoadObjectSeed {
        distance: 422,
        lane: 1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 506,
        lane: 0,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 614,
        lane: -1,
        kind: RoadObjectKind::Truck,
    },
    RoadObjectSeed {
        distance: 712,
        lane: 1,
        kind: RoadObjectKind::Traffic,
    },
];

const SUNSET_OBJECTS: [RoadObjectSeed; 9] = [
    RoadObjectSeed {
        distance: 98,
        lane: 0,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 150,
        lane: -1,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 224,
        lane: 1,
        kind: RoadObjectKind::Truck,
    },
    RoadObjectSeed {
        distance: 308,
        lane: 0,
        kind: RoadObjectKind::Barrier,
    },
    RoadObjectSeed {
        distance: 382,
        lane: -1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 474,
        lane: 1,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 566,
        lane: 0,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 654,
        lane: -1,
        kind: RoadObjectKind::Barrier,
    },
    RoadObjectSeed {
        distance: 738,
        lane: 1,
        kind: RoadObjectKind::Truck,
    },
];

const NIGHT_OBJECTS: [RoadObjectSeed; 10] = [
    RoadObjectSeed {
        distance: 92,
        lane: -1,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 148,
        lane: 1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 206,
        lane: 0,
        kind: RoadObjectKind::Barrier,
    },
    RoadObjectSeed {
        distance: 282,
        lane: -1,
        kind: RoadObjectKind::Truck,
    },
    RoadObjectSeed {
        distance: 354,
        lane: 1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 420,
        lane: 0,
        kind: RoadObjectKind::Cone,
    },
    RoadObjectSeed {
        distance: 498,
        lane: -1,
        kind: RoadObjectKind::Barrier,
    },
    RoadObjectSeed {
        distance: 582,
        lane: 1,
        kind: RoadObjectKind::Traffic,
    },
    RoadObjectSeed {
        distance: 670,
        lane: 0,
        kind: RoadObjectKind::Truck,
    },
    RoadObjectSeed {
        distance: 760,
        lane: -1,
        kind: RoadObjectKind::Traffic,
    },
];

const TRACKS: [TrackDefinition; TRACK_COUNT] = [
    TrackDefinition {
        name_en: "SEASIDE RUN",
        name_zh: "海濱快線",
        blurb_en: "WIDE CURVES / BRIGHT SKY / CLEAN CHECKPOINTS",
        blurb_zh: "寬路線、亮天空、節奏平穩",
        sky: color::rgb565(102, 173, 232),
        horizon: color::rgb565(255, 223, 122),
        ground_a: color::rgb565(63, 150, 79),
        ground_b: color::rgb565(48, 126, 68),
        road_a: color::rgb565(78, 82, 86),
        road_b: color::rgb565(70, 73, 78),
        stripe_a: color::rgb565(255, 96, 88),
        stripe_b: color::rgb565(255, 248, 240),
        checkpoint_time_ms: 16_000,
        segments: &SEASIDE_SEGMENTS,
        objects: &SEASIDE_OBJECTS,
    },
    TrackDefinition {
        name_en: "SUNSET RIDGE",
        name_zh: "晚霞山脊",
        blurb_en: "TIGHT CORNERS / HILLS / WARM GLOW",
        blurb_zh: "彎道多、坡度高、晚霞視野",
        sky: color::rgb565(255, 146, 102),
        horizon: color::rgb565(255, 219, 156),
        ground_a: color::rgb565(132, 96, 46),
        ground_b: color::rgb565(108, 76, 34),
        road_a: color::rgb565(78, 66, 70),
        road_b: color::rgb565(68, 58, 60),
        stripe_a: color::rgb565(255, 199, 84),
        stripe_b: color::rgb565(255, 248, 234),
        checkpoint_time_ms: 17_500,
        segments: &SUNSET_SEGMENTS,
        objects: &SUNSET_OBJECTS,
    },
    TrackDefinition {
        name_en: "NIGHT CIRCUIT",
        name_zh: "夜行賽道",
        blurb_en: "HIGH SPEED / DARK ROAD / STROBE POSTS",
        blurb_zh: "高速、暗夜、壓力最高",
        sky: color::rgb565(22, 36, 82),
        horizon: color::rgb565(123, 92, 214),
        ground_a: color::rgb565(18, 55, 48),
        ground_b: color::rgb565(12, 38, 34),
        road_a: color::rgb565(44, 48, 68),
        road_b: color::rgb565(34, 38, 56),
        stripe_a: color::rgb565(99, 255, 240),
        stripe_b: color::rgb565(255, 255, 255),
        checkpoint_time_ms: 18_500,
        segments: &NIGHT_SEGMENTS,
        objects: &NIGHT_OBJECTS,
    },
];

pub struct PseudoRacerApp {
    state: RacerState,
    selected_track: usize,
    showcase_autopilot: bool,
    full_redraw_pending: bool,
    render_pending: bool,
    render_accum_ms: u16,
    countdown_ms: u16,
    elapsed_ms: u32,
    time_left_ms: u32,
    distance: f32,
    speed: f32,
    player_x: f32,
    steering_tilt: f32,
    crash_flash_ms: u16,
    checkpoint_flash_ms: u16,
    offroad_warning_ms: u16,
    boost_flash_ms: u16,
    checkpoint_index: usize,
    current_track_total: f32,
    best_time_ms: [u32; TRACK_COUNT],
    objects: [RoadObject; MAX_OBJECTS],
    object_count: usize,
    persist_requested: bool,
}

impl PseudoRacerApp {
    pub const fn new() -> Self {
        Self {
            state: RacerState::TrackSelect,
            selected_track: 0,
            showcase_autopilot: false,
            full_redraw_pending: false,
            render_pending: false,
            render_accum_ms: 0,
            countdown_ms: 0,
            elapsed_ms: 0,
            time_left_ms: 0,
            distance: 0.0,
            speed: CRUISE_SPEED,
            player_x: 0.0,
            steering_tilt: 0.0,
            crash_flash_ms: 0,
            checkpoint_flash_ms: 0,
            offroad_warning_ms: 0,
            boost_flash_ms: 0,
            checkpoint_index: 0,
            current_track_total: 0.0,
            best_time_ms: [0; TRACK_COUNT],
            objects: [BLANK_OBJECT; MAX_OBJECTS],
            object_count: 0,
            persist_requested: false,
        }
    }

    pub fn enter(&mut self) {
        self.state = RacerState::TrackSelect;
        self.showcase_autopilot = false;
        self.full_redraw_pending = true;
        self.render_pending = true;
        self.render_accum_ms = 0;
        self.countdown_ms = 0;
        self.elapsed_ms = 0;
        self.time_left_ms = 0;
        self.distance = 0.0;
        self.speed = CRUISE_SPEED;
        self.player_x = 0.0;
        self.steering_tilt = 0.0;
        self.crash_flash_ms = 0;
        self.checkpoint_flash_ms = 0;
        self.offroad_warning_ms = 0;
        self.boost_flash_ms = 0;
        self.checkpoint_index = 0;
    }

    pub fn snapshot(&self) -> PersistedPseudoRacerData {
        PersistedPseudoRacerData {
            selected_track: self.selected_track.min((TRACK_COUNT - 1) as usize) as u8,
            best_time_ms: self.best_time_ms,
        }
    }

    pub fn restore(&mut self, state: PersistedPseudoRacerData) {
        self.selected_track = (state.selected_track as usize).min(TRACK_COUNT - 1);
        self.best_time_ms = state.best_time_ms;
        self.enter();
    }

    pub fn start_showcase(&mut self, track_index: usize) {
        self.selected_track = track_index % TRACK_COUNT;
        self.showcase_autopilot = true;
        self.start_run();
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

    pub fn take_persist_request(&mut self) -> bool {
        let persist = self.persist_requested;
        self.persist_requested = false;
        persist
    }

    fn request_persist(&mut self) {
        self.persist_requested = true;
    }

    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> PseudoRacerAction {
        if input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            self.state = RacerState::TrackSelect;
            self.showcase_autopilot = false;
            return PseudoRacerAction::ExitGameCenter;
        }

        match self.state {
            RacerState::TrackSelect => self.update_track_select(input, touch),
            RacerState::Countdown => {
                self.countdown_ms = self.countdown_ms.saturating_sub(dt_ms as u16);
                if self.countdown_ms == 0 {
                    self.state = RacerState::Racing;
                    self.render_pending = true;
                }
                self.queue_runtime_frame(dt_ms);
            }
            RacerState::Racing => self.update_race(input, touch, dt_ms),
            RacerState::Finish => {
                if input.k1_just_pressed || touch_released_in_rect(touch, 100, 205, 120, 20) {
                    self.start_run();
                }
                if input.wkup_just_pressed {
                    self.state = RacerState::TrackSelect;
                    self.showcase_autopilot = false;
                    self.full_redraw_pending = true;
                    self.render_pending = true;
                }
            }
        }

        PseudoRacerAction::Stay
    }

    pub fn needs_animation(&self) -> bool {
        (matches!(self.state, RacerState::Countdown | RacerState::Racing)
            || self.crash_flash_ms > 0
            || self.checkpoint_flash_ms > 0)
            && self.render_pending
    }

    pub fn can_partial_render(&self) -> bool {
        matches!(
            self.state,
            RacerState::Countdown | RacerState::Racing | RacerState::Finish
        )
    }

    pub fn render(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 54);
        draw_shell_window(display, ui.orange, &ui);
        draw_title_bar(
            display,
            if zh_mode {
                "假 3D 賽車"
            } else {
                "PSEUDO RACER"
            },
            if zh_mode {
                "scanline road / checkpoint sprint"
            } else {
                "scanline road / checkpoint sprint"
            },
            ui.orange,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.white, &ui);

        match self.state {
            RacerState::TrackSelect => self.render_track_select(display, zh_mode, &ui),
            RacerState::Countdown | RacerState::Racing | RacerState::Finish => {
                self.render_race(display, zh_mode, &ui)
            }
        }
    }

    pub fn render_partial(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        if !self.can_partial_render() {
            self.render(display, theme, zh_mode);
            return;
        }
        let ui = palette(theme);
        self.render_race_partial(display, zh_mode, &ui);
    }

    fn update_track_select(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        if input.k0_just_pressed {
            self.selected_track = (self.selected_track + TRACK_COUNT - 1) % TRACK_COUNT;
            self.render_pending = true;
        }
        if input.wkup_just_pressed {
            self.selected_track = (self.selected_track + 1) % TRACK_COUNT;
            self.render_pending = true;
        }
        if input.k1_just_pressed {
            self.start_run();
            return;
        }

        if touch.just_released {
            for index in 0..TRACK_COUNT {
                let y = TRACK_CARD_Y + index as u16 * (TRACK_CARD_H + TRACK_CARD_GAP);
                if touch_released_in_rect(touch, TRACK_CARD_X, y, TRACK_CARD_W, TRACK_CARD_H) {
                    if self.selected_track == index {
                        self.start_run();
                    } else {
                        self.selected_track = index;
                    }
                    return;
                }
            }
        }
    }

    fn update_race(&mut self, input: &ButtonSnapshot, touch: &TouchState, dt_ms: u32) {
        let dt = dt_ms as f32 * 0.001;
        let left_touch = touch_active_in_rect(touch, VIEW_X, VIEW_Y, VIEW_W / 3, VIEW_H);
        let right_touch = touch_active_in_rect(
            touch,
            VIEW_X + VIEW_W - VIEW_W / 3,
            VIEW_Y,
            VIEW_W / 3,
            VIEW_H,
        );
        let brake_touch = touch_active_in_rect(
            touch,
            VIEW_X + VIEW_W / 3,
            VIEW_Y + VIEW_H - 34,
            VIEW_W / 3,
            34,
        );

        let current_curve = self.sample_road(self.distance + 18.0).curve;
        let steer = if self.showcase_autopilot {
            let preview_curve = self.sample_road(self.distance + 36.0).curve;
            let target_x = clampf(
                (-preview_curve * 0.42) + sinf(self.distance * 0.018) * 0.10,
                -0.36,
                0.36,
            );
            clampf((target_x - self.player_x) * 2.6, -1.0, 1.0)
        } else if (input.k0 || left_touch) && !(input.wkup || right_touch) {
            -1.0
        } else if (input.wkup || right_touch) && !(input.k0 || left_touch) {
            1.0
        } else {
            0.0
        };
        let braking = if self.showcase_autopilot {
            false
        } else {
            input.k1 || brake_touch
        };

        self.player_x = clampf(
            self.player_x + steer * dt * 1.12 - current_curve * dt * 1.8,
            PLAYER_MIN_X,
            PLAYER_MAX_X,
        );
        self.steering_tilt = (self.steering_tilt * 0.82) + steer * 0.18;

        let target_speed = if braking { BRAKE_SPEED } else { MAX_SPEED };
        let response = if braking { 0.10 } else { 0.035 };
        self.speed += (target_speed - self.speed) * response;
        if !braking && self.speed < CRUISE_SPEED {
            self.speed += dt * 18.0;
        }
        self.speed = clampf(self.speed, BRAKE_SPEED, MAX_SPEED);

        let offroad = fabsf(self.player_x) > OFFROAD_THRESHOLD;
        if offroad {
            let severity = clampf(
                (fabsf(self.player_x) - OFFROAD_THRESHOLD)
                    / (OFFROAD_DANGER_THRESHOLD - OFFROAD_THRESHOLD),
                0.0,
                1.0,
            );
            self.speed = (self.speed - dt * (16.0 + severity * 22.0)).max(BRAKE_SPEED - 4.0);
            self.offroad_warning_ms = 120;
        }

        self.elapsed_ms = self.elapsed_ms.saturating_add(dt_ms);
        self.time_left_ms = self.time_left_ms.saturating_sub(dt_ms);
        self.distance += self.speed * dt;

        if self.crash_flash_ms > 0 {
            self.crash_flash_ms = self.crash_flash_ms.saturating_sub(dt_ms as u16);
        }
        if self.checkpoint_flash_ms > 0 {
            self.checkpoint_flash_ms = self.checkpoint_flash_ms.saturating_sub(dt_ms as u16);
        }
        if self.offroad_warning_ms > 0 {
            self.offroad_warning_ms = self.offroad_warning_ms.saturating_sub(dt_ms as u16);
        }
        if self.boost_flash_ms > 0 {
            self.boost_flash_ms = self.boost_flash_ms.saturating_sub(dt_ms as u16);
        }

        self.check_collisions();
        self.check_checkpoints();
        self.queue_runtime_frame(dt_ms);

        if self.distance >= self.current_track_total {
            let best = &mut self.best_time_ms[self.selected_track];
            if *best == 0 || self.elapsed_ms < *best {
                *best = self.elapsed_ms;
                self.request_persist();
            }
            self.state = RacerState::Finish;
            self.render_pending = true;
        } else if self.time_left_ms == 0 {
            self.state = RacerState::Finish;
            self.render_pending = true;
        }
    }

    fn render_track_select(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.fill_rect(22, 46, 120, 14, color::mix(ui.panel_alt, ui.orange, 34));
        display.stroke_rect(22, 46, 120, 14, 1, ui.orange);
        display.text(
            28,
            49,
            if zh_mode {
                "選擇賽道"
            } else {
                "SELECT TRACK"
            },
            ui.text,
            color::mix(ui.panel_alt, ui.orange, 34),
            1,
        );
        display.fill_rect(176, 46, 126, 14, color::mix(ui.panel_alt, ui.cyan, 28));
        display.stroke_rect(176, 46, 126, 14, 1, ui.cyan);
        display.text(
            184,
            49,
            if zh_mode {
                "K1 開始 / K0 WK 切換"
            } else {
                "K1 START / K0 WK SELECT"
            },
            ui.text_muted,
            color::mix(ui.panel_alt, ui.cyan, 28),
            1,
        );

        for (index, track) in TRACKS.iter().enumerate() {
            let y = TRACK_CARD_Y + index as u16 * (TRACK_CARD_H + TRACK_CARD_GAP);
            let selected = self.selected_track == index;
            let fill = if selected { ui.panel_alt } else { ui.panel };
            let border = if selected { ui.orange } else { ui.steel };
            display.panel(TRACK_CARD_X, y, TRACK_CARD_W, TRACK_CARD_H, fill, border);

            let stripe = if index == 0 {
                ui.cyan
            } else if index == 1 {
                ui.amber
            } else {
                ui.rose
            };
            display.fill_rect(
                TRACK_CARD_X + 8,
                y + 7,
                34,
                20,
                color::mix(fill, stripe, 24),
            );
            display.stroke_rect(TRACK_CARD_X + 8, y + 7, 34, 20, 1, stripe);
            draw_track_stamp(display, TRACK_CARD_X + 12, y + 10, index, stripe, fill, ui);

            let title = fit_text(
                display,
                if zh_mode {
                    track.name_zh
                } else {
                    track.name_en
                },
                132,
            );
            let blurb = fit_text(
                display,
                if zh_mode {
                    track.blurb_zh
                } else {
                    track.blurb_en
                },
                140,
            );
            display.text(TRACK_CARD_X + 50, y + 8, &title, ui.text, fill, 1);
            display.text(TRACK_CARD_X + 50, y + 20, &blurb, ui.text_muted, fill, 1);

            let mut best_line: String<18> = String::new();
            let best = self.best_time_ms[index];
            if best == 0 {
                let _ = write!(
                    &mut best_line,
                    "{}",
                    if zh_mode { "NEW ROUTE" } else { "NEW ROUTE" }
                );
            } else {
                let _ = write!(
                    &mut best_line,
                    "{} {:02}.{:01}",
                    if zh_mode { "最佳" } else { "BEST" },
                    (best / 1000),
                    (best % 1000) / 100
                );
            }
            display.text(TRACK_CARD_X + 192, y + 8, &best_line, ui.white, fill, 1);
            display.text(
                TRACK_CARD_X + 196,
                y + 20,
                if selected {
                    if zh_mode {
                        "READY"
                    } else {
                        "READY"
                    }
                } else if zh_mode {
                    "選擇"
                } else {
                    "SELECT"
                },
                color::mix(ui.white, border, 120),
                fill,
                1,
            );
        }

        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK 切換賽道  K1 開始  觸控點選卡片"
            } else {
                "K0/WK SWITCH TRACK  K1 START  TAP CARD TO RUN"
            },
            ui.orange,
            ui,
        );
    }

    fn render_race(&mut self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        let track = &TRACKS[self.selected_track];
        let horizon_y = self.road_horizon_y();
        self.render_buffered_viewport(display, track, horizon_y, ui);
        self.draw_objects(display, track);
        self.draw_checkpoint_banner(display, track);
        self.draw_player_car(display, track, ui);
        self.draw_hud(display, track, zh_mode, ui);

        if self.checkpoint_flash_ms > 0 {
            display.fill_rect(
                VIEW_X,
                VIEW_Y,
                VIEW_W,
                10,
                color::mix(track.horizon, ui.white, 96),
            );
            display.centered_text(
                VIEW_X + VIEW_W / 2,
                VIEW_Y + 2,
                if zh_mode {
                    "CHECKPOINT +"
                } else {
                    "CHECKPOINT +"
                },
                ui.text,
                color::mix(track.horizon, ui.white, 96),
                1,
            );
        }

        if self.crash_flash_ms > 0 {
            let flash = color::mix(ui.rose, ui.white, 180);
            display.stroke_rect(VIEW_X, VIEW_Y, VIEW_W, VIEW_H, 2, flash);
        }
        if self.offroad_warning_ms > 0 {
            let warn = color::mix(track.stripe_a, ui.white, 90);
            display.fill_rect(VIEW_X, VIEW_Y + VIEW_H - 12, VIEW_W, 4, warn);
        }
        if self.boost_flash_ms > 0 {
            let boost = color::mix(ui.cyan, ui.white, 110);
            display.fill_rect(VIEW_X, VIEW_Y + 12, VIEW_W, 3, boost);
        }

        match self.state {
            RacerState::Countdown => {
                let count = (self.countdown_ms / 1000).saturating_add(1);
                self.draw_center_overlay(
                    display,
                    if zh_mode { "準備" } else { "READY" },
                    if count > 0 { Some(count) } else { None },
                    ui,
                );
            }
            RacerState::Finish => {
                self.draw_finish_overlay(display, track, zh_mode, ui);
            }
            _ => {}
        }
    }

    fn render_race_partial(
        &mut self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        self.render_race(display, zh_mode, ui);
    }

    fn render_buffered_viewport(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        unsafe {
            let buffer = &mut *core::ptr::addr_of_mut!(ROAD_VIEW_BUFFER);
            self.draw_road_buffer(buffer, track, horizon_y, ui);
            display.draw_rgb565_scaled(
                VIEW_X,
                VIEW_Y,
                ROAD_BUF_W as u16,
                ROAD_BUF_H as u16,
                ROAD_BUF_SCALE,
                buffer,
            );
        }
    }

    fn draw_road_buffer(
        &self,
        buffer: &mut [u16; ROAD_BUF_PIXELS],
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        road_buffer_clear(buffer, track.sky);
        let horizon_low = full_to_road_y(horizon_y as f32).clamp(0, ROAD_BUF_H as i16 - 1) as usize;

        self.draw_buffer_backdrop(buffer, track, horizon_low as i16, ui);
        self.draw_buffer_hills(buffer, track, horizon_low as i16, ui);

        if horizon_low < ROAD_BUF_H {
            road_buffer_fill_rect(
                buffer,
                0,
                horizon_low as i16,
                ROAD_BUF_W as u16,
                (ROAD_BUF_H - horizon_low) as u16,
                track.ground_a,
            );
        }
        road_buffer_fill_rect(
            buffer,
            0,
            horizon_low.saturating_sub(2) as i16,
            ROAD_BUF_W as u16,
            2,
            track.horizon,
        );

        let view_bottom = VIEW_Y + VIEW_H;
        for row in 0..ROAD_BUF_H {
            let full_y = VIEW_Y + row as u16 * ROAD_BUF_SCALE + ROAD_BUF_SCALE / 2;
            if full_y < horizon_y {
                continue;
            }

            let depth =
                (full_y - horizon_y) as f32 / (view_bottom.saturating_sub(horizon_y).max(1)) as f32;
            let sample_dist = self.distance + TRACK_FAR_CLAMP + depth * depth * VISIBLE_DISTANCE;
            let road = self.sample_road(sample_dist);
            let road_half_full = (18.0 + depth * depth * 108.0) * road.width;
            let road_half = road_half_full / ROAD_BUF_SCALE as f32;
            let center = ROAD_BUF_W as f32 * 0.5
                + (road.offset * (0.32 + depth * 0.55)) / ROAD_BUF_SCALE as f32
                - self.player_x * road_half * 0.92;
            let left = clampf(center - road_half, 0.0, (ROAD_BUF_W - 1) as f32);
            let right = clampf(center + road_half, 0.0, (ROAD_BUF_W - 1) as f32);
            let left_u = left as usize;
            let right_u = right as usize;
            let strip_toggle = ((sample_dist / 22.0) as i32 & 1) == 0;
            let row_color = if strip_toggle {
                track.ground_a
            } else {
                track.ground_b
            };
            road_buffer_fill_rect(buffer, 0, row as i16, ROAD_BUF_W as u16, 1, row_color);

            if right_u > left_u {
                let rumble_w =
                    ((2.0 + depth * depth * 8.0) / ROAD_BUF_SCALE as f32).max(1.0) as usize;
                let road_fill = if strip_toggle {
                    track.road_a
                } else {
                    track.road_b
                };
                let stripe_fill = if strip_toggle {
                    track.stripe_a
                } else {
                    track.stripe_b
                };
                if left_u > 0 {
                    let start = left_u.saturating_sub(rumble_w);
                    road_buffer_fill_rect(
                        buffer,
                        start as i16,
                        row as i16,
                        (left_u - start) as u16,
                        1,
                        stripe_fill,
                    );
                }
                road_buffer_fill_rect(
                    buffer,
                    left_u as i16,
                    row as i16,
                    (right_u - left_u).max(1) as u16,
                    1,
                    road_fill,
                );
                if right_u < ROAD_BUF_W {
                    let width = rumble_w.min(ROAD_BUF_W - right_u);
                    road_buffer_fill_rect(
                        buffer,
                        right_u as i16,
                        row as i16,
                        width as u16,
                        1,
                        stripe_fill,
                    );
                }
                if strip_toggle && (row & 0x01) == 0 {
                    let lane_w = (road_half * 0.10).max(1.0) as usize;
                    let lane_x =
                        clampf(center - lane_w as f32 / 2.0, left, right - lane_w as f32) as usize;
                    road_buffer_fill_rect(
                        buffer,
                        lane_x as i16,
                        row as i16,
                        lane_w.max(1) as u16,
                        1,
                        track.stripe_b,
                    );
                }
            }
        }

        self.draw_buffer_posts(buffer, track, horizon_y);
        self.draw_buffer_decor(buffer, track, horizon_y, ui);
    }

    fn draw_buffer_backdrop(
        &self,
        buffer: &mut [u16; ROAD_BUF_PIXELS],
        track: &TrackDefinition,
        horizon_low: i16,
        ui: &crate::display::Palette,
    ) {
        match self.selected_track {
            0 => {
                road_buffer_fill_rect(buffer, 54, 4, 8, 8, color::mix(track.horizon, ui.white, 58));
                for i in 0..4i16 {
                    let x = 8 + i * 16 - (((self.distance * 0.12) as i32).rem_euclid(10) as i16);
                    road_buffer_fill_rect(
                        buffer,
                        x,
                        5 + (i & 1),
                        5,
                        2,
                        color::mix(ui.white, track.sky, 40),
                    );
                    road_buffer_fill_rect(
                        buffer,
                        x + 1,
                        4 + (i & 1),
                        3,
                        1,
                        color::mix(ui.white, track.sky, 28),
                    );
                }
            }
            1 => {
                road_buffer_fill_rect(buffer, 55, 4, 6, 6, color::mix(track.horizon, ui.white, 42));
                for i in 0..6i16 {
                    let x = i * 10;
                    let height = 3 + ((i as usize + self.checkpoint_index) % 3) as u16 * 2;
                    road_buffer_fill_rect(
                        buffer,
                        x,
                        horizon_low - height as i16,
                        4,
                        height,
                        color::mix(track.horizon, ui.shadow, 90),
                    );
                }
            }
            _ => {
                road_buffer_fill_rect(buffer, 56, 4, 4, 4, color::mix(ui.white, track.horizon, 24));
                road_buffer_fill_rect(buffer, 57, 5, 3, 3, color::mix(ui.white, track.sky, 12));
                for i in 0..10usize {
                    let x = ((i * 6 + (self.distance as usize / 2)) % ROAD_BUF_W) as i16;
                    let y = 2 + ((i * 3) % 5) as i16;
                    road_buffer_fill_rect(buffer, x, y, 1, 1, ui.white);
                }
            }
        }
    }

    fn draw_buffer_hills(
        &self,
        buffer: &mut [u16; ROAD_BUF_PIXELS],
        track: &TrackDefinition,
        horizon_low: i16,
        ui: &crate::display::Palette,
    ) {
        let hill_color = color::mix(track.horizon, ui.indigo, 90);
        let base = horizon_low.saturating_sub(5);
        let x_shift = (((self.distance * 0.18) as i32) % 12) as i16;
        for index in 0..6i16 {
            let x = index * 12 - x_shift;
            let peak = base.saturating_sub((index as u16 % 3 * 2) as i16);
            road_buffer_fill_rect(
                buffer,
                x,
                peak,
                7,
                (horizon_low - peak).max(1) as u16,
                hill_color,
            );
        }
    }

    fn draw_buffer_posts(
        &self,
        buffer: &mut [u16; ROAD_BUF_PIXELS],
        track: &TrackDefinition,
        horizon_y: u16,
    ) {
        let mut row = full_to_road_y((horizon_y + 8) as f32).max(0) as usize;
        while row + 1 < ROAD_BUF_H - 3 {
            let full_y = VIEW_Y + row as u16 * ROAD_BUF_SCALE + ROAD_BUF_SCALE / 2;
            let depth = (full_y - horizon_y) as f32 / (VIEW_Y + VIEW_H - horizon_y).max(1) as f32;
            let sample_dist = self.distance + 10.0 + depth * depth * VISIBLE_DISTANCE;
            let road = self.sample_road(sample_dist);
            let road_half_full = (18.0 + depth * depth * 108.0) * road.width;
            let road_half = road_half_full / ROAD_BUF_SCALE as f32;
            let center = ROAD_BUF_W as f32 * 0.5
                + (road.offset * (0.32 + depth * 0.55)) / ROAD_BUF_SCALE as f32
                - self.player_x * road_half * 0.92;
            let post_h = ((2.0 + depth * depth * 10.0) / ROAD_BUF_SCALE as f32).max(1.0) as u16;
            let post_color = if ((sample_dist / 28.0) as i32 & 1) == 0 {
                track.stripe_b
            } else {
                track.stripe_a
            };
            let left_x = clampf(center - road_half - 3.0, 0.0, (ROAD_BUF_W - 1) as f32) as i16;
            let right_x = clampf(center + road_half + 2.0, 0.0, (ROAD_BUF_W - 1) as f32) as i16;
            road_buffer_fill_rect(
                buffer,
                left_x,
                row as i16 - post_h as i16,
                1,
                post_h,
                post_color,
            );
            road_buffer_fill_rect(
                buffer,
                right_x,
                row as i16 - post_h as i16,
                1,
                post_h,
                post_color,
            );
            row = row
                .saturating_add(((6.0 + depth * 12.0) / ROAD_BUF_SCALE as f32).max(1.0) as usize);
        }
    }

    fn draw_buffer_decor(
        &self,
        buffer: &mut [u16; ROAD_BUF_PIXELS],
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        let mut row = full_to_road_y((horizon_y + 16) as f32).max(0) as usize;
        while row + 2 < ROAD_BUF_H - 6 {
            let full_y = VIEW_Y + row as u16 * ROAD_BUF_SCALE + ROAD_BUF_SCALE / 2;
            let depth = (full_y - horizon_y) as f32 / (VIEW_Y + VIEW_H - horizon_y).max(1) as f32;
            let sample_dist = self.distance + 22.0 + depth * depth * VISIBLE_DISTANCE;
            if ((sample_dist / 54.0) as i32 & 1) != 0 {
                row = row.saturating_add(
                    ((10.0 + depth * 16.0) / ROAD_BUF_SCALE as f32).max(1.0) as usize
                );
                continue;
            }
            let road = self.sample_road(sample_dist);
            let road_half_full = (18.0 + depth * depth * 108.0) * road.width;
            let road_half = road_half_full / ROAD_BUF_SCALE as f32;
            let center = ROAD_BUF_W as f32 * 0.5
                + (road.offset * (0.32 + depth * 0.55)) / ROAD_BUF_SCALE as f32
                - self.player_x * road_half * 0.92;
            let decor_h = ((6.0 + depth * depth * 18.0) / ROAD_BUF_SCALE as f32).max(2.0) as u16;
            let left_x = clampf(center - road_half - 4.0, 0.0, (ROAD_BUF_W - 3) as f32) as i16;
            let right_x = clampf(center + road_half + 2.0, 0.0, (ROAD_BUF_W - 3) as f32) as i16;
            draw_buffer_roadside_sprite(
                buffer,
                left_x,
                row as i16,
                decor_h,
                self.selected_track,
                track,
                ui,
            );
            draw_buffer_roadside_sprite(
                buffer,
                right_x,
                row as i16,
                decor_h,
                self.selected_track,
                track,
                ui,
            );
            row = row
                .saturating_add(((14.0 + depth * 16.0) / ROAD_BUF_SCALE as f32).max(1.0) as usize);
        }
    }

    fn draw_center_overlay(
        &self,
        display: &mut Display,
        label: &str,
        count: Option<u16>,
        ui: &crate::display::Palette,
    ) {
        display.fill_rect(108, 94, 104, 44, color::mix(ui.panel, ui.shadow, 36));
        display.stroke_rect(108, 94, 104, 44, 2, ui.orange);
        display.centered_text(
            160,
            100,
            label,
            ui.text_muted,
            color::mix(ui.panel, ui.shadow, 36),
            1,
        );
        if let Some(count) = count {
            let mut line: String<4> = String::new();
            let _ = write!(&mut line, "{}", count);
            display.centered_text(
                160,
                112,
                &line,
                ui.white,
                color::mix(ui.panel, ui.shadow, 36),
                2,
            );
        }
    }

    fn draw_finish_overlay(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        let won = self.distance >= self.current_track_total;
        display.fill_rect(76, 82, 168, 78, color::mix(ui.panel, ui.shadow, 42));
        display.stroke_rect(76, 82, 168, 78, 2, if won { ui.cyan } else { ui.rose });
        display.centered_text(
            160,
            92,
            if won {
                if zh_mode {
                    "完成賽段"
                } else {
                    "STAGE CLEAR"
                }
            } else if zh_mode {
                "時間耗盡"
            } else {
                "TIME UP"
            },
            ui.white,
            color::mix(ui.panel, ui.shadow, 42),
            1,
        );

        let mut line: String<24> = String::new();
        let _ = write!(
            &mut line,
            "{} {:02}.{:01}s",
            if zh_mode { "用時" } else { "TIME" },
            self.elapsed_ms / 1000,
            (self.elapsed_ms % 1000) / 100
        );
        display.centered_text(
            160,
            110,
            &line,
            ui.text,
            color::mix(ui.panel, ui.shadow, 42),
            1,
        );

        let mut best_line: String<28> = String::new();
        let best = self.best_time_ms[self.selected_track];
        if best > 0 {
            let _ = write!(
                &mut best_line,
                "{} {:02}.{:01}s",
                if zh_mode { "最佳" } else { "BEST" },
                best / 1000,
                (best % 1000) / 100
            );
            display.centered_text(
                160,
                123,
                &best_line,
                color::mix(ui.white, ui.cyan, 110),
                color::mix(ui.panel, ui.shadow, 42),
                1,
            );
        }

        display.centered_text(
            160,
            136,
            if zh_mode {
                "K1 重跑  WK 回選關"
            } else {
                "K1 RETRY  WK TRACKS"
            },
            ui.text_muted,
            color::mix(ui.panel, ui.shadow, 42),
            1,
        );
        display.fill_rect(100, 205, 120, 20, color::mix(track.horizon, ui.white, 46));
        display.stroke_rect(100, 205, 120, 20, 1, ui.white);
        display.centered_text(
            160,
            211,
            if zh_mode { "重新出發" } else { "RUN AGAIN" },
            ui.text,
            color::mix(track.horizon, ui.white, 46),
            1,
        );
    }

    fn draw_hud(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.fill_rect(
            VIEW_X + 6,
            VIEW_Y + 6,
            102,
            26,
            color::mix(ui.panel, ui.shadow, 38),
        );
        display.stroke_rect(VIEW_X + 6, VIEW_Y + 6, 102, 26, 1, ui.white);
        display.text(
            VIEW_X + 12,
            VIEW_Y + 11,
            &fit_text(
                display,
                if zh_mode {
                    track.name_zh
                } else {
                    track.name_en
                },
                94,
            ),
            ui.white,
            color::mix(ui.panel, ui.shadow, 38),
            1,
        );
        display.text(
            VIEW_X + 12,
            VIEW_Y + 22,
            if self.boost_flash_ms > 0 {
                if zh_mode {
                    "BOOST"
                } else {
                    "BOOST"
                }
            } else if self.offroad_warning_ms > 0 {
                if zh_mode {
                    "OFFROAD"
                } else {
                    "OFFROAD"
                }
            } else if self.speed > 56.0 {
                "HIGH SPD"
            } else {
                "CRUISE"
            },
            if self.offroad_warning_ms > 0 {
                ui.rose
            } else {
                ui.text_muted
            },
            color::mix(ui.panel, ui.shadow, 38),
            1,
        );

        let mut speed_line: String<18> = String::new();
        let _ = write!(&mut speed_line, "SPD {:02}", self.speed as u16);
        display.fill_rect(
            VIEW_X + 194,
            VIEW_Y + 6,
            84,
            12,
            color::mix(ui.panel, ui.shadow, 34),
        );
        display.stroke_rect(VIEW_X + 194, VIEW_Y + 6, 84, 12, 1, ui.cyan);
        display.text(
            VIEW_X + 202,
            VIEW_Y + 9,
            &speed_line,
            ui.text,
            color::mix(ui.panel, ui.shadow, 34),
            1,
        );
        let speed_bar = ((self.speed / MAX_SPEED) * 28.0) as u16;
        display.fill_rect(
            VIEW_X + 244,
            VIEW_Y + 9,
            28,
            4,
            color::mix(ui.shadow, ui.panel, 70),
        );
        display.fill_rect(VIEW_X + 244, VIEW_Y + 9, speed_bar.min(28), 4, ui.cyan);

        let mut time_line: String<18> = String::new();
        let _ = write!(
            &mut time_line,
            "T {:02}.{:01}",
            self.time_left_ms / 1000,
            (self.time_left_ms % 1000) / 100
        );
        display.fill_rect(
            VIEW_X + 194,
            VIEW_Y + 20,
            84,
            12,
            color::mix(ui.panel, ui.shadow, 34),
        );
        display.stroke_rect(VIEW_X + 194, VIEW_Y + 20, 84, 12, 1, ui.amber);
        display.text(
            VIEW_X + 202,
            VIEW_Y + 23,
            &time_line,
            ui.text,
            color::mix(ui.panel, ui.shadow, 34),
            1,
        );
        let best = self.best_time_ms[self.selected_track];
        if best > 0 {
            let mut best_line: String<18> = String::new();
            let _ = write!(
                &mut best_line,
                "B {:02}.{:01}",
                best / 1000,
                (best % 1000) / 100
            );
            display.text(
                VIEW_X + 242,
                VIEW_Y + 23,
                &best_line,
                ui.white,
                color::mix(ui.panel, ui.shadow, 34),
                1,
            );
        }

        let progress = clampf(
            if self.current_track_total > 0.0 {
                self.distance / self.current_track_total
            } else {
                0.0
            },
            0.0,
            1.0,
        );
        display.fill_rect(
            VIEW_X + 12,
            VIEW_Y + VIEW_H - 22,
            VIEW_W - 24,
            10,
            color::mix(ui.shadow, ui.panel, 70),
        );
        display.stroke_rect(
            VIEW_X + 12,
            VIEW_Y + VIEW_H - 22,
            VIEW_W - 24,
            10,
            1,
            ui.steel,
        );
        display.fill_rect(
            VIEW_X + 14,
            VIEW_Y + VIEW_H - 20,
            ((VIEW_W - 28) as f32 * progress) as u16,
            6,
            ui.orange,
        );

        for checkpoint in 0..CHECKPOINT_COUNT {
            let x = VIEW_X + 14 + (((VIEW_W - 28) as f32 * ((checkpoint + 1) as f32 / 4.0)) as u16);
            let active = self.checkpoint_index > checkpoint;
            display.fill_rect(
                x.saturating_sub(1),
                VIEW_Y + VIEW_H - 24,
                3,
                14,
                if active { ui.cyan } else { ui.white },
            );
        }

        let mut cp_line: String<16> = String::new();
        let _ = write!(
            &mut cp_line,
            "CP {}/{}",
            self.checkpoint_index.min(CHECKPOINT_COUNT),
            CHECKPOINT_COUNT
        );
        display.fill_rect(
            VIEW_X + VIEW_W - 56,
            VIEW_Y + VIEW_H - 38,
            44,
            10,
            color::mix(ui.panel, ui.shadow, 40),
        );
        display.stroke_rect(
            VIEW_X + VIEW_W - 56,
            VIEW_Y + VIEW_H - 38,
            44,
            10,
            1,
            ui.white,
        );
        display.text(
            VIEW_X + VIEW_W - 52,
            VIEW_Y + VIEW_H - 36,
            &cp_line,
            ui.white,
            color::mix(ui.panel, ui.shadow, 40),
            1,
        );

        display.text(
            VIEW_X + 18,
            VIEW_Y + VIEW_H - 36,
            if zh_mode {
                "K0/WK 轉向  K1 煞車"
            } else {
                "K0/WK STEER  K1 BRAKE"
            },
            ui.white,
            track.road_a,
            1,
        );
    }

    #[allow(dead_code)]
    fn draw_backdrop(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        match self.selected_track {
            0 => {
                display.fill_rect(
                    VIEW_X + 216,
                    VIEW_Y + 16,
                    30,
                    30,
                    color::mix(track.horizon, ui.white, 58),
                );
                for i in 0..4u16 {
                    let x = VIEW_X + 30 + i * 66
                        - (((self.distance * 0.12) as i32).rem_euclid(40) as u16);
                    display.fill_rect(
                        x,
                        VIEW_Y + 18 + (i & 1) * 2,
                        18,
                        6,
                        color::mix(ui.white, track.sky, 40),
                    );
                    display.fill_rect(
                        x + 5,
                        VIEW_Y + 14 + (i & 1) * 2,
                        12,
                        4,
                        color::mix(ui.white, track.sky, 28),
                    );
                }
            }
            1 => {
                display.fill_rect(
                    VIEW_X + 222,
                    VIEW_Y + 18,
                    24,
                    24,
                    color::mix(track.horizon, ui.white, 42),
                );
                for i in 0..6u16 {
                    let x = VIEW_X + i * 42;
                    let height = 10 + ((i as usize + self.checkpoint_index) % 3) as u16 * 6;
                    display.fill_rect(
                        x,
                        horizon_y.saturating_sub(height),
                        18,
                        height,
                        color::mix(track.horizon, ui.shadow, 90),
                    );
                }
            }
            _ => {
                display.fill_rect(
                    VIEW_X + 224,
                    VIEW_Y + 14,
                    18,
                    18,
                    color::mix(ui.white, track.horizon, 24),
                );
                display.fill_rect(
                    VIEW_X + 229,
                    VIEW_Y + 18,
                    12,
                    12,
                    color::mix(ui.white, track.sky, 12),
                );
                for i in 0..10u16 {
                    let x = VIEW_X + ((i * 23 + (self.distance as u16 * 2)) % VIEW_W);
                    let y = VIEW_Y + 8 + ((i * 7) % 18);
                    display.fill_rect(x, y, 1, 1, ui.white);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn draw_hills(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        let hill_color = color::mix(track.horizon, ui.indigo, 90);
        let base = horizon_y.saturating_sub(20);
        let x_shift = (((self.distance * 0.18) as i32) % 48) as i16;
        for index in 0..6u16 {
            let x = VIEW_X as i16 + index as i16 * 48 - x_shift;
            let peak = base.saturating_sub((index % 3) * 6);
            if x + 28 < VIEW_X as i16 || x > (VIEW_X + VIEW_W) as i16 {
                continue;
            }
            display.fill_rect(
                x.max(VIEW_X as i16) as u16,
                peak,
                28,
                horizon_y.saturating_sub(peak),
                hill_color,
            );
        }
    }

    #[allow(dead_code)]
    fn draw_roadside_posts(&self, display: &mut Display, track: &TrackDefinition, horizon_y: u16) {
        let mut y = horizon_y + 8;
        while y < VIEW_Y + VIEW_H - 18 {
            let depth = (y - horizon_y) as f32 / (VIEW_Y + VIEW_H - horizon_y).max(1) as f32;
            let sample_dist = self.distance + 10.0 + depth * depth * VISIBLE_DISTANCE;
            let road = self.sample_road(sample_dist);
            let road_half = (18.0 + depth * depth * 108.0) * road.width;
            let center = VIEW_X as f32 + VIEW_W as f32 * 0.5 + road.offset * (0.32 + depth * 0.55)
                - self.player_x * road_half * 0.92;
            let post_h = (2.0 + depth * depth * 10.0) as u16;
            let post_color = if ((sample_dist / 28.0) as i32 & 1) == 0 {
                track.stripe_b
            } else {
                track.stripe_a
            };
            let left_x = clampf(
                center - road_half - 12.0,
                VIEW_X as f32,
                (VIEW_X + VIEW_W - 2) as f32,
            ) as u16;
            let right_x = clampf(
                center + road_half + 10.0,
                VIEW_X as f32,
                (VIEW_X + VIEW_W - 2) as f32,
            ) as u16;
            display.fill_rect(left_x, y.saturating_sub(post_h), 2, post_h, post_color);
            display.fill_rect(right_x, y.saturating_sub(post_h), 2, post_h, post_color);
            y = y.saturating_add((6.0 + depth * 12.0) as u16);
        }
    }

    #[allow(dead_code)]
    fn draw_roadside_decor(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        horizon_y: u16,
        ui: &crate::display::Palette,
    ) {
        let mut y = horizon_y + 16;
        while y < VIEW_Y + VIEW_H - 32 {
            let depth = (y - horizon_y) as f32 / (VIEW_Y + VIEW_H - horizon_y).max(1) as f32;
            let sample_dist = self.distance + 22.0 + depth * depth * VISIBLE_DISTANCE;
            if ((sample_dist / 54.0) as i32 & 1) != 0 {
                y = y.saturating_add((10.0 + depth * 16.0) as u16);
                continue;
            }
            let road = self.sample_road(sample_dist);
            let road_half = (18.0 + depth * depth * 108.0) * road.width;
            let center = VIEW_X as f32 + VIEW_W as f32 * 0.5 + road.offset * (0.32 + depth * 0.55)
                - self.player_x * road_half * 0.92;
            let decor_h = (6.0 + depth * depth * 18.0) as u16;
            let left_x = clampf(
                center - road_half - 18.0,
                VIEW_X as f32,
                (VIEW_X + VIEW_W - 10) as f32,
            ) as u16;
            let right_x = clampf(
                center + road_half + 10.0,
                VIEW_X as f32,
                (VIEW_X + VIEW_W - 10) as f32,
            ) as u16;
            draw_roadside_sprite(
                display,
                left_x,
                y,
                decor_h.max(6),
                self.selected_track,
                track,
                ui,
            );
            draw_roadside_sprite(
                display,
                right_x,
                y,
                decor_h.max(6),
                self.selected_track,
                track,
                ui,
            );
            y = y.saturating_add((14.0 + depth * 16.0) as u16);
        }
    }

    fn draw_objects(&self, display: &mut Display, track: &TrackDefinition) {
        let mut order = [0usize; MAX_OBJECTS];
        let mut count = 0usize;
        for index in 0..self.object_count {
            if !self.objects[index].active {
                continue;
            }
            let rel = self.objects[index].distance - self.distance;
            if rel <= 4.0 || rel > VISIBLE_DISTANCE {
                continue;
            }
            order[count] = index;
            count += 1;
        }

        for i in 0..count {
            for j in (i + 1)..count {
                let a = self.objects[order[i]].distance;
                let b = self.objects[order[j]].distance;
                if a < b {
                    order.swap(i, j);
                }
            }
        }

        for order_index in order[..count].iter().rev() {
            let object = self.objects[*order_index];
            let rel = object.distance - self.distance;
            let depth = 1.0 - rel / VISIBLE_DISTANCE;
            let depth = clampf(depth, 0.02, 1.0);
            let road = self.sample_road(object.distance);
            let road_half = (18.0 + depth * depth * 108.0) * road.width;
            let center = VIEW_X as f32 + VIEW_W as f32 * 0.5 + road.offset * (0.32 + depth * 0.55)
                - self.player_x * road_half * 0.92;
            let lane = self.object_lane(*order_index, object);
            let sprite_x = center + lane * road_half * 0.72;
            let bob = sinf(self.elapsed_ms as f32 * 0.003 + *order_index as f32) * depth * 2.0;
            let sprite_y = VIEW_Y as f32 + 30.0 + depth * depth * (VIEW_H as f32 - 44.0) + bob;
            let size = (6.0 + depth * depth * 18.0) as u16;
            draw_road_object(
                display,
                sprite_x as i16,
                sprite_y as i16,
                size.max(4),
                object.kind,
                track,
            );
        }
    }

    fn draw_checkpoint_banner(&self, display: &mut Display, track: &TrackDefinition) {
        if self.checkpoint_index >= CHECKPOINT_COUNT {
            return;
        }
        let checkpoint_distance =
            self.current_track_total * ((self.checkpoint_index + 1) as f32 / 4.0);
        let rel = checkpoint_distance - self.distance;
        if !(32.0..110.0).contains(&rel) {
            return;
        }
        let depth = 1.0 - rel / 110.0;
        let road = self.sample_road(checkpoint_distance);
        let road_half = (18.0 + depth * depth * 108.0) * road.width;
        let center = VIEW_X as f32 + VIEW_W as f32 * 0.5 + road.offset * (0.32 + depth * 0.55)
            - self.player_x * road_half * 0.92;
        let y = (VIEW_Y as f32 + 22.0 + depth * depth * (VIEW_H as f32 - 62.0)) as u16;
        let left = clampf(
            center - road_half - 2.0,
            VIEW_X as f32,
            (VIEW_X + VIEW_W - 30) as f32,
        ) as u16;
        let width = ((road_half * 2.0) as u16).min(120).max(42);
        display.fill_rect(
            left,
            y,
            width,
            3,
            color::mix(track.horizon, color::WHITE, 120),
        );
        display.fill_rect(left + 2, y + 3, 2, 12, track.stripe_a);
        display.fill_rect(left + width.saturating_sub(4), y + 3, 2, 12, track.stripe_a);
    }

    fn draw_player_car(
        &self,
        display: &mut Display,
        track: &TrackDefinition,
        ui: &crate::display::Palette,
    ) {
        let center_x = VIEW_X + VIEW_W / 2;
        let y = VIEW_Y + VIEW_H - 38;
        let shift = (self.steering_tilt * 8.0) as i16;
        let body = if self.crash_flash_ms > 0 {
            ui.white
        } else {
            color::mix(track.horizon, ui.white, 34)
        };
        let x = center_x as i16 + shift - 12;
        if self.speed > 54.0 {
            let flame = color::mix(ui.amber, ui.white, 72);
            display.fill_rect((x + 9).max(0) as u16, y + 24, 2, 4, flame);
            display.fill_rect((x + 13).max(0) as u16, y + 24, 2, 4, flame);
        }
        display.fill_rect(x.max(0) as u16, y + 8, 24, 14, body);
        display.stroke_rect(x.max(0) as u16, y + 8, 24, 14, 1, ui.shadow);
        display.fill_rect((x + 5).max(0) as u16, y + 4, 14, 8, ui.cyan);
        display.stroke_rect((x + 5).max(0) as u16, y + 4, 14, 8, 1, ui.white);
        display.fill_rect((x + 2).max(0) as u16, y + 22, 6, 4, ui.shadow);
        display.fill_rect((x + 16).max(0) as u16, y + 22, 6, 4, ui.shadow);
        display.fill_rect((x + 4).max(0) as u16, y + 12, 3, 2, ui.amber);
        display.fill_rect((x + 17).max(0) as u16, y + 12, 3, 2, ui.rose);
        if self.state == RacerState::Racing && self.speed < CRUISE_SPEED {
            display.fill_rect((x + 9).max(0) as u16, y + 24, 2, 4, ui.rose);
            display.fill_rect((x + 13).max(0) as u16, y + 24, 2, 4, ui.rose);
        }
    }

    fn start_run(&mut self) {
        let track = &TRACKS[self.selected_track];
        self.state = RacerState::Countdown;
        self.full_redraw_pending = true;
        self.render_pending = true;
        self.render_accum_ms = 0;
        self.countdown_ms = 2_800;
        self.elapsed_ms = 0;
        self.time_left_ms = track.checkpoint_time_ms;
        self.distance = 0.0;
        self.speed = CRUISE_SPEED;
        self.player_x = 0.0;
        self.steering_tilt = 0.0;
        self.crash_flash_ms = 0;
        self.checkpoint_flash_ms = 0;
        self.offroad_warning_ms = 0;
        self.boost_flash_ms = 0;
        self.checkpoint_index = 0;
        self.current_track_total = track_length(track);
        self.load_objects(track);
    }

    fn queue_runtime_frame(&mut self, dt_ms: u32) {
        self.render_accum_ms = self
            .render_accum_ms
            .saturating_add(dt_ms.min(u16::MAX as u32) as u16);
        if self.render_accum_ms >= RACER_FRAME_INTERVAL_MS {
            self.render_accum_ms = self.render_accum_ms.saturating_sub(RACER_FRAME_INTERVAL_MS);
            self.render_pending = true;
        }
    }

    fn load_objects(&mut self, track: &TrackDefinition) {
        self.objects = [BLANK_OBJECT; MAX_OBJECTS];
        self.object_count = track.objects.len().min(MAX_OBJECTS);
        for (index, seed) in track.objects.iter().take(self.object_count).enumerate() {
            self.objects[index] = RoadObject {
                distance: seed.distance as f32,
                lane: seed.lane as f32 * 0.45,
                kind: seed.kind,
                active: true,
            };
        }
    }

    fn check_collisions(&mut self) {
        let elapsed_ms = self.elapsed_ms;
        for (index, object) in self.objects.iter_mut().take(self.object_count).enumerate() {
            if !object.active {
                continue;
            }
            let rel = object.distance - self.distance;
            if !(0.0..10.0).contains(&rel) {
                continue;
            }
            let dynamic_lane = match object.kind {
                RoadObjectKind::Traffic => {
                    let sway = sinf(elapsed_ms as f32 * 0.0022 + index as f32 * 0.8) * 0.08;
                    clampf(object.lane + sway, -0.72, 0.72)
                }
                RoadObjectKind::Truck => clampf(object.lane, -0.65, 0.65),
                _ => object.lane,
            };
            let lane_threshold = match object.kind {
                RoadObjectKind::Truck => 0.35,
                RoadObjectKind::Barrier => 0.28,
                RoadObjectKind::Traffic => 0.24,
                RoadObjectKind::Cone => 0.16,
            };
            if fabsf(self.player_x - dynamic_lane) <= lane_threshold {
                self.speed = (self.speed * 0.58).max(BRAKE_SPEED);
                self.crash_flash_ms = 220;
                object.active = false;
            }
        }
    }

    fn check_checkpoints(&mut self) {
        while self.checkpoint_index < CHECKPOINT_COUNT {
            let checkpoint_distance =
                self.current_track_total * ((self.checkpoint_index + 1) as f32 / 4.0);
            if self.distance < checkpoint_distance {
                break;
            }
            self.checkpoint_index += 1;
            self.time_left_ms = self
                .time_left_ms
                .saturating_add(TRACKS[self.selected_track].checkpoint_time_ms / 2);
            self.checkpoint_flash_ms = 500;
            self.boost_flash_ms = BOOST_FLASH_MS;
            self.speed = clampf(self.speed + 12.0, BRAKE_SPEED, MAX_SPEED);
        }
    }

    fn object_lane(&self, index: usize, object: RoadObject) -> f32 {
        match object.kind {
            RoadObjectKind::Traffic => {
                let sway = sinf(self.elapsed_ms as f32 * 0.0022 + index as f32 * 0.8) * 0.08;
                clampf(object.lane + sway, -0.72, 0.72)
            }
            RoadObjectKind::Truck => clampf(object.lane, -0.65, 0.65),
            _ => object.lane,
        }
    }

    fn road_horizon_y(&self) -> u16 {
        let road = self.sample_road(self.distance + 10.0);
        clampf(
            (VIEW_Y + 38) as f32 - road.hill * 10.0,
            VIEW_Y as f32 + 22.0,
            VIEW_Y as f32 + 58.0,
        ) as u16
    }

    fn sample_road(&self, sample_distance: f32) -> RoadState {
        let track = &TRACKS[self.selected_track];
        let track_len = self.current_track_total.max(1.0);
        let sample = clampf(sample_distance, 0.0, track_len - 1.0);
        let mut cursor = self.distance.min(sample);
        let mut offset = 0.0f32;
        while cursor + 0.1 < sample {
            let (_, segment, seg_end) = segment_for_distance(track, cursor);
            let step = (seg_end - cursor).min(sample - cursor);
            offset += segment.curve as f32 * step * 0.0085;
            cursor += step.max(1.0);
        }
        let (_, segment, _) = segment_for_distance(track, sample);
        RoadState {
            offset,
            hill: segment.hill as f32 / 12.0,
            width: segment.width as f32 / 100.0,
            curve: segment.curve as f32 / 30.0,
        }
    }
}

fn track_length(track: &TrackDefinition) -> f32 {
    track
        .segments
        .iter()
        .map(|segment| segment.length as f32)
        .sum()
}

fn segment_for_distance(track: &TrackDefinition, distance: f32) -> (usize, TrackSegment, f32) {
    let mut cursor = 0.0f32;
    for (index, segment) in track.segments.iter().enumerate() {
        cursor += segment.length as f32;
        if distance < cursor {
            return (index, *segment, cursor);
        }
    }
    let last = track.segments[track.segments.len() - 1];
    (track.segments.len() - 1, last, cursor)
}

fn draw_track_stamp(
    display: &mut Display,
    x: u16,
    y: u16,
    index: usize,
    accent: u16,
    bg: u16,
    ui: &crate::display::Palette,
) {
    match index {
        0 => {
            display.fill_rect(x + 1, y + 7, 16, 3, accent);
            display.fill_rect(x + 2, y + 2, 12, 5, color::mix(accent, ui.white, 80));
            display.fill_rect(x + 5, y + 1, 6, 2, ui.white);
        }
        1 => {
            display.fill_rect(x + 2, y + 10, 15, 2, accent);
            display.fill_rect(x + 3, y + 6, 12, 4, color::mix(accent, ui.amber, 80));
            display.fill_rect(x + 8, y + 2, 4, 4, ui.white);
        }
        _ => {
            display.fill_rect(x + 2, y + 9, 14, 4, color::mix(bg, ui.white, 30));
            display.stroke_rect(x + 2, y + 9, 14, 4, 1, accent);
            display.fill_rect(x + 6, y + 2, 5, 5, ui.white);
            display.fill_rect(x + 8, y + 1, 1, 1, accent);
        }
    }
}

fn draw_road_object(
    display: &mut Display,
    x: i16,
    y: i16,
    size: u16,
    kind: RoadObjectKind,
    track: &TrackDefinition,
) {
    let base_x = x.saturating_sub(size as i16 / 2).max(0) as u16;
    let base_y = y.saturating_sub(size as i16).max(0) as u16;
    match kind {
        RoadObjectKind::Traffic => {
            display.fill_rect(
                base_x,
                base_y + size / 2,
                size,
                size / 2,
                color::rgb565(255, 90, 82),
            );
            display.fill_rect(
                base_x + size / 4,
                base_y + size / 4,
                size / 2,
                size / 3,
                color::rgb565(190, 235, 255),
            );
            display.fill_rect(
                base_x + size / 6,
                base_y + size,
                size / 5,
                size / 6,
                color::rgb565(20, 20, 26),
            );
            display.fill_rect(
                base_x + size - size / 3,
                base_y + size,
                size / 5,
                size / 6,
                color::rgb565(20, 20, 26),
            );
        }
        RoadObjectKind::Truck => {
            display.fill_rect(
                base_x,
                base_y + size / 3,
                size,
                size * 2 / 3,
                color::rgb565(97, 210, 233),
            );
            display.fill_rect(
                base_x + size / 2,
                base_y,
                size / 2,
                size / 2,
                color::rgb565(238, 247, 255),
            );
            display.fill_rect(
                base_x + size / 8,
                base_y + size,
                size / 5,
                size / 6,
                color::rgb565(18, 18, 22),
            );
            display.fill_rect(
                base_x + size - size / 3,
                base_y + size,
                size / 5,
                size / 6,
                color::rgb565(18, 18, 22),
            );
        }
        RoadObjectKind::Cone => {
            display.fill_rect(
                base_x + size / 4,
                base_y + size / 3,
                size / 2,
                size / 2,
                color::rgb565(255, 164, 40),
            );
            display.fill_rect(
                base_x + size / 3,
                base_y + size / 2,
                size / 3,
                size / 6,
                track.stripe_b,
            );
        }
        RoadObjectKind::Barrier => {
            display.fill_rect(base_x, base_y + size / 2, size, size / 3, track.stripe_a);
            display.fill_rect(
                base_x,
                base_y + size * 2 / 3,
                size,
                size / 6,
                track.stripe_b,
            );
        }
    }
}

#[allow(dead_code)]
fn draw_roadside_sprite(
    display: &mut Display,
    x: u16,
    baseline_y: u16,
    size: u16,
    track_index: usize,
    track: &TrackDefinition,
    ui: &crate::display::Palette,
) {
    let top = baseline_y.saturating_sub(size);
    match track_index {
        0 => {
            display.fill_rect(
                x + size / 3,
                top,
                size / 6,
                size,
                color::rgb565(121, 82, 40),
            );
            display.fill_rect(
                x,
                top + size / 5,
                size,
                size / 3,
                color::rgb565(52, 204, 132),
            );
            display.fill_rect(
                x + size / 5,
                top,
                size * 3 / 5,
                size / 4,
                color::rgb565(92, 230, 170),
            );
        }
        1 => {
            display.fill_rect(
                x,
                top + size / 3,
                size,
                size * 2 / 3,
                color::mix(track.horizon, ui.shadow, 90),
            );
            display.fill_rect(
                x + size / 5,
                top,
                size / 2,
                size / 3,
                color::mix(ui.amber, ui.white, 64),
            );
            display.fill_rect(x + size / 3, top + size / 2, size / 4, size / 5, ui.white);
        }
        _ => {
            display.fill_rect(
                x + size / 3,
                top,
                size / 6,
                size,
                color::rgb565(155, 155, 188),
            );
            display.fill_rect(
                x,
                top + size / 5,
                size,
                size / 5,
                color::mix(ui.cyan, ui.white, 90),
            );
            display.fill_rect(
                x + size / 4,
                top + size / 2,
                size / 2,
                size / 4,
                color::mix(track.horizon, ui.white, 36),
            );
        }
    }
}

fn road_buffer_clear(buffer: &mut [u16; ROAD_BUF_PIXELS], color: u16) {
    for pixel in buffer.iter_mut() {
        *pixel = color;
    }
}

fn road_buffer_fill_rect(
    buffer: &mut [u16; ROAD_BUF_PIXELS],
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    color: u16,
) {
    if width == 0 || height == 0 {
        return;
    }

    let x0 = x.clamp(0, ROAD_BUF_W as i16);
    let y0 = y.clamp(0, ROAD_BUF_H as i16);
    let x1 = (x + width as i16).clamp(0, ROAD_BUF_W as i16);
    let y1 = (y + height as i16).clamp(0, ROAD_BUF_H as i16);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    for row in y0 as usize..y1 as usize {
        let start = row * ROAD_BUF_W + x0 as usize;
        let end = row * ROAD_BUF_W + x1 as usize;
        buffer[start..end].fill(color);
    }
}

fn draw_buffer_roadside_sprite(
    buffer: &mut [u16; ROAD_BUF_PIXELS],
    x: i16,
    baseline_y: i16,
    size: u16,
    track_index: usize,
    track: &TrackDefinition,
    ui: &crate::display::Palette,
) {
    let top = baseline_y.saturating_sub(size as i16);
    match track_index {
        0 => {
            road_buffer_fill_rect(
                buffer,
                x + size as i16 / 3,
                top,
                (size / 6).max(1),
                size,
                color::rgb565(121, 82, 40),
            );
            road_buffer_fill_rect(
                buffer,
                x,
                top + size as i16 / 5,
                size.max(2),
                (size / 3).max(1),
                color::rgb565(52, 204, 132),
            );
            road_buffer_fill_rect(
                buffer,
                x + size as i16 / 5,
                top,
                (size * 3 / 5).max(2),
                (size / 4).max(1),
                color::rgb565(92, 230, 170),
            );
        }
        1 => {
            road_buffer_fill_rect(
                buffer,
                x,
                top + size as i16 / 3,
                size.max(2),
                (size * 2 / 3).max(1),
                color::mix(track.horizon, ui.shadow, 90),
            );
            road_buffer_fill_rect(
                buffer,
                x + size as i16 / 5,
                top,
                (size * 3 / 5).max(2),
                (size / 3).max(1),
                color::mix(track.horizon, ui.amber, 60),
            );
        }
        _ => {
            road_buffer_fill_rect(
                buffer,
                x + size as i16 / 3,
                top + size as i16 / 4,
                (size / 5).max(1),
                (size * 3 / 4).max(1),
                color::rgb565(110, 102, 174),
            );
            road_buffer_fill_rect(
                buffer,
                x,
                top + size as i16 / 3,
                size.max(2),
                (size / 3).max(1),
                color::mix(ui.cyan, track.sky, 60),
            );
        }
    }
}

fn full_to_road_y(y: f32) -> i16 {
    floorf((y - VIEW_Y as f32) / ROAD_BUF_SCALE as f32) as i16
}

fn clampf(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
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
