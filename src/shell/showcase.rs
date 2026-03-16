use super::*;

impl MiniOs {
    pub(super) fn enter_showcase_mode(&mut self) {
        self.showcase_mode = Some(ShowcaseMode {
            scene_index: 0,
            scene_elapsed_ms: 0,
            cycle_count: 0,
            last_countdown_sec: 0,
            paused: false,
        });
        self.begin_showcase_scene(0, false);
    }

    pub(super) fn exit_showcase_mode(&mut self) {
        self.showcase_mode = None;
        self.switch_screen(Screen::Settings);
    }

    pub(super) fn showcase_active(&self) -> bool {
        self.showcase_mode.is_some()
    }

    pub(super) fn current_showcase_scene(&self) -> Option<ShowcaseScene> {
        self.showcase_mode
            .map(|mode| SHOWCASE_SCENES[mode.scene_index % SHOWCASE_SCENES.len()])
    }

    pub(super) fn showcase_countdown_sec(&self) -> u8 {
        let Some(mode) = self.showcase_mode else {
            return 0;
        };
        let duration = SHOWCASE_SCENES[mode.scene_index % SHOWCASE_SCENES.len()].duration_ms();
        ((duration
            .saturating_sub(mode.scene_elapsed_ms)
            .saturating_add(999))
            / 1000)
            .clamp(0, u8::MAX as u32) as u8
    }

    pub(super) fn showcase_progress_pct(&self) -> u8 {
        let Some(mode) = self.showcase_mode else {
            return 0;
        };
        let duration = SHOWCASE_SCENES[mode.scene_index % SHOWCASE_SCENES.len()]
            .duration_ms()
            .max(1);
        ((mode.scene_elapsed_ms.min(duration) * 100) / duration) as u8
    }

    pub(super) fn showcase_paused(&self) -> bool {
        self.showcase_mode.map(|mode| mode.paused).unwrap_or(false)
    }

    fn begin_showcase_scene(&mut self, scene_index: usize, bump_cycle: bool) {
        let Some(mut mode) = self.showcase_mode else {
            return;
        };

        if bump_cycle {
            mode.cycle_count = mode.cycle_count.wrapping_add(1);
        }
        mode.scene_index = scene_index % SHOWCASE_SCENES.len();
        mode.scene_elapsed_ms = 0;
        mode.paused = false;
        mode.last_countdown_sec =
            ((SHOWCASE_SCENES[mode.scene_index].duration_ms() + 999) / 1000) as u8;
        self.showcase_mode = Some(mode);

        match SHOWCASE_SCENES[mode.scene_index] {
            ShowcaseScene::Home => {
                self.last_uptime_second = millis() / 1000;
                self.switch_screen(Screen::Home);
            }
            ShowcaseScene::Album => {
                self.album.prepare_showcase();
                self.album_redraw = Some(AlbumRedraw::Full);
                self.performance_focus_app = Some(AppId::Album);
                self.switch_screen(Screen::Album);
            }
            ShowcaseScene::AutoBattle => {
                let stage_index = (mode.cycle_count as usize) % STATION_HUNTER_STAGE_COUNT;
                self.auto_battle.start_showcase(stage_index);
                self.auto_battle_redraw = Some(AutoBattleRedraw::Full);
                self.performance_focus_app = Some(AppId::AutoBattle);
                self.switch_screen(Screen::AutoBattle);
            }
            ShowcaseScene::PseudoRacer => {
                let track_index = (mode.cycle_count as usize) % 3;
                self.pseudo_racer.start_showcase(track_index);
                self.performance_focus_app = Some(AppId::PseudoRacer);
                self.switch_screen(Screen::PseudoRacer);
            }
            ShowcaseScene::GraphicsLab => {
                let mode_index = (mode.cycle_count as usize) % 6;
                self.graphics_lab.start_showcase(mode_index);
                self.performance_focus_app = Some(AppId::GraphicsLab);
                self.switch_screen(Screen::GraphicsLab);
            }
            ShowcaseScene::Diagnostics => {
                self.diagnostics_return_screen = Screen::Settings;
                self.switch_screen(Screen::Diagnostics);
            }
        }
    }

    fn step_showcase_scene(&mut self, delta: i8) {
        let Some(mode) = self.showcase_mode else {
            return;
        };
        let count = SHOWCASE_SCENES.len() as i32;
        let next = (mode.scene_index as i32 + delta as i32).rem_euclid(count) as usize;
        self.begin_showcase_scene(next, true);
    }

    pub(super) fn update_showcase_mode(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> bool {
        if input.home_chord() || showcase_back_released(touch) {
            self.exit_showcase_mode();
            return true;
        }

        if input.k0_just_pressed {
            self.step_showcase_scene(-1);
            return true;
        }
        if input.wkup_just_pressed {
            self.step_showcase_scene(1);
            return true;
        }
        if input.k1_just_pressed {
            if let Some(mode) = self.showcase_mode.as_mut() {
                mode.paused = !mode.paused;
            }
            self.force_full_redraw = true;
            return true;
        }

        let scene = self.current_showcase_scene();
        let neutral_input = ButtonSnapshot::default();
        let neutral_touch = TouchState::default();
        let mut dirty = false;

        match scene {
            Some(ShowcaseScene::Home) => {
                let uptime_second = millis() / 1000;
                if uptime_second != self.last_uptime_second {
                    self.last_uptime_second = uptime_second;
                    dirty = true;
                }
            }
            Some(ShowcaseScene::Album) => {
                let _ = self.album.update(&neutral_input, &neutral_touch, dt_ms);
                self.album_redraw = self.album.take_redraw_request();
                dirty = self.album_redraw.is_some();
            }
            Some(ShowcaseScene::AutoBattle) => {
                let _ = self
                    .auto_battle
                    .update(&neutral_input, &neutral_touch, dt_ms);
                if self.auto_battle.take_persist_request() {
                    self.save_storage();
                }
                self.auto_battle_redraw = self.auto_battle.take_redraw_request();
                dirty = self.auto_battle_redraw.is_some();
            }
            Some(ShowcaseScene::PseudoRacer) => {
                let _ = self
                    .pseudo_racer
                    .update(&neutral_input, &neutral_touch, dt_ms);
                if self.pseudo_racer.take_persist_request() {
                    self.save_storage();
                }
                dirty = self.pseudo_racer.needs_animation();
            }
            Some(ShowcaseScene::GraphicsLab) => {
                let _ = self
                    .graphics_lab
                    .update(&neutral_input, &neutral_touch, dt_ms);
                dirty = self.graphics_lab.needs_animation();
            }
            Some(ShowcaseScene::Diagnostics) => {
                let uptime_second = millis() / 1000;
                if uptime_second != self.last_uptime_second {
                    self.last_uptime_second = uptime_second;
                    dirty = true;
                }
            }
            None => {}
        }

        let mut advance = false;
        if let Some(mode) = self.showcase_mode.as_mut() {
            if !mode.paused {
                mode.scene_elapsed_ms = mode.scene_elapsed_ms.saturating_add(dt_ms);
                let duration = SHOWCASE_SCENES[mode.scene_index].duration_ms();
                let remaining = ((duration
                    .saturating_sub(mode.scene_elapsed_ms)
                    .saturating_add(999))
                    / 1000)
                    .clamp(0, u8::MAX as u32) as u8;
                if remaining != mode.last_countdown_sec {
                    mode.last_countdown_sec = remaining;
                    dirty = true;
                }
                if mode.scene_elapsed_ms >= duration {
                    advance = true;
                }
            }
        }

        if advance {
            self.step_showcase_scene(1);
            return true;
        }

        dirty
    }
}

fn showcase_back_released(touch: &TouchState) -> bool {
    if !touch.just_released {
        return false;
    }
    let center_x = ((touch.start_x as u32 + touch.release_x as u32) / 2) as u16;
    let center_y = ((touch.start_y as u32 + touch.release_y as u32) / 2) as u16;
    center_x >= NAV_BACK_X.saturating_sub(10)
        && center_x < NAV_BACK_X + NAV_BACK_W + 10
        && center_y >= NAV_BACK_Y.saturating_sub(10)
        && center_y < NAV_BACK_Y + NAV_BACK_H + 10
}
