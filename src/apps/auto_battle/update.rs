use libm::{atan2f, sqrtf};

use crate::board::ButtonSnapshot;
use crate::touch::TouchState;
use crate::ui::{NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y};

use super::super::{touch_active_in_rect, touch_released_in_rect};
use super::*;

const RESULT_BUTTON_GAP: u16 = 8;

impl AutoBattleApp {
    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> AutoBattleAction {
        if input.home_chord() {
            return AutoBattleAction::ExitGameCenter;
        }

        match self.state {
            BattleState::Profile => return self.update_profile(input, touch),
            BattleState::StageSelect => return self.update_stage_select(input, touch),
            BattleState::Running => {
                if input.k0_just_pressed
                    || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    return AutoBattleAction::ExitGameCenter;
                }
                self.update_running(touch, dt_ms);
            }
            BattleState::BossReward => {
                if input.k0_just_pressed
                    || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    return AutoBattleAction::ExitGameCenter;
                }
                self.update_reward(input, touch);
            }
            BattleState::StageClear | BattleState::Defeat => {
                if input.k0_just_pressed
                    || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    self.enter();
                    return AutoBattleAction::Stay;
                }
                self.update_result(input, touch);
            }
        }

        AutoBattleAction::Stay
    }

    fn update_profile(&mut self, input: &ButtonSnapshot, touch: &TouchState) -> AutoBattleAction {
        if input.k0_just_pressed {
            self.profile_cursor =
                (self.profile_cursor + ProfileAction::ALL.len() - 1) % ProfileAction::ALL.len();
            self.request_redraw(AutoBattleRedraw::Full);
        }
        if input.wkup_just_pressed {
            self.profile_cursor = (self.profile_cursor + 1) % ProfileAction::ALL.len();
            self.request_redraw(AutoBattleRedraw::Full);
        }
        if input.k1_just_pressed {
            self.activate_profile_action(ProfileAction::ALL[self.profile_cursor]);
        }

        if touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H) {
            return AutoBattleAction::ExitGameCenter;
        }

        for (index, action) in ProfileAction::ALL.iter().copied().enumerate() {
            let (x, y, w, h) = profile_action_rect(action);
            if touch_released_in_rect(touch, x, y, w, h) {
                if self.profile_cursor == index {
                    self.activate_profile_action(action);
                } else {
                    self.profile_cursor = index;
                    self.request_redraw(AutoBattleRedraw::Full);
                }
                break;
            }
        }

        AutoBattleAction::Stay
    }

    fn activate_profile_action(&mut self, action: ProfileAction) {
        match action {
            ProfileAction::Deploy => self.open_stage_select(),
            _ => self.spend_profile_upgrade(action),
        }
    }

    fn update_stage_select(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
    ) -> AutoBattleAction {
        if input.k0_just_pressed {
            self.stage_select_index = (self.stage_select_index + STAGE_COUNT - 1) % STAGE_COUNT;
            self.request_redraw(AutoBattleRedraw::Full);
        }
        if input.wkup_just_pressed {
            self.stage_select_index = (self.stage_select_index + 1) % STAGE_COUNT;
            self.request_redraw(AutoBattleRedraw::Full);
        }
        if input.k1_just_pressed && self.is_stage_unlocked(self.stage_select_index) {
            self.start_selected_stage();
        }

        if touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H) {
            self.enter();
            return AutoBattleAction::Stay;
        }

        for index in 0..STAGE_COUNT {
            let y = STAGE_CARD_Y + index as u16 * (STAGE_CARD_H + STAGE_CARD_GAP);
            if touch_released_in_rect(touch, STAGE_CARD_X, y, STAGE_CARD_W, STAGE_CARD_H) {
                if self.stage_select_index == index && self.is_stage_unlocked(index) {
                    self.start_selected_stage();
                } else {
                    self.stage_select_index = index;
                    self.request_redraw(AutoBattleRedraw::Full);
                }
                break;
            }
        }

        AutoBattleAction::Stay
    }

    fn update_result(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        if input.k0_just_pressed || input.wkup_just_pressed {
            self.result_choice = (self.result_choice + 1) % 2;
            self.request_redraw(AutoBattleRedraw::Full);
        }
        if input.k1_just_pressed {
            self.apply_result_choice();
            return;
        }

        for index in 0..2 {
            let y = RESULT_BUTTON_Y + index as u16 * (RESULT_BUTTON_H + RESULT_BUTTON_GAP);
            if touch_released_in_rect(touch, RESULT_BUTTON_X, y, RESULT_BUTTON_W, RESULT_BUTTON_H) {
                if self.result_choice == index {
                    self.apply_result_choice();
                } else {
                    self.result_choice = index;
                    self.request_redraw(AutoBattleRedraw::Full);
                }
                break;
            }
        }
    }

    fn apply_result_choice(&mut self) {
        match self.state {
            BattleState::StageClear => {
                if self.result_choice == 0 {
                    self.enter();
                } else {
                    self.open_stage_select();
                }
            }
            BattleState::Defeat => {
                if self.result_choice == 0 {
                    self.start_selected_stage();
                } else {
                    self.enter();
                }
            }
            _ => {}
        }
    }

    fn update_reward(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        let previous_choice = self.selected_choice;
        if input.k0_just_pressed {
            self.selected_choice = (self.selected_choice + LEVEL_UP_CHOICES - 1) % LEVEL_UP_CHOICES;
        }
        if input.wkup_just_pressed {
            self.selected_choice = (self.selected_choice + 1) % LEVEL_UP_CHOICES;
        }
        if input.k1_just_pressed {
            let buff = self.reward_choices[self.selected_choice];
            self.apply_buff(buff);
            return;
        }

        for index in 0..LEVEL_UP_CHOICES {
            let y = BUFF_Y + index as u16 * (BUFF_H + BUFF_GAP);
            if touch_released_in_rect(touch, BUFF_X, y, BUFF_W, BUFF_H) {
                self.selected_choice = index;
                let buff = self.reward_choices[index];
                self.apply_buff(buff);
                return;
            }
        }

        if self.selected_choice != previous_choice {
            self.request_redraw(AutoBattleRedraw::Full);
        }
    }

    fn update_running(&mut self, touch: &TouchState, dt_ms: u32) {
        let panel_before = self.panel_snapshot();
        self.hit_invuln_ms = self.hit_invuln_ms.saturating_sub(dt_ms as u16);
        self.shot_cooldown_ms = self.shot_cooldown_ms.saturating_sub(dt_ms as u16);
        self.damage_flash_ms = self.damage_flash_ms.saturating_sub(dt_ms as u16);
        self.heal_flash_ms = self.heal_flash_ms.saturating_sub(dt_ms as u16);
        self.weapon_flash_ms = self.weapon_flash_ms.saturating_sub(dt_ms as u16);
        self.wave_banner_ms = self.wave_banner_ms.saturating_sub(dt_ms as u16);
        self.boss_banner_ms = self.boss_banner_ms.saturating_sub(dt_ms as u16);
        self.boss_intro_ms = self.boss_intro_ms.saturating_sub(dt_ms as u16);

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

        self.update_pickups();
        if self.boss_intro_ms > 0 {
            if self.panel_snapshot() != panel_before {
                self.request_redraw(AutoBattleRedraw::ArenaAndPanel);
            } else {
                self.request_redraw(AutoBattleRedraw::Arena);
            }
            return;
        }

        self.update_enemies(dt_ms as u16);
        self.update_projectiles(dt_ms as u16);
        self.update_wave_spawning(dt_ms as u16);

        if self.health <= 0 {
            self.health = 0;
            self.fail_stage();
            return;
        }

        if self.wave_tracker.remaining_to_kill == 0 && self.active_enemy_count() == 0 {
            if self.wave_tracker.wave >= WAVES_PER_STAGE {
                self.complete_stage();
                return;
            }

            self.roll_reward_choices();
            self.selected_choice = 0;
            self.projectiles = [Projectile::empty(); MAX_PROJECTILES];
            self.state = BattleState::BossReward;
            self.request_redraw(AutoBattleRedraw::Full);
            return;
        }

        let arena_fx_active = self.damage_flash_ms > 0
            || self.heal_flash_ms > 0
            || self.weapon_flash_ms > 0
            || self.banner_active();

        if self.panel_snapshot() != panel_before {
            self.request_redraw(AutoBattleRedraw::ArenaAndPanel);
        } else if arena_fx_active {
            self.request_redraw(AutoBattleRedraw::Arena);
        } else {
            self.request_redraw(AutoBattleRedraw::Arena);
        }
    }

    fn update_wave_spawning(&mut self, dt_ms: u16) {
        if self.wave_tracker.kind == WaveKind::Boss {
            return;
        }
        if self.wave_tracker.spawned >= self.wave_tracker.total_to_spawn {
            return;
        }
        if self.active_enemy_count() >= self.wave_tracker.active_cap as usize {
            return;
        }

        self.wave_tracker.spawn_timer_ms = self.wave_tracker.spawn_timer_ms.saturating_add(dt_ms);
        while self.wave_tracker.spawn_timer_ms >= self.wave_tracker.spawn_interval_ms
            && self.wave_tracker.spawned < self.wave_tracker.total_to_spawn
            && self.active_enemy_count() < self.wave_tracker.active_cap as usize
        {
            self.wave_tracker.spawn_timer_ms = self
                .wave_tracker
                .spawn_timer_ms
                .saturating_sub(self.wave_tracker.spawn_interval_ms);
            if !self.spawn_wave_enemy() {
                break;
            }
        }
    }

    fn update_pickups(&mut self) {
        let mut changed = false;
        for pickup in &mut self.pickups {
            if !pickup.active {
                continue;
            }
            let dx = pickup.x - self.player_x;
            let dy = pickup.y - self.player_y;
            let radius = (pickup.size as f32 * 0.5 + PLAYER_SIZE as f32 * 0.5).max(12.0);
            if dx * dx + dy * dy <= radius * radius {
                pickup.active = false;
                match pickup.kind {
                    PickupKind::MedKit => {
                        self.health = (self.health + PICKUP_HEAL_AMOUNT).min(self.max_health);
                        self.heal_flash_ms = HEAL_FLASH_MS;
                    }
                }
                changed = true;
            }
        }
        if changed {
            self.request_redraw(AutoBattleRedraw::ArenaAndPanel);
        }
    }

    fn update_enemies(&mut self, dt_ms: u16) {
        use libm::sinf;

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
                                Some((enemy.kind, enemy.x, enemy.y, atan2f(dy, dx)));
                            queued_shot_count += 1;
                            enemy.attack_timer_ms = 920 + (enemy.phase % 180);
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
                            enemy.attack_timer_ms = 920 + (enemy.phase % 180);
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
                        if enemy.attack_timer_ms == 0
                            && enemy.charges > 0
                            && queued_summon_count < MAX_ENEMIES
                        {
                            queued_summons[queued_summon_count] =
                                Some((enemy.x, enemy.y, enemy.phase, false));
                            queued_summon_count += 1;
                            enemy.attack_timer_ms = 1_500 + (enemy.phase % 320);
                            enemy.charges = enemy.charges.saturating_sub(1);
                        }
                        (
                            dir_x * advance + perp_x * orbit,
                            dir_y * advance + perp_y * orbit,
                        )
                    }
                    EnemyKind::BossRam => {
                        if enemy.burst_timer_ms == 0 && enemy.attack_timer_ms == 0 && dist > 32.0 {
                            enemy.burst_timer_ms = 240;
                            enemy.attack_timer_ms = 720 + (enemy.phase % 180);
                        }
                        if enemy.burst_timer_ms > 0 {
                            (dir_x * 3.2, dir_y * 3.2)
                        } else {
                            (dir_x * 0.82, dir_y * 0.82)
                        }
                    }
                    EnemyKind::BossBurst => {
                        let preferred_dist = 88.0;
                        let advance = if dist > preferred_dist + 16.0 {
                            0.52
                        } else if dist < preferred_dist - 10.0 {
                            -0.34
                        } else {
                            0.0
                        };
                        let orbit = sinf(enemy.phase as f32 * 0.010) * 0.64;
                        if enemy.attack_timer_ms == 0 && queued_shot_count < MAX_ENEMIES {
                            queued_shots[queued_shot_count] =
                                Some((enemy.kind, enemy.x, enemy.y, atan2f(dy, dx)));
                            queued_shot_count += 1;
                            enemy.attack_timer_ms = 600 + (enemy.phase % 120);
                        }
                        (
                            dir_x * advance + perp_x * orbit,
                            dir_y * advance + perp_y * orbit,
                        )
                    }
                    EnemyKind::BossNest => {
                        let preferred_dist = 92.0;
                        let advance = if dist > preferred_dist + 12.0 {
                            0.42
                        } else if dist < preferred_dist - 12.0 {
                            -0.28
                        } else {
                            0.0
                        };
                        let orbit = sinf(enemy.phase as f32 * 0.009) * 0.42;
                        if enemy.attack_timer_ms == 0
                            && enemy.charges > 0
                            && queued_summon_count < MAX_ENEMIES
                        {
                            queued_summons[queued_summon_count] =
                                Some((enemy.x, enemy.y, enemy.phase, true));
                            queued_summon_count += 1;
                            enemy.attack_timer_ms = 760 + (enemy.phase % 160);
                            enemy.charges = enemy.charges.saturating_sub(1);
                        }
                        (
                            dir_x * advance + perp_x * orbit,
                            dir_y * advance + perp_y * orbit,
                        )
                    }
                    EnemyKind::BossRing => {
                        let preferred_dist = 94.0;
                        let advance = if dist > preferred_dist + 12.0 {
                            0.42
                        } else if dist < preferred_dist - 12.0 {
                            -0.26
                        } else {
                            0.0
                        };
                        let orbit = sinf(enemy.phase as f32 * 0.008) * 0.58;
                        if enemy.attack_timer_ms == 0 && queued_shot_count < MAX_ENEMIES {
                            queued_shots[queued_shot_count] =
                                Some((enemy.kind, enemy.x, enemy.y, atan2f(dy, dx)));
                            queued_shot_count += 1;
                            enemy.attack_timer_ms = 760 + (enemy.phase % 140);
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
                self.health -= if enemy.kind.is_boss() { 2 } else { 1 };
                self.hit_invuln_ms = self.hit_invuln_base_ms;
                self.damage_flash_ms = DAMAGE_FLASH_MS;
                enemy.touch_timer_ms = ENEMY_TOUCH_DAMAGE_MS;
            }
        }

        for shot in queued_shots.into_iter().flatten() {
            match shot.0 {
                EnemyKind::Shooter => {
                    self.spawn_enemy_projectile(shot.1, shot.2, shot.3, 1, ENEMY_PROJECTILE_SPEED);
                }
                EnemyKind::BossBurst => {
                    self.spawn_enemy_projectile(
                        shot.1,
                        shot.2,
                        shot.3 - 0.22,
                        1,
                        ENEMY_PROJECTILE_SPEED + 0.01,
                    );
                    self.spawn_enemy_projectile(
                        shot.1,
                        shot.2,
                        shot.3,
                        1,
                        ENEMY_PROJECTILE_SPEED + 0.01,
                    );
                    self.spawn_enemy_projectile(
                        shot.1,
                        shot.2,
                        shot.3 + 0.22,
                        1,
                        ENEMY_PROJECTILE_SPEED + 0.01,
                    );
                }
                EnemyKind::BossRing => {
                    for index in 0..6 {
                        let angle = shot.3 + index as f32 * 1.047;
                        self.spawn_enemy_projectile(
                            shot.1,
                            shot.2,
                            angle,
                            1,
                            ENEMY_PROJECTILE_SPEED,
                        );
                    }
                }
                _ => {}
            }
        }
        for summon in queued_summons.into_iter().flatten() {
            let _ = self.spawn_summoned_runner(summon.0, summon.1, summon.2, summon.3);
        }
    }

    fn update_projectiles(&mut self, dt_ms: u16) {
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
                        self.damage_flash_ms = DAMAGE_FLASH_MS;
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
                        self.wave_tracker.remaining_to_kill =
                            self.wave_tracker.remaining_to_kill.saturating_sub(1);
                    }
                    break;
                }
            }
        }

        self.profile.best_kills = self.profile.best_kills.max(self.kills);
    }
}

fn profile_action_rect(action: ProfileAction) -> (u16, u16, u16, u16) {
    match action {
        ProfileAction::Attack => (
            PROFILE_LEFT_X,
            PROFILE_TOP_Y,
            PROFILE_CARD_W,
            PROFILE_CARD_H,
        ),
        ProfileAction::Vitality => (
            PROFILE_RIGHT_X,
            PROFILE_TOP_Y,
            PROFILE_CARD_W,
            PROFILE_CARD_H,
        ),
        ProfileAction::Trigger => (
            PROFILE_LEFT_X,
            PROFILE_BOTTOM_Y,
            PROFILE_CARD_W,
            PROFILE_CARD_H,
        ),
        ProfileAction::Thrusters => (
            PROFILE_RIGHT_X,
            PROFILE_BOTTOM_Y,
            PROFILE_CARD_W,
            PROFILE_CARD_H,
        ),
        ProfileAction::Deploy => (
            PROFILE_DEPLOY_X,
            PROFILE_DEPLOY_Y,
            PROFILE_DEPLOY_W,
            PROFILE_DEPLOY_H,
        ),
    }
}
