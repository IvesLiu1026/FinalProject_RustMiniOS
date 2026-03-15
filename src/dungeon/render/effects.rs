use super::super::math::*;
use super::super::*;
use super::primitives::{
    buffer_blend_circle, buffer_blend_line, buffer_blend_rect, buffer_stroke_circle,
};

pub(super) fn render_shot_fx(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
) {
    if dungeon.muzzle_flash_ms == 0 {
        return;
    }

    let flash_ms = dungeon.weapon.flash_ms().max(1) as u32;
    let intensity = ((dungeon.muzzle_flash_ms as u32 * 255) / flash_ms).min(255) as u8;
    let weapon_color = dungeon.weapon.accent(ui);
    let tracer_color = crate::display::color::mix(
        weapon_color,
        if dungeon.shot_hit_enemy {
            ui.rose
        } else {
            ui.white
        },
        intensity,
    );
    let center_x = VIEW_W / 2;
    let muzzle_y = VIEW_H_USIZE.saturating_sub(view_span(6));
    let impact_y =
        ((VIEW_H as f32 * 0.5) + (dungeon.shot_depth.clamp(0.6, 8.0) * view_f(3.0))) as usize;
    let impact_y = impact_y
        .min(VIEW_H_USIZE.saturating_sub(view_span(12)))
        .max(view_span(26));

    for y in impact_y..=muzzle_y {
        let row = y * VIEW_W;
        for dx in center_x.saturating_sub(1)..=(center_x + 1).min(VIEW_W - 1) {
            buffer[row + dx] = tracer_color;
        }
    }

    let spark = if dungeon.shot_hit_enemy {
        ui.rose
    } else {
        crate::display::color::mix(weapon_color, ui.white, 70)
    };
    let spark_y = impact_y;
    for offset in 0..view_span(5).max(2) {
        let up = spark_y.saturating_sub(offset);
        let down = (spark_y + offset).min(VIEW_H_USIZE - 1);
        let left = center_x.saturating_sub(offset);
        let right = (center_x + offset).min(VIEW_W - 1);
        buffer[up * VIEW_W + center_x] = spark;
        buffer[down * VIEW_W + center_x] = spark;
        buffer[spark_y * VIEW_W + left] = spark;
        buffer[spark_y * VIEW_W + right] = spark;
    }
}

pub(super) fn render_heal_fx(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
) {
    if dungeon.heal_flash_ms == 0 {
        return;
    }

    let pulse = dungeon.heal_flash_ms as f32 / 220.0;
    let center_x = VIEW_W / 2;
    let center_y = VIEW_H_USIZE / 2 + view_span(12);
    let burst_radius = view_f(18.0 + (1.0 - pulse) * 22.0) as i32;
    let ring_alpha = (pulse * 164.0).clamp(36.0, 164.0) as u8;
    let core_alpha = (pulse * 88.0).clamp(22.0, 88.0) as u8;
    let glow = crate::display::color::mix(ui.lime, ui.white, 124);

    buffer_blend_circle(buffer, center_x, center_y, burst_radius, glow, core_alpha);
    buffer_stroke_circle(
        buffer,
        center_x,
        center_y,
        burst_radius,
        view_span(2) as i32,
        crate::display::color::mix(ui.white, ui.lime, 96),
        ring_alpha,
    );

    let spoke = (burst_radius + view_span(6) as i32).max(view_span(14) as i32);
    for &(dx, dy) in &[
        (1.0f32, 0.0f32),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (0.7, 0.7),
        (-0.7, 0.7),
        (0.7, -0.7),
        (-0.7, -0.7),
    ] {
        let end_x = center_x as f32 + dx * spoke as f32;
        let end_y = center_y as f32 + dy * spoke as f32;
        buffer_blend_line(
            buffer,
            center_x as i32,
            center_y as i32,
            round_to_i32(end_x),
            round_to_i32(end_y),
            glow,
            (ring_alpha / 2).max(48),
        );
    }

    let cross_color = crate::display::color::mix(ui.white, ui.lime, 110);
    buffer_blend_rect(
        buffer,
        center_x.saturating_sub(view_span(2)),
        center_y.saturating_sub(view_span(10)),
        view_span(4),
        view_span(20),
        cross_color,
        ring_alpha,
    );
    buffer_blend_rect(
        buffer,
        center_x.saturating_sub(view_span(10)),
        center_y.saturating_sub(view_span(2)),
        view_span(20),
        view_span(4),
        cross_color,
        ring_alpha,
    );
}
