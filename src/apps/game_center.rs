use crate::app_registry::{descriptor, game_center_apps, game_center_slot_for_app, AppId};
use crate::board::ButtonSnapshot;
use crate::display::{palette, Display, ThemeMode};
use crate::touch::TouchState;
use crate::ui::{
    draw_gradient_background, render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const CARD_X: u16 = 20;
const CARD_W: u16 = 280;
const CARD_H: u16 = 44;

pub enum GameCenterAction {
    Stay,
    Launch(AppId),
    ExitHome,
}

pub struct GameCenterApp {
    selected: usize,
}

impl GameCenterApp {
    pub const fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn update(&mut self, input: &ButtonSnapshot, touch: &TouchState) -> GameCenterAction {
        if input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            return GameCenterAction::ExitHome;
        }

        let app_count = game_center_apps().len();
        if input.k0_just_pressed {
            self.selected = (self.selected + app_count - 1) % app_count;
        }
        if input.wkup_just_pressed {
            self.selected = (self.selected + 1) % app_count;
        }
        if input.k1_just_pressed {
            return self.activate_selected();
        }

        if touch.just_released {
            for index in 0..app_count {
                let y = 64 + index as u16 * 50;
                if touch_released_in_rect(touch, CARD_X, y, CARD_W, CARD_H) {
                    if self.selected == index {
                        return self.activate_selected();
                    }
                    self.selected = index;
                }
            }
        }

        GameCenterAction::Stay
    }

    pub fn render(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 30);

        display.panel(14, 10, 292, 34, ui.panel, ui.rose);
        render_nav_back(display, zh_mode, ui.orange, &ui);
        display.text(
            74,
            18,
            if zh_mode {
                "遊戲中心"
            } else {
                "GAME CENTER"
            },
            ui.text,
            ui.panel,
            2,
        );
        display.text(
            158,
            20,
            if zh_mode {
                "地城、自動獵手與街機小遊戲"
            } else {
                "DUNGEON, AUTO HUNTER, AND ARCADE MODES"
            },
            ui.text_muted,
            ui.panel,
            1,
        );

        for (index, app_id) in game_center_apps().iter().copied().enumerate() {
            let app = descriptor(app_id);
            self.render_card(
                display,
                index,
                app.title(zh_mode),
                app.subtitle(zh_mode),
                app.accent.resolve(&ui),
                &ui,
            );
        }

        display.panel(18, 222, 284, 16, ui.panel, ui.white);
        display.text(
            28,
            226,
            if zh_mode {
                "K0/WK 切換  K1 進入  K0+WK 回首頁"
            } else {
                "K0/WK SWITCH  K1 OPEN  K0+WK HOME"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
    }

    pub fn select_app(&mut self, app_id: AppId) {
        if let Some(index) = game_center_slot_for_app(app_id) {
            self.selected = index;
        }
    }

    fn render_card(
        &self,
        display: &mut Display,
        index: usize,
        title: &str,
        subtitle: &str,
        accent: u16,
        ui: &crate::display::Palette,
    ) {
        let y = 64 + index as u16 * 50;
        let selected = self.selected == index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { accent } else { ui.steel };
        display.panel(CARD_X, y, CARD_W, CARD_H, fill, border);
        display.text(30, y + 8, title, ui.text, fill, 2);
        display.text(30, y + 28, subtitle, ui.text_muted, fill, 1);

        let badge_x = 226;
        display.panel(badge_x, y + 10, 62, 18, fill, accent);
        display.centered_text(badge_x + 31, y + 15, "PLAY", ui.text, fill, 1);
    }

    fn activate_selected(&self) -> GameCenterAction {
        game_center_apps()
            .get(self.selected)
            .copied()
            .map(GameCenterAction::Launch)
            .unwrap_or(GameCenterAction::Stay)
    }
}
