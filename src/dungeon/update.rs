use super::math::*;
use super::*;

impl DungeonApp {
    pub(super) fn current_map(&self) -> &'static MapDef {
        &MAPS[self.map_index]
    }

    pub(super) fn load_current_map(&mut self) {
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
            && touch_started_in_rect(
                touch,
                MAP_BUTTON_X,
                MAP_BUTTON_Y,
                MAP_BUTTON_W,
                MAP_BUTTON_H,
            )
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
            self.touch_mode = if weapon_switch_tapped {
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

    fn try_shoot(&mut self) {
        let (dir_x, dir_y, _, _) = direction_and_plane(self.angle);
        let center_hit = cast_ray(
            self.current_map(),
            self.player_x,
            self.player_y,
            dir_x,
            dir_y,
        );
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
        let center_hit = cast_ray(
            self.current_map(),
            self.player_x,
            self.player_y,
            dir_x,
            dir_y,
        );
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

            if !line_of_sight(
                self.current_map(),
                self.player_x,
                self.player_y,
                enemy.x,
                enemy.y,
            ) {
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
            try_enemy_move(
                &mut enemy.x,
                &mut enemy.y,
                dir_x * speed,
                dir_y * speed,
                map,
            );
        }
    }
}
