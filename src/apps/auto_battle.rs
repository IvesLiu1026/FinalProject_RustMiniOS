#[path = "auto_battle/geometry.rs"]
mod geometry;
#[path = "auto_battle/render.rs"]
mod render;
#[path = "auto_battle/update.rs"]
mod update;

use geometry::{ArenaFrame, EnemyFrame, PickupFrame, ProjectileFrame};

use crate::storage::{PersistedStationHunterData, STATION_HUNTER_STAGE_COUNT};

const ARENA_X: u16 = 6;
const ARENA_Y: u16 = 34;
const ARENA_W: u16 = 308;
const ARENA_H: u16 = 200;
const ARENA_INNER_X: u16 = ARENA_X + 4;
const ARENA_INNER_Y: u16 = ARENA_Y + 4;
const ARENA_INNER_W: u16 = ARENA_W - 8;
const ARENA_INNER_H: u16 = ARENA_H - 8;
const PANEL_W: u16 = 124;
const PANEL_H: u16 = 58;
const PANEL_X: u16 = ARENA_X + ARENA_W - PANEL_W - 8;
const PANEL_Y: u16 = ARENA_Y + 8;

const PLAYER_SIZE: i16 = 8;
const PLAYER_BASE_SPEED: f32 = 0.095;
const PLAYER_BASE_HP: i16 = 5;
const PLAYER_BASE_ATTACK: i16 = 1;
const BASE_SHOT_COOLDOWN_MS: u16 = 250;
const BASE_PROJECTILE_SPEED: f32 = 0.17;
const ENEMY_PROJECTILE_SPEED: f32 = 0.10;
const HIT_INVULN_MS: u16 = 420;
const ENEMY_TOUCH_DAMAGE_MS: u16 = 520;

const MAX_ENEMIES: usize = 8;
const MAX_PROJECTILES: usize = 24;
const MAX_PICKUPS: usize = 2;
const LEVEL_UP_CHOICES: usize = 3;
const BUFF_KIND_COUNT: usize = 9;
const MAX_ARENA_DIRTY_RECTS: usize = 40;
const STAGE_COUNT: usize = STATION_HUNTER_STAGE_COUNT;
const WAVES_PER_STAGE: u8 = 30;
const BOSS_INTERVAL: u8 = 10;
const PICKUP_HEAL_AMOUNT: i16 = 2;
const DAMAGE_FLASH_MS: u16 = 180;
const HEAL_FLASH_MS: u16 = 240;
const WEAPON_FLASH_MS: u16 = 70;
const WAVE_BANNER_MS: u16 = 820;
const BOSS_BANNER_MS: u16 = 1_240;
const BOSS_INTRO_MS: u16 = 560;

const BUFF_X: u16 = 30;
const BUFF_Y: u16 = 78;
const BUFF_W: u16 = 260;
const BUFF_H: u16 = 38;
const BUFF_GAP: u16 = 10;

const PROFILE_CARD_W: u16 = 128;
const PROFILE_CARD_H: u16 = 42;
const PROFILE_LEFT_X: u16 = 20;
const PROFILE_RIGHT_X: u16 = 172;
const PROFILE_TOP_Y: u16 = 76;
const PROFILE_BOTTOM_Y: u16 = 126;
const PROFILE_DEPLOY_X: u16 = 188;
const PROFILE_DEPLOY_Y: u16 = 184;
const PROFILE_DEPLOY_W: u16 = 92;
const PROFILE_DEPLOY_H: u16 = 22;

const STAGE_CARD_X: u16 = 22;
const STAGE_CARD_Y: u16 = 54;
const STAGE_CARD_W: u16 = 276;
const STAGE_CARD_H: u16 = 28;
const STAGE_CARD_GAP: u16 = 6;

const RESULT_BUTTON_X: u16 = 176;
const RESULT_BUTTON_Y: u16 = 184;
const RESULT_BUTTON_W: u16 = 104;
const RESULT_BUTTON_H: u16 = 18;

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
    BossRam,
    BossBurst,
    BossNest,
    BossRing,
}

impl EnemyKind {
    const fn is_boss(self) -> bool {
        matches!(
            self,
            Self::BossRam | Self::BossBurst | Self::BossNest | Self::BossRing
        )
    }

    const fn title_en(self) -> &'static str {
        match self {
            Self::Runner => "RUNNER",
            Self::Shooter => "SHOOTER",
            Self::Bruiser => "BRUISER",
            Self::Dasher => "DASHER",
            Self::Summoner => "SUMMONER",
            Self::BossRam => "RAM CORE",
            Self::BossBurst => "BURST NODE",
            Self::BossNest => "NEST CORE",
            Self::BossRing => "RING CORE",
        }
    }

    const fn title_zh(self) -> &'static str {
        match self {
            Self::Runner => "突進者",
            Self::Shooter => "射手",
            Self::Bruiser => "重裝體",
            Self::Dasher => "衝刺體",
            Self::Summoner => "召喚體",
            Self::BossRam => "衝撞核心",
            Self::BossBurst => "爆裂核心",
            Self::BossNest => "巢穴核心",
            Self::BossRing => "環形核心",
        }
    }
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
    charges: u8,
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
            charges: 0,
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
enum PickupKind {
    MedKit,
}

#[derive(Clone, Copy)]
struct Pickup {
    active: bool,
    kind: PickupKind,
    x: f32,
    y: f32,
    size: u16,
}

impl Pickup {
    const fn empty() -> Self {
        Self {
            active: false,
            kind: PickupKind::MedKit,
            x: 0.0,
            y: 0.0,
            size: 12,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BattleState {
    Profile,
    StageSelect,
    Running,
    BossReward,
    StageClear,
    Defeat,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WaveKind {
    Standard,
    Pressure,
    Elite,
    Boss,
}

impl WaveKind {
    const fn title_en(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Pressure => "PRESSURE",
            Self::Elite => "ELITE",
            Self::Boss => "BOSS",
        }
    }

    const fn title_zh(self) -> &'static str {
        match self {
            Self::Standard => "一般波",
            Self::Pressure => "壓力波",
            Self::Elite => "精英波",
            Self::Boss => "Boss 波",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProfileAction {
    Attack,
    Vitality,
    Trigger,
    Thrusters,
    Deploy,
}

impl ProfileAction {
    const ALL: [Self; 5] = [
        Self::Attack,
        Self::Vitality,
        Self::Trigger,
        Self::Thrusters,
        Self::Deploy,
    ];

    const fn title_en(self) -> &'static str {
        match self {
            Self::Attack => "ATTACK",
            Self::Vitality => "VITAL",
            Self::Trigger => "TRIGGER",
            Self::Thrusters => "THRUST",
            Self::Deploy => "DEPLOY",
        }
    }

    const fn title_zh(self) -> &'static str {
        match self {
            Self::Attack => "攻擊",
            Self::Vitality => "生命",
            Self::Trigger => "射速",
            Self::Thrusters => "移速",
            Self::Deploy => "出擊",
        }
    }
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

#[derive(Clone, Copy)]
struct WaveTracker {
    stage_index: usize,
    wave: u8,
    kind: WaveKind,
    total_to_spawn: u8,
    spawned: u8,
    remaining_to_kill: u8,
    active_cap: u8,
    spawn_interval_ms: u16,
    spawn_timer_ms: u16,
    boss_kind: Option<EnemyKind>,
}

impl WaveTracker {
    const fn empty() -> Self {
        Self {
            stage_index: 0,
            wave: 0,
            kind: WaveKind::Standard,
            total_to_spawn: 0,
            spawned: 0,
            remaining_to_kill: 0,
            active_cap: 0,
            spawn_interval_ms: 0,
            spawn_timer_ms: 0,
            boss_kind: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AutoBattleAction {
    Stay,
    ExitGameCenter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AutoBattleRedraw {
    Full,
    Arena,
    ArenaAndPanel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PanelSnapshot {
    moving: bool,
    health: i16,
    max_health: i16,
    kills: u16,
    stage: u8,
    wave: u8,
    remaining: u8,
    best_kills: u16,
    wave_kind: WaveKind,
    medkit_active: bool,
    boss_hp: i16,
    boss_max_hp: i16,
}

#[derive(Clone, Copy)]
struct ResultSummary {
    stage: u8,
    wave_reached: u8,
    kills: u16,
    xp_gain: u16,
    level_gained: u8,
    upgrade_points_gain: u8,
    unlocked_stage: Option<u8>,
}

impl ResultSummary {
    const fn empty() -> Self {
        Self {
            stage: 1,
            wave_reached: 0,
            kills: 0,
            xp_gain: 0,
            level_gained: 0,
            upgrade_points_gain: 0,
            unlocked_stage: None,
        }
    }
}

#[derive(Clone, Copy)]
struct StageDefinition {
    title_en: &'static str,
    title_zh: &'static str,
    note_en: &'static str,
    note_zh: &'static str,
    enemy_pool: [EnemyKind; 4],
    bosses: [EnemyKind; 3],
}

const STAGES: [StageDefinition; STAGE_COUNT] = [
    StageDefinition {
        title_en: "STAGE 1 / BOOT CAMP",
        title_zh: "第一關 / 啟動訓練",
        note_en: "LEARN TO MOVE, PLANT, AND FIRE",
        note_zh: "學會走位、停下、開火",
        enemy_pool: [
            EnemyKind::Runner,
            EnemyKind::Bruiser,
            EnemyKind::Runner,
            EnemyKind::Shooter,
        ],
        bosses: [EnemyKind::BossRam, EnemyKind::BossBurst, EnemyKind::BossRam],
    },
    StageDefinition {
        title_en: "STAGE 2 / CROSSLINE",
        title_zh: "第二關 / 火線交叉",
        note_en: "RANGED PRESSURE ENTERS THE FIELD",
        note_zh: "遠程壓力正式加入戰場",
        enemy_pool: [
            EnemyKind::Runner,
            EnemyKind::Shooter,
            EnemyKind::Bruiser,
            EnemyKind::Shooter,
        ],
        bosses: [
            EnemyKind::BossBurst,
            EnemyKind::BossRam,
            EnemyKind::BossBurst,
        ],
    },
    StageDefinition {
        title_en: "STAGE 3 / SNAP DASH",
        title_zh: "第三關 / 閃擊節奏",
        note_en: "DASHERS BREAK YOUR STANDSTILL",
        note_zh: "衝刺敵人開始打亂站樁節奏",
        enemy_pool: [
            EnemyKind::Dasher,
            EnemyKind::Runner,
            EnemyKind::Shooter,
            EnemyKind::Dasher,
        ],
        bosses: [
            EnemyKind::BossRam,
            EnemyKind::BossBurst,
            EnemyKind::BossRing,
        ],
    },
    StageDefinition {
        title_en: "STAGE 4 / OVERGROWTH",
        title_zh: "第四關 / 場控蔓延",
        note_en: "SUMMONERS TURN EMPTY SPACE HOSTILE",
        note_zh: "召喚型敵人開始佔滿戰場",
        enemy_pool: [
            EnemyKind::Summoner,
            EnemyKind::Shooter,
            EnemyKind::Bruiser,
            EnemyKind::Summoner,
        ],
        bosses: [
            EnemyKind::BossNest,
            EnemyKind::BossRing,
            EnemyKind::BossNest,
        ],
    },
    StageDefinition {
        title_en: "STAGE 5 / STATION ZERO",
        title_zh: "第五關 / 原點試煉",
        note_en: "ALL THREATS COLLIDE IN THE FINAL RUN",
        note_zh: "所有威脅混合成最終試煉",
        enemy_pool: [
            EnemyKind::Runner,
            EnemyKind::Dasher,
            EnemyKind::Shooter,
            EnemyKind::Summoner,
        ],
        bosses: [
            EnemyKind::BossRing,
            EnemyKind::BossNest,
            EnemyKind::BossRing,
        ],
    },
];

pub struct AutoBattleApp {
    state: BattleState,
    profile: PersistedStationHunterData,
    player_x: f32,
    player_y: f32,
    target_x: f32,
    target_y: f32,
    moving: bool,
    health: i16,
    max_health: i16,
    kills: u16,
    current_stage: u8,
    profile_cursor: usize,
    stage_select_index: usize,
    result_choice: usize,
    wave_tracker: WaveTracker,
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
    reward_choices: [BuffKind; LEVEL_UP_CHOICES],
    selected_choice: usize,
    damage_flash_ms: u16,
    heal_flash_ms: u16,
    weapon_flash_ms: u16,
    wave_banner_ms: u16,
    boss_banner_ms: u16,
    boss_intro_ms: u16,
    enemies: [Enemy; MAX_ENEMIES],
    projectiles: [Projectile; MAX_PROJECTILES],
    pickups: [Pickup; MAX_PICKUPS],
    result_summary: ResultSummary,
    redraw_pending: Option<AutoBattleRedraw>,
    persist_requested: bool,
    last_arena_frame: ArenaFrame,
    arena_frame_valid: bool,
}

impl AutoBattleApp {
    pub const fn new() -> Self {
        Self {
            state: BattleState::Profile,
            profile: PersistedStationHunterData {
                selected_stage: 1,
                player_level: 1,
                player_xp: 0,
                upgrade_points: 0,
                unlocked_stage: 1,
                base_attack: 0,
                base_hp: 0,
                base_fire_rate: 0,
                base_move_speed: 0,
                best_kills: 0,
                stage_best_wave: [0; STAGE_COUNT],
                stage_best_kills: [0; STAGE_COUNT],
                stage_clear_count: [0; STAGE_COUNT],
            },
            player_x: ARENA_W as f32 * 0.5,
            player_y: ARENA_H as f32 * 0.5,
            target_x: ARENA_W as f32 * 0.5,
            target_y: ARENA_H as f32 * 0.5,
            moving: false,
            health: PLAYER_BASE_HP,
            max_health: PLAYER_BASE_HP,
            kills: 0,
            current_stage: 1,
            profile_cursor: 0,
            stage_select_index: 0,
            result_choice: 0,
            wave_tracker: WaveTracker::empty(),
            wave_seed: 0xA17C_02D2,
            shot_cooldown_ms: 0,
            shot_cooldown_base_ms: BASE_SHOT_COOLDOWN_MS,
            hit_invuln_ms: 0,
            hit_invuln_base_ms: HIT_INVULN_MS,
            bullet_damage: PLAYER_BASE_ATTACK,
            projectile_count: 1,
            projectile_speed: BASE_PROJECTILE_SPEED,
            projectile_ttl_ms: 900,
            projectile_pierce: 0,
            player_speed: PLAYER_BASE_SPEED,
            buff_counts: [0; BUFF_KIND_COUNT],
            reward_choices: [BuffKind::MultiShot, BuffKind::VitalCore, BuffKind::Impact],
            selected_choice: 0,
            damage_flash_ms: 0,
            heal_flash_ms: 0,
            weapon_flash_ms: 0,
            wave_banner_ms: 0,
            boss_banner_ms: 0,
            boss_intro_ms: 0,
            enemies: [Enemy::empty(); MAX_ENEMIES],
            projectiles: [Projectile::empty(); MAX_PROJECTILES],
            pickups: [Pickup::empty(); MAX_PICKUPS],
            result_summary: ResultSummary::empty(),
            redraw_pending: None,
            persist_requested: false,
            last_arena_frame: ArenaFrame::empty(),
            arena_frame_valid: false,
        }
    }

    pub fn enter(&mut self) {
        self.state = BattleState::Profile;
        self.profile_cursor = 0;
        self.stage_select_index = self.profile.selected_stage.saturating_sub(1) as usize;
        self.result_choice = 0;
        self.request_redraw(AutoBattleRedraw::Full);
    }

    pub fn snapshot(&self) -> PersistedStationHunterData {
        self.profile
    }

    pub fn restore(&mut self, mut state: PersistedStationHunterData) {
        state.selected_stage = state.selected_stage.clamp(1, STAGE_COUNT as u8);
        state.player_level = state.player_level.max(1);
        state.unlocked_stage = state.unlocked_stage.clamp(1, STAGE_COUNT as u8);
        if state.selected_stage > state.unlocked_stage {
            state.selected_stage = state.unlocked_stage;
        }
        self.profile = state;
        self.current_stage = self.profile.selected_stage;
        self.stage_select_index = self.profile.selected_stage.saturating_sub(1) as usize;
        self.enter();
    }

    pub fn take_redraw_request(&mut self) -> Option<AutoBattleRedraw> {
        let redraw = self.redraw_pending;
        self.redraw_pending = None;
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

    fn request_redraw(&mut self, redraw: AutoBattleRedraw) {
        self.redraw_pending = Some(match (self.redraw_pending, redraw) {
            (Some(AutoBattleRedraw::Full), _) | (_, AutoBattleRedraw::Full) => {
                AutoBattleRedraw::Full
            }
            (Some(AutoBattleRedraw::ArenaAndPanel), _) | (_, AutoBattleRedraw::ArenaAndPanel) => {
                AutoBattleRedraw::ArenaAndPanel
            }
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

    fn stage_def(&self, stage_index: usize) -> &'static StageDefinition {
        &STAGES[stage_index.min(STAGE_COUNT - 1)]
    }

    fn stage_index(&self) -> usize {
        self.current_stage
            .saturating_sub(1)
            .min((STAGE_COUNT - 1) as u8) as usize
    }

    fn is_stage_unlocked(&self, stage_index: usize) -> bool {
        stage_index < self.profile.unlocked_stage as usize
    }

    fn xp_to_next_level(level: u8) -> u16 {
        28 + level as u16 * 14
    }

    fn stage_best_wave(&self, stage_index: usize) -> u8 {
        self.profile.stage_best_wave[stage_index]
    }

    fn stage_best_kills(&self, stage_index: usize) -> u16 {
        self.profile.stage_best_kills[stage_index]
    }

    fn stage_clear_count(&self, stage_index: usize) -> u8 {
        self.profile.stage_clear_count[stage_index]
    }

    fn spend_profile_upgrade(&mut self, action: ProfileAction) {
        if self.profile.upgrade_points == 0 {
            return;
        }
        match action {
            ProfileAction::Attack => {
                self.profile.base_attack = self.profile.base_attack.saturating_add(1)
            }
            ProfileAction::Vitality => {
                self.profile.base_hp = self.profile.base_hp.saturating_add(1)
            }
            ProfileAction::Trigger => {
                self.profile.base_fire_rate = self.profile.base_fire_rate.saturating_add(1)
            }
            ProfileAction::Thrusters => {
                self.profile.base_move_speed = self.profile.base_move_speed.saturating_add(1)
            }
            ProfileAction::Deploy => return,
        }
        self.profile.upgrade_points = self.profile.upgrade_points.saturating_sub(1);
        self.request_persist();
        self.request_redraw(AutoBattleRedraw::Full);
    }

    fn open_stage_select(&mut self) {
        self.state = BattleState::StageSelect;
        self.stage_select_index = self.profile.selected_stage.saturating_sub(1) as usize;
        self.request_redraw(AutoBattleRedraw::Full);
    }

    fn start_selected_stage(&mut self) {
        self.current_stage = (self.stage_select_index + 1) as u8;
        self.profile.selected_stage = self.current_stage;
        self.player_x = ARENA_W as f32 * 0.5;
        self.player_y = ARENA_H as f32 * 0.5;
        self.target_x = self.player_x;
        self.target_y = self.player_y;
        self.moving = false;
        self.max_health = PLAYER_BASE_HP + self.profile.base_hp as i16 * 2;
        self.health = self.max_health;
        self.kills = 0;
        self.shot_cooldown_ms = 120;
        self.shot_cooldown_base_ms = BASE_SHOT_COOLDOWN_MS
            .saturating_sub(self.profile.base_fire_rate as u16 * 16)
            .max(120);
        self.hit_invuln_ms = 0;
        self.hit_invuln_base_ms = HIT_INVULN_MS;
        self.bullet_damage = PLAYER_BASE_ATTACK + self.profile.base_attack as i16;
        self.projectile_count = 1;
        self.projectile_speed = BASE_PROJECTILE_SPEED;
        self.projectile_ttl_ms = 900;
        self.projectile_pierce = 0;
        self.player_speed = PLAYER_BASE_SPEED + self.profile.base_move_speed as f32 * 0.006;
        self.buff_counts = [0; BUFF_KIND_COUNT];
        self.damage_flash_ms = 0;
        self.heal_flash_ms = 0;
        self.weapon_flash_ms = 0;
        self.wave_banner_ms = 0;
        self.boss_banner_ms = 0;
        self.boss_intro_ms = 0;
        self.projectiles = [Projectile::empty(); MAX_PROJECTILES];
        self.enemies = [Enemy::empty(); MAX_ENEMIES];
        self.pickups = [Pickup::empty(); MAX_PICKUPS];
        self.result_summary = ResultSummary::empty();
        self.wave_tracker = WaveTracker::empty();
        self.arena_frame_valid = false;
        self.state = BattleState::Running;
        self.request_persist();
        self.begin_wave(1);
    }

    fn begin_wave(&mut self, wave: u8) {
        let stage_index = self.stage_index();
        let kind = if wave % BOSS_INTERVAL == 0 {
            WaveKind::Boss
        } else if wave % 7 == 0 {
            WaveKind::Elite
        } else if wave % 5 == 0 {
            WaveKind::Pressure
        } else {
            WaveKind::Standard
        };
        let stage_depth = stage_index as u8;
        let wave_depth = (wave.saturating_sub(1)) / 5;
        let total_to_spawn = match kind {
            WaveKind::Standard => 3 + stage_depth + wave_depth,
            WaveKind::Pressure => 5 + stage_depth + wave_depth,
            WaveKind::Elite => 4 + stage_depth + wave_depth,
            WaveKind::Boss => 1,
        };
        let active_cap = match kind {
            WaveKind::Standard => 3 + (stage_depth / 2),
            WaveKind::Pressure => 4 + (stage_depth / 2),
            WaveKind::Elite => 3 + (stage_depth / 2),
            WaveKind::Boss => 1,
        }
        .min(MAX_ENEMIES as u8);
        let spawn_interval_ms = match kind {
            WaveKind::Standard => {
                430u16.saturating_sub(stage_depth as u16 * 22 + wave_depth as u16 * 10)
            }
            WaveKind::Pressure => {
                310u16.saturating_sub(stage_depth as u16 * 18 + wave_depth as u16 * 8)
            }
            WaveKind::Elite => {
                340u16.saturating_sub(stage_depth as u16 * 12 + wave_depth as u16 * 6)
            }
            WaveKind::Boss => 0,
        }
        .max(120);
        let boss_kind = if kind == WaveKind::Boss {
            Some(self.stage_def(stage_index).bosses[((wave / BOSS_INTERVAL) - 1) as usize])
        } else {
            None
        };
        self.wave_tracker = WaveTracker {
            stage_index,
            wave,
            kind,
            total_to_spawn,
            spawned: 0,
            remaining_to_kill: 0,
            active_cap,
            spawn_interval_ms,
            spawn_timer_ms: 0,
            boss_kind,
        };
        self.wave_banner_ms = WAVE_BANNER_MS;
        self.boss_banner_ms = if kind == WaveKind::Boss {
            BOSS_BANNER_MS
        } else {
            0
        };
        self.boss_intro_ms = if kind == WaveKind::Boss {
            BOSS_INTRO_MS
        } else {
            0
        };
        self.spawn_wave_until_cap();
        self.request_redraw(AutoBattleRedraw::Full);
    }

    fn spawn_wave_until_cap(&mut self) {
        while self.wave_tracker.spawned < self.wave_tracker.total_to_spawn
            && self.active_enemy_count() < self.wave_tracker.active_cap as usize
        {
            if !self.spawn_wave_enemy() {
                break;
            }
        }
    }

    fn spawn_wave_enemy(&mut self) -> bool {
        let Some(slot_index) = self.enemies.iter().position(|enemy| !enemy.active) else {
            return false;
        };
        let stage_index = self.wave_tracker.stage_index;
        let wave = self.wave_tracker.wave;
        let spawn = SPAWN_POINTS[(self.next_rand() as usize) % SPAWN_POINTS.len()];
        let phase = (self.next_rand() & 0xFFFF) as u16;
        let tier = stage_index as i16 + (wave as i16 / 6);
        let kind = if let Some(boss_kind) = self.wave_tracker.boss_kind {
            boss_kind
        } else {
            self.roll_stage_enemy_kind(stage_index, self.wave_tracker.kind)
        };
        self.enemies[slot_index] = self.build_enemy(kind, spawn, tier, phase);
        self.wave_tracker.spawned = self.wave_tracker.spawned.saturating_add(1);
        self.wave_tracker.remaining_to_kill = self.wave_tracker.remaining_to_kill.saturating_add(1);
        true
    }

    fn roll_stage_enemy_kind(&mut self, stage_index: usize, wave_kind: WaveKind) -> EnemyKind {
        let stage = self.stage_def(stage_index);
        let roll = (self.next_rand() % 100) as u8;
        match wave_kind {
            WaveKind::Standard => {
                if roll < 38 {
                    stage.enemy_pool[0]
                } else if roll < 64 {
                    stage.enemy_pool[1]
                } else if roll < 84 {
                    stage.enemy_pool[2]
                } else {
                    stage.enemy_pool[3]
                }
            }
            WaveKind::Pressure => {
                if roll < 24 {
                    stage.enemy_pool[0]
                } else if roll < 52 {
                    stage.enemy_pool[1]
                } else if roll < 76 {
                    stage.enemy_pool[2]
                } else {
                    stage.enemy_pool[3]
                }
            }
            WaveKind::Elite => {
                if roll < 20 {
                    EnemyKind::Bruiser
                } else if roll < 42 {
                    stage.enemy_pool[1]
                } else if roll < 68 {
                    stage.enemy_pool[3]
                } else {
                    stage.enemy_pool[2]
                }
            }
            WaveKind::Boss => stage.bosses[0],
        }
    }

    fn build_enemy(&self, kind: EnemyKind, spawn: (f32, f32), tier: i16, phase: u16) -> Enemy {
        let stage_boost = self.stage_index() as i16;
        let (hp, speed, size, attack_timer_ms, charges) = match kind {
            EnemyKind::Runner => (
                2 + tier / 2 + stage_boost / 2,
                0.0155 + stage_boost as f32 * 0.0008 + tier as f32 * 0.0004,
                10 + (stage_boost as u16 / 2).min(2),
                0,
                0,
            ),
            EnemyKind::Shooter => (
                3 + tier / 2 + stage_boost / 2,
                0.0116 + stage_boost as f32 * 0.0005 + tier as f32 * 0.0003,
                11 + (stage_boost as u16 / 2).min(2),
                580 + (phase % 220),
                0,
            ),
            EnemyKind::Bruiser => (
                5 + tier + stage_boost / 2,
                0.0106 + stage_boost as f32 * 0.0004 + tier as f32 * 0.0002,
                14 + (stage_boost as u16).min(4),
                0,
                0,
            ),
            EnemyKind::Dasher => (
                3 + tier / 2 + stage_boost / 2,
                0.0128 + stage_boost as f32 * 0.0006 + tier as f32 * 0.0003,
                10 + (stage_boost as u16 / 2).min(3),
                420 + (phase % 180),
                0,
            ),
            EnemyKind::Summoner => (
                4 + tier / 2 + stage_boost,
                0.0102 + stage_boost as f32 * 0.0003 + tier as f32 * 0.0002,
                12 + (stage_boost as u16 / 2).min(3),
                900 + (phase % 220),
                1 + (stage_boost as u8 / 2).min(1),
            ),
            EnemyKind::BossRam => (18 + tier * 2, 0.0112, 22, 700 + (phase % 180), 0),
            EnemyKind::BossBurst => (20 + tier * 2, 0.0095, 20, 540 + (phase % 140), 0),
            EnemyKind::BossNest => (22 + tier * 2, 0.0088, 22, 860 + (phase % 180), 4),
            EnemyKind::BossRing => (24 + tier * 2, 0.0092, 24, 760 + (phase % 200), 0),
        };

        Enemy {
            active: true,
            kind,
            x: spawn.0,
            y: spawn.1,
            hp,
            max_hp: hp,
            speed,
            size,
            touch_timer_ms: 0,
            attack_timer_ms,
            burst_timer_ms: 0,
            flash_ms: 0,
            phase,
            charges,
        }
    }

    fn active_enemy_count(&self) -> usize {
        self.enemies.iter().filter(|enemy| enemy.active).count()
    }

    fn nearest_enemy_position(&self) -> Option<(f32, f32)> {
        let mut best = None;
        let mut best_dist = f32::MAX;
        for enemy in &self.enemies {
            if !enemy.active {
                continue;
            }
            let dx = enemy.x - self.player_x;
            let dy = enemy.y - self.player_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < best_dist {
                best_dist = dist_sq;
                best = Some((enemy.x, enemy.y));
            }
        }
        best
    }

    fn nearest_enemy_angle(&self) -> Option<(usize, f32)> {
        use libm::atan2f;

        let mut best = None;
        let mut best_dist = f32::MAX;
        for (index, enemy) in self.enemies.iter().enumerate() {
            if !enemy.active {
                continue;
            }
            let dx = enemy.x - self.player_x;
            let dy = enemy.y - self.player_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < best_dist {
                best_dist = dist_sq;
                best = Some((index, atan2f(dy, dx)));
            }
        }
        best
    }

    fn fire_at_nearest_enemy(&mut self) {
        use libm::{cosf, sinf};

        let Some((enemy_index, angle)) = self.nearest_enemy_angle() else {
            return;
        };
        if !self.enemies[enemy_index].active {
            return;
        }

        let shot_count = self.projectile_count.max(1) as i32;
        let spread = 0.15f32;
        let mid = (shot_count - 1) as f32 * 0.5;
        for shot_idx in 0..shot_count {
            let Some(projectile) = self.projectiles.iter_mut().find(|shot| !shot.active) else {
                break;
            };
            let offset = (shot_idx as f32 - mid) * spread;
            let shot_angle = angle + offset;
            projectile.active = true;
            projectile.from_enemy = false;
            projectile.x = self.player_x;
            projectile.y = self.player_y;
            projectile.vx = cosf(shot_angle) * self.projectile_speed;
            projectile.vy = sinf(shot_angle) * self.projectile_speed;
            projectile.damage = self.bullet_damage;
            projectile.pierce_left = self.projectile_pierce;
            projectile.ttl_ms = self.projectile_ttl_ms;
        }
        self.shot_cooldown_ms = self.shot_cooldown_base_ms;
        self.weapon_flash_ms = WEAPON_FLASH_MS;
    }

    fn spawn_enemy_projectile(&mut self, x: f32, y: f32, angle: f32, damage: i16, speed: f32) {
        use libm::{cosf, sinf};

        let Some(projectile) = self.projectiles.iter_mut().find(|shot| !shot.active) else {
            return;
        };

        projectile.active = true;
        projectile.from_enemy = true;
        projectile.x = x;
        projectile.y = y;
        projectile.vx = cosf(angle) * speed;
        projectile.vy = sinf(angle) * speed;
        projectile.damage = damage;
        projectile.pierce_left = 0;
        projectile.ttl_ms = 1_300;
    }

    fn spawn_summoned_runner(&mut self, x: f32, y: f32, phase: u16, boss_summon: bool) -> bool {
        use libm::{cosf, sinf};

        let Some(slot_index) = self.enemies.iter().position(|enemy| !enemy.active) else {
            return false;
        };

        let angle = (phase as f32 * 0.024) + ((self.next_rand() & 0x1F) as f32 * 0.08);
        let offset = if boss_summon { 24.0 } else { 18.0 };
        let spawn = (
            (x + cosf(angle) * offset).clamp(14.0, ARENA_W as f32 - 14.0),
            (y + sinf(angle) * offset).clamp(14.0, ARENA_H as f32 - 14.0),
        );
        let tier = self.stage_index() as i16 + self.wave_tracker.wave as i16 / 8;
        self.enemies[slot_index] = self.build_enemy(EnemyKind::Runner, spawn, tier, phase);
        self.enemies[slot_index].phase = phase.wrapping_add(111);
        self.enemies[slot_index].flash_ms = 90;
        self.wave_tracker.remaining_to_kill = self.wave_tracker.remaining_to_kill.saturating_add(1);
        true
    }

    fn roll_reward_choices(&mut self) {
        let mut selected = [BuffKind::MultiShot; LEVEL_UP_CHOICES];
        let mut count = 0usize;
        while count < LEVEL_UP_CHOICES {
            let candidate = ALL_BUFFS[(self.next_rand() as usize) % ALL_BUFFS.len()];
            if selected[..count].contains(&candidate) {
                continue;
            }
            selected[count] = candidate;
            count += 1;
        }
        self.reward_choices = selected;
    }

    fn apply_buff(&mut self, buff: BuffKind) {
        self.buff_counts[buff.index()] = self.buff_counts[buff.index()].saturating_add(1);
        match buff {
            BuffKind::MultiShot => {
                self.projectile_count = (self.projectile_count + 1).min(4);
            }
            BuffKind::VitalCore => {
                self.max_health += 2;
                self.health = (self.health + 2).min(self.max_health);
            }
            BuffKind::Impact => {
                self.bullet_damage += 1;
            }
            BuffKind::QuickTrigger => {
                self.shot_cooldown_base_ms = self.shot_cooldown_base_ms.saturating_sub(38).max(100);
            }
            BuffKind::Velocity => {
                self.projectile_speed += 0.028;
            }
            BuffKind::Thrusters => {
                self.player_speed = (self.player_speed + 0.011).min(0.15);
            }
            BuffKind::PhaseRound => {
                self.projectile_pierce = (self.projectile_pierce + 1).min(3);
            }
            BuffKind::LongBarrel => {
                self.projectile_ttl_ms = (self.projectile_ttl_ms + 160).min(1_700);
            }
            BuffKind::GuardShell => {
                self.hit_invuln_base_ms = (self.hit_invuln_base_ms + 110).min(920);
                self.health = (self.health + 1).min(self.max_health);
            }
        }
        let next_wave = self.wave_tracker.wave.saturating_add(1);
        self.state = BattleState::Running;
        self.begin_wave(next_wave);
        self.spawn_wave_medkit();
    }

    fn complete_stage(&mut self) {
        let stage_index = self.stage_index();
        let stage_number = stage_index as u8 + 1;
        let first_clear = self.profile.stage_clear_count[stage_index] == 0;
        self.profile.stage_clear_count[stage_index] =
            self.profile.stage_clear_count[stage_index].saturating_add(1);
        self.profile.stage_best_wave[stage_index] = WAVES_PER_STAGE;
        self.profile.stage_best_kills[stage_index] =
            self.profile.stage_best_kills[stage_index].max(self.kills);
        self.profile.best_kills = self.profile.best_kills.max(self.kills);

        let xp_gain = 18 + stage_number as u16 * 10;
        self.profile.player_xp = self.profile.player_xp.saturating_add(xp_gain);
        let mut levels_gained = 0u8;
        while self.profile.player_xp >= Self::xp_to_next_level(self.profile.player_level) {
            self.profile.player_xp = self
                .profile
                .player_xp
                .saturating_sub(Self::xp_to_next_level(self.profile.player_level));
            self.profile.player_level = self.profile.player_level.saturating_add(1);
            self.profile.upgrade_points = self.profile.upgrade_points.saturating_add(1);
            levels_gained = levels_gained.saturating_add(1);
        }
        self.profile.upgrade_points = self.profile.upgrade_points.saturating_add(1);

        let mut unlocked = None;
        if first_clear && self.profile.unlocked_stage < STAGE_COUNT as u8 {
            self.profile.unlocked_stage = self.profile.unlocked_stage.saturating_add(1);
            unlocked = Some(self.profile.unlocked_stage);
        }

        self.result_summary = ResultSummary {
            stage: stage_number,
            wave_reached: WAVES_PER_STAGE,
            kills: self.kills,
            xp_gain,
            level_gained: levels_gained,
            upgrade_points_gain: 1 + levels_gained,
            unlocked_stage: unlocked,
        };
        self.state = BattleState::StageClear;
        self.request_persist();
        self.request_redraw(AutoBattleRedraw::Full);
    }

    fn fail_stage(&mut self) {
        let stage_index = self.stage_index();
        self.profile.stage_best_wave[stage_index] =
            self.profile.stage_best_wave[stage_index].max(self.wave_tracker.wave);
        self.profile.stage_best_kills[stage_index] =
            self.profile.stage_best_kills[stage_index].max(self.kills);
        self.profile.best_kills = self.profile.best_kills.max(self.kills);
        self.result_summary = ResultSummary {
            stage: self.current_stage,
            wave_reached: self.wave_tracker.wave,
            kills: self.kills,
            xp_gain: 0,
            level_gained: 0,
            upgrade_points_gain: 0,
            unlocked_stage: None,
        };
        self.state = BattleState::Defeat;
        self.request_persist();
        self.request_redraw(AutoBattleRedraw::Full);
    }

    fn panel_snapshot(&self) -> PanelSnapshot {
        let (boss_hp, boss_max_hp) = self.active_boss_stats().unwrap_or((0, 0));
        PanelSnapshot {
            moving: self.moving,
            health: self.health,
            max_health: self.max_health,
            kills: self.kills,
            stage: self.current_stage,
            wave: self.wave_tracker.wave,
            remaining: self.wave_tracker.remaining_to_kill,
            best_kills: self.profile.best_kills,
            wave_kind: self.wave_tracker.kind,
            medkit_active: self.pickups.iter().any(|pickup| pickup.active),
            boss_hp,
            boss_max_hp,
        }
    }

    fn active_boss_stats(&self) -> Option<(i16, i16)> {
        self.enemies
            .iter()
            .find(|enemy| enemy.active && enemy.kind.is_boss())
            .map(|enemy| (enemy.hp.max(0), enemy.max_hp.max(1)))
    }

    fn active_boss_kind(&self) -> Option<EnemyKind> {
        self.enemies
            .iter()
            .find(|enemy| enemy.active && enemy.kind.is_boss())
            .map(|enemy| enemy.kind)
            .or(self.wave_tracker.boss_kind)
    }

    fn banner_active(&self) -> bool {
        self.wave_banner_ms > 0 || self.boss_banner_ms > 0
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

        let mut pickups = [PickupFrame::empty(); MAX_PICKUPS];
        for (index, pickup) in self.pickups.iter().enumerate() {
            if pickup.active {
                pickups[index] = PickupFrame {
                    active: true,
                    x: ARENA_X as i16 + pickup.x as i16,
                    y: ARENA_Y as i16 + pickup.y as i16,
                    size: pickup.size as i16,
                };
            }
        }

        ArenaFrame {
            player_x: ARENA_X as i16 + self.player_x as i16,
            player_y: ARENA_Y as i16 + self.player_y as i16,
            target_x: ARENA_X as i16 + self.target_x as i16,
            target_y: ARENA_Y as i16 + self.target_y as i16,
            moving: self.moving,
            banner_active: self.banner_active(),
            nearest_enemy: if self.moving {
                None
            } else {
                self.nearest_enemy_position()
                    .map(|(x, y)| (ARENA_X as i16 + x as i16, ARENA_Y as i16 + y as i16))
            },
            enemies,
            projectiles,
            pickups,
        }
    }

    fn clear_pickups(&mut self) {
        self.pickups = [Pickup::empty(); MAX_PICKUPS];
    }

    fn spawn_wave_medkit(&mut self) {
        self.clear_pickups();
        let slot = 0;
        let stage_offset = self.stage_index() as f32 * 8.0;
        let x = 44.0 + ((self.next_rand() % 180) as f32) + stage_offset.min(18.0);
        let y = 52.0 + ((self.next_rand() % 108) as f32);
        self.pickups[slot] = Pickup {
            active: true,
            kind: PickupKind::MedKit,
            x: x.clamp(24.0, ARENA_W as f32 - 24.0),
            y: y.clamp(24.0, ARENA_H as f32 - 24.0),
            size: 12,
        };
    }
}
