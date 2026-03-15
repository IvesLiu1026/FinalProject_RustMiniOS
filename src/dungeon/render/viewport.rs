use super::super::math::*;
use super::super::*;
use super::controls::render_touch_controls;
use super::effects::{render_heal_fx, render_shot_fx};
use super::floor::render_sky_floor;
use super::hud::{render_crosshair, render_health_bar, render_minimap};
use super::sprites::{render_pickups, render_sprites};
use super::weapon::render_weapon;

pub(super) fn draw_shell(display: &mut Display, ui: &crate::display::Palette, zh_mode: bool) {
    display.fill_rect(0, 0, SCREEN_WIDTH, 240, ui.canvas);
    display.panel(10, 8, 300, 30, ui.panel, ui.cyan);
    display.text(
        22,
        16,
        if zh_mode {
            "地城核心"
        } else {
            "DUNGEON CORE"
        },
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

pub(super) fn draw_viewport(
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
    render_sky_floor(
        buffer,
        dungeon,
        ui,
        dir_x,
        dir_y,
        plane_x,
        plane_y,
        render_strategy,
    );
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
        let wall_top = (((VIEW_H as i32 - line_height) / 2).max(0)) as u16;
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
        let top_end = wall_top as usize;
        let wall_end = core::cmp::max(wall_bottom as usize, top_end + 1);

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
    display.draw_rgb565_scaled(0, VIEW_Y, VIEW_W as u16, VIEW_H, VIEW_SCALE, buffer);
}
