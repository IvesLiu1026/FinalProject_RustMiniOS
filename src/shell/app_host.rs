use super::*;
use super::update::touch_started_in_rect;
use crate::shell_contract::{
    reduce_dungeon_host, reduce_map_select, DungeonHostIntent, DungeonHostSignals,
    HostedAppNavigation, MapSelectIntent,
};

#[derive(Clone, Copy)]
struct HostedAppUpdateResult {
    dirty: bool,
    navigation: HostedAppNavigation,
}

impl HostedAppUpdateResult {
    const fn stay(dirty: bool) -> Self {
        Self {
            dirty,
            navigation: HostedAppNavigation::Stay,
        }
    }

    const fn launch(app_id: AppId) -> Self {
        Self {
            dirty: true,
            navigation: HostedAppNavigation::Launch(app_id),
        }
    }

    const fn exit(app_id: AppId, persist_state: bool) -> Self {
        Self {
            dirty: true,
            navigation: HostedAppNavigation::Exit {
                app_id,
                persist_state,
            },
        }
    }
}

impl MiniOs {
    pub(super) fn update_hosted_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<bool> {
        let result = match self.screen {
            Screen::Album => self.update_album_app(input, touch, dt_ms),
            Screen::GameCenter => self.update_game_center_app(input, touch),
            Screen::MapSelect => self.update_map_select_app(input, touch),
            Screen::DungeonCore => self.update_dungeon_app(input, touch, dt_ms),
            Screen::Paint => self.update_paint_app(input, touch),
            Screen::AutoBattle => self.update_auto_battle_app(input, touch, dt_ms),
            Screen::TapRush => self.update_tap_rush_app(input, touch, dt_ms),
            Screen::PseudoRacer => self.update_pseudo_racer_app(input, touch, dt_ms),
            Screen::GraphicsLab => self.update_graphics_lab_app(input, touch, dt_ms),
            _ => None,
        }?;

        Some(self.apply_hosted_app_update(result))
    }

    pub(super) fn render_hosted_app(
        &mut self,
        display: &mut Display,
        touch: &TouchState,
        full_refresh: bool,
    ) -> bool {
        match self.screen {
            Screen::Album => {
                if !full_refresh && self.album_redraw == Some(AlbumRedraw::MotionFrame) {
                    self.album.render_motion_frame(display);
                } else {
                    self.album
                        .render(display, self.theme, self.language.is_zh());
                }
                self.album_redraw = None;
                true
            }
            Screen::GameCenter => {
                self.game_center
                    .render(display, self.theme, self.language.is_zh());
                true
            }
            Screen::MapSelect => {
                render_map_select(display, self.map_index, self.theme, self.language.is_zh());
                true
            }
            Screen::DungeonCore => {
                self.dungeon.render(
                    display,
                    touch,
                    full_refresh,
                    self.theme,
                    self.language.is_zh(),
                    self.fps_estimate,
                    self.render_strategy,
                );
                true
            }
            Screen::Paint => {
                if full_refresh {
                    self.paint
                        .render(display, self.theme, self.language.is_zh());
                } else if let Some(redraw) = self.paint_redraw {
                    self.paint
                        .render_partial(display, self.theme, self.language.is_zh(), redraw);
                }
                self.paint_redraw = None;
                true
            }
            Screen::AutoBattle => {
                if full_refresh {
                    self.auto_battle
                        .render(display, self.theme, self.language.is_zh());
                } else if let Some(redraw) = self.auto_battle_redraw {
                    self.auto_battle.render_partial(
                        display,
                        self.theme,
                        self.language.is_zh(),
                        redraw,
                    );
                }
                self.auto_battle_redraw = None;
                true
            }
            Screen::TapRush => {
                self.tap_rush
                    .render(display, self.theme, self.language.is_zh());
                true
            }
            Screen::PseudoRacer => {
                if !full_refresh && self.pseudo_racer.can_partial_render() {
                    self.pseudo_racer
                        .render_partial(display, self.theme, self.language.is_zh());
                } else {
                    self.pseudo_racer
                        .render(display, self.theme, self.language.is_zh());
                }
                true
            }
            Screen::GraphicsLab => {
                if !full_refresh && self.graphics_lab.can_partial_render() {
                    self.graphics_lab
                        .render_partial(display, self.theme, self.language.is_zh());
                } else {
                    self.graphics_lab
                        .render(display, self.theme, self.language.is_zh());
                }
                true
            }
            _ => false,
        }
    }

    fn apply_hosted_app_update(&mut self, result: HostedAppUpdateResult) -> bool {
        match result.navigation {
            HostedAppNavigation::Stay => result.dirty,
            HostedAppNavigation::Launch(app_id) => {
                self.begin_app_launch(app_id, true);
                true
            }
            HostedAppNavigation::Switch(screen) => {
                self.switch_screen(screen);
                true
            }
            HostedAppNavigation::Exit {
                app_id,
                persist_state,
            } => self.exit_app(app_id, persist_state),
        }
    }

    fn update_album_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        match self.album.update(input, touch, dt_ms) {
            AlbumAction::ExitHome => Some(HostedAppUpdateResult::exit(AppId::Album, true)),
            AlbumAction::Stay => {
                self.album_redraw = self.album.take_redraw_request();
                Some(HostedAppUpdateResult::stay(self.album_redraw.is_some()))
            }
        }
    }

    fn update_game_center_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
    ) -> Option<HostedAppUpdateResult> {
        match self.game_center.update(input, touch) {
            GameCenterAction::Launch(app_id) => Some(HostedAppUpdateResult::launch(app_id)),
            GameCenterAction::ExitHome => Some(HostedAppUpdateResult::exit(AppId::GameCenter, false)),
            GameCenterAction::Stay => Some(HostedAppUpdateResult::stay(
                input.k0_just_pressed || input.wkup_just_pressed || touch.just_released,
            )),
        }
    }

    fn update_dungeon_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        let intent = match self.dungeon.update(input, touch, dt_ms) {
            DungeonAction::ExitHome => DungeonHostIntent::ExitToGameCenter,
            DungeonAction::OpenMapSelect => DungeonHostIntent::OpenMapSelect,
            DungeonAction::Stay => DungeonHostIntent::Stay,
        };
        let outcome = reduce_dungeon_host(
            intent,
            DungeonHostSignals {
                animation_active: self.dungeon.needs_animation(),
                redraw_requested: self.dungeon.take_redraw_request(),
                k0_just_pressed: input.k0_just_pressed,
                k1_just_pressed: input.k1_just_pressed,
                wkup_just_pressed: input.wkup_just_pressed,
                home_chord: input.home_chord(),
                touch_just_pressed: touch.just_pressed,
                touch_just_released: touch.just_released,
            },
        );
        Some(HostedAppUpdateResult {
            dirty: outcome.dirty,
            navigation: outcome.navigation,
        })
    }

    fn update_map_select_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
    ) -> Option<HostedAppUpdateResult> {
        let mut intent = MapSelectIntent::Idle;
        if input.k0_just_pressed {
            intent = MapSelectIntent::Previous;
        } else if input.wkup_just_pressed {
            intent = MapSelectIntent::Next;
        } else if input.k1_just_pressed {
            intent = MapSelectIntent::LaunchCurrent;
        } else if input.home_chord() {
            intent = MapSelectIntent::ExitToGameCenter;
        } else if touch.just_released {
            if touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H) {
                intent = MapSelectIntent::ExitToGameCenter;
            }
            for index in 0..DungeonApp::map_count() {
                let y = 72 + index as u16 * 44;
                if touch_started_in_rect(touch, 20, y, 280, 36) {
                    intent = MapSelectIntent::SelectMap(index);
                    break;
                }
            }
            if touch_started_in_rect(touch, 22, 208, 276, 20) {
                intent = MapSelectIntent::ExitToGameCenter;
            }
        }

        let outcome = reduce_map_select(self.map_index, DungeonApp::map_count(), intent);
        self.map_index = outcome.next_map_index;
        if outcome.prepare_dungeon_launch {
            self.prepare_map_launch();
        }
        Some(HostedAppUpdateResult {
            dirty: outcome.dirty,
            navigation: outcome.navigation,
        })
    }

    fn update_paint_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
    ) -> Option<HostedAppUpdateResult> {
        match self.paint.update(input, touch) {
            PaintAction::ExitHome => Some(HostedAppUpdateResult::exit(AppId::Paint, true)),
            PaintAction::Stay => {
                self.paint_redraw = self.paint.take_redraw_request();
                Some(HostedAppUpdateResult::stay(self.paint_redraw.is_some()))
            }
        }
    }

    fn update_auto_battle_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        match self.auto_battle.update(input, touch, dt_ms) {
            AutoBattleAction::ExitGameCenter => {
                Some(HostedAppUpdateResult::exit(AppId::AutoBattle, true))
            }
            AutoBattleAction::Stay => {
                if self.auto_battle.take_persist_request() {
                    self.request_storage_save();
                }
                self.auto_battle_redraw = self.auto_battle.take_redraw_request();
                Some(HostedAppUpdateResult::stay(self.auto_battle_redraw.is_some()))
            }
        }
    }

    fn update_tap_rush_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        match self.tap_rush.update(input, touch, dt_ms) {
            TapRushAction::ExitGameCenter => Some(HostedAppUpdateResult::exit(AppId::TapRush, true)),
            TapRushAction::Stay => Some(HostedAppUpdateResult::stay(
                self.tap_rush.needs_animation() || self.tap_rush.take_redraw_request(),
            )),
        }
    }

    fn update_pseudo_racer_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        match self.pseudo_racer.update(input, touch, dt_ms) {
            PseudoRacerAction::ExitGameCenter => {
                Some(HostedAppUpdateResult::exit(AppId::PseudoRacer, true))
            }
            PseudoRacerAction::Stay => {
                if self.pseudo_racer.take_persist_request() {
                    self.request_storage_save();
                }
                if self.pseudo_racer.take_full_redraw_request() {
                    self.force_full_redraw = true;
                }
                Some(HostedAppUpdateResult::stay(
                    self.force_full_redraw
                        || self.pseudo_racer.take_render_request()
                        || input.k0_just_pressed
                        || input.k1_just_pressed
                        || input.wkup_just_pressed
                        || touch.just_released,
                ))
            }
        }
    }

    fn update_graphics_lab_app(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> Option<HostedAppUpdateResult> {
        match self.graphics_lab.update(input, touch, dt_ms) {
            GraphicsLabAction::ExitGameCenter => {
                Some(HostedAppUpdateResult::exit(AppId::GraphicsLab, false))
            }
            GraphicsLabAction::Stay => {
                if self.graphics_lab.take_full_redraw_request() {
                    self.force_full_redraw = true;
                }
                Some(HostedAppUpdateResult::stay(
                    self.force_full_redraw
                        || self.graphics_lab.take_render_request()
                        || input.k0_just_pressed
                        || input.k1_just_pressed
                        || input.wkup_just_pressed
                        || touch.just_released,
                ))
            }
        }
    }
}
