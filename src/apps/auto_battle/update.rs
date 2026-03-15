use libm::{atan2f, cosf, sinf, sqrtf};

use crate::board::ButtonSnapshot;
use crate::touch::TouchState;
use crate::ui::{NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y};

use super::super::{touch_active_in_rect, touch_released_in_rect};
use super::*;

impl AutoBattleApp {
    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> AutoBattleAction {
        if input.home_chord()
            || input.k0_just_pressed
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            return AutoBattleAction::ExitGameCenter;
        }

        match self.state {
            BattleState::Ready | BattleState::Victory | BattleState::Defeat => {
                if input.k1_just_pressed
                    || touch_released_in_rect(
                        touch,
                        OVERLAY_ACTION_X,
                        OVERLAY_ACTION_Y,
                        OVERLAY_ACTION_W,
                        OVERLAY_ACTION_H,
                    )
                {
                    self.reset();
                }
            }
            BattleState::LevelUp => self.update_level_up(input, touch),
            BattleState::Running => self.update_running(touch, dt_ms),
        }

        AutoBattleAction::Stay
    }

    fn update_running(&mut self, touch: &TouchState, dt_ms: u32) {
        let panel_before = self.panel_snapshot();
        self.hit_invuln_ms = self.hit_invuln_ms.saturating_sub(dt_ms as u16);
        self.shot_cooldown_ms = self.shot_cooldown_ms.saturating_sub(dt_ms as u16);

        if touch_active_in_rect(touch, ARENA_X, ARENA_Y, ARENA_W, ARENA_H) {
            self.target_x = (touch.x - ARENA_X).clamp(10, ARENA_W - 10) as f32;
            self.target_y = (touch.y - ARENA_Y).clamp(10, ARENA_H - 10) as f32;
            self.moving = true;
        } else {
            self.moving = false;
        }

        if self.moving {
            let dx = self.target_x - self.player_x;
            let dy = self.target_y - self.player_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > 9.0 {
                let dist = sqrtf(dist_sq).max(0.001);
                let step = self.player_speed * dt_ms as f32;
                self.player_x =
                    (self.player_x + dx / dist * step).clamp(16.0, ARENA_W as f32 - 16.0);
                self.player_y =
                    (self.player_y + dy / dist * step).clamp(16.0, ARENA_H as f32 - 16.0);
            }
        } else if self.shot_cooldown_ms == 0 {
            self.fire_at_nearest_enemy();
        }

        self.update_enemies(dt_ms as u16);
        self.update_projectiles(dt_ms as u16);

        if self.health <= 0 {
            self.health = 0;
            self.sync_best_kills();
            self.state = BattleState::Defeat;
            self.request_redraw(AutoBattleRedraw::Full);
            return;
        }

        if self.kills >= TARGET_KILLS {
            self.sync_best_kills();
            self.state = BattleState::Victory;
            self.request_redraw(AutoBattleRedraw::Full);
            return;
        }

        if self.kills >= self.next_level_kills {
            self.roll_level_up_choices();
            self.selected_choice = 0;
            self.projectiles = [Projectile::empty(); MAX_PROJECTILES];
            self.shot_cooldown_ms = 0;
            self.state = BattleState::LevelUp;
            self.request_redraw(AutoBattleRedraw::Full);
            return;
        }

        if self.panel_snapshot() != panel_before {
            self.request_redraw(AutoBattleRedraw::ArenaAndPanel);
        } else {
            self.request_redraw(AutoBattleRedraw::Arena);
        }
    }

    fn update_level_up(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        let previous_choice = self.selected_choice;

        if input.k0_just_pressed {
            self.selected_choice = (self.selected_choice + LEVEL_UP_CHOICES - 1) % LEVEL_UP_CHOICES;
        }
        if input.wkup_just_pressed {
            self.selected_choice = (self.selected_choice + 1) % LEVEL_UP_CHOICES;
        }
        if input.k1_just_pressed {
            let buff = self.level_up_choices[self.selected_choice];
            self.apply_buff(buff);
            return;
        }

        for index in 0..LEVEL_UP_CHOICES {
            let y = BUFF_Y + index as u16 * (BUFF_H + BUFF_GAP);
            if touch_released_in_rect(touch, BUFF_X, y, BUFF_W, BUFF_H) {
                self.selected_choice = index;
                let buff = self.level_up_choices[index];
                self.apply_buff(buff);
                return;
            }
        }

        if self.selected_choice != previous_choice {
            self.request_redraw(AutoBattleRedraw::Overlay);
        }
    }

    fn update_enemies(&mut self, dt_ms: u16) {
        let mut queued_shots = [None; MAX_ENEMIES];
        let mut queued_shot_count = 0usize;
        let mut queued_summons = [None; MAX_ENEMIES];
        let mut queued_summon_count = 0usize;

        for enemy in &mut self.enemies {
            if !enemy.active {
                continue;
            }

            enemy.phase = enemy.phase.wrapping_add(dt_ms);
            enemy.flash_ms = enemy.flash_ms.saturating_sub(dt_ms);
            enemy.touch_timer_ms = enemy.touch_timer_ms.saturating_sub(dt_ms);
            enemy.attack_timer_ms = enemy.attack_timer_ms.saturating_sub(dt_ms);
            enemy.burst_timer_ms = enemy.burst_timer_ms.saturating_sub(dt_ms);

            let dx = self.player_x - enemy.x;
            let dy = self.player_y - enemy.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > 0.25 {
                let dist = sqrtf(dist_sq).max(0.001);
                let dir_x = dx / dist;
                let dir_y = dy / dist;
                let perp_x = -dir_y;
                let perp_y = dir_x;

                let (move_x, move_y) = match enemy.kind {
                    EnemyKind::Runner => {
                        let sway = sinf(enemy.phase as f32 * 0.021) * 0.18;
                        (dir_x + perp_x * sway, dir_y + perp_y * sway)
                    }
                    EnemyKind::Shooter => {
                        let preferred_dist = 72.0;
                        let advance = if dist > preferred_dist + 10.0 {
                            0.68
                        } else if dist < preferred_dist - 8.0 {
                            -0.44
                        } else {
                            0.0
                        };
                        let strafe = sinf(enemy.phase as f32 * 0.013) * 0.74;
                        if enemy.attack_timer_ms == 0
                            && dist < 124.0
                            && queued_shot_count < MAX_ENEMIES
                        {
                            queued_shots[queued_shot_count] =
                                Some((enemy.x, enemy.y, atan2f(dy, dx)));
                            queued_shot_count += 1;
                            enemy.attack_timer_ms = 980 + (enemy.phase % 220);
                        }
                        (
                            dir_x * advance + perp_x * strafe,
                            dir_y * advance + perp_y * strafe,
                        )
                    }
                    EnemyKind::Bruiser => {
                        let sway = sinf(enemy.phase as f32 * 0.009) * 0.05;
                        (dir_x + perp_x * sway, dir_y + perp_y * sway)
                    }
                    EnemyKind::Dasher => {
                        if enemy.burst_timer_ms == 0
                            && enemy.attack_timer_ms == 0
                            && dist > 28.0
                            && dist < 138.0
                        {
                            enemy.burst_timer_ms = 160;
                            enemy.attack_timer_ms = 940 + (enemy.phase % 180);
                        }

                        if enemy.burst_timer_ms > 0 {
                            (dir_x * 2.8, dir_y * 2.8)
                        } else {
                            let sway = sinf(enemy.phase as f32 * 0.024) * 0.28;
                            (dir_x * 0.64 + perp_x * sway, dir_y * 0.64 + perp_y * sway)
                        }
                    }
                    EnemyKind::Summoner => {
                        let preferred_dist = 96.0;
                        let advance = if dist > preferred_dist + 14.0 {
                            0.46
                        } else if dist < preferred_dist - 10.0 {
                            -0.32
                        } else {
                            0.0
                        };
                        let orbit = sinf(enemy.phase as f32 * 0.010) * 0.52;
                        if enemy.attack_timer_ms == 0 && queued_summon_count < MAX_ENEMIES {
                            queued_summons[queued_summon_count] =
                                Some((enemy.x, enemy.y, enemy.phase));
                            queued_summon_count += 1;
                            enemy.attack_timer_ms = 1_800 + (enemy.phase % 320);
                        }
                        (
                            dir_x * advance + perp_x * orbit,
                            dir_y * advance + perp_y * orbit,
                        )
                    }
                };

                let move_len_sq = move_x * move_x + move_y * move_y;
                if move_len_sq > 0.001 {
                    let move_len = sqrtf(move_len_sq);
                    let step = enemy.speed * dt_ms as f32;
                    enemy.x =
                        (enemy.x + move_x / move_len * step).clamp(14.0, ARENA_W as f32 - 14.0);
                    enemy.y =
                        (enemy.y + move_y / move_len * step).clamp(14.0, ARENA_H as f32 - 14.0);
                }
            }

            let touch_radius = (enemy.size as f32 * 0.52 + PLAYER_SIZE as f32 * 0.45).max(11.0);
            if dist_sq < touch_radius * touch_radius
                && enemy.touch_timer_ms == 0
                && self.hit_invuln_ms == 0
            {
                self.health -= 1;
                self.hit_invuln_ms = self.hit_invuln_base_ms;
                enemy.touch_timer_ms = ENEMY_TOUCH_DAMAGE_MS;
            }
        }

        for shot in queued_shots.into_iter().flatten() {
            self.spawn_enemy_projectile(shot.0, shot.1, shot.2);
        }
        for summon in queued_summons.into_iter().flatten() {
            self.spawn_summoned_runner(summon.0, summon.1, summon.2);
        }
    }

    fn update_projectiles(&mut self, dt_ms: u16) {
        let mut respawns = 0u8;

        for projectile in &mut self.projectiles {
            if !projectile.active {
                continue;
            }
            projectile.ttl_ms = projectile.ttl_ms.saturating_sub(dt_ms);
            projectile.x += projectile.vx * dt_ms as f32;
            projectile.y += projectile.vy * dt_ms as f32;
            if projectile.ttl_ms == 0
                || projectile.x < 8.0
                || projectile.y < 8.0
                || projectile.x > ARENA_W as f32 - 8.0
                || projectile.y > ARENA_H as f32 - 8.0
            {
                projectile.active = false;
            }
        }

        for projectile in &mut self.projectiles {
            if !projectile.active {
                continue;
            }

            if projectile.from_enemy {
                let dx = projectile.x - self.player_x;
                let dy = projectile.y - self.player_y;
                if dx * dx + dy * dy < 72.0 {
                    projectile.active = false;
                    if self.hit_invuln_ms == 0 {
                        self.health -= projectile.damage;
                        self.hit_invuln_ms = self.hit_invuln_base_ms;
                    }
                }
                continue;
            }

            for enemy in &mut self.enemies {
                if !enemy.active {
                    continue;
                }

                let dx = projectile.x - enemy.x;
                let dy = projectile.y - enemy.y;
                let hit_radius = (enemy.size as f32 * 0.7).max(8.0);
                if dx * dx + dy * dy < hit_radius * hit_radius {
                    enemy.hp -= projectile.damage;
                    enemy.flash_ms = 120;
                    if projectile.pierce_left == 0 {
                        projectile.active = false;
                    } else {
                        projectile.pierce_left -= 1;
                    }
                    if enemy.hp <= 0 {
                        enemy.active = false;
                        self.kills = self.kills.saturating_add(1);
                        respawns = respawns.saturating_add(1);
                    }
                    break;
                }
            }
        }

        self.sync_best_kills();

        for _ in 0..respawns {
            self.try_spawn_replacement();
        }
    }

    fn fire_at_nearest_enemy(&mut self) {
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
    }

    fn try_spawn_replacement(&mut self) {
        if self.kills >= TARGET_KILLS || matches!(self.state, BattleState::LevelUp) {
            return;
        }
        let Some(slot_index) = self.enemies.iter().position(|enemy| !enemy.active) else {
            return;
        };

        let spawn = SPAWN_POINTS[(self.next_rand() as usize) % SPAWN_POINTS.len()];
        let tier = (self.kills / 20) as i16;
        let phase = (self.next_rand() & 0xFFFF) as u16;
        let kind = self.roll_enemy_kind();
        self.enemies[slot_index] = self.build_enemy(kind, spawn, tier, phase);
    }

    pub(super) fn spawn_opening_wave(&mut self) {
        let opening_kinds = [
            EnemyKind::Runner,
            EnemyKind::Runner,
            EnemyKind::Bruiser,
            EnemyKind::Runner,
            EnemyKind::Shooter,
            EnemyKind::Dasher,
            EnemyKind::Bruiser,
            EnemyKind::Summoner,
        ];
        for index in 0..self.enemies.len() {
            let spawn = SPAWN_POINTS[index];
            self.enemies[index] =
                self.build_enemy(opening_kinds[index], spawn, 0, (index as u16) * 73);
        }
    }

    fn spawn_enemy_projectile(&mut self, x: f32, y: f32, angle: f32) {
        let Some(projectile) = self.projectiles.iter_mut().find(|shot| !shot.active) else {
            return;
        };

        projectile.active = true;
        projectile.from_enemy = true;
        projectile.x = x;
        projectile.y = y;
        projectile.vx = cosf(angle) * ENEMY_PROJECTILE_SPEED;
        projectile.vy = sinf(angle) * ENEMY_PROJECTILE_SPEED;
        projectile.damage = 1;
        projectile.pierce_left = 0;
        projectile.ttl_ms = 1_200;
    }

    fn spawn_summoned_runner(&mut self, x: f32, y: f32, phase: u16) {
        let Some(slot_index) = self.enemies.iter().position(|enemy| !enemy.active) else {
            return;
        };

        let angle = (phase as f32 * 0.024) + ((self.next_rand() & 0x1F) as f32 * 0.08);
        let offset = 18.0 + (self.next_rand() & 0x7) as f32;
        let spawn = (
            (x + cosf(angle) * offset).clamp(14.0, ARENA_W as f32 - 14.0),
            (y + sinf(angle) * offset).clamp(14.0, ARENA_H as f32 - 14.0),
        );
        self.enemies[slot_index] =
            self.build_enemy(EnemyKind::Runner, spawn, self.kills as i16 / 24, phase);
        self.enemies[slot_index].phase = phase.wrapping_add(111);
        self.enemies[slot_index].flash_ms = 90;
    }

    fn roll_enemy_kind(&mut self) -> EnemyKind {
        let roll = (self.next_rand() % 100) as u8;
        if self.level < 3 {
            if roll < 42 {
                EnemyKind::Runner
            } else if roll < 66 {
                EnemyKind::Bruiser
            } else if roll < 88 {
                EnemyKind::Shooter
            } else {
                EnemyKind::Dasher
            }
        } else if self.level < 6 {
            if roll < 32 {
                EnemyKind::Runner
            } else if roll < 54 {
                EnemyKind::Bruiser
            } else if roll < 74 {
                EnemyKind::Shooter
            } else if roll < 90 {
                EnemyKind::Dasher
            } else {
                EnemyKind::Summoner
            }
        } else if roll < 24 {
            EnemyKind::Runner
        } else if roll < 44 {
            EnemyKind::Bruiser
        } else if roll < 64 {
            EnemyKind::Shooter
        } else if roll < 82 {
            EnemyKind::Dasher
        } else {
            EnemyKind::Summoner
        }
    }

    fn build_enemy(&self, kind: EnemyKind, spawn: (f32, f32), tier: i16, phase: u16) -> Enemy {
        let level_boost = self.level as i16 / 3;
        let (hp, speed, size, attack_timer_ms) = match kind {
            EnemyKind::Runner => (
                2 + level_boost + tier / 2,
                0.0155 + self.level as f32 * 0.0008 + tier as f32 * 0.0005,
                10 + (self.level as u16 / 4).min(2),
                0,
            ),
            EnemyKind::Shooter => (
                3 + level_boost + tier / 2,
                0.0118 + self.level as f32 * 0.0005 + tier as f32 * 0.0003,
                11 + (self.level as u16 / 5).min(2),
                620 + (phase % 260),
            ),
            EnemyKind::Bruiser => (
                5 + level_boost + tier,
                0.0108 + self.level as f32 * 0.0004 + tier as f32 * 0.0003,
                14 + ((self.level as u16 + self.kills / 28) / 5).min(4),
                0,
            ),
            EnemyKind::Dasher => (
                3 + level_boost + tier / 2,
                0.0126 + self.level as f32 * 0.0006 + tier as f32 * 0.0004,
                10 + (self.level as u16 / 4).min(3),
                420 + (phase % 220),
            ),
            EnemyKind::Summoner => (
                4 + level_boost + tier,
                0.0102 + self.level as f32 * 0.0003 + tier as f32 * 0.0002,
                12 + (self.level as u16 / 6).min(3),
                1_100 + (phase % 320),
            ),
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
        }
    }

    pub(super) fn nearest_enemy_position(&self) -> Option<(f32, f32)> {
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

    fn roll_level_up_choices(&mut self) {
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
        self.level_up_choices = selected;
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
                self.shot_cooldown_base_ms = self.shot_cooldown_base_ms.saturating_sub(40).max(110);
            }
            BuffKind::Velocity => {
                self.projectile_speed += 0.028;
            }
            BuffKind::Thrusters => {
                self.player_speed = (self.player_speed + 0.011).min(0.145);
            }
            BuffKind::PhaseRound => {
                self.projectile_pierce = (self.projectile_pierce + 1).min(3);
            }
            BuffKind::LongBarrel => {
                self.projectile_ttl_ms = (self.projectile_ttl_ms + 160).min(1_600);
            }
            BuffKind::GuardShell => {
                self.hit_invuln_base_ms = (self.hit_invuln_base_ms + 110).min(900);
                self.health = (self.health + 1).min(self.max_health);
            }
        }

        self.level = self.level.saturating_add(1);
        self.next_level_kills = self.next_level_kills.saturating_add(KILLS_PER_LEVEL);
        self.state = BattleState::Running;
        self.request_redraw(AutoBattleRedraw::Full);
    }
}
