#[path = "auto_battle/geometry.rs"]
mod geometry;
#[path = "auto_battle/render.rs"]
mod render;
#[path = "auto_battle/update.rs"]
mod update;

use geometry::{ArenaFrame, EnemyFrame, ProjectileFrame};

const ARENA_X: u16 = 6;
const ARENA_Y: u16 = 34;
const ARENA_W: u16 = 308;
const ARENA_H: u16 = 200;
const ARENA_INNER_X: u16 = ARENA_X + 4;
const ARENA_INNER_Y: u16 = ARENA_Y + 4;
const ARENA_INNER_W: u16 = ARENA_W - 8;
const ARENA_INNER_H: u16 = ARENA_H - 8;
const PANEL_W: u16 = 104;
const PANEL_H: u16 = 36;
const PANEL_X: u16 = ARENA_X + ARENA_W - PANEL_W - 8;
const PANEL_Y: u16 = ARENA_Y + 8;

const PLAYER_SIZE: i16 = 8;
const PLAYER_SPEED: f32 = 0.095;
const PLAYER_START_HP: i16 = 5;
const BASE_SHOT_COOLDOWN_MS: u16 = 250;
const BASE_PROJECTILE_SPEED: f32 = 0.17;
const ENEMY_PROJECTILE_SPEED: f32 = 0.10;
const HIT_INVULN_MS: u16 = 420;
const ENEMY_TOUCH_DAMAGE_MS: u16 = 520;
const KILLS_PER_LEVEL: u16 = 8;

const MAX_ENEMIES: usize = 8;
const MAX_PROJECTILES: usize = 24;
const TARGET_KILLS: u16 = 100;
const LEVEL_UP_CHOICES: usize = 3;
const BUFF_KIND_COUNT: usize = 9;
const MAX_ARENA_DIRTY_RECTS: usize = 40;

const OVERLAY_ACTION_X: u16 = 236;
const OVERLAY_ACTION_Y: u16 = 188;
const OVERLAY_ACTION_W: u16 = 58;
const OVERLAY_ACTION_H: u16 = 20;

const BUFF_X: u16 = 30;
const BUFF_Y: u16 = 82;
const BUFF_W: u16 = 260;
const BUFF_H: u16 = 36;
const BUFF_GAP: u16 = 10;

const SPAWN_POINTS: [(f32, f32); MAX_ENEMIES] = [
    (14.0, 16.0),
    (104.0, 18.0),
    (194.0, 22.0),
    (18.0, 82.0),
    (192.0, 88.0),
    (16.0, 156.0),
    (102.0, 160.0),
    (194.0, 150.0),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyKind {
    Runner,
    Shooter,
    Bruiser,
    Dasher,
    Summoner,
}

#[derive(Clone, Copy)]
struct Enemy {
    active: bool,
    kind: EnemyKind,
    x: f32,
    y: f32,
    hp: i16,
    max_hp: i16,
    speed: f32,
    size: u16,
    touch_timer_ms: u16,
    attack_timer_ms: u16,
    burst_timer_ms: u16,
    flash_ms: u16,
    phase: u16,
}

impl Enemy {
    const fn empty() -> Self {
        Self {
            active: false,
            kind: EnemyKind::Runner,
            x: 0.0,
            y: 0.0,
            hp: 0,
            max_hp: 0,
            speed: 0.0,
            size: 10,
            touch_timer_ms: 0,
            attack_timer_ms: 0,
            burst_timer_ms: 0,
            flash_ms: 0,
            phase: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Projectile {
    active: bool,
    from_enemy: bool,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: i16,
    pierce_left: u8,
    ttl_ms: u16,
}

impl Projectile {
    const fn empty() -> Self {
        Self {
            active: false,
            from_enemy: false,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            damage: 0,
            pierce_left: 0,
            ttl_ms: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BattleState {
    Ready,
    Running,
    LevelUp,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuffKind {
    MultiShot,
    VitalCore,
    Impact,
    QuickTrigger,
    Velocity,
    Thrusters,
    PhaseRound,
    LongBarrel,
    GuardShell,
}

impl BuffKind {
    const fn index(self) -> usize {
        match self {
            Self::MultiShot => 0,
            Self::VitalCore => 1,
            Self::Impact => 2,
            Self::QuickTrigger => 3,
            Self::Velocity => 4,
            Self::Thrusters => 5,
            Self::PhaseRound => 6,
            Self::LongBarrel => 7,
            Self::GuardShell => 8,
        }
    }

    const fn title_en(self) -> &'static str {
        match self {
            Self::MultiShot => "TWIN SHOT",
            Self::VitalCore => "VITAL CORE",
            Self::Impact => "IMPACT+",
            Self::QuickTrigger => "QUICK TRIGGER",
            Self::Velocity => "VELOCITY",
            Self::Thrusters => "THRUSTERS",
            Self::PhaseRound => "PHASE ROUND",
            Self::LongBarrel => "LONG BARREL",
            Self::GuardShell => "GUARD SHELL",
        }
    }

    const fn title_zh(self) -> &'static str {
        match self {
            Self::MultiShot => "雙重射擊",
            Self::VitalCore => "生命核心",
            Self::Impact => "重擊升級",
            Self::QuickTrigger => "快速扳機",
            Self::Velocity => "彈速提升",
            Self::Thrusters => "推進模組",
            Self::PhaseRound => "穿透彈",
            Self::LongBarrel => "長射程",
            Self::GuardShell => "守護外殼",
        }
    }

    const fn desc_en(self) -> &'static str {
        match self {
            Self::MultiShot => "ADD ONE EXTRA PROJECTILE",
            Self::VitalCore => "MAX HP +2 AND HEAL +2",
            Self::Impact => "PROJECTILES DEAL +1 DMG",
            Self::QuickTrigger => "SHORTER AUTO-FIRE COOLDOWN",
            Self::Velocity => "SHOTS TRAVEL FASTER",
            Self::Thrusters => "MOVE FASTER WHILE DRAGGING",
            Self::PhaseRound => "SHOTS PIERCE ONE MORE TARGET",
            Self::LongBarrel => "PROJECTILES TRAVEL FARTHER",
            Self::GuardShell => "LONGER I-FRAMES AND HEAL +1",
        }
    }

    const fn desc_zh(self) -> &'static str {
        match self {
            Self::MultiShot => "多一發子彈同時射出",
            Self::VitalCore => "最大血量 +2 並回 2 血",
            Self::Impact => "每發子彈傷害 +1",
            Self::QuickTrigger => "自動射擊冷卻更短",
            Self::Velocity => "子彈飛得更快",
            Self::Thrusters => "拖曳移動時跑得更快",
            Self::PhaseRound => "子彈多穿透一個敵人",
            Self::LongBarrel => "子彈能飛得更遠",
            Self::GuardShell => "受傷無敵更久並回 1 血",
        }
    }
}

const ALL_BUFFS: [BuffKind; BUFF_KIND_COUNT] = [
    BuffKind::MultiShot,
    BuffKind::VitalCore,
    BuffKind::Impact,
    BuffKind::QuickTrigger,
    BuffKind::Velocity,
    BuffKind::Thrusters,
    BuffKind::PhaseRound,
    BuffKind::LongBarrel,
    BuffKind::GuardShell,
];

pub enum AutoBattleAction {
    Stay,
    ExitGameCenter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AutoBattleRedraw {
    Full,
    Arena,
    ArenaAndPanel,
    Overlay,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PanelSnapshot {
    moving: bool,
    health: i16,
    max_health: i16,
    kills: u16,
    level: u8,
    next_level_kills: u16,
    shot_cooldown_base_ms: u16,
    bullet_damage: i16,
    projectile_count: u8,
    buff_counts: [u8; BUFF_KIND_COUNT],
}

pub struct AutoBattleApp {
    state: BattleState,
    player_x: f32,
    player_y: f32,
    target_x: f32,
    target_y: f32,
    moving: bool,
    health: i16,
    max_health: i16,
    kills: u16,
    best_kills: u16,
    level: u8,
    next_level_kills: u16,
    wave_seed: u32,
    shot_cooldown_ms: u16,
    shot_cooldown_base_ms: u16,
    hit_invuln_ms: u16,
    hit_invuln_base_ms: u16,
    bullet_damage: i16,
    projectile_count: u8,
    projectile_speed: f32,
    projectile_ttl_ms: u16,
    projectile_pierce: u8,
    player_speed: f32,
    buff_counts: [u8; BUFF_KIND_COUNT],
    level_up_choices: [BuffKind; LEVEL_UP_CHOICES],
    selected_choice: usize,
    enemies: [Enemy; MAX_ENEMIES],
    projectiles: [Projectile; MAX_PROJECTILES],
    redraw_pending: Option<AutoBattleRedraw>,
    last_arena_frame: ArenaFrame,
    arena_frame_valid: bool,
}

impl AutoBattleApp {
    pub const fn new() -> Self {
        Self {
            state: BattleState::Ready,
            player_x: ARENA_W as f32 * 0.5,
            player_y: ARENA_H as f32 * 0.5,
            target_x: ARENA_W as f32 * 0.5,
            target_y: ARENA_H as f32 * 0.5,
            moving: false,
            health: PLAYER_START_HP,
            max_health: PLAYER_START_HP,
            kills: 0,
            best_kills: 0,
            level: 1,
            next_level_kills: KILLS_PER_LEVEL,
            wave_seed: 0xA17C_02D2,
            shot_cooldown_ms: 0,
            shot_cooldown_base_ms: BASE_SHOT_COOLDOWN_MS,
            hit_invuln_ms: 0,
            hit_invuln_base_ms: HIT_INVULN_MS,
            bullet_damage: 1,
            projectile_count: 1,
            projectile_speed: BASE_PROJECTILE_SPEED,
            projectile_ttl_ms: 900,
            projectile_pierce: 0,
            player_speed: PLAYER_SPEED,
            buff_counts: [0; BUFF_KIND_COUNT],
            level_up_choices: [BuffKind::MultiShot, BuffKind::VitalCore, BuffKind::Impact],
            selected_choice: 0,
            enemies: [Enemy::empty(); MAX_ENEMIES],
            projectiles: [Projectile::empty(); MAX_PROJECTILES],
            redraw_pending: None,
            last_arena_frame: ArenaFrame::empty(),
            arena_frame_valid: false,
        }
    }

    pub fn reset(&mut self) {
        self.state = BattleState::Running;
        self.player_x = ARENA_W as f32 * 0.5;
        self.player_y = ARENA_H as f32 * 0.5;
        self.target_x = self.player_x;
        self.target_y = self.player_y;
        self.moving = false;
        self.max_health = PLAYER_START_HP;
        self.health = PLAYER_START_HP;
        self.kills = 0;
        self.level = 1;
        self.next_level_kills = KILLS_PER_LEVEL;
        self.shot_cooldown_ms = 120;
        self.shot_cooldown_base_ms = BASE_SHOT_COOLDOWN_MS;
        self.hit_invuln_ms = 0;
        self.hit_invuln_base_ms = HIT_INVULN_MS;
        self.bullet_damage = 1;
        self.projectile_count = 1;
        self.projectile_speed = BASE_PROJECTILE_SPEED;
        self.projectile_ttl_ms = 900;
        self.projectile_pierce = 0;
        self.player_speed = PLAYER_SPEED;
        self.buff_counts = [0; BUFF_KIND_COUNT];
        self.projectiles = [Projectile::empty(); MAX_PROJECTILES];
        self.enemies = [Enemy::empty(); MAX_ENEMIES];
        self.spawn_opening_wave();
        self.arena_frame_valid = false;
        self.request_redraw(AutoBattleRedraw::Full);
    }

    pub fn best_kills(&self) -> u16 {
        self.best_kills
    }

    pub fn set_best_kills(&mut self, best_kills: u16) {
        self.best_kills = best_kills;
        self.request_redraw(AutoBattleRedraw::Full);
    }

    pub fn take_redraw_request(&mut self) -> Option<AutoBattleRedraw> {
        let redraw = self.redraw_pending;
        self.redraw_pending = None;
        redraw
    }

    pub(super) fn sync_best_kills(&mut self) {
        self.best_kills = self.best_kills.max(self.kills);
    }

    fn capture_arena_frame(&self) -> ArenaFrame {
        let mut enemies = [EnemyFrame::empty(); MAX_ENEMIES];
        for (index, enemy) in self.enemies.iter().enumerate() {
            if enemy.active {
                enemies[index] = EnemyFrame {
                    active: true,
                    x: ARENA_X as i16 + enemy.x as i16,
                    y: ARENA_Y as i16 + enemy.y as i16,
                    size: enemy.size as i16,
                };
            }
        }

        let mut projectiles = [ProjectileFrame::empty(); MAX_PROJECTILES];
        for (index, projectile) in self.projectiles.iter().enumerate() {
            if projectile.active {
                let px = ARENA_X as i16 + projectile.x as i16;
                let py = ARENA_Y as i16 + projectile.y as i16;
                projectiles[index] = ProjectileFrame {
                    active: true,
                    x: px,
                    y: py,
                    tail_x: px - (projectile.vx * 18.0) as i16,
                    tail_y: py - (projectile.vy * 18.0) as i16,
                };
            }
        }

        ArenaFrame {
            player_x: ARENA_X as i16 + self.player_x as i16,
            player_y: ARENA_Y as i16 + self.player_y as i16,
            target_x: ARENA_X as i16 + self.target_x as i16,
            target_y: ARENA_Y as i16 + self.target_y as i16,
            moving: self.moving,
            nearest_enemy: if self.moving {
                None
            } else {
                self.nearest_enemy_position()
                    .map(|(x, y)| (ARENA_X as i16 + x as i16, ARENA_Y as i16 + y as i16))
            },
            enemies,
            projectiles,
        }
    }

    fn panel_snapshot(&self) -> PanelSnapshot {
        PanelSnapshot {
            moving: self.moving,
            health: self.health,
            max_health: self.max_health,
            kills: self.kills,
            level: self.level,
            next_level_kills: self.next_level_kills,
            shot_cooldown_base_ms: self.shot_cooldown_base_ms,
            bullet_damage: self.bullet_damage,
            projectile_count: self.projectile_count,
            buff_counts: self.buff_counts,
        }
    }

    fn request_redraw(&mut self, redraw: AutoBattleRedraw) {
        self.redraw_pending = Some(match (self.redraw_pending, redraw) {
            (Some(AutoBattleRedraw::Full), _) | (_, AutoBattleRedraw::Full) => {
                AutoBattleRedraw::Full
            }
            (Some(AutoBattleRedraw::ArenaAndPanel), _) | (_, AutoBattleRedraw::ArenaAndPanel) => {
                AutoBattleRedraw::ArenaAndPanel
            }
            (Some(AutoBattleRedraw::Overlay), AutoBattleRedraw::Arena)
            | (Some(AutoBattleRedraw::Arena), AutoBattleRedraw::Overlay) => AutoBattleRedraw::Full,
            (Some(existing), _) => existing,
            (None, value) => value,
        });
    }

    fn next_rand(&mut self) -> u32 {
        self.wave_seed = self
            .wave_seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.wave_seed
    }
}
