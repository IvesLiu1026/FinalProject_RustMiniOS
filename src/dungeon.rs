mod data;
mod strategy;
mod weapon;

use core::fmt::Write;

use heapless::String;
use libm::{atan2f, cosf, fabsf, floorf, sinf, sqrtf};

use crate::assets::{self, EnemySprite, TextureId, TEX_SIZE};
use crate::board::ButtonSnapshot;
use crate::display::{palette, shade, Display, ThemeMode, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::touch::TouchState;
use data::{MapDef, MAPS, MAP_H, MAP_W, MAX_ENEMIES, MAX_PICKUPS};
pub use strategy::RenderStrategy;
use weapon::WeaponKind;

const COLUMN_WIDTH: u16 = 1;
const RAY_COUNT: u16 = SCREEN_WIDTH / COLUMN_WIDTH;
const CAMERA_PLANE_SCALE: f32 = 0.577_350_26;
const VIEW_W: usize = SCREEN_WIDTH as usize;
const VIEW_Y: u16 = 40;
const VIEW_H: u16 = SCREEN_HEIGHT - VIEW_Y;
const VIEW_BOTTOM: u16 = VIEW_Y + VIEW_H;
const VIEW_H_USIZE: usize = VIEW_H as usize;
const VIEW_PIXELS: usize = VIEW_W * VIEW_H_USIZE;

const CELL: u16 = 4;
const MAP_ORIGIN_X: u16 = 16;
const MAP_ORIGIN_Y: u16 = 48;
const MAP_BUTTON_X: u16 = 206;
const MAP_BUTTON_Y: u16 = 162;
const MAP_BUTTON_W: u16 = 102;
const MAP_BUTTON_H: u16 = 18;
const OVERLAY_RETRY_X: u16 = 70;
const OVERLAY_RETRY_Y: u16 = 138;
const OVERLAY_RETRY_W: u16 = 84;
const OVERLAY_RETRY_H: u16 = 22;
const OVERLAY_MAPS_X: u16 = 168;
const OVERLAY_MAPS_Y: u16 = 138;
const OVERLAY_MAPS_W: u16 = 84;
const OVERLAY_MAPS_H: u16 = 22;

const CONTROL_CENTER_X: u16 = 74;
const CONTROL_CENTER_Y: u16 = 207;
const CONTROL_RING_RADIUS: u16 = 30;
const CONTROL_BASE_RADIUS: u16 = 27;
const CONTROL_KNOB_RADIUS: u16 = 11;
const CONTROL_INPUT_RADIUS: f32 = 18.0;
const CONTROL_TOUCH_RADIUS: f32 = 34.0;
const CONTROL_DEADZONE: f32 = 0.16;

const FIRE_CENTER_X: u16 = 262;
const FIRE_CENTER_Y: u16 = 207;
const FIRE_RING_RADIUS: u16 = 28;
const FIRE_BASE_RADIUS: u16 = 24;
const FIRE_KNOB_RADIUS: u16 = 10;
const FIRE_INPUT_RADIUS: f32 = 10.0;
const FIRE_TOUCH_RADIUS: f32 = 30.0;
const WEAPON_TAP_CENTER_X: u16 = SCREEN_WIDTH / 2;
const WEAPON_TAP_CENTER_Y: u16 = 206;
const WEAPON_TAP_RADIUS: f32 = 26.0;

const PLAYER_MAX_HP: i16 = 100;
const PLAYER_RADIUS: f32 = 0.18;
const ENEMY_RADIUS: f32 = 0.16;

static mut VIEWPORT_BUFFER: [u16; VIEW_PIXELS] = [0; VIEW_PIXELS];
static mut ZBUFFER: [f32; RAY_COUNT as usize] = [0.0; RAY_COUNT as usize];

#[derive(Clone, Copy, PartialEq, Eq)]
enum TouchMode {
    None,
    Control,
    Fire,
}

#[derive(Clone, Copy)]
struct Enemy {
    kind: EnemySprite,
    x: f32,
    y: f32,
    hp: i16,
    attack_cooldown_ms: u16,
    hit_flash_ms: u16,
    death_anim_ms: u16,
    phase: u16,
    alive: bool,
}

#[derive(Clone, Copy)]
struct Pickup {
    x: f32,
    y: f32,
    amount: i16,
    phase: u16,
    active: bool,
}

impl Enemy {
    const fn dead() -> Self {
        Self {
            kind: EnemySprite::Imp,
            x: 0.0,
            y: 0.0,
            hp: 0,
            attack_cooldown_ms: 0,
            hit_flash_ms: 0,
            death_anim_ms: 0,
            phase: 0,
            alive: false,
        }
    }
}

impl Pickup {
    const fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            amount: 0,
            phase: 0,
            active: false,
        }
    }
}

pub enum DungeonAction {
    Stay,
    ExitHome,
    OpenMapSelect,
}

pub struct DungeonApp {
    player_x: f32,
    player_y: f32,
    angle: f32,
    map_index: usize,
    health: i16,
    score: u32,
    kills: u16,
    level_cleared: bool,
    game_over: bool,
    exit_hold_ms: u16,
    weapon: WeaponKind,
    fire_cooldown_ms: u16,
    muzzle_flash_ms: u16,
    shot_depth: f32,
    shot_hit_enemy: bool,
    touch_mode: TouchMode,
    heal_flash_ms: u16,
    intro_ms: u16,
    phase_ms: u16,
    redraw_pending: bool,
    prev_hud_health: i16,
    prev_hud_score: u32,
    prev_hud_kills: u16,
    prev_hud_map_index: usize,
    prev_hud_weapon: Option<WeaponKind>,
    prev_hud_fps: u16,
    prev_hud_exit_hold: bool,
    enemies: [Enemy; MAX_ENEMIES],
    pickups: [Pickup; MAX_PICKUPS],
}

impl DungeonApp {
    pub const fn new() -> Self {
        Self {
            player_x: 1.5,
            player_y: 1.5,
            angle: 0.15,
            map_index: 0,
            health: PLAYER_MAX_HP,
            score: 0,
            kills: 0,
            level_cleared: false,
            game_over: false,
            exit_hold_ms: 0,
            weapon: WeaponKind::Pulse,
            fire_cooldown_ms: 0,
            muzzle_flash_ms: 0,
            shot_depth: 1.0,
            shot_hit_enemy: false,
            touch_mode: TouchMode::None,
            heal_flash_ms: 0,
            intro_ms: 0,
            phase_ms: 0,
            redraw_pending: false,
            prev_hud_health: -1,
            prev_hud_score: u32::MAX,
            prev_hud_kills: u16::MAX,
            prev_hud_map_index: usize::MAX,
            prev_hud_weapon: None,
            prev_hud_fps: u16::MAX,
            prev_hud_exit_hold: false,
            enemies: [Enemy::dead(); MAX_ENEMIES],
            pickups: [Pickup::empty(); MAX_PICKUPS],
        }
    }

    pub fn map_count() -> usize {
        MAPS.len()
    }

    pub fn map_name(map_index: usize, zh_mode: bool) -> &'static str {
        let map = &MAPS[map_index % MAPS.len()];
        if zh_mode {
            map.name_zh
        } else {
            map.name_en
        }
    }

    pub fn set_map(&mut self, map_index: usize) {
        self.map_index = map_index % MAPS.len();
        self.load_current_map();
    }

    pub fn needs_animation(&self) -> bool {
        self.intro_ms == 0 && !self.game_over && !self.level_cleared
    }

    pub fn take_redraw_request(&mut self) -> bool {
        let redraw = self.redraw_pending;
        self.redraw_pending = false;
        redraw
    }

    fn load_current_map(&mut self) {
        let map = &MAPS[self.map_index];
        self.player_x = map.spawn_x;
        self.player_y = map.spawn_y;
        self.angle = map.spawn_angle;
        self.health = PLAYER_MAX_HP;
        self.level_cleared = false;
        self.game_over = false;
        self.exit_hold_ms = 0;
        self.fire_cooldown_ms = 0;
        self.muzzle_flash_ms = 0;
        self.shot_depth = 1.0;
        self.shot_hit_enemy = false;
        self.touch_mode = TouchMode::None;
        self.heal_flash_ms = 0;
        self.intro_ms = 900;
        self.phase_ms = 0;
        self.redraw_pending = true;
        self.prev_hud_health = -1;
        self.prev_hud_score = u32::MAX;
        self.prev_hud_kills = u16::MAX;
        self.prev_hud_map_index = usize::MAX;
        self.prev_hud_weapon = None;
        self.prev_hud_fps = u16::MAX;
        self.prev_hud_exit_hold = false;

        for enemy in &mut self.enemies {
            *enemy = Enemy::dead();
        }
        for pickup in &mut self.pickups {
            *pickup = Pickup::empty();
        }

        for (slot, spawn) in map.enemies.iter().enumerate() {
            if slot >= MAX_ENEMIES {
                break;
            }
            self.enemies[slot] = Enemy {
                kind: spawn.kind,
                x: spawn.x,
                y: spawn.y,
                hp: match spawn.kind {
                    EnemySprite::Imp => 3,
                    EnemySprite::Bat => 2,
                    EnemySprite::Hound => 4,
                },
                attack_cooldown_ms: 0,
                hit_flash_ms: 0,
                death_anim_ms: 0,
                phase: (slot as u16) * 27,
                alive: true,
            };
        }

        for (slot, spawn) in map.pickups.iter().enumerate() {
            if slot >= MAX_PICKUPS {
                break;
            }
            self.pickups[slot] = Pickup {
                x: spawn.x,
                y: spawn.y,
                amount: spawn.amount,
                phase: (slot as u16) * 61,
                active: true,
            };
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> DungeonAction {
        let map = &MAPS[self.map_index];
        let turn_speed = (dt_ms as f32) * 0.0031;
        let move_speed = (dt_ms as f32) * 0.0023;
        let mut turn_input = 0.0f32;
        let mut forward_input = 0.0f32;
        self.phase_ms = self.phase_ms.wrapping_add(dt_ms as u16);
        let weapon_switch_tapped = touch.just_pressed
            && point_in_circle(
                touch.x,
                touch.y,
                WEAPON_TAP_CENTER_X,
                WEAPON_TAP_CENTER_Y,
                WEAPON_TAP_RADIUS,
            );
        let switch_prev = input.k0 && input.k1_just_pressed;
        let switch_next = weapon_switch_tapped || (input.wkup && input.k1_just_pressed);
        let switching_weapon = switch_prev || switch_next;

        if self.fire_cooldown_ms > 0 {
            self.fire_cooldown_ms = self.fire_cooldown_ms.saturating_sub(dt_ms as u16);
        }
        if self.muzzle_flash_ms > 0 {
            self.muzzle_flash_ms = self.muzzle_flash_ms.saturating_sub(dt_ms as u16);
        }
        if self.heal_flash_ms > 0 {
            self.heal_flash_ms = self.heal_flash_ms.saturating_sub(dt_ms as u16);
        }

        if input.home_chord() {
            let held = self.exit_hold_ms.saturating_add(dt_ms as u16);
            self.exit_hold_ms = held;
            if held > 500 {
                self.exit_hold_ms = 0;
                return DungeonAction::ExitHome;
            }
        } else {
            self.exit_hold_ms = 0;
        }

        if touch.just_released
            && touch_started_in_rect(touch, MAP_BUTTON_X, MAP_BUTTON_Y, MAP_BUTTON_W, MAP_BUTTON_H)
        {
            return DungeonAction::OpenMapSelect;
        }

        if self.game_over || self.level_cleared {
            if input.k1_just_pressed || touch.just_released {
                if touch.just_released
                    && touch_started_in_rect(
                        touch,
                        OVERLAY_MAPS_X,
                        OVERLAY_MAPS_Y,
                        OVERLAY_MAPS_W,
                        OVERLAY_MAPS_H,
                    )
                {
                    return DungeonAction::OpenMapSelect;
                }
                if !touch.just_released
                    || touch_started_in_rect(
                        touch,
                        OVERLAY_RETRY_X,
                        OVERLAY_RETRY_Y,
                        OVERLAY_RETRY_W,
                        OVERLAY_RETRY_H,
                    )
                {
                    self.load_current_map();
                }
            }
            return DungeonAction::Stay;
        }

        if self.intro_ms > 0 {
            let previous = self.intro_ms;
            self.intro_ms = self.intro_ms.saturating_sub(dt_ms as u16);
            if input.k1_just_pressed || touch.just_released {
                self.intro_ms = 0;
            }
            if previous != 0 && self.intro_ms == 0 {
                self.redraw_pending = true;
            }
            return DungeonAction::Stay;
        }

        if switch_prev {
            self.weapon = self.weapon.previous();
            self.prev_hud_weapon = None;
        } else if switch_next {
            self.weapon = self.weapon.next();
            self.prev_hud_weapon = None;
            self.touch_mode = TouchMode::None;
        }

        if touch.just_pressed {
            self.touch_mode =
                if weapon_switch_tapped {
                    TouchMode::None
                } else if point_in_circle(
                    touch.x,
                    touch.y,
                    CONTROL_CENTER_X,
                    CONTROL_CENTER_Y,
                    CONTROL_TOUCH_RADIUS,
                ) {
                    TouchMode::Control
                } else if point_in_circle(
                    touch.x,
                    touch.y,
                    FIRE_CENTER_X,
                    FIRE_CENTER_Y,
                    FIRE_TOUCH_RADIUS,
                ) {
                    TouchMode::Fire
                } else {
                    TouchMode::None
                };
        } else if !touch.active {
            self.touch_mode = TouchMode::None;
        }

        if input.k0 && !input.wkup {
            turn_input -= 1.0;
        } else if input.wkup && !input.k0 {
            turn_input += 1.0;
        }

        if input.k1 {
            forward_input += 1.0;
        }

        if touch.active {
            match self.touch_mode {
                TouchMode::Control => {
                    let (dx, dy) = clamp_circle_delta(
                        touch.x as f32 - CONTROL_CENTER_X as f32,
                        touch.y as f32 - CONTROL_CENTER_Y as f32,
                        CONTROL_INPUT_RADIUS,
                    );
                    let turn = apply_deadzone(dx / CONTROL_INPUT_RADIUS, CONTROL_DEADZONE);
                    let forward = apply_deadzone((-dy) / CONTROL_INPUT_RADIUS, CONTROL_DEADZONE);
                    turn_input += turn * (1.15 + 0.2 * forward.max(0.0));
                    forward_input += forward;
                }
                TouchMode::Fire | TouchMode::None => {}
            }
        }

        self.angle += turn_input * turn_speed;
        if self.angle < 0.0 {
            self.angle += core::f32::consts::TAU;
        }
        if self.angle >= core::f32::consts::TAU {
            self.angle -= core::f32::consts::TAU;
        }

        if forward_input != 0.0 {
            try_move(
                &mut self.player_x,
                &mut self.player_y,
                cosf(self.angle) * move_speed * forward_input,
                sinf(self.angle) * move_speed * forward_input,
                map,
            );
        }

        let fired = !switching_weapon
            && (input.k1_just_pressed || (touch.active && self.touch_mode == TouchMode::Fire))
            && self.fire_cooldown_ms == 0;
        if fired {
            self.fire_cooldown_ms = self.weapon.cooldown_ms();
            self.muzzle_flash_ms = self.weapon.flash_ms();
            self.try_shoot();
        }

        self.update_pickups(dt_ms as u16);
        self.update_enemies(dt_ms as u16);
        let was_cleared = self.level_cleared;
        self.level_cleared = self.enemies.iter().all(|enemy| !enemy.alive);
        if !was_cleared && self.level_cleared {
            self.redraw_pending = true;
        }

        DungeonAction::Stay
    }

    pub fn render(
        &mut self,
        display: &mut Display,
        touch: &TouchState,
        full_refresh: bool,
        theme: ThemeMode,
        zh_mode: bool,
        fps: u16,
        render_strategy: RenderStrategy,
    ) {
        let ui = palette(theme);

        if full_refresh {
            draw_shell(display, &ui, zh_mode);
            self.prev_hud_health = -1;
            self.prev_hud_score = u32::MAX;
            self.prev_hud_kills = u16::MAX;
            self.prev_hud_map_index = usize::MAX;
            self.prev_hud_weapon = None;
            self.prev_hud_fps = u16::MAX;
            self.prev_hud_exit_hold = false;
        }

        draw_viewport(display, self, touch, &ui, render_strategy);
        self.draw_hud(display, &ui, zh_mode, full_refresh, fps);

        if self.intro_ms > 0 {
            draw_intro_overlay(display, &ui, self.current_map(), zh_mode);
        } else if self.game_over {
            draw_overlay(
                display,
                &ui,
                if zh_mode { "遊戲結束" } else { "GAME OVER" },
                if zh_mode {
                    "按 K1 或點擊重來"
                } else {
                    "PRESS K1 OR TAP TO RETRY"
                },
                if zh_mode { "重新開始" } else { "RETRY" },
                if zh_mode { "地圖選單" } else { "MAP SELECT" },
                ui.rose,
            );
        } else if self.level_cleared {
            draw_overlay(
                display,
                &ui,
                if zh_mode { "關卡完成" } else { "AREA CLEARED" },
                if zh_mode {
                    "按 K1 或點擊重玩"
                } else {
                    "PRESS K1 OR TAP TO RELOAD"
                },
                if zh_mode { "再次挑戰" } else { "RETRY" },
                if zh_mode { "地圖選單" } else { "MAP SELECT" },
                ui.lime,
            );
        }
    }

    fn current_map(&self) -> &'static MapDef {
        &MAPS[self.map_index]
    }

    fn try_shoot(&mut self) {
        let (dir_x, dir_y, _, _) = direction_and_plane(self.angle);
        let center_hit = cast_ray(self.current_map(), self.player_x, self.player_y, dir_x, dir_y);
        self.shot_depth = center_hit.distance.max(0.6);
        self.shot_hit_enemy = false;

        match self.weapon {
            WeaponKind::Pulse => self.fire_hitscan(0.0, 2, 0.16),
            WeaponKind::Carbine => self.fire_hitscan(0.0, 1, 0.11),
            WeaponKind::Scatter => {
                for offset in [-0.11f32, -0.05, 0.0, 0.05, 0.11] {
                    self.fire_hitscan(offset, 1, 0.14);
                }
            }
        }
    }

    fn fire_hitscan(&mut self, angle_offset: f32, damage: i16, hit_window: f32) {
        let shot_angle = self.angle + angle_offset;
        let (dir_x, dir_y, _, _) = direction_and_plane(shot_angle);
        let center_hit = cast_ray(self.current_map(), self.player_x, self.player_y, dir_x, dir_y);
        if !self.shot_hit_enemy {
            self.shot_depth = center_hit.distance.max(0.6);
        }

        let mut best_index = None;
        let mut best_distance = 1.0e9f32;

        for (index, enemy) in self.enemies.iter().enumerate() {
            if !enemy.alive {
                continue;
            }

            let dx = enemy.x - self.player_x;
            let dy = enemy.y - self.player_y;
            let distance = sqrtf(dx * dx + dy * dy);
            if distance > 8.0 {
                continue;
            }

            let bearing = atan2f(dy, dx);
            let delta = wrap_angle(bearing - shot_angle);
            if fabsf(delta) > hit_window {
                continue;
            }

            if !line_of_sight(self.current_map(), self.player_x, self.player_y, enemy.x, enemy.y) {
                continue;
            }

            if distance < best_distance {
                best_distance = distance;
                best_index = Some(index);
            }
        }

        if let Some(index) = best_index {
            let enemy = &mut self.enemies[index];
            self.shot_depth = self.shot_depth.min(best_distance.max(0.4));
            self.shot_hit_enemy = true;
            enemy.hp -= damage;
            enemy.hit_flash_ms = 140;
            if enemy.hp <= 0 && enemy.alive {
                enemy.alive = false;
                enemy.death_anim_ms = 280;
                self.score = self.score.saturating_add(match enemy.kind {
                    EnemySprite::Bat => 80,
                    EnemySprite::Imp => 125,
                    EnemySprite::Hound => 160,
                });
                self.kills = self.kills.saturating_add(1);
            }
        }
    }

    fn update_pickups(&mut self, dt_ms: u16) {
        for pickup in &mut self.pickups {
            if !pickup.active {
                continue;
            }

            pickup.phase = pickup.phase.wrapping_add(dt_ms);
            let dx = pickup.x - self.player_x;
            let dy = pickup.y - self.player_y;
            let distance = sqrtf(dx * dx + dy * dy);

            if distance < 0.42 && self.health < PLAYER_MAX_HP {
                self.health = (self.health + pickup.amount).min(PLAYER_MAX_HP);
                pickup.active = false;
                self.heal_flash_ms = 220;
                self.prev_hud_health = -1;
            }
        }
    }

    fn update_enemies(&mut self, dt_ms: u16) {
        let map = self.current_map();

        for enemy in &mut self.enemies {
            enemy.hit_flash_ms = enemy.hit_flash_ms.saturating_sub(dt_ms);
            if !enemy.alive {
                enemy.death_anim_ms = enemy.death_anim_ms.saturating_sub(dt_ms);
                continue;
            }

            enemy.phase = enemy.phase.wrapping_add(dt_ms);
            enemy.attack_cooldown_ms = enemy.attack_cooldown_ms.saturating_sub(dt_ms);

            let dx = self.player_x - enemy.x;
            let dy = self.player_y - enemy.y;
            let distance = sqrtf(dx * dx + dy * dy);

            if distance < 0.75 {
                if enemy.attack_cooldown_ms == 0 {
                    let hit = match enemy.kind {
                        EnemySprite::Bat => 6,
                        EnemySprite::Imp => 10,
                        EnemySprite::Hound => 14,
                    };
                    self.health = self.health.saturating_sub(hit);
                    enemy.attack_cooldown_ms = 850;
                    if self.health <= 0 {
                        self.health = 0;
                        self.game_over = true;
                        self.redraw_pending = true;
                    }
                }
                continue;
            }

            if distance > 6.5 {
                continue;
            }

            if !line_of_sight(map, enemy.x, enemy.y, self.player_x, self.player_y) {
                continue;
            }

            let speed = match enemy.kind {
                EnemySprite::Bat => 0.0009,
                EnemySprite::Imp => 0.0007,
                EnemySprite::Hound => 0.0011,
            } * dt_ms as f32;

            let dir_x = dx / distance;
            let dir_y = dy / distance;
            try_enemy_move(&mut enemy.x, &mut enemy.y, dir_x * speed, dir_y * speed, map);
        }
    }

    fn draw_hud(
        &mut self,
        display: &mut Display,
        ui: &crate::display::Palette,
        zh_mode: bool,
        force: bool,
        fps: u16,
    ) {
        let map_name = hud_map_name(self.map_index, zh_mode);

        if force || self.prev_hud_fps != fps {
            let mut fps_line: String<16> = String::new();
            let _ = write!(&mut fps_line, "{}FPS", fps);
            display.fill_rect(250, 14, 48, 14, ui.panel);
            display.text(256, 18, &fps_line, ui.amber, ui.panel, 1);
            self.prev_hud_fps = fps;
        }

        if force || self.prev_hud_map_index != self.map_index {
            display.fill_rect(206, 44, 102, 18, ui.panel_alt);
            display.text(212, 50, map_name, ui.text, ui.panel_alt, 1);
            self.prev_hud_map_index = self.map_index;
        }

        if force || self.prev_hud_health != self.health {
            display.fill_rect(206, 66, 102, 18, ui.panel_alt);
            display.text(
                212,
                72,
                if zh_mode { "生命" } else { "HP" },
                ui.text_muted,
                ui.panel_alt,
                1,
            );
            display.fill_rect(246, 71, 54, 8, ui.shadow);
            let hp_width = ((self.health.max(0) as u32 * 54) / PLAYER_MAX_HP as u32) as u16;
            display.fill_rect(246, 71, hp_width, 8, ui.lime);
            self.prev_hud_health = self.health;
        }

        if force || self.prev_hud_score != self.score {
            let mut line: String<32> = String::new();
            let _ = write!(
                &mut line,
                "{} {}",
                if zh_mode { "分數" } else { "SCORE" },
                self.score
            );
            display.fill_rect(206, 90, 102, 16, ui.panel_alt);
            display.text(212, 95, &line, ui.text, ui.panel_alt, 1);
            self.prev_hud_score = self.score;
        }

        if force || self.prev_hud_kills != self.kills {
            let mut kills: String<24> = String::new();
            let _ = write!(
                &mut kills,
                "{} {}",
                if zh_mode { "擊殺" } else { "KILLS" },
                self.kills
            );
            display.fill_rect(206, 108, 102, 16, ui.panel_alt);
            display.text(212, 113, &kills, ui.text_muted, ui.panel_alt, 1);
            self.prev_hud_kills = self.kills;
        }

        if force || self.prev_hud_weapon != Some(self.weapon) {
            display.fill_rect(206, 126, 102, 16, ui.panel_alt);
            display.text(
                212,
                131,
                if zh_mode {
                    self.weapon.label_zh()
                } else {
                    self.weapon.label_en()
                },
                self.weapon.accent(ui),
                ui.panel_alt,
                1,
            );
            self.prev_hud_weapon = Some(self.weapon);
        }

        display.fill_rect(MAP_BUTTON_X, MAP_BUTTON_Y, MAP_BUTTON_W, MAP_BUTTON_H, ui.panel);
        display.stroke_rect(MAP_BUTTON_X, MAP_BUTTON_Y, MAP_BUTTON_W, MAP_BUTTON_H, 1, ui.cyan);
        display.text(
            MAP_BUTTON_X + 28,
            MAP_BUTTON_Y + 5,
            if zh_mode { "選圖" } else { "MAPS" },
            ui.text_muted,
            ui.panel,
            1,
        );

        let exit_active = self.exit_hold_ms > 0;
        if force || self.prev_hud_exit_hold != exit_active {
            display.fill_rect(206, 144, 102, 18, ui.panel_alt);
            display.text(
                212,
                150,
                if exit_active {
                    if zh_mode {
                        "返回主頁中"
                    } else {
                        "EXITING"
                    }
                } else if zh_mode {
                    "長按返回"
                } else {
                    "HOLD EXIT"
                },
                ui.text_muted,
                ui.panel_alt,
                1,
            );
            self.prev_hud_exit_hold = exit_active;
        }
    }
}

fn hud_map_name(map_index: usize, zh_mode: bool) -> &'static str {
    match (map_index % DungeonApp::map_count(), zh_mode) {
        (0, true) => "遺跡",
        (1, true) => "熔爐",
        (2, true) => "墓穴",
        (0, false) => "RUINS",
        (1, false) => "FORGE",
        _ => "CRYPT",
    }
}

fn render_shot_fx(buffer: &mut [u16], dungeon: &DungeonApp, ui: &crate::display::Palette) {
    if dungeon.muzzle_flash_ms == 0 {
        return;
    }

    let flash_ms = dungeon.weapon.flash_ms().max(1) as u32;
    let intensity = ((dungeon.muzzle_flash_ms as u32 * 255) / flash_ms).min(255) as u8;
    let weapon_color = dungeon.weapon.accent(ui);
    let tracer_color = crate::display::color::mix(
        weapon_color,
        if dungeon.shot_hit_enemy { ui.rose } else { ui.white },
        intensity,
    );
    let center_x = VIEW_W / 2;
    let muzzle_y = VIEW_H_USIZE.saturating_sub(6);
    let impact_y = ((VIEW_H as f32 * 0.5) + (dungeon.shot_depth.clamp(0.6, 8.0) * 3.0)) as usize;
    let impact_y = impact_y.min(VIEW_H_USIZE.saturating_sub(12)).max(26);

    for y in impact_y..=muzzle_y {
        let row = y * VIEW_W;
        for dx in center_x.saturating_sub(1)..=(center_x + 1).min(VIEW_W - 1) {
            buffer[row + dx] = tracer_color;
        }
    }

    let spark = if dungeon.shot_hit_enemy {
        ui.rose
    } else {
        crate::display::color::mix(weapon_color, ui.white, 70)
    };
    let spark_y = impact_y;
    for offset in 0..5usize {
        let up = spark_y.saturating_sub(offset);
        let down = (spark_y + offset).min(VIEW_H_USIZE - 1);
        let left = center_x.saturating_sub(offset);
        let right = (center_x + offset).min(VIEW_W - 1);
        buffer[up * VIEW_W + center_x] = spark;
        buffer[down * VIEW_W + center_x] = spark;
        buffer[spark_y * VIEW_W + left] = spark;
        buffer[spark_y * VIEW_W + right] = spark;
    }
}

fn render_heal_fx(buffer: &mut [u16], dungeon: &DungeonApp, ui: &crate::display::Palette) {
    if dungeon.heal_flash_ms == 0 {
        return;
    }

    let pulse = dungeon.heal_flash_ms as f32 / 220.0;
    let center_x = VIEW_W / 2;
    let center_y = VIEW_H_USIZE / 2 + 12;
    let burst_radius = (18.0 + (1.0 - pulse) * 22.0) as i32;
    let ring_alpha = (pulse * 164.0).clamp(36.0, 164.0) as u8;
    let core_alpha = (pulse * 88.0).clamp(22.0, 88.0) as u8;
    let glow = crate::display::color::mix(ui.lime, ui.white, 124);

    buffer_blend_circle(buffer, center_x, center_y, burst_radius, glow, core_alpha);
    buffer_stroke_circle(
        buffer,
        center_x,
        center_y,
        burst_radius,
        2,
        crate::display::color::mix(ui.white, ui.lime, 96),
        ring_alpha,
    );

    let spoke = (burst_radius + 6).max(14);
    for &(dx, dy) in &[
        (1.0f32, 0.0f32),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.7, 0.7),
        (-0.7, 0.7),
        (0.7, -0.7),
        (-0.7, -0.7),
    ] {
        let end_x = center_x as f32 + dx * spoke as f32;
        let end_y = center_y as f32 + dy * spoke as f32;
        buffer_blend_line(
            buffer,
            center_x as i32,
            center_y as i32,
            round_to_i32(end_x),
            round_to_i32(end_y),
            glow,
            (ring_alpha / 2).max(48),
        );
    }

    let cross_color = crate::display::color::mix(ui.white, ui.lime, 110);
    buffer_blend_rect(
        buffer,
        center_x.saturating_sub(2),
        center_y.saturating_sub(10),
        4,
        20,
        cross_color,
        ring_alpha,
    );
    buffer_blend_rect(
        buffer,
        center_x.saturating_sub(10),
        center_y.saturating_sub(2),
        20,
        4,
        cross_color,
        ring_alpha,
    );
}

fn draw_shell(display: &mut Display, ui: &crate::display::Palette, zh_mode: bool) {
    display.fill_rect(0, 0, SCREEN_WIDTH, 240, ui.canvas);
    display.panel(10, 8, 300, 30, ui.panel, ui.cyan);
    display.text(
        22,
        16,
        if zh_mode { "地城核心" } else { "DUNGEON CORE" },
        ui.text,
        ui.panel,
        2,
    );
    display.text(
        176,
        16,
        if zh_mode {
            "戰術光線投射"
        } else {
            "TACTICAL RAYCAST"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    let map_w = MAP_W as u16 * CELL + 12;
    let map_h = MAP_H as u16 * CELL + 12;
    display.panel(10, 42, map_w, map_h, ui.panel, ui.cyan);
    display.panel(204, 42, 106, 24, ui.panel_alt, ui.orange);
}

fn draw_viewport(
    display: &mut Display,
    dungeon: &DungeonApp,
    touch: &TouchState,
    ui: &crate::display::Palette,
    render_strategy: RenderStrategy,
) {
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(VIEWPORT_BUFFER) as *mut u16,
            VIEW_PIXELS,
        )
    };
    let zbuffer = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(ZBUFFER) as *mut f32,
            RAY_COUNT as usize,
        )
    };

    let (dir_x, dir_y, plane_x, plane_y) = direction_and_plane(dungeon.angle);
    render_sky_floor(buffer, dungeon, ui, dir_x, dir_y, plane_x, plane_y, render_strategy);
    let wall_stride = render_strategy.wall_stride();

    let mut column = 0u16;
    while column < RAY_COUNT {
        let camera_x = 2.0 * ((column as f32 + 0.5) / RAY_COUNT as f32) - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;
        let hit = cast_ray(
            dungeon.current_map(),
            dungeon.player_x,
            dungeon.player_y,
            ray_dir_x,
            ray_dir_y,
        );
        let distance = hit.distance.max(0.16);
        let mut copy = 0usize;
        while copy < wall_stride && column as usize + copy < zbuffer.len() {
            zbuffer[column as usize + copy] = distance;
            copy += 1;
        }

        let line_height = (VIEW_H as f32 * 0.82 / distance) as i32;
        let wall_top =
            (VIEW_Y as i32 + ((VIEW_H as i32 - line_height) / 2)).max(VIEW_Y as i32) as u16;
        let wall_bottom = ((wall_top as i32 + line_height).min((VIEW_BOTTOM - 1) as i32)) as u16;

        let texture = assets::texture(texture_for_tile(hit.tile));
        let distance_factor = (255.0 - (distance * 28.0)).clamp(56.0, 255.0) as u8;
        let side_factor = if hit.side == 0 {
            distance_factor
        } else {
            distance_factor.saturating_sub(26)
        };
        let mut tex_x = (hit.wall_x * TEX_SIZE as f32) as usize;
        tex_x = tex_x.min(TEX_SIZE - 1);
        if (hit.side == 0 && hit.dir_x > 0.0) || (hit.side == 1 && hit.dir_y < 0.0) {
            tex_x = TEX_SIZE - 1 - tex_x;
        }

        let x = (column * COLUMN_WIDTH) as usize;
        let top_end = wall_top.saturating_sub(VIEW_Y) as usize;
        let wall_end = core::cmp::max(wall_bottom.saturating_sub(VIEW_Y) as usize, top_end + 1);

        for y in top_end..wall_end {
            let row = y * VIEW_W;
            let rel = y - top_end;
            let denom = (wall_end - top_end).max(1);
            let tex_y = (rel * TEX_SIZE / denom).min(TEX_SIZE - 1);
            let sample = assets::texture_sample(texture, tex_x, tex_y);
            let shaded = shade(sample, side_factor);
            let mut fill = 0usize;
            while fill < wall_stride && x + fill < VIEW_W {
                buffer[row + x + fill] = shaded;
                fill += 1;
            }
        }
        column += wall_stride as u16;
    }

    render_sprites(buffer, zbuffer, dungeon, ui, dir_x, dir_y, plane_x, plane_y);
    render_pickups(buffer, zbuffer, dungeon, ui, dir_x, dir_y, plane_x, plane_y);
    render_shot_fx(buffer, dungeon, ui);
    render_heal_fx(buffer, dungeon, ui);
    render_weapon(buffer, dungeon, ui);
    render_health_bar(buffer, dungeon, ui);
    render_minimap(buffer, dungeon, ui);
    render_touch_controls(buffer, touch, dungeon.touch_mode, ui);
    render_crosshair(buffer, ui, dungeon.muzzle_flash_ms > 0);
    display.draw_rgb565(0, VIEW_Y, SCREEN_WIDTH, VIEW_H, buffer);
}

fn render_sprites(
    buffer: &mut [u16],
    zbuffer: &[f32],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
    dir_x: f32,
    dir_y: f32,
    plane_x: f32,
    plane_y: f32,
) {
    let mut order = [usize::MAX; MAX_ENEMIES];
    let mut count = 0usize;
    for (index, enemy) in dungeon.enemies.iter().enumerate() {
        if enemy.alive || enemy.death_anim_ms > 0 {
            order[count] = index;
            count += 1;
        }
    }

    for i in 0..count {
        let mut best = i;
        let mut best_dist = distance_sq(
            dungeon.enemies[order[i]].x - dungeon.player_x,
            dungeon.enemies[order[i]].y - dungeon.player_y,
        );
        for j in (i + 1)..count {
            let dist = distance_sq(
                dungeon.enemies[order[j]].x - dungeon.player_x,
                dungeon.enemies[order[j]].y - dungeon.player_y,
            );
            if dist > best_dist {
                best = j;
                best_dist = dist;
            }
        }
        if best != i {
            order.swap(i, best);
        }
    }

    for index in order.into_iter().take(count) {
        let enemy = dungeon.enemies[index];
        let sprite = assets::enemy_sprite(enemy.kind);

        let sprite_x = enemy.x - dungeon.player_x;
        let sprite_y = enemy.y - dungeon.player_y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
        let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y);

        if transform_y <= 0.1 {
            continue;
        }

        let sprite_screen_x =
            ((VIEW_W as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
        let death_t = if enemy.alive {
            1.0
        } else {
            (enemy.death_anim_ms as f32 / 280.0).clamp(0.0, 1.0)
        };
        let bob = if enemy.alive {
            (sinf(enemy.phase as f32 * 0.01) * 3.0) as i32
        } else {
            0
        };
        let sprite_height =
            ((VIEW_H as f32 / transform_y).clamp(12.0, 110.0) * death_t.max(0.35)) as i32;
        let sprite_width = (sprite_height as f32 * sprite.width as f32 / sprite.height as f32)
            .clamp(10.0, 110.0) as i32;

        let sink = ((1.0 - death_t) * 18.0) as i32;
        let floor_anchor = (VIEW_H as f32 * 0.82) as i32 + bob + sink;
        let draw_end_y = floor_anchor.clamp(16, VIEW_H as i32 - 1);
        let draw_start_y = (draw_end_y - sprite_height).max(0);
        let draw_start_x = (sprite_screen_x - sprite_width / 2).max(0);
        let draw_end_x = (sprite_screen_x + sprite_width / 2).min(VIEW_W as i32);

        for stripe in draw_start_x..draw_end_x {
            let ray_column = (stripe as usize / COLUMN_WIDTH as usize).min(zbuffer.len() - 1);
            if transform_y >= zbuffer[ray_column].max(0.001) {
                continue;
            }

            let tex_x =
                (((stripe - (sprite_screen_x - sprite_width / 2)) * sprite.width as i32) / sprite_width)
                    as usize;
            if tex_x >= sprite.width {
                continue;
            }

            for y in draw_start_y..draw_end_y {
                let tex_y = (((y - draw_start_y) * sprite.height as i32) / sprite_height) as usize;
                if tex_y >= sprite.height {
                    continue;
                }

                if let Some(pixel) = assets::sprite_sample(&sprite, tex_x, tex_y) {
                    let shade_factor = (255.0 - transform_y * 24.0).clamp(92.0, 255.0) as u8;
                    let flash_factor = if enemy.hit_flash_ms > 0 {
                        crate::display::color::mix(pixel, ui.white, 160)
                    } else if !enemy.alive {
                        crate::display::color::mix(pixel, ui.rose, 84)
                    } else if dungeon.muzzle_flash_ms > 0 {
                        crate::display::color::mix(pixel, ui.white, 36)
                    } else {
                        pixel
                    };
                    let viewport_y = y as usize;
                    let idx = viewport_y * VIEW_W + stripe as usize;
                    if idx < buffer.len() {
                        buffer[idx] = shade(flash_factor, shade_factor);
                    }
                }
            }
        }
    }
}

fn render_pickups(
    buffer: &mut [u16],
    zbuffer: &[f32],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
    dir_x: f32,
    dir_y: f32,
    plane_x: f32,
    plane_y: f32,
) {
    for pickup in dungeon.pickups.iter().filter(|pickup| pickup.active) {
        let sprite_x = pickup.x - dungeon.player_x;
        let sprite_y = pickup.y - dungeon.player_y;

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
        let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y);

        if transform_y <= 0.18 {
            continue;
        }

        let screen_x = ((VIEW_W as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
        let bob = (sinf(pickup.phase as f32 * 0.012) * 2.0) as i32;
        let size = (VIEW_H as f32 / transform_y).clamp(10.0, 26.0) as i32;
        let draw_end_y = ((VIEW_H as f32 * 0.82) as i32 + bob).clamp(14, VIEW_H as i32 - 1);
        let draw_start_y = (draw_end_y - size).max(0);
        let draw_start_x = (screen_x - size / 2).max(0);
        let draw_end_x = (screen_x + size / 2).min(VIEW_W as i32);

        for stripe in draw_start_x..draw_end_x {
            let ray_column = stripe as usize;
            if ray_column >= zbuffer.len() || transform_y >= zbuffer[ray_column] {
                continue;
            }

            for y in draw_start_y..draw_end_y {
                let local_x = stripe - draw_start_x;
                let local_y = y - draw_start_y;
                let idx = y as usize * VIEW_W + stripe as usize;
                if idx >= buffer.len() {
                    continue;
                }

                let border = local_x < 1 || local_y < 1 || local_x >= size - 1 || local_y >= size - 1;
                let cross_h = local_y >= (size / 2 - 1) && local_y <= (size / 2 + 1);
                let cross_v = local_x >= (size / 2 - 1) && local_x <= (size / 2 + 1);

                if border {
                    buffer[idx] = crate::display::color::mix(buffer[idx], ui.white, 205);
                } else if cross_h || cross_v {
                    buffer[idx] = crate::display::color::mix(buffer[idx], ui.rose, 230);
                } else {
                    buffer[idx] = crate::display::color::mix(buffer[idx], ui.panel_alt, 155);
                }
            }
        }
    }
}

fn render_health_bar(buffer: &mut [u16], dungeon: &DungeonApp, ui: &crate::display::Palette) {
    let panel_x = 96usize;
    let panel_y = 6usize;
    let panel_w = 126usize;
    let panel_h = 20usize;
    let fill_w = ((dungeon.health.max(0) as usize * 110) / PLAYER_MAX_HP as usize).min(110);
    let bar_fill = if dungeon.heal_flash_ms > 0 {
        crate::display::color::mix(ui.lime, ui.white, 120)
    } else if dungeon.health < 28 {
        ui.rose
    } else {
        ui.lime
    };

    buffer_blend_rect(buffer, panel_x + 2, panel_y + 2, panel_w, panel_h, ui.shadow, 112);
    buffer_blend_rect(buffer, panel_x, panel_y, panel_w, panel_h, ui.panel, 210);
    buffer_stroke_rect(buffer, panel_x, panel_y, panel_w, panel_h, 1, ui.cyan, 220);
    buffer_blend_rect(buffer, panel_x + 10, panel_y + 7, 110, 7, ui.shadow, 255);
    if fill_w > 0 {
        buffer_blend_rect(buffer, panel_x + 10, panel_y + 7, fill_w, 7, bar_fill, 255);
    }
    buffer_blend_rect(
        buffer,
        panel_x + 10 + fill_w.min(108),
        panel_y + 7,
        2,
        7,
        crate::display::color::mix(ui.white, ui.cyan, 160),
        220,
    );
}

fn render_sky_floor(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
    dir_x: f32,
    dir_y: f32,
    plane_x: f32,
    plane_y: f32,
    render_strategy: RenderStrategy,
) {
    let floor_stride = render_strategy.floor_stride();
    let horizon = VIEW_H as f32 * 0.5;
    let left_ray_x = dir_x - plane_x;
    let left_ray_y = dir_y - plane_y;
    let right_ray_x = dir_x + plane_x;
    let right_ray_y = dir_y + plane_y;
    let sky_top = crate::display::color::mix(ui.canvas, ui.sky, 96);
    let sky_horizon = crate::display::color::mix(ui.sky, ui.white, 18);
    let ceiling_texture = assets::texture(ceiling_texture_for_map(dungeon.map_index));
    let floor_texture = assets::texture(floor_texture_for_map(dungeon.map_index));

    for y in 0..VIEW_H_USIZE {
        let row = y * VIEW_W;
        if (y as f32) < horizon {
            let perspective = (horizon - y as f32).max(1.0);
            let row_distance = (0.5 * VIEW_H as f32) / perspective;
            let step_x = row_distance * (right_ray_x - left_ray_x) / VIEW_W as f32;
            let step_y = row_distance * (right_ray_y - left_ray_y) / VIEW_W as f32;
            let mut ceil_x = dungeon.player_x + row_distance * left_ray_x;
            let mut ceil_y = dungeon.player_y + row_distance * left_ray_y;
            let t = ((y as f32 / horizon) * 255.0).clamp(0.0, 255.0) as u8;
            let sky = crate::display::color::mix(sky_top, sky_horizon, t);

            let mut x = 0usize;
            while x < VIEW_W {
                let cell_x = floorf(ceil_x) as i32;
                let cell_y = floorf(ceil_y) as i32;
                let frac_x = ceil_x - cell_x as f32;
                let frac_y = ceil_y - cell_y as f32;
                let tex_x = ((frac_x * TEX_SIZE as f32) as usize).min(TEX_SIZE - 1);
                let tex_y = ((frac_y * TEX_SIZE as f32) as usize).min(TEX_SIZE - 1);
                let sample = assets::texture_sample(ceiling_texture, tex_x, tex_y);
                let mixed =
                    crate::display::color::mix(sample, sky, render_strategy.ceiling_mix_alpha());
                let stripe = if ((cell_x + cell_y) & 1) == 0 { 214u8 } else { 184u8 };
                let fade = (255.0 - row_distance * 15.0).clamp(72.0, stripe as f32) as u8;
                let pixel = shade(mixed, fade);
                buffer[row + x] = pixel;
                let mut copy = 1usize;
                while copy < floor_stride && x + copy < VIEW_W {
                    buffer[row + x + copy] = pixel;
                    copy += 1;
                }
                ceil_x += step_x * floor_stride as f32;
                ceil_y += step_y * floor_stride as f32;
                x += floor_stride;
            }
            continue;
        }

        let perspective = (y as f32 - horizon).max(1.0);
        let row_distance = (0.5 * VIEW_H as f32) / perspective;
        let step_x = row_distance * (right_ray_x - left_ray_x) / VIEW_W as f32;
        let step_y = row_distance * (right_ray_y - left_ray_y) / VIEW_W as f32;
        let mut floor_x = dungeon.player_x + row_distance * left_ray_x;
        let mut floor_y = dungeon.player_y + row_distance * left_ray_y;

        let mut x = 0usize;
        while x < VIEW_W {
            let cell_x = floorf(floor_x) as i32;
            let cell_y = floorf(floor_y) as i32;
            let frac_x = floor_x - cell_x as f32;
            let frac_y = floor_y - cell_y as f32;
            let tex_x = ((frac_x * TEX_SIZE as f32) as usize).min(TEX_SIZE - 1);
            let tex_y = ((frac_y * TEX_SIZE as f32) as usize).min(TEX_SIZE - 1);
            let sample = crate::display::color::mix(
                assets::texture_sample(floor_texture, tex_x, tex_y),
                ui.floor,
                render_strategy.floor_mix_alpha(),
            );
            let checker = if ((cell_x + cell_y) & 1) == 0 { 214u8 } else { 170u8 };
            let fade = (255.0 - row_distance * 18.0).clamp(48.0, checker as f32) as u8;
            let pixel = shade(sample, fade);
            buffer[row + x] = pixel;
            let mut copy = 1usize;
            while copy < floor_stride && x + copy < VIEW_W {
                buffer[row + x + copy] = pixel;
                copy += 1;
            }
            floor_x += step_x * floor_stride as f32;
            floor_y += step_y * floor_stride as f32;
            x += floor_stride;
        }
    }
}

fn render_touch_controls(
    buffer: &mut [u16],
    touch: &TouchState,
    mode: TouchMode,
    ui: &crate::display::Palette,
) {
    let control_fill = crate::display::color::mix(ui.panel_alt, ui.cyan, 54);
    let control_core = crate::display::color::mix(ui.canvas, ui.panel, 112);
    let fire_fill = crate::display::color::mix(ui.panel_alt, ui.amber, 58);
    let fire_core = crate::display::color::mix(ui.canvas, ui.panel, 112);
    let control_cx = CONTROL_CENTER_X as usize;
    let control_cy = CONTROL_CENTER_Y.saturating_sub(VIEW_Y) as usize;
    let fire_cx = FIRE_CENTER_X as usize;
    let fire_cy = FIRE_CENTER_Y.saturating_sub(VIEW_Y) as usize;

    buffer_blend_circle(
        buffer,
        control_cx + 2,
        control_cy + 2,
        CONTROL_RING_RADIUS as i32,
        crate::display::color::mix(ui.shadow, ui.indigo, 40),
        112,
    );
    buffer_blend_circle(
        buffer,
        control_cx,
        control_cy,
        CONTROL_RING_RADIUS as i32,
        control_fill,
        120,
    );
    buffer_blend_circle(
        buffer,
        control_cx,
        control_cy,
        CONTROL_BASE_RADIUS as i32,
        control_core,
        130,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        CONTROL_RING_RADIUS as i32,
        2,
        ui.cyan,
        220,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        CONTROL_BASE_RADIUS as i32,
        1,
        crate::display::color::mix(ui.steel, ui.white, 62),
        200,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        CONTROL_INPUT_RADIUS as i32,
        1,
        crate::display::color::mix(ui.steel, ui.cyan, 96),
        180,
    );
    buffer_blend_rect(buffer, control_cx.saturating_sub(1), control_cy.saturating_sub(12), 2, 24, ui.steel, 150);
    buffer_blend_rect(buffer, control_cx.saturating_sub(12), control_cy.saturating_sub(1), 24, 2, ui.steel, 150);

    buffer_blend_circle(
        buffer,
        fire_cx + 2,
        fire_cy + 2,
        FIRE_RING_RADIUS as i32,
        crate::display::color::mix(ui.shadow, ui.orange, 30),
        104,
    );
    buffer_blend_circle(buffer, fire_cx, fire_cy, FIRE_RING_RADIUS as i32, fire_fill, 122);
    buffer_blend_circle(buffer, fire_cx, fire_cy, FIRE_BASE_RADIUS as i32, fire_core, 132);
    buffer_stroke_circle(buffer, fire_cx, fire_cy, FIRE_RING_RADIUS as i32, 2, ui.amber, 220);
    buffer_stroke_circle(
        buffer,
        fire_cx,
        fire_cy,
        FIRE_BASE_RADIUS as i32,
        1,
        crate::display::color::mix(ui.steel, ui.white, 72),
        200,
    );

    let (control_dx, control_dy) = if touch.active && mode == TouchMode::Control {
        clamp_circle_delta(
            touch.x as f32 - CONTROL_CENTER_X as f32,
            touch.y as f32 - CONTROL_CENTER_Y as f32,
            CONTROL_INPUT_RADIUS,
        )
    } else {
        (0.0, 0.0)
    };
    let control_knob_x = round_to_usize(CONTROL_CENTER_X as f32 + control_dx);
    let control_knob_y = round_to_usize(CONTROL_CENTER_Y as f32 + control_dy - VIEW_Y as f32);
    buffer_blend_circle(
        buffer,
        control_knob_x + 1,
        control_knob_y + 1,
        (CONTROL_KNOB_RADIUS + 1) as i32,
        crate::display::color::mix(ui.shadow, ui.cyan, 56),
        145,
    );
    buffer_blend_circle(
        buffer,
        control_knob_x,
        control_knob_y,
        CONTROL_KNOB_RADIUS as i32,
        crate::display::color::mix(ui.panel_alt, ui.cyan, 142),
        220,
    );
    buffer_stroke_circle(
        buffer,
        control_knob_x,
        control_knob_y,
        CONTROL_KNOB_RADIUS as i32,
        2,
        ui.white,
        220,
    );

    let (fire_dx, fire_dy) = if touch.active && mode == TouchMode::Fire {
        clamp_circle_delta(
            touch.x as f32 - FIRE_CENTER_X as f32,
            touch.y as f32 - FIRE_CENTER_Y as f32,
            FIRE_INPUT_RADIUS,
        )
    } else {
        (0.0, 0.0)
    };
    let fire_hot = touch.active && mode == TouchMode::Fire;
    let fire_knob_x = round_to_usize(FIRE_CENTER_X as f32 + fire_dx);
    let fire_knob_y = round_to_usize(FIRE_CENTER_Y as f32 + fire_dy - VIEW_Y as f32);
    buffer_blend_circle(
        buffer,
        fire_knob_x + 1,
        fire_knob_y + 1,
        (FIRE_KNOB_RADIUS + 1) as i32,
        crate::display::color::mix(ui.shadow, ui.orange, 64),
        152,
    );
    buffer_blend_circle(
        buffer,
        fire_knob_x,
        fire_knob_y,
        FIRE_KNOB_RADIUS as i32,
        if fire_hot {
            crate::display::color::mix(ui.orange, ui.white, 140)
        } else {
            crate::display::color::mix(ui.panel_alt, ui.amber, 138)
        },
        if fire_hot { 236 } else { 214 },
    );
    buffer_stroke_circle(
        buffer,
        fire_knob_x,
        fire_knob_y,
        FIRE_KNOB_RADIUS as i32,
        2,
        ui.white,
        220,
    );

    let glyph = if fire_hot {
        crate::display::color::mix(ui.orange, ui.white, 180)
    } else {
        crate::display::color::mix(ui.amber, ui.white, 90)
    };
    buffer_blend_rect(buffer, fire_cx.saturating_sub(1), fire_cy.saturating_sub(8), 2, 16, glyph, 220);
    buffer_blend_rect(buffer, fire_cx.saturating_sub(8), fire_cy.saturating_sub(1), 16, 2, glyph, 220);
}

fn render_weapon(buffer: &mut [u16], dungeon: &DungeonApp, ui: &crate::display::Palette) {
    let recoil = if dungeon.muzzle_flash_ms > 0 {
        let flash_ms = dungeon.weapon.flash_ms().max(1) as u32;
        (((dungeon.muzzle_flash_ms as u32 * 8) / flash_ms).min(8)) as usize
    } else {
        0
    };
    let base_y = VIEW_H_USIZE.saturating_sub(26).saturating_add(recoil);
    let center_x = VIEW_W / 2;

    let accent = dungeon.weapon.accent(ui);
    buffer_blend_circle(
        buffer,
        center_x,
        base_y + 14,
        18,
        crate::display::color::mix(ui.panel, accent, 54),
        118,
    );
    buffer_stroke_circle(
        buffer,
        center_x,
        base_y + 14,
        18,
        1,
        crate::display::color::mix(accent, ui.white, 40),
        170,
    );

    match dungeon.weapon {
        WeaponKind::Pulse => {
            buffer_fill_rect(buffer, center_x.saturating_sub(24), base_y + 10, 48, 12, ui.shadow);
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(18),
                base_y + 12,
                36,
                9,
                crate::display::color::mix(ui.panel_alt, ui.steel, 110),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(8),
                base_y + 5,
                16,
                8,
                crate::display::color::mix(ui.panel_alt, ui.white, 66),
            );
            buffer_fill_rect(buffer, center_x.saturating_sub(3), base_y + 1, 6, 6, ui.white);
        }
        WeaponKind::Carbine => {
            buffer_fill_rect(buffer, center_x.saturating_sub(30), base_y + 11, 60, 10, ui.shadow);
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(26),
                base_y + 12,
                52,
                7,
                crate::display::color::mix(ui.panel_alt, ui.steel, 122),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(6),
                base_y + 6,
                20,
                6,
                crate::display::color::mix(ui.panel_alt, ui.white, 60),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(18),
                base_y + 16,
                10,
                5,
                crate::display::color::mix(ui.panel, ui.lime, 64),
            );
        }
        WeaponKind::Scatter => {
            buffer_fill_rect(buffer, center_x.saturating_sub(28), base_y + 10, 56, 14, ui.shadow);
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(16),
                base_y + 12,
                32,
                11,
                crate::display::color::mix(ui.panel_alt, ui.steel, 108),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(14),
                base_y + 4,
                28,
                10,
                crate::display::color::mix(ui.panel_alt, ui.white, 52),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(4),
                base_y + 1,
                8,
                5,
                ui.white,
            );
        }
    }

    for slot in 0..3usize {
        let slot_x = center_x.saturating_sub(12) + slot * 12;
        let active = slot
            == match dungeon.weapon {
                WeaponKind::Pulse => 0,
                WeaponKind::Carbine => 1,
                WeaponKind::Scatter => 2,
            };
        buffer_blend_circle(
            buffer,
            slot_x,
            base_y + 30,
            4,
            if active {
                accent
            } else {
                crate::display::color::mix(ui.panel_alt, ui.steel, 100)
            },
            if active { 220 } else { 160 },
        );
    }

    if dungeon.muzzle_flash_ms > 0 {
        let flash = crate::display::color::mix(accent, ui.white, 178);
        let tip_y = base_y.saturating_sub(2);
        buffer_fill_rect(buffer, center_x.saturating_sub(2), tip_y, 4, 4, flash);
        buffer_fill_rect(buffer, center_x.saturating_sub(6), tip_y + 2, 12, 3, flash);
        buffer_fill_rect(buffer, center_x.saturating_sub(10), tip_y + 4, 20, 2, accent);
    }
}

fn buffer_fill_rect(buffer: &mut [u16], x: usize, y: usize, width: usize, height: usize, color: u16) {
    let x_end = (x + width).min(VIEW_W);
    let y_end = (y + height).min(VIEW_H_USIZE);
    for py in y..y_end {
        let row = py * VIEW_W;
        for px in x..x_end {
            buffer[row + px] = color;
        }
    }
}

fn buffer_blend_rect(
    buffer: &mut [u16],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u16,
    alpha: u8,
) {
    let x_end = (x + width).min(VIEW_W);
    let y_end = (y + height).min(VIEW_H_USIZE);
    for py in y..y_end {
        let row = py * VIEW_W;
        for px in x..x_end {
            let idx = row + px;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

fn buffer_stroke_rect(
    buffer: &mut [u16],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    thickness: usize,
    color: u16,
    alpha: u8,
) {
    buffer_blend_rect(buffer, x, y, width, thickness, color, alpha);
    buffer_blend_rect(
        buffer,
        x,
        y.saturating_add(height.saturating_sub(thickness)),
        width,
        thickness,
        color,
        alpha,
    );
    buffer_blend_rect(buffer, x, y, thickness, height, color, alpha);
    buffer_blend_rect(
        buffer,
        x.saturating_add(width.saturating_sub(thickness)),
        y,
        thickness,
        height,
        color,
        alpha,
    );
}

fn buffer_blend_circle(
    buffer: &mut [u16],
    center_x: usize,
    center_y: usize,
    radius: i32,
    color: u16,
    alpha: u8,
) {
    if radius <= 0 {
        return;
    }

    let cx = center_x as i32;
    let cy = center_y as i32;

    for dy in -radius..=radius {
        let y = cy + dy;
        if !(0..VIEW_H as i32).contains(&y) {
            continue;
        }

        let dx = buffer_circle_dx(radius, dy);
        let start_x = (cx - dx).max(0);
        let end_x = (cx + dx).min(VIEW_W as i32 - 1);
        let row = y as usize * VIEW_W;
        for px in start_x..=end_x {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

fn buffer_stroke_circle(
    buffer: &mut [u16],
    center_x: usize,
    center_y: usize,
    radius: i32,
    thickness: i32,
    color: u16,
    alpha: u8,
) {
    let outer = radius.max(0);
    let inner = (outer - thickness).max(0);
    let cx = center_x as i32;
    let cy = center_y as i32;

    for dy in -outer..=outer {
        let y = cy + dy;
        if !(0..VIEW_H as i32).contains(&y) {
            continue;
        }

        let outer_dx = buffer_circle_dx(outer, dy);
        let inner_dx = if dy.abs() <= inner {
            buffer_circle_dx(inner, dy)
        } else {
            -1
        };
        let row = y as usize * VIEW_W;
        let left_outer = (cx - outer_dx).max(0);
        let right_outer = (cx + outer_dx).min(VIEW_W as i32 - 1);

        if inner_dx < 0 {
            for px in left_outer..=right_outer {
                let idx = row + px as usize;
                buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
            }
            continue;
        }

        let left_inner = (cx - inner_dx).max(0);
        let right_inner = (cx + inner_dx).min(VIEW_W as i32 - 1);

        for px in left_outer..left_inner {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
        for px in (right_inner + 1)..=right_outer {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

fn buffer_blend_line(
    buffer: &mut [u16],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u16,
    alpha: u8,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if (0..VIEW_W as i32).contains(&x0) && (0..VIEW_H as i32).contains(&y0) {
            let idx = y0 as usize * VIEW_W + x0 as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }

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

fn buffer_circle_dx(radius: i32, dy: i32) -> i32 {
    let rr = radius * radius;
    let yy = dy * dy;
    let mut dx = radius;
    while dx > 0 && (dx * dx + yy) > rr {
        dx -= 1;
    }
    dx
}

fn render_minimap(buffer: &mut [u16], dungeon: &DungeonApp, ui: &crate::display::Palette) {
    let map = dungeon.current_map();
    let panel_x = 10usize;
    let panel_y = 2usize;
    let panel_w = MAP_W * CELL as usize + 12;
    let panel_h = MAP_H * CELL as usize + 12;

    buffer_blend_rect(buffer, panel_x + 3, panel_y + 4, panel_w, panel_h, ui.shadow, 96);
    buffer_blend_rect(buffer, panel_x, panel_y, panel_w, panel_h, ui.panel, 220);
    buffer_stroke_rect(buffer, panel_x, panel_y, panel_w, panel_h, 2, ui.cyan, 235);
    buffer_stroke_rect(
        buffer,
        panel_x + 2,
        panel_y + 2,
        panel_w.saturating_sub(4),
        panel_h.saturating_sub(4),
        1,
        ui.white,
        210,
    );

    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let tile_color = tile_fill(map.layout[row][col], ui);
            buffer_blend_rect(
                buffer,
                MAP_ORIGIN_X as usize + col * CELL as usize,
                (MAP_ORIGIN_Y - VIEW_Y) as usize + row * CELL as usize,
                (CELL - 1) as usize,
                (CELL - 1) as usize,
                tile_color,
                255,
            );
        }
    }

    let player_px = MAP_ORIGIN_X as f32 + dungeon.player_x * CELL as f32;
    let player_py = (MAP_ORIGIN_Y - VIEW_Y) as f32 + dungeon.player_y * CELL as f32;
    let (dir_x, dir_y, _, _) = direction_and_plane(dungeon.angle);
    let tip_x = player_px + dir_x * 5.0;
    let tip_y = player_py + dir_y * 5.0;
    let left_x = player_px + cosf(dungeon.angle + 2.5) * 2.0;
    let left_y = player_py + sinf(dungeon.angle + 2.5) * 2.0;
    let right_x = player_px + cosf(dungeon.angle - 2.5) * 2.0;
    let right_y = player_py + sinf(dungeon.angle - 2.5) * 2.0;
    let fov_left_x = player_px + cosf(dungeon.angle - 0.42) * 4.0;
    let fov_left_y = player_py + sinf(dungeon.angle - 0.42) * 4.0;
    let fov_right_x = player_px + cosf(dungeon.angle + 0.42) * 4.0;
    let fov_right_y = player_py + sinf(dungeon.angle + 0.42) * 4.0;

    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(fov_left_x),
        round_to_i32(fov_left_y),
        crate::display::color::mix(ui.cyan, ui.white, 48),
        124,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(fov_right_x),
        round_to_i32(fov_right_y),
        crate::display::color::mix(ui.cyan, ui.white, 48),
        124,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        ui.lime,
        255,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        round_to_i32(left_x),
        round_to_i32(left_y),
        ui.white,
        220,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        round_to_i32(right_x),
        round_to_i32(right_y),
        ui.white,
        220,
    );
    buffer_blend_circle(
        buffer,
        round_to_usize(player_px),
        round_to_usize(player_py),
        1,
        ui.lime,
        255,
    );

    for enemy in dungeon.enemies.iter().filter(|enemy| enemy.alive) {
        let ex = MAP_ORIGIN_X as f32 + enemy.x * CELL as f32;
        let ey = (MAP_ORIGIN_Y - VIEW_Y) as f32 + enemy.y * CELL as f32;
        buffer_blend_circle(
            buffer,
            round_to_usize(ex),
            round_to_usize(ey),
            1,
            ui.rose,
            255,
        );
    }

    for pickup in dungeon.pickups.iter().filter(|pickup| pickup.active) {
        let px = MAP_ORIGIN_X as f32 + pickup.x * CELL as f32;
        let py = (MAP_ORIGIN_Y - VIEW_Y) as f32 + pickup.y * CELL as f32;
        let px = round_to_usize(px);
        let py = round_to_usize(py);
        buffer_blend_rect(buffer, px.saturating_sub(1), py.saturating_sub(2), 3, 5, ui.white, 255);
        buffer_blend_rect(buffer, px.saturating_sub(2), py.saturating_sub(1), 5, 3, ui.rose, 255);
    }
}

fn render_crosshair(buffer: &mut [u16], ui: &crate::display::Palette, hot: bool) {
    let color = if hot { ui.amber } else { ui.white };
    buffer_fill_rect(buffer, 159, 66, 2, 10, color);
    buffer_fill_rect(buffer, 155, 70, 10, 2, color);
}

fn draw_overlay(
    display: &mut Display,
    ui: &crate::display::Palette,
    title: &str,
    subtitle: &str,
    retry_label: &str,
    map_label: &str,
    accent: u16,
) {
    let glow = crate::display::color::mix(accent, ui.white, 84);
    const PANEL_Y: u16 = 80;
    display.fill_rect(44, PANEL_Y + 8, 232, 88, crate::display::color::mix(ui.shadow, accent, 26));
    display.panel(52, PANEL_Y, 216, 88, ui.panel_alt, accent);
    display.stroke_rect(56, PANEL_Y + 4, 208, 80, 1, glow);
    display.centered_text(160, PANEL_Y + 14, title, ui.text, ui.panel_alt, 2);
    display.centered_text(160, PANEL_Y + 38, subtitle, ui.text_muted, ui.panel_alt, 1);

    display.panel(OVERLAY_RETRY_X, OVERLAY_RETRY_Y, OVERLAY_RETRY_W, OVERLAY_RETRY_H, ui.panel, accent);
    display.centered_text(
        OVERLAY_RETRY_X + OVERLAY_RETRY_W / 2,
        OVERLAY_RETRY_Y + 6,
        retry_label,
        ui.text,
        ui.panel,
        1,
    );
    display.panel(OVERLAY_MAPS_X, OVERLAY_MAPS_Y, OVERLAY_MAPS_W, OVERLAY_MAPS_H, ui.panel, ui.cyan);
    display.centered_text(
        OVERLAY_MAPS_X + OVERLAY_MAPS_W / 2,
        OVERLAY_MAPS_Y + 6,
        map_label,
        ui.text,
        ui.panel,
        1,
    );
}

fn draw_intro_overlay(
    display: &mut Display,
    ui: &crate::display::Palette,
    map: &MapDef,
    zh_mode: bool,
) {
    let accent = match map.spawn_angle > 0.0 {
        true => ui.cyan,
        false => ui.orange,
    };
    let band = crate::display::color::mix(ui.panel_alt, accent, 78);
    display.fill_rect(34, 66, 252, 80, crate::display::color::mix(ui.shadow, accent, 28));
    display.panel(42, 58, 236, 74, ui.panel_alt, accent);
    display.fill_rect(54, 96, 212, 8, band);
    display.centered_text(
        160,
        72,
        if zh_mode { "任務部署中" } else { "MISSION DEPLOY" },
        ui.text,
        ui.panel_alt,
        2,
    );
    display.centered_text(
        160,
        98,
        if zh_mode { map.name_zh } else { map.name_en },
        ui.white,
        ui.panel_alt,
        2,
    );
    display.centered_text(
        160,
        120,
        if zh_mode {
            "點擊或按 K1 可略過"
        } else {
            "TAP OR PRESS K1 TO SKIP"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.fill_rect(58, 92, 198, 2, crate::display::color::mix(band, ui.white, 64));
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

fn texture_for_tile(tile: u8) -> TextureId {
    match tile {
        1 => TextureId::WallLight,
        2 => TextureId::WallMid,
        3 => TextureId::WallDark,
        4 => TextureId::DoorDark,
        5 => TextureId::WindowDark,
        _ => TextureId::WallMid,
    }
}

fn floor_texture_for_map(map_index: usize) -> TextureId {
    match map_index % MAPS.len() {
        0 => TextureId::WallMid,
        1 => TextureId::DoorDark,
        _ => TextureId::WallDark,
    }
}

fn ceiling_texture_for_map(map_index: usize) -> TextureId {
    match map_index % MAPS.len() {
        0 => TextureId::WallLight,
        1 => TextureId::WindowDark,
        _ => TextureId::DoorDark,
    }
}

fn tile_fill(tile: u8, ui: &crate::display::Palette) -> u16 {
    match tile {
        0 => crate::display::color::mix(ui.panel, ui.panel_alt, 70),
        1 => crate::display::color::mix(ui.cyan, ui.sky, 88),
        2 => crate::display::color::mix(ui.orange, ui.amber, 92),
        3 => crate::display::color::mix(ui.rose, ui.indigo, 110),
        4 => crate::display::color::mix(ui.amber, ui.orange, 140),
        _ => crate::display::color::mix(ui.rose, ui.sky, 150),
    }
}

fn try_move(player_x: &mut f32, player_y: &mut f32, delta_x: f32, delta_y: f32, map: &MapDef) {
    let next_x = *player_x + delta_x;
    let next_y = *player_y + delta_y;
    if !is_blocked_circle(map, next_x, *player_y, PLAYER_RADIUS) {
        *player_x = next_x;
    }
    if !is_blocked_circle(map, *player_x, next_y, PLAYER_RADIUS) {
        *player_y = next_y;
    }
}

fn try_enemy_move(enemy_x: &mut f32, enemy_y: &mut f32, delta_x: f32, delta_y: f32, map: &MapDef) {
    let next_x = *enemy_x + delta_x;
    let next_y = *enemy_y + delta_y;
    if !is_blocked_circle(map, next_x, *enemy_y, ENEMY_RADIUS) {
        *enemy_x = next_x;
    }
    if !is_blocked_circle(map, *enemy_x, next_y, ENEMY_RADIUS) {
        *enemy_y = next_y;
    }
}

struct RayHit {
    distance: f32,
    tile: u8,
    side: u8,
    wall_x: f32,
    dir_x: f32,
    dir_y: f32,
}

fn cast_ray(map: &MapDef, px: f32, py: f32, dir_x: f32, dir_y: f32) -> RayHit {
    let mut map_x = floorf(px) as i32;
    let mut map_y = floorf(py) as i32;

    let delta_dist_x = if dir_x == 0.0 { 1.0e30 } else { fabsf(1.0 / dir_x) };
    let delta_dist_y = if dir_y == 0.0 { 1.0e30 } else { fabsf(1.0 / dir_y) };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (px - map_x as f32) * delta_dist_x)
    } else {
        (1, ((map_x + 1) as f32 - px) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (py - map_y as f32) * delta_dist_y)
    } else {
        (1, ((map_y + 1) as f32 - py) * delta_dist_y)
    };

    let mut tile = 1;
    let mut side = 0u8;
    for _ in 0..32 {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        if map_x < 0 || map_y < 0 || map_x as usize >= MAP_W || map_y as usize >= MAP_H {
            break;
        }
        tile = map.layout[map_y as usize][map_x as usize];
        if tile != 0 {
            break;
        }
    }

    let distance = if side == 0 {
        (side_dist_x - delta_dist_x).max(0.001)
    } else {
        (side_dist_y - delta_dist_y).max(0.001)
    };

    let wall_x = if side == 0 {
        py + distance * dir_y
    } else {
        px + distance * dir_x
    };

    RayHit {
        distance,
        tile,
        side,
        wall_x: wall_x - floorf(wall_x),
        dir_x,
        dir_y,
    }
}

fn direction_and_plane(angle: f32) -> (f32, f32, f32, f32) {
    let dir_x = cosf(angle);
    let dir_y = sinf(angle);
    let plane_x = -dir_y * CAMERA_PLANE_SCALE;
    let plane_y = dir_x * CAMERA_PLANE_SCALE;
    (dir_x, dir_y, plane_x, plane_y)
}

fn is_wall(map: &MapDef, x: f32, y: f32) -> bool {
    let mx = floorf(x) as i32;
    let my = floorf(y) as i32;
    if mx < 0 || my < 0 || mx as usize >= MAP_W || my as usize >= MAP_H {
        return true;
    }
    map.layout[my as usize][mx as usize] != 0
}

fn is_blocked_circle(map: &MapDef, x: f32, y: f32, radius: f32) -> bool {
    is_wall(map, x, y)
        || is_wall(map, x - radius, y)
        || is_wall(map, x + radius, y)
        || is_wall(map, x, y - radius)
        || is_wall(map, x, y + radius)
        || is_wall(map, x - radius * 0.7, y - radius * 0.7)
        || is_wall(map, x + radius * 0.7, y - radius * 0.7)
        || is_wall(map, x - radius * 0.7, y + radius * 0.7)
        || is_wall(map, x + radius * 0.7, y + radius * 0.7)
}

fn line_of_sight(map: &MapDef, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let distance = sqrtf(dx * dx + dy * dy);
    let steps = (distance * 10.0) as i32;
    if steps <= 1 {
        return true;
    }
    for step in 1..steps {
        let t = step as f32 / steps as f32;
        let x = x0 + dx * t;
        let y = y0 + dy * t;
        if is_wall(map, x, y) {
            return false;
        }
    }
    true
}

fn point_in_circle(x: u16, y: u16, center_x: u16, center_y: u16, radius: f32) -> bool {
    let dx = x as f32 - center_x as f32;
    let dy = y as f32 - center_y as f32;
    (dx * dx) + (dy * dy) <= radius * radius
}

fn clamp_circle_delta(dx: f32, dy: f32, radius: f32) -> (f32, f32) {
    let magnitude = sqrtf(dx * dx + dy * dy);
    if magnitude <= radius || magnitude <= 0.0001 {
        (dx, dy)
    } else {
        let scale = radius / magnitude;
        (dx * scale, dy * scale)
    }
}

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if fabsf(value) <= deadzone {
        0.0
    } else if value > 0.0 {
        (value - deadzone) / (1.0 - deadzone)
    } else {
        (value + deadzone) / (1.0 - deadzone)
    }
}

fn round_to_usize(value: f32) -> usize {
    floorf(value + 0.5) as usize
}

fn round_to_i32(value: f32) -> i32 {
    floorf(value + 0.5) as i32
}

fn distance_sq(dx: f32, dy: f32) -> f32 {
    dx * dx + dy * dy
}

fn wrap_angle(angle: f32) -> f32 {
    let mut wrapped = angle;
    while wrapped > core::f32::consts::PI {
        wrapped -= core::f32::consts::TAU;
    }
    while wrapped < -core::f32::consts::PI {
        wrapped += core::f32::consts::TAU;
    }
    wrapped
}
