use super::*;

const STORAGE_COMMIT_DELAY_MS: u32 = 1_500;

impl MiniOs {
    pub fn apply_persisted_state(&mut self, state: PersistedState, _touch: &mut Touch) {
        let settings = state.system;
        self.theme = settings.theme;
        self.language = if settings.language_zh {
            Language::ZhTw
        } else {
            Language::English
        };
        self.render_strategy = settings.render_strategy;
        self.touch_calibration = settings.touch_calibration;
        self.touch_ready = false;
        self.album.restore(apply_album_restore(&state.apps));
        self.paint.restore(apply_paint_restore(&state.apps));
        self.auto_battle.restore(state.apps.station_hunter);
        self.pseudo_racer.restore(state.apps.pseudo_racer);
        self.tap_rush.set_best_score(state.apps.tap_rush_best_score);
        self.recent_app = state.apps.recent_app;
        if let Some(app_id) = self.recent_app {
            self.home_index = app_registry::home_slot_for_app(app_id);
            self.game_center.select_app(app_id);
        }
        self.screen = Screen::TouchCalibrate;
        self.touch_return_screen = Screen::Home;
        self.calibration_step = 0;
        self.calibration_raw_x = [0; 5];
        self.calibration_raw_y = [0; 5];
        self.storage_dirty = false;
        self.storage_dirty_since_ms = 0;
        self.storage_flush_requested = false;
        self.force_full_redraw = true;
    }

    pub fn service_background_tasks(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        self.service_storage_commit(input, touch);
    }

    pub(super) fn request_storage_save(&mut self) {
        self.storage_dirty = true;
        self.storage_dirty_since_ms = millis();
    }

    pub(super) fn request_storage_flush(&mut self) {
        self.request_storage_save();
        self.storage_flush_requested = true;
    }

    pub(super) fn flush_storage(&mut self) -> bool {
        let saved = storage::save(&self.build_persisted_state());
        if saved {
            self.storage_dirty = false;
            self.storage_flush_requested = false;
        } else {
            self.storage_dirty = true;
            self.storage_dirty_since_ms = millis();
            self.storage_flush_requested = false;
        }
        saved
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
                station_hunter: self.auto_battle.snapshot(),
                pseudo_racer: self.pseudo_racer.snapshot(),
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
        self.pseudo_racer = PseudoRacerApp::new();
        self.graphics_lab = GraphicsLabApp::new();
        self.recent_app = None;
        self.home_index = 0;
        self.settings_index = 0;
        self.settings_scroll_top_row = 0;
        self.settings_drag_active = false;
        self.map_index = 0;
        self.album_redraw = None;
        self.paint_redraw = None;
        self.auto_battle_redraw = None;
        self.flush_storage()
    }

    pub(super) fn perform_factory_reset(&mut self, touch_driver: &mut Touch) -> bool {
        if !storage::erase_all() {
            return false;
        }
        touch_driver.set_calibration(TouchCalibration::default());
        *self = MiniOs::new();
        true
    }

    fn service_storage_commit(&mut self, input: &ButtonSnapshot, touch: &TouchState) {
        if !self.storage_dirty {
            return;
        }

        if input.k0
            || input.k1
            || input.wkup
            || touch.active
            || touch.just_pressed
            || touch.just_released
        {
            return;
        }

        let now = millis();
        if !self.storage_flush_requested
            && now.wrapping_sub(self.storage_dirty_since_ms) < STORAGE_COMMIT_DELAY_MS
        {
            return;
        }

        let _ = self.flush_storage();
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
