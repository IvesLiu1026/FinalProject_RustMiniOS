use super::*;

impl MiniOs {
    pub fn apply_persisted_state(&mut self, state: PersistedState, touch: &mut Touch) {
        let settings = state.system;
        self.theme = settings.theme;
        self.language = if settings.language_zh {
            Language::ZhTw
        } else {
            Language::English
        };
        self.render_strategy = settings.render_strategy;
        self.touch_calibration = settings.touch_calibration;
        self.touch_ready = settings.touch_ready && settings.touch_calibration.valid;
        self.album.restore(apply_album_restore(&state.apps));
        self.paint.restore(apply_paint_restore(&state.apps));
        self.auto_battle
            .set_best_kills(state.apps.auto_battle_best_kills);
        self.tap_rush.set_best_score(state.apps.tap_rush_best_score);
        self.recent_app = state.apps.recent_app;
        if let Some(app_id) = self.recent_app {
            self.home_index = app_registry::home_slot_for_app(app_id);
            self.game_center.select_app(app_id);
        }
        if self.touch_ready {
            touch.set_calibration(settings.touch_calibration);
            self.screen = Screen::Home;
            self.touch_return_screen = Screen::Home;
        }
        self.force_full_redraw = true;
    }

    pub(super) fn save_storage(&self) -> bool {
        storage::save(&self.build_persisted_state())
    }

    fn build_persisted_state(&self) -> PersistedState {
        let album = self.album.snapshot();
        let paint = self.paint.snapshot();
        PersistedState {
            system: PersistedSystemSettings {
                theme: self.theme,
                language_zh: self.language.is_zh(),
                render_strategy: self.render_strategy,
                touch_ready: self.touch_ready,
                touch_calibration: self.touch_calibration,
            },
            apps: PersistedAppData {
                recent_app: self.recent_app,
                album_motion_tab: album.motion_tab,
                album_still_index: album.still_index,
                album_motion_index: album.motion_index,
                album_playing: album.playing,
                paint_selected_color: paint.selected_color,
                paint_pixels: paint.pixels,
                auto_battle_best_kills: self.auto_battle.best_kills(),
                tap_rush_best_score: self.tap_rush.best_score(),
            },
        }
    }

    pub(super) fn clear_app_save_data(&mut self) -> bool {
        self.album = AlbumApp::new();
        self.game_center = GameCenterApp::new();
        self.auto_battle = AutoBattleApp::new();
        self.paint = PaintApp::new();
        self.tap_rush = TapRushApp::new();
        self.recent_app = None;
        self.home_index = 0;
        self.map_index = 0;
        self.album_redraw = None;
        self.paint_redraw = None;
        self.auto_battle_redraw = None;
        self.save_storage()
    }

    pub(super) fn perform_factory_reset(&mut self, touch_driver: &mut Touch) -> bool {
        if !storage::erase_all() {
            return false;
        }
        touch_driver.set_calibration(TouchCalibration::default());
        *self = MiniOs::new();
        true
    }
}

fn apply_album_restore(apps: &PersistedAppData) -> AlbumState {
    AlbumState {
        motion_tab: apps.album_motion_tab,
        still_index: apps.album_still_index,
        motion_index: apps.album_motion_index,
        playing: apps.album_playing,
    }
}

fn apply_paint_restore(apps: &PersistedAppData) -> PaintState {
    PaintState {
        pixels: apps.paint_pixels,
        selected_color: apps.paint_selected_color,
    }
}
