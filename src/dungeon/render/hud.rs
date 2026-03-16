use core::fmt::Write;

use heapless::String;

use super::super::math::*;
use super::super::*;
use super::primitives::{
    buffer_blend_circle, buffer_blend_line, buffer_blend_rect, buffer_fill_rect, buffer_stroke_rect,
};

impl DungeonApp {
    pub(super) fn draw_hud(
        &mut self,
        display: &mut Display,
        ui: &crate::display::Palette,
        zh_mode: bool,
        force: bool,
        fps: u16,
    ) {
        let map_name = hud_map_name(self.map_index, zh_mode);
        let active_enemies = self.enemies.iter().filter(|enemy| enemy.alive).count() as u16;
        let active_pickups = self.pickups.iter().filter(|pickup| pickup.active).count() as u16;

        if force || self.prev_hud_fps != fps {
            let mut fps_line: String<16> = String::new();
            let _ = write!(&mut fps_line, "{}FPS", fps);
            display.fill_rect(250, 14, 48, 14, ui.panel);
            display.text(256, 18, &fps_line, ui.amber, ui.panel, 1);
            self.prev_hud_fps = fps;
        }

        if force || self.prev_hud_map_index != self.map_index {
            display.fill_rect(206, 44, 102, 18, ui.panel_alt);
            display.text(212, 50, map_name, ui.text, ui.panel_alt, 1);
            self.prev_hud_map_index = self.map_index;
        }

        if force || self.prev_hud_health != self.health {
            display.fill_rect(206, 66, 102, 18, ui.panel_alt);
            display.text(
                212,
                72,
                if zh_mode { "生命" } else { "HP" },
                ui.text_muted,
                ui.panel_alt,
                1,
            );
            display.fill_rect(246, 71, 54, 8, ui.shadow);
            let hp_width = ((self.health.max(0) as u32 * 54) / PLAYER_MAX_HP as u32) as u16;
            display.fill_rect(246, 71, hp_width, 8, ui.lime);
            self.prev_hud_health = self.health;
        }

        if force || self.prev_hud_score != self.score {
            let mut line: String<32> = String::new();
            let _ = write!(
                &mut line,
                "{} {}",
                if zh_mode { "分數" } else { "SCORE" },
                self.score
            );
            display.fill_rect(206, 90, 102, 16, ui.panel_alt);
            display.text(212, 95, &line, ui.text, ui.panel_alt, 1);
            self.prev_hud_score = self.score;
        }

        if force || self.prev_hud_kills != self.kills {
            let mut kills: String<24> = String::new();
            let _ = write!(
                &mut kills,
                "{} {}",
                if zh_mode { "擊殺" } else { "KILLS" },
                self.kills
            );
            display.fill_rect(206, 108, 102, 16, ui.panel_alt);
            display.text(212, 113, &kills, ui.text_muted, ui.panel_alt, 1);
            self.prev_hud_kills = self.kills;
        }

        if force || self.prev_hud_weapon != Some(self.weapon) {
            display.fill_rect(206, 126, 102, 16, ui.panel_alt);
            display.text(
                212,
                131,
                if zh_mode {
                    self.weapon.label_zh()
                } else {
                    self.weapon.label_en()
                },
                self.weapon.accent(ui),
                ui.panel_alt,
                1,
            );
            self.prev_hud_weapon = Some(self.weapon);
        }

        display.fill_rect(
            MAP_BUTTON_X,
            MAP_BUTTON_Y,
            MAP_BUTTON_W,
            MAP_BUTTON_H,
            ui.panel,
        );
        display.stroke_rect(
            MAP_BUTTON_X,
            MAP_BUTTON_Y,
            MAP_BUTTON_W,
            MAP_BUTTON_H,
            1,
            ui.cyan,
        );
        display.text(
            MAP_BUTTON_X + 28,
            MAP_BUTTON_Y + 5,
            if zh_mode { "選圖" } else { "MAPS" },
            ui.text_muted,
            ui.panel,
            1,
        );

        let exit_active = self.exit_hold_ms > 0;
        if force || self.prev_hud_exit_hold != exit_active {
            display.fill_rect(206, 144, 102, 18, ui.panel_alt);
            if exit_active {
                display.text(
                    212,
                    150,
                    if zh_mode {
                        "返回主頁中"
                    } else {
                        "EXITING"
                    },
                    ui.text_muted,
                    ui.panel_alt,
                    1,
                );
            } else {
                let mut objective: String<24> = String::new();
                let _ = write!(
                    &mut objective,
                    "{} {} / {} {}",
                    if zh_mode { "敵" } else { "ENM" },
                    active_enemies,
                    if zh_mode { "補" } else { "MED" },
                    active_pickups
                );
                display.text(212, 146, &objective, ui.text, ui.panel_alt, 1);
                display.text(
                    212,
                    154,
                    if zh_mode {
                        "長按返回主頁"
                    } else {
                        "HOLD TO EXIT"
                    },
                    ui.text_muted,
                    ui.panel_alt,
                    1,
                );
            }
            self.prev_hud_exit_hold = exit_active;
        }
    }
}

fn hud_map_name(map_index: usize, zh_mode: bool) -> &'static str {
    match (map_index % DungeonApp::map_count(), zh_mode) {
        (0, true) => "遺跡",
        (1, true) => "熔爐",
        (2, true) => "墓穴",
        (0, false) => "RUINS",
        (1, false) => "FORGE",
        _ => "CRYPT",
    }
}

pub(super) fn render_health_bar(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
) {
    let panel_x = view_px(96);
    let panel_y = view_px(6);
    let panel_w = view_span(126);
    let panel_h = view_span(20);
    let bar_x = panel_x + view_span(10);
    let bar_y = panel_y + view_span(7);
    let bar_w = view_span(110);
    let fill_w = ((dungeon.health.max(0) as usize * bar_w) / PLAYER_MAX_HP as usize).min(bar_w);
    let bar_fill = if dungeon.heal_flash_ms > 0 {
        crate::display::color::mix(ui.lime, ui.white, 120)
    } else if dungeon.health < 28 {
        ui.rose
    } else {
        ui.lime
    };

    buffer_blend_rect(
        buffer,
        panel_x + view_span(2),
        panel_y + view_span(2),
        panel_w,
        panel_h,
        ui.shadow,
        112,
    );
    buffer_blend_rect(buffer, panel_x, panel_y, panel_w, panel_h, ui.panel, 210);
    buffer_stroke_rect(
        buffer,
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        view_span(1),
        ui.cyan,
        220,
    );
    buffer_blend_rect(buffer, bar_x, bar_y, bar_w, view_span(7), ui.shadow, 255);
    if fill_w > 0 {
        buffer_blend_rect(buffer, bar_x, bar_y, fill_w, view_span(7), bar_fill, 255);
    }
    buffer_blend_rect(
        buffer,
        bar_x + fill_w.min(bar_w.saturating_sub(view_span(2))),
        bar_y,
        view_span(2),
        view_span(7),
        crate::display::color::mix(ui.white, ui.cyan, 160),
        220,
    );
}

pub(super) fn render_minimap(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
) {
    let map = dungeon.current_map();
    let panel_x = view_px(10);
    let panel_y = view_px(2);
    let cell = view_span(CELL).max(1);
    let panel_w = MAP_W * cell + view_span(12);
    let panel_h = MAP_H * cell + view_span(12);

    buffer_blend_rect(
        buffer,
        panel_x + view_span(3),
        panel_y + view_span(4),
        panel_w,
        panel_h,
        ui.shadow,
        96,
    );
    buffer_blend_rect(buffer, panel_x, panel_y, panel_w, panel_h, ui.panel, 220);
    buffer_stroke_rect(
        buffer,
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        view_span(2),
        ui.cyan,
        235,
    );
    buffer_stroke_rect(
        buffer,
        panel_x + view_span(2),
        panel_y + view_span(2),
        panel_w.saturating_sub(view_span(4)),
        panel_h.saturating_sub(view_span(4)),
        view_span(1),
        ui.white,
        210,
    );

    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let tile_color = tile_fill(map.layout[row][col], ui);
            buffer_blend_rect(
                buffer,
                view_px(MAP_ORIGIN_X) + col * cell,
                view_screen_y(MAP_ORIGIN_Y) + row * cell,
                cell.saturating_sub(1).max(1),
                cell.saturating_sub(1).max(1),
                tile_color,
                255,
            );
        }
    }

    let player_px = view_f(MAP_ORIGIN_X as f32) + dungeon.player_x * cell as f32;
    let player_py = view_f((MAP_ORIGIN_Y - VIEW_Y) as f32) + dungeon.player_y * cell as f32;
    let (dir_x, dir_y, _, _) = direction_and_plane(dungeon.angle);
    let tip_x = player_px + dir_x * view_f(5.0);
    let tip_y = player_py + dir_y * view_f(5.0);
    let left_x = player_px + cosf(dungeon.angle + 2.5) * view_f(2.0);
    let left_y = player_py + sinf(dungeon.angle + 2.5) * view_f(2.0);
    let right_x = player_px + cosf(dungeon.angle - 2.5) * view_f(2.0);
    let right_y = player_py + sinf(dungeon.angle - 2.5) * view_f(2.0);
    let fov_left_x = player_px + cosf(dungeon.angle - 0.42) * view_f(4.0);
    let fov_left_y = player_py + sinf(dungeon.angle - 0.42) * view_f(4.0);
    let fov_right_x = player_px + cosf(dungeon.angle + 0.42) * view_f(4.0);
    let fov_right_y = player_py + sinf(dungeon.angle + 0.42) * view_f(4.0);

    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(fov_left_x),
        round_to_i32(fov_left_y),
        crate::display::color::mix(ui.cyan, ui.white, 48),
        124,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(fov_right_x),
        round_to_i32(fov_right_y),
        crate::display::color::mix(ui.cyan, ui.white, 48),
        124,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(player_px),
        round_to_i32(player_py),
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        ui.lime,
        255,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        round_to_i32(left_x),
        round_to_i32(left_y),
        ui.white,
        220,
    );
    buffer_blend_line(
        buffer,
        round_to_i32(tip_x),
        round_to_i32(tip_y),
        round_to_i32(right_x),
        round_to_i32(right_y),
        ui.white,
        220,
    );
    buffer_blend_circle(
        buffer,
        round_to_usize(player_px),
        round_to_usize(player_py),
        1,
        ui.lime,
        255,
    );

    for enemy in dungeon.enemies.iter().filter(|enemy| enemy.alive) {
        let ex = view_f(MAP_ORIGIN_X as f32) + enemy.x * cell as f32;
        let ey = view_f((MAP_ORIGIN_Y - VIEW_Y) as f32) + enemy.y * cell as f32;
        buffer_blend_circle(
            buffer,
            round_to_usize(ex),
            round_to_usize(ey),
            1,
            ui.rose,
            255,
        );
    }

    for pickup in dungeon.pickups.iter().filter(|pickup| pickup.active) {
        let px = view_f(MAP_ORIGIN_X as f32) + pickup.x * cell as f32;
        let py = view_f((MAP_ORIGIN_Y - VIEW_Y) as f32) + pickup.y * cell as f32;
        let px = round_to_usize(px);
        let py = round_to_usize(py);
        buffer_blend_rect(
            buffer,
            px.saturating_sub(view_span(1)),
            py.saturating_sub(view_span(2)),
            view_span(3),
            view_span(5),
            ui.white,
            255,
        );
        buffer_blend_rect(
            buffer,
            px.saturating_sub(view_span(2)),
            py.saturating_sub(view_span(1)),
            view_span(5),
            view_span(3),
            ui.rose,
            255,
        );
    }
}

pub(super) fn render_crosshair(buffer: &mut [u16], ui: &crate::display::Palette, hot: bool) {
    let color = if hot { ui.amber } else { ui.white };
    let center_x = VIEW_W / 2;
    let center_y = view_span(66);
    buffer_fill_rect(
        buffer,
        center_x.saturating_sub(view_span(1)),
        center_y.saturating_sub(view_span(5)),
        view_span(2),
        view_span(10),
        color,
    );
    buffer_fill_rect(
        buffer,
        center_x.saturating_sub(view_span(5)),
        center_y.saturating_sub(view_span(1)),
        view_span(10),
        view_span(2),
        color,
    );
}
