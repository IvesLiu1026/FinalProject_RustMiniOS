mod data;
#[path = "dungeon/math.rs"]
mod math;
#[path = "dungeon/render.rs"]
mod render;
mod strategy;
#[path = "dungeon/update.rs"]
mod update;
mod weapon;

use libm::{atan2f, cosf, fabsf, floorf, sinf, sqrtf};

use crate::assets::{self, EnemySprite, TextureId, TEX_SIZE};
use crate::board::ButtonSnapshot;
use crate::display::{palette, shade, Display, ThemeMode, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::touch::TouchState;
use data::{MapDef, MAPS, MAP_H, MAP_W, MAX_ENEMIES, MAX_PICKUPS};
pub use strategy::RenderStrategy;
use weapon::WeaponKind;

const COLUMN_WIDTH: u16 = 1;
const CAMERA_PLANE_SCALE: f32 = 0.577_350_26;
const VIEW_Y: u16 = 40;
const VIEW_SCALE: u16 = 2;
const VIEW_W: usize = SCREEN_WIDTH as usize / VIEW_SCALE as usize;
const VIEW_H: u16 = (SCREEN_HEIGHT - VIEW_Y) / VIEW_SCALE;
const VIEW_BOTTOM: u16 = VIEW_H;
const VIEW_H_USIZE: usize = VIEW_H as usize;
const VIEW_PIXELS: usize = VIEW_W * VIEW_H_USIZE;
const RAY_COUNT: u16 = VIEW_W as u16 / COLUMN_WIDTH;

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
}

struct RayHit {
    distance: f32,
    tile: u8,
    side: u8,
    wall_x: f32,
    dir_x: f32,
    dir_y: f32,
}
