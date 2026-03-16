#[path = "render/controls.rs"]
mod controls;
#[path = "render/effects.rs"]
mod effects;
#[path = "render/floor.rs"]
mod floor;
#[path = "render/hud.rs"]
mod hud;
#[path = "render/overlay.rs"]
mod overlay;
#[path = "render/primitives.rs"]
mod primitives;
#[path = "render/sprites.rs"]
mod sprites;
#[path = "render/viewport.rs"]
mod viewport;
#[path = "render/weapon.rs"]
mod weapon;

use super::*;
use overlay::{draw_intro_overlay, draw_overlay};
use viewport::{draw_shell, draw_viewport};

impl DungeonApp {
    pub fn render(
        &mut self,
        display: &mut Display,
        touch: &TouchState,
        full_refresh: bool,
        theme: ThemeMode,
        zh_mode: bool,
        fps: u16,
        render_strategy: RenderStrategy,
    ) {
        let ui = palette(theme);

        if full_refresh {
            draw_shell(display, &ui, zh_mode);
            self.prev_hud_health = -1;
            self.prev_hud_score = u32::MAX;
            self.prev_hud_kills = u16::MAX;
            self.prev_hud_map_index = usize::MAX;
            self.prev_hud_weapon = None;
            self.prev_hud_fps = u16::MAX;
            self.prev_hud_exit_hold = false;
        }

        draw_viewport(display, self, touch, &ui, render_strategy);
        self.draw_hud(display, &ui, zh_mode, full_refresh, fps);

        if self.intro_ms > 0 {
            draw_intro_overlay(
                display,
                &ui,
                self.current_map(),
                zh_mode,
                self.current_map().enemies.len() as u8,
                self.current_map().pickups.len() as u8,
            );
        } else if self.game_over {
            draw_overlay(
                display,
                &ui,
                if zh_mode { "遊戲結束" } else { "GAME OVER" },
                if zh_mode {
                    "按 K1 或點擊重來"
                } else {
                    "PRESS K1 OR TAP TO RETRY"
                },
                if zh_mode { "重新開始" } else { "RETRY" },
                if zh_mode {
                    "地圖選單"
                } else {
                    "MAP SELECT"
                },
                ui.rose,
                if zh_mode {
                    self.current_map().name_zh
                } else {
                    self.current_map().name_en
                },
                self.score,
                self.kills,
                zh_mode,
            );
        } else if self.level_cleared {
            draw_overlay(
                display,
                &ui,
                if zh_mode {
                    "關卡完成"
                } else {
                    "AREA CLEARED"
                },
                if zh_mode {
                    "按 K1 或點擊重玩"
                } else {
                    "PRESS K1 OR TAP TO RELOAD"
                },
                if zh_mode { "再次挑戰" } else { "RETRY" },
                if zh_mode {
                    "地圖選單"
                } else {
                    "MAP SELECT"
                },
                ui.lime,
                if zh_mode {
                    self.current_map().name_zh
                } else {
                    self.current_map().name_en
                },
                self.score,
                self.kills,
                zh_mode,
            );
        }
    }
}
