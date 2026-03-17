use heapless::String;

use crate::app_registry::{descriptor, game_center_apps, game_center_slot_for_app, AppId};
use crate::board::ButtonSnapshot;
use crate::display::{color, palette, Display, ThemeMode};
use crate::shell_contract::{reduce_game_center, GameCenterIntent, HostedAppNavigation};
use crate::touch::TouchState;
use crate::ui::{
    draw_app_icon, draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window,
    draw_title_bar, render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const PREVIEW_X: u16 = 18;
const PREVIEW_Y: u16 = 54;
const PREVIEW_W: u16 = 136;
const PREVIEW_H: u16 = 150;

const LIST_X: u16 = 164;
const LIST_Y: u16 = 54;
const LIST_W: u16 = 138;
const LIST_CARD_H: u16 = 24;
const LIST_STEP: u16 = 28;

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
        let mut intent = GameCenterIntent::Idle;
        if input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            intent = GameCenterIntent::ExitHome;
        } else if input.k0_just_pressed {
            intent = GameCenterIntent::Previous;
        } else if input.wkup_just_pressed {
            intent = GameCenterIntent::Next;
        } else if input.k1_just_pressed {
            intent = GameCenterIntent::LaunchCurrent;
        } else if touch.just_released {
            for index in 0..game_center_apps().len() {
                let y = LIST_Y + 8 + index as u16 * LIST_STEP;
                if touch_released_in_rect(touch, LIST_X + 8, y, LIST_W - 16, LIST_CARD_H) {
                    intent = GameCenterIntent::SelectSlot(index);
                    break;
                }
            }
        }

        let outcome = reduce_game_center(self.selected, game_center_apps(), intent);
        self.selected = outcome.next_selected;
        match outcome.navigation {
            HostedAppNavigation::Stay => GameCenterAction::Stay,
            HostedAppNavigation::Launch(app_id) => GameCenterAction::Launch(app_id),
            HostedAppNavigation::Exit {
                app_id: AppId::GameCenter,
                persist_state: false,
            } => GameCenterAction::ExitHome,
            _ => GameCenterAction::Stay,
        }
    }

    pub fn render(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 30);
        draw_shell_window(display, ui.rose, &ui);
        draw_title_bar(
            display,
            if zh_mode {
                "遊戲中心"
            } else {
                "GAME CENTER"
            },
            if zh_mode {
                "arcade launcher / dungeon / hunter"
            } else {
                "arcade launcher / dungeon / hunter"
            },
            ui.rose,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.orange, &ui);

        let selected_app = descriptor(game_center_apps()[self.selected]);
        draw_info_strip(
            display,
            18,
            40,
            132,
            if zh_mode { "櫃台" } else { "CABINET" },
            if zh_mode { "主打遊戲" } else { "HEADLINER" },
            selected_app.accent.resolve(&ui),
            &ui,
        );
        draw_info_strip(
            display,
            164,
            40,
            138,
            if zh_mode { "操作" } else { "INPUT" },
            if zh_mode {
                "K1 啟動 / 點兩下"
            } else {
                "K1 RUN / DOUBLE TAP"
            },
            ui.cyan,
            &ui,
        );

        self.render_preview(
            display,
            selected_app.title(zh_mode),
            teaser_for_app(game_center_apps()[self.selected], zh_mode),
            selected_app.icon,
            selected_app.accent.resolve(&ui),
            &ui,
        );

        for (index, app_id) in game_center_apps().iter().copied().enumerate() {
            let app = descriptor(app_id);
            self.render_card(
                display,
                index,
                app.desktop_label(zh_mode),
                app.icon,
                app.accent.resolve(&ui),
                &ui,
            );
        }

        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK SWITCH  K1 OPEN  K0+WK HOME"
            } else {
                "K0/WK SWITCH  K1 OPEN  K0+WK HOME"
            },
            ui.white,
            &ui,
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
        icon: crate::app_registry::AppIcon,
        accent: u16,
        ui: &crate::display::Palette,
    ) {
        let y = LIST_Y + 8 + index as u16 * LIST_STEP;
        let selected = self.selected == index;
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { accent } else { ui.steel };
        display.panel(LIST_X + 8, y, LIST_W - 16, LIST_CARD_H, fill, border);
        display.fill_rect(LIST_X + 16, y + 4, 20, 16, color_box(fill, accent, ui));
        display.stroke_rect(LIST_X + 16, y + 4, 20, 16, 1, accent);
        draw_app_icon(
            display,
            LIST_X + 18,
            y + 4,
            icon,
            accent,
            color_box(fill, accent, ui),
            ui,
        );
        let card_title = fit_text_to_width(display, title, 52, 1);
        display.text(LIST_X + 42, y + 5, &card_title, ui.text, fill, 1);
        display.text(
            LIST_X + 42,
            y + 14,
            if selected { "READY" } else { "ARCADE" },
            ui.text_muted,
            fill,
            1,
        );

        let badge_x = LIST_X + LIST_W - 42;
        display.fill_rect(badge_x, y + 6, 20, 12, color_box(fill, accent, ui));
        display.stroke_rect(badge_x, y + 6, 20, 12, 1, accent);
        display.centered_text(
            badge_x + 10,
            y + 9,
            if selected { "GO" } else { "RUN" },
            ui.text,
            color_box(fill, accent, ui),
            1,
        );
    }

    fn render_preview(
        &self,
        display: &mut Display,
        title: &str,
        subtitle: &str,
        icon: crate::app_registry::AppIcon,
        accent: u16,
        ui: &crate::display::Palette,
    ) {
        display.panel(PREVIEW_X, PREVIEW_Y, PREVIEW_W, PREVIEW_H, ui.panel, accent);
        display.fill_rect(
            PREVIEW_X + 12,
            PREVIEW_Y + 14,
            PREVIEW_W - 24,
            54,
            color_box(ui.panel, accent, ui),
        );
        display.stroke_rect(
            PREVIEW_X + 12,
            PREVIEW_Y + 14,
            PREVIEW_W - 24,
            54,
            1,
            accent,
        );
        display.fill_rect(PREVIEW_X + 22, PREVIEW_Y + 24, 34, 24, ui.white);
        display.stroke_rect(PREVIEW_X + 22, PREVIEW_Y + 24, 34, 24, 1, accent);
        draw_app_icon(
            display,
            PREVIEW_X + 30,
            PREVIEW_Y + 28,
            icon,
            accent,
            ui.white,
            ui,
        );
        let preview_title = fit_text_to_width(display, title, 54, 1);
        display.text(
            PREVIEW_X + 66,
            PREVIEW_Y + 26,
            &preview_title,
            ui.text,
            color_box(ui.panel, accent, ui),
            1,
        );
        display.text(
            PREVIEW_X + 66,
            PREVIEW_Y + 40,
            "ARCADE",
            ui.text_muted,
            color_box(ui.panel, accent, ui),
            1,
        );
        display.fill_rect(
            PREVIEW_X + 12,
            PREVIEW_Y + 76,
            PREVIEW_W - 24,
            48,
            ui.panel_alt,
        );
        display.stroke_rect(
            PREVIEW_X + 12,
            PREVIEW_Y + 76,
            PREVIEW_W - 24,
            48,
            1,
            ui.steel,
        );
        let preview_subtitle = fit_text_to_width(display, subtitle, PREVIEW_W - 36, 1);
        display.text(
            PREVIEW_X + 18,
            PREVIEW_Y + 84,
            &preview_subtitle,
            ui.text,
            ui.panel_alt,
            1,
        );
        display.text(
            PREVIEW_X + 18,
            PREVIEW_Y + 98,
            "SHOWCASE READY",
            ui.text_muted,
            ui.panel_alt,
            1,
        );
        display.fill_rect(
            PREVIEW_X + 24,
            PREVIEW_Y + 132,
            PREVIEW_W - 48,
            16,
            color_box(ui.panel_alt, accent, ui),
        );
        display.stroke_rect(
            PREVIEW_X + 24,
            PREVIEW_Y + 132,
            PREVIEW_W - 48,
            16,
            1,
            accent,
        );
        display.centered_text(
            PREVIEW_X + PREVIEW_W / 2,
            PREVIEW_Y + 137,
            "PRESS PLAY",
            ui.text,
            color_box(ui.panel_alt, accent, ui),
            1,
        );
    }
}

fn color_box(fill: u16, accent: u16, ui: &crate::display::Palette) -> u16 {
    color::mix(fill, accent, if accent == ui.white { 28 } else { 22 })
}

fn teaser_for_app(app_id: AppId, zh_mode: bool) -> &'static str {
    match (app_id, zh_mode) {
        (AppId::DungeonCore, true) => "3D 地城冒險",
        (AppId::DungeonCore, false) => "3D QUEST + MAPS",
        (AppId::AutoBattle, true) => "停下鎖定射擊，五關挑戰",
        (AppId::AutoBattle, false) => "5 STAGES / LOCK TO FIRE",
        (AppId::PseudoRacer, true) => "假 3D 道路與檢查點衝刺",
        (AppId::PseudoRacer, false) => "SCANLINE ROAD / TIME RUN",
        (AppId::GraphicsLab, true) => "六種數學圖形即時展示",
        (AppId::GraphicsLab, false) => "6 REALTIME MATH FX MODES",
        (AppId::TapRush, true) => "快節奏反應挑戰",
        (AppId::TapRush, false) => "FAST REACTION ARCADE",
        _ => "",
    }
}

fn fit_text_to_width(display: &Display, text: &str, max_width: u16, scale: u16) -> String<48> {
    let mut exact = String::<48>::new();
    let _ = exact.push_str(text);
    if display.measure_text(&exact, scale) <= max_width {
        return exact;
    }

    let mut fitted = String::<48>::new();
    let ellipsis = "..";
    for ch in text.chars() {
        let mut candidate = fitted.clone();
        if candidate.push(ch).is_err() {
            break;
        }
        let _ = candidate.push_str(ellipsis);
        if display.measure_text(&candidate, scale) > max_width {
            break;
        }
        let _ = fitted.push(ch);
    }
    let _ = fitted.push_str(ellipsis);
    fitted
}
