use super::*;

impl MiniOs {
    pub fn update_frame_timing(&mut self, frame_dt_ms: u32) {
        let instant_fps = (1000u32 / frame_dt_ms.max(1)).min(99) as u16;
        self.fps_estimate = if self.fps_estimate == 0 {
            instant_fps
        } else {
            (((self.fps_estimate as u32 * 7) + instant_fps as u32) / 8) as u16
        };
    }

    pub fn update(
        &mut self,
        board: &mut Board,
        input: &ButtonSnapshot,
        touch: &TouchState,
        touch_driver: &mut Touch,
        dt_ms: u32,
    ) -> bool {
        if self.showcase_active() {
            return self.update_showcase_mode(input, touch, dt_ms);
        }

        if let Some(dirty) = self.update_hosted_app(input, touch, dt_ms) {
            return dirty;
        }

        let mut dirty = false;
        match self.screen {
            Screen::Home => {
                let home_app_count = home_apps().len();
                if input.k0_just_pressed {
                    self.home_index =
                        self.home_index.wrapping_add(home_app_count - 1) % home_app_count;
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.home_index = (self.home_index + 1) % home_app_count;
                    dirty = true;
                }
                if input.k1_just_pressed {
                    self.open_selected_screen();
                    dirty = true;
                }

                if touch.just_released {
                    for index in 0..home_app_count {
                        let (x, y, width, height) = desktop_icon_rect(index);
                        if touch_started_in_rect(touch, x, y, width, height) {
                            if self.home_index == index {
                                self.open_selected_screen();
                            } else {
                                self.home_index = index;
                            }
                            dirty = true;
                            break;
                        }
                    }
                }
            }
            Screen::Settings => {
                if input.home_chord() {
                    self.switch_screen(Screen::Home);
                    return true;
                }

                if input.k0_just_pressed {
                    self.settings_index =
                        (self.settings_index + SETTINGS_ITEM_COUNT - 1) % SETTINGS_ITEM_COUNT;
                    self.sync_settings_scroll_to_selection();
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.settings_index = (self.settings_index + 1) % SETTINGS_ITEM_COUNT;
                    self.sync_settings_scroll_to_selection();
                    dirty = true;
                }
                if input.k1_just_pressed {
                    return self.activate_settings_item();
                }

                if touch.just_pressed && settings_list_contains(touch.x, touch.y) {
                    self.settings_drag_active = true;
                    self.settings_drag_anchor_y = touch.y;
                    self.settings_drag_origin_row = self.settings_scroll_top_row;
                }

                if self.settings_drag_active && touch.active && touch.dragging {
                    let delta = self.settings_drag_anchor_y as i32 - touch.y as i32;
                    let steps = delta / SETTINGS_ROW_HEIGHT as i32;
                    let top = (self.settings_drag_origin_row as i32 + steps)
                        .clamp(0, settings_max_scroll_top() as i32)
                        as usize;
                    if top != self.settings_scroll_top_row {
                        self.settings_scroll_top_row = top;
                        dirty = true;
                    }
                }

                if touch.just_released {
                    let was_drag = self.settings_drag_active && touch.dragging;
                    self.settings_drag_active = false;
                    if touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                    {
                        self.switch_screen(Screen::Home);
                        return true;
                    }
                    if was_drag {
                        return dirty;
                    }
                    if let Some((tap_x, tap_y)) = released_tap_point(touch) {
                        if let Some(index) =
                            settings_item_at_point(tap_x, tap_y, self.settings_scroll_top_row)
                        {
                            if self.settings_index == index {
                                return self.activate_settings_item();
                            }
                            self.settings_index = index;
                            self.sync_settings_scroll_to_selection();
                            dirty = true;
                        }
                    }
                }
            }
            Screen::PerformanceConsole => {
                if input.home_chord() {
                    self.switch_screen(Screen::Settings);
                    return true;
                }
                if input.k1_just_pressed {
                    self.enter_benchmark_mode();
                    return true;
                }
                let uptime_second = millis() / 1000;
                if uptime_second != self.last_uptime_second {
                    self.last_uptime_second = uptime_second;
                    dirty = true;
                }
                if touch.just_released
                    && touch_started_in_rect(
                        touch,
                        PERF_BENCH_X,
                        PERF_BENCH_Y,
                        PERF_BENCH_W,
                        PERF_BENCH_H,
                    )
                {
                    self.enter_benchmark_mode();
                    return true;
                }
                if touch.just_released
                    && touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    self.switch_screen(Screen::Settings);
                    return true;
                }
            }
            Screen::Benchmark => {
                return self.update_benchmark(input, touch, dt_ms);
            }
            Screen::About => {
                if input.home_chord() {
                    self.switch_screen(Screen::Settings);
                    return true;
                }
                if touch.just_released
                    && touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                {
                    self.switch_screen(Screen::Settings);
                    return true;
                }
            }
            Screen::Diagnostics => {
                if input.home_chord() {
                    self.switch_screen(self.diagnostics_return_screen);
                    return true;
                }
                if input.k0_just_pressed {
                    self.select_diagnostics_action(
                        self.diagnostics_action_index + DIAG_ACTION_COUNT - 1,
                    );
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.select_diagnostics_action(self.diagnostics_action_index + 1);
                    dirty = true;
                }
                if input.k1_just_pressed {
                    return self.activate_diagnostics_action(touch_driver);
                }
                if touch.just_released {
                    if touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                    {
                        self.switch_screen(self.diagnostics_return_screen);
                        return true;
                    }
                    if touch_started_in_rect(
                        touch,
                        DIAG_CLEAR_X,
                        DIAG_ACTION_Y,
                        DIAG_ACTION_W,
                        DIAG_ACTION_H,
                    ) {
                        if self.diagnostics_action_index == 0 {
                            return self.activate_diagnostics_action(touch_driver);
                        }
                        self.select_diagnostics_action(0);
                        dirty = true;
                    } else if touch_started_in_rect(
                        touch,
                        DIAG_RESET_X,
                        DIAG_ACTION_Y,
                        DIAG_ACTION_W,
                        DIAG_ACTION_H,
                    ) {
                        if self.diagnostics_action_index == 1 {
                            return self.activate_diagnostics_action(touch_driver);
                        }
                        self.select_diagnostics_action(1);
                        dirty = true;
                    }
                }
            }
            Screen::SafeMode => {
                const SAFE_MODE_COUNT: usize = 3;
                if input.k0_just_pressed {
                    self.safe_mode_index =
                        (self.safe_mode_index + SAFE_MODE_COUNT - 1) % SAFE_MODE_COUNT;
                    dirty = true;
                }
                if input.wkup_just_pressed {
                    self.safe_mode_index = (self.safe_mode_index + 1) % SAFE_MODE_COUNT;
                    dirty = true;
                }
                if input.k1_just_pressed {
                    return self.activate_safe_mode_item();
                }
                if touch.just_released {
                    for index in 0..SAFE_MODE_COUNT {
                        let y = 110 + index as u16 * 34;
                        if touch_started_in_rect(touch, 18, y, 284, 28) {
                            if self.safe_mode_index == index {
                                return self.activate_safe_mode_item();
                            }
                            self.safe_mode_index = index;
                            dirty = true;
                            break;
                        }
                    }
                }
            }
            Screen::TouchCalibrate => {
                if self.touch_ready && (input.k0_just_pressed || input.home_chord()) {
                    self.return_from_touch_calibration();
                    return true;
                }
                if touch.just_released {
                    if self.touch_ready
                        && touch_started_in_rect(
                            touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H,
                        )
                    {
                        self.return_from_touch_calibration();
                        return true;
                    }
                    dirty = true;
                    let index = self.calibration_step as usize;
                    if index < 5 {
                        self.calibration_raw_x[index] = touch.raw_x;
                        self.calibration_raw_y[index] = touch.raw_y;
                        self.calibration_step = self.calibration_step.saturating_add(1);
                    }

                    if self.calibration_step >= 5 {
                        if self.commit_touch_calibration(touch_driver) {
                            self.return_from_touch_calibration();
                        } else {
                            self.calibration_step = 0;
                            self.calibration_raw_x = [0; 5];
                            self.calibration_raw_y = [0; 5];
                            self.force_full_redraw = true;
                        }
                    }
                }
            }
            Screen::ControlRoom => {
                if input.k1_just_pressed {
                    board.toggle_led();
                    dirty = true;
                }
                if touch.just_released {
                    if touch_started_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
                    {
                        self.switch_screen(Screen::Settings);
                        return true;
                    }
                    if touch_started_in_rect(touch, 18, 56, 284, 70)
                        || touch_started_in_rect(touch, 20, 138, 85, 52)
                    {
                        board.toggle_led();
                        dirty = true;
                    } else if touch_started_in_rect(touch, 18, 206, 284, 24) {
                        self.switch_screen(Screen::Settings);
                        return true;
                    }
                }
                if input.home_chord() {
                    self.switch_screen(Screen::Settings);
                    return true;
                }
                let uptime_second = millis() / 1000;
                if uptime_second != self.last_uptime_second {
                    self.last_uptime_second = uptime_second;
                    dirty = true;
                }
            }
            Screen::Album
            | Screen::GameCenter
            | Screen::MapSelect
            | Screen::DungeonCore
            | Screen::Paint
            | Screen::AutoBattle
            | Screen::TapRush
            | Screen::PseudoRacer
            | Screen::GraphicsLab => unreachable!("hosted app screens are handled above"),
        }
        dirty
    }

    fn open_control_room(&mut self) {
        self.last_uptime_second = millis() / 1000;
        self.switch_screen(Screen::ControlRoom);
    }

    fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.request_storage_save();
        self.force_full_redraw = true;
    }

    fn open_selected_screen(&mut self) {
        if let Some(app_id) = home_apps().get(self.home_index).copied() {
            self.launch_app(app_id);
        }
    }

    fn launch_app(&mut self, app_id: AppId) {
        self.begin_app_launch(app_id, true);
    }

    fn activate_settings_item(&mut self) -> bool {
        match self.settings_index {
            0 => {
                self.toggle_theme();
                true
            }
            1 => {
                self.language.toggle();
                self.request_storage_save();
                true
            }
            2 => {
                self.render_strategy = self.render_strategy.next();
                self.request_storage_save();
                true
            }
            3 => {
                self.open_control_room();
                true
            }
            4 => {
                self.enter_touch_calibration(Screen::Settings);
                true
            }
            5 => {
                self.enter_showcase_mode();
                true
            }
            6 => {
                self.last_uptime_second = millis() / 1000;
                self.switch_screen(Screen::PerformanceConsole);
                true
            }
            7 => {
                self.diagnostics_return_screen = Screen::Settings;
                self.switch_screen(Screen::Diagnostics);
                true
            }
            _ => {
                self.switch_screen(Screen::About);
                true
            }
        }
    }

    pub(super) fn sync_settings_scroll_to_selection(&mut self) {
        let visual_row = settings_visual_row_for_item(self.settings_index);
        if visual_row < self.settings_scroll_top_row {
            self.settings_scroll_top_row = visual_row;
        } else if visual_row >= self.settings_scroll_top_row + SETTINGS_VISIBLE_ROWS {
            self.settings_scroll_top_row = visual_row + 1 - SETTINGS_VISIBLE_ROWS;
        }
        self.settings_scroll_top_row = self.settings_scroll_top_row.min(settings_max_scroll_top());
    }

    fn activate_safe_mode_item(&mut self) -> bool {
        match self.safe_mode_index {
            0 => {
                self.switch_screen(Screen::Home);
                true
            }
            1 => {
                self.enter_touch_calibration(Screen::SafeMode);
                true
            }
            _ => {
                self.diagnostics_return_screen = Screen::SafeMode;
                self.switch_screen(Screen::Diagnostics);
                true
            }
        }
    }

    fn select_diagnostics_action(&mut self, index: usize) {
        self.diagnostics_action_index = index % DIAG_ACTION_COUNT;
        self.diagnostics_armed = false;
        self.diagnostics_notice = None;
    }

    fn activate_diagnostics_action(&mut self, touch_driver: &mut Touch) -> bool {
        if self.diagnostics_armed {
            self.diagnostics_armed = false;
            match self.diagnostics_action_index {
                0 => {
                    if self.clear_app_save_data() {
                        self.diagnostics_notice = Some(DiagnosticsNotice::Cleared);
                    } else {
                        self.diagnostics_notice = Some(DiagnosticsNotice::ClearFailed);
                    }
                    self.force_full_redraw = true;
                    true
                }
                _ => {
                    if self.perform_factory_reset(touch_driver) {
                        true
                    } else {
                        self.diagnostics_notice = Some(DiagnosticsNotice::ResetFailed);
                        self.force_full_redraw = true;
                        true
                    }
                }
            }
        } else {
            self.diagnostics_armed = true;
            self.diagnostics_notice = Some(match self.diagnostics_action_index {
                0 => DiagnosticsNotice::ClearReady,
                _ => DiagnosticsNotice::ResetReady,
            });
            self.force_full_redraw = true;
            true
        }
    }
}

pub(super) fn touch_started_in_rect(
    touch: &TouchState,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> bool {
    if touch.dragging {
        return point_in_rect_with_slop(touch.start_x, touch.start_y, x, y, width, height)
            && point_in_rect_with_slop(touch.release_x, touch.release_y, x, y, width, height);
    }

    let (tap_x, tap_y) = match released_tap_point(touch) {
        Some(point) => point,
        None => return false,
    };
    point_in_rect_with_slop(tap_x, tap_y, x, y, width, height)
}

fn released_tap_point(touch: &TouchState) -> Option<(u16, u16)> {
    if !touch.just_released {
        return None;
    }
    Some((
        ((touch.start_x as u32 + touch.release_x as u32) / 2) as u16,
        ((touch.start_y as u32 + touch.release_y as u32) / 2) as u16,
    ))
}

fn point_in_rect_with_slop(px: u16, py: u16, x: u16, y: u16, width: u16, height: u16) -> bool {
    let slop = 10u16;
    let left = x.saturating_sub(slop);
    let top = y.saturating_sub(slop);
    let right = x.saturating_add(width).saturating_add(slop);
    let bottom = y.saturating_add(height).saturating_add(slop);
    px >= left && px < right && py >= top && py < bottom
}
