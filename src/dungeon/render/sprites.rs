use super::super::math::*;
use super::super::*;

pub(super) fn render_sprites(
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

        let sprite_screen_x = ((VIEW_W as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
        let death_t = if enemy.alive {
            1.0
        } else {
            (enemy.death_anim_ms as f32 / 280.0).clamp(0.0, 1.0)
        };
        let bob = if enemy.alive {
            (sinf(enemy.phase as f32 * 0.01) * view_f(3.0)) as i32
        } else {
            0
        };
        let sprite_height = ((VIEW_H as f32 / transform_y).clamp(view_f(12.0), view_f(110.0))
            * death_t.max(0.35)) as i32;
        let sprite_width = (sprite_height as f32 * sprite.width as f32 / sprite.height as f32)
            .clamp(view_f(10.0), view_f(110.0)) as i32;

        let sink = ((1.0 - death_t) * view_f(18.0)) as i32;
        let floor_anchor = (VIEW_H as f32 * 0.82) as i32 + bob + sink;
        let draw_end_y = floor_anchor.clamp(view_span(16) as i32, VIEW_H as i32 - 1);
        let draw_start_y = (draw_end_y - sprite_height).max(0);
        let draw_start_x = (sprite_screen_x - sprite_width / 2).max(0);
        let draw_end_x = (sprite_screen_x + sprite_width / 2).min(VIEW_W as i32);

        for stripe in draw_start_x..draw_end_x {
            let ray_column = (stripe as usize / COLUMN_WIDTH as usize).min(zbuffer.len() - 1);
            if transform_y >= zbuffer[ray_column].max(0.001) {
                continue;
            }

            let tex_x = (((stripe - (sprite_screen_x - sprite_width / 2)) * sprite.width as i32)
                / sprite_width) as usize;
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

pub(super) fn render_pickups(
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
        let bob = (sinf(pickup.phase as f32 * 0.012) * view_f(2.0)) as i32;
        let size = (VIEW_H as f32 / transform_y).clamp(view_f(10.0), view_f(26.0)) as i32;
        let draw_end_y =
            ((VIEW_H as f32 * 0.82) as i32 + bob).clamp(view_span(14) as i32, VIEW_H as i32 - 1);
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

                let border =
                    local_x < 1 || local_y < 1 || local_x >= size - 1 || local_y >= size - 1;
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
