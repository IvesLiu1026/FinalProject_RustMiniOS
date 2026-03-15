use super::super::math::*;
use super::super::*;

pub(super) fn render_sky_floor(
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
                let stripe = if ((cell_x + cell_y) & 1) == 0 {
                    214u8
                } else {
                    184u8
                };
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
            let checker = if ((cell_x + cell_y) & 1) == 0 {
                214u8
            } else {
                170u8
            };
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
