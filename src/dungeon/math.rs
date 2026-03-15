use super::*;

pub(super) fn touch_started_in_rect(
    touch: &TouchState,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> bool {
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

pub(super) fn texture_for_tile(tile: u8) -> TextureId {
    match tile {
        1 => TextureId::WallLight,
        2 => TextureId::WallMid,
        3 => TextureId::WallDark,
        4 => TextureId::DoorDark,
        5 => TextureId::WindowDark,
        _ => TextureId::WallMid,
    }
}

pub(super) fn floor_texture_for_map(map_index: usize) -> TextureId {
    match map_index % MAPS.len() {
        0 => TextureId::WallMid,
        1 => TextureId::DoorDark,
        _ => TextureId::WallDark,
    }
}

pub(super) fn ceiling_texture_for_map(map_index: usize) -> TextureId {
    match map_index % MAPS.len() {
        0 => TextureId::WallLight,
        1 => TextureId::WindowDark,
        _ => TextureId::DoorDark,
    }
}

pub(super) fn tile_fill(tile: u8, ui: &crate::display::Palette) -> u16 {
    match tile {
        0 => crate::display::color::mix(ui.panel, ui.panel_alt, 70),
        1 => crate::display::color::mix(ui.cyan, ui.sky, 88),
        2 => crate::display::color::mix(ui.orange, ui.amber, 92),
        3 => crate::display::color::mix(ui.rose, ui.indigo, 110),
        4 => crate::display::color::mix(ui.amber, ui.orange, 140),
        _ => crate::display::color::mix(ui.rose, ui.sky, 150),
    }
}

pub(super) fn try_move(
    player_x: &mut f32,
    player_y: &mut f32,
    delta_x: f32,
    delta_y: f32,
    map: &MapDef,
) {
    let next_x = *player_x + delta_x;
    let next_y = *player_y + delta_y;
    if !is_blocked_circle(map, next_x, *player_y, PLAYER_RADIUS) {
        *player_x = next_x;
    }
    if !is_blocked_circle(map, *player_x, next_y, PLAYER_RADIUS) {
        *player_y = next_y;
    }
}

pub(super) fn try_enemy_move(
    enemy_x: &mut f32,
    enemy_y: &mut f32,
    delta_x: f32,
    delta_y: f32,
    map: &MapDef,
) {
    let next_x = *enemy_x + delta_x;
    let next_y = *enemy_y + delta_y;
    if !is_blocked_circle(map, next_x, *enemy_y, ENEMY_RADIUS) {
        *enemy_x = next_x;
    }
    if !is_blocked_circle(map, *enemy_x, next_y, ENEMY_RADIUS) {
        *enemy_y = next_y;
    }
}

pub(super) fn cast_ray(map: &MapDef, px: f32, py: f32, dir_x: f32, dir_y: f32) -> RayHit {
    let mut map_x = floorf(px) as i32;
    let mut map_y = floorf(py) as i32;

    let delta_dist_x = if dir_x == 0.0 {
        1.0e30
    } else {
        fabsf(1.0 / dir_x)
    };
    let delta_dist_y = if dir_y == 0.0 {
        1.0e30
    } else {
        fabsf(1.0 / dir_y)
    };

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

pub(super) fn direction_and_plane(angle: f32) -> (f32, f32, f32, f32) {
    let dir_x = cosf(angle);
    let dir_y = sinf(angle);
    let plane_x = -dir_y * CAMERA_PLANE_SCALE;
    let plane_y = dir_x * CAMERA_PLANE_SCALE;
    (dir_x, dir_y, plane_x, plane_y)
}

pub(super) fn is_wall(map: &MapDef, x: f32, y: f32) -> bool {
    let mx = floorf(x) as i32;
    let my = floorf(y) as i32;
    if mx < 0 || my < 0 || mx as usize >= MAP_W || my as usize >= MAP_H {
        return true;
    }
    map.layout[my as usize][mx as usize] != 0
}

pub(super) fn is_blocked_circle(map: &MapDef, x: f32, y: f32, radius: f32) -> bool {
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

pub(super) fn line_of_sight(map: &MapDef, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
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

pub(super) fn point_in_circle(x: u16, y: u16, center_x: u16, center_y: u16, radius: f32) -> bool {
    let dx = x as f32 - center_x as f32;
    let dy = y as f32 - center_y as f32;
    (dx * dx) + (dy * dy) <= radius * radius
}

pub(super) fn clamp_circle_delta(dx: f32, dy: f32, radius: f32) -> (f32, f32) {
    let magnitude = sqrtf(dx * dx + dy * dy);
    if magnitude <= radius || magnitude <= 0.0001 {
        (dx, dy)
    } else {
        let scale = radius / magnitude;
        (dx * scale, dy * scale)
    }
}

pub(super) fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if fabsf(value) <= deadzone {
        0.0
    } else if value > 0.0 {
        (value - deadzone) / (1.0 - deadzone)
    } else {
        (value + deadzone) / (1.0 - deadzone)
    }
}

pub(super) fn view_px(value: u16) -> usize {
    value as usize / VIEW_SCALE as usize
}

pub(super) fn view_span(value: u16) -> usize {
    ((value as usize) + VIEW_SCALE as usize - 1) / VIEW_SCALE as usize
}

pub(super) fn view_screen_y(value: u16) -> usize {
    view_px(value.saturating_sub(VIEW_Y))
}

pub(super) fn view_f(value: f32) -> f32 {
    value / VIEW_SCALE as f32
}

pub(super) fn round_to_usize(value: f32) -> usize {
    floorf(value + 0.5) as usize
}

pub(super) fn round_to_i32(value: f32) -> i32 {
    floorf(value + 0.5) as i32
}

pub(super) fn distance_sq(dx: f32, dy: f32) -> f32 {
    dx * dx + dy * dy
}

pub(super) fn wrap_angle(angle: f32) -> f32 {
    let mut wrapped = angle;
    while wrapped > core::f32::consts::PI {
        wrapped -= core::f32::consts::TAU;
    }
    while wrapped < -core::f32::consts::PI {
        wrapped += core::f32::consts::TAU;
    }
    wrapped
}
