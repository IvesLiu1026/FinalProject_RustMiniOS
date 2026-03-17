use super::*;

impl MiniOs {
    pub fn theme(&self) -> ThemeMode {
        self.theme
    }

    pub fn take_full_redraw(&mut self) -> bool {
        let value = self.force_full_redraw;
        self.force_full_redraw = false;
        value
    }

    pub fn render(
        &mut self,
        display: &mut Display,
        board: &Board,
        touch: &TouchState,
        full_refresh: bool,
    ) {
        self.render_current_screen(display, board, touch, full_refresh, self.screen);

        if let Some(scene) = self.current_showcase_scene() {
            render_showcase_overlay(
                display,
                self.theme,
                self.language.is_zh(),
                scene.title(self.language.is_zh()),
                scene.subtitle(self.language.is_zh()),
                self.showcase_mode.map(|mode| mode.scene_index).unwrap_or(0),
                SHOWCASE_SCENES.len(),
                self.showcase_paused(),
                self.showcase_countdown_sec(),
                self.showcase_progress_pct(),
            );
        }
    }

    fn render_current_screen(
        &mut self,
        display: &mut Display,
        board: &Board,
        touch: &TouchState,
        full_refresh: bool,
        screen: Screen,
    ) {
        if self.render_hosted_app(display, touch, full_refresh) {
            return;
        }

        match screen {
            Screen::Home => render_home(
                display,
                self.home_index,
                self.theme,
                self.language.is_zh(),
                self.fps_estimate,
                millis() / 1000,
            ),
            Screen::Settings => render_settings(
                display,
                self.theme,
                self.language.is_zh(),
                self.render_strategy,
                self.settings_index,
                self.settings_scroll_top_row,
            ),
            Screen::PerformanceConsole => render_performance_console(
                display,
                self.theme,
                self.language.is_zh(),
                self.performance_focus_screen_label(),
                self.performance_focus_title(),
                self.performance_focus_subtitle(),
                self.performance_recent_app_label(),
                self.performance_render_pipeline(),
                self.performance_render_cadence(),
                self.fps_estimate,
                self.render_strategy,
            ),
            Screen::Benchmark => self.render_benchmark(display, full_refresh),
            Screen::About => render_about(
                display,
                self.theme,
                self.language.is_zh(),
                self.safe_boot_session,
            ),
            Screen::Diagnostics => render_diagnostics(
                display,
                board,
                self.theme,
                self.language.is_zh(),
                self.diagnostics_return_screen.label(self.language.is_zh()),
                self.fps_estimate,
                self.touch_ready,
                self.render_strategy,
                storage::inspect(),
                self.safe_boot_session,
                self.diagnostics_action_index,
                self.diagnostics_armed,
                self.diagnostics_notice,
            ),
            Screen::SafeMode => render_safe_mode(
                display,
                self.theme,
                self.language.is_zh(),
                self.safe_mode_index,
                self.touch_ready,
            ),
            Screen::TouchCalibrate => render_touch_calibration(
                display,
                self.calibration_step,
                self.theme,
                self.language.is_zh(),
            ),
            Screen::ControlRoom => {
                render_control_room(display, board, self.theme, self.language.is_zh())
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
    }

    fn clear_transient_redraws(&mut self) {
        self.album_redraw = None;
        self.paint_redraw = None;
        self.auto_battle_redraw = None;
    }

    pub(super) fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.clear_transient_redraws();
        self.settings_drag_active = false;
        self.diagnostics_armed = false;
        self.diagnostics_notice = None;
        if matches!(screen, Screen::Settings) {
            self.sync_settings_scroll_to_selection();
        }
        self.force_full_redraw = true;
    }

    fn performance_focus_title(&self) -> &'static str {
        if let Some(app_id) = self.performance_focus_app {
            return app_registry::descriptor(app_id).title(self.language.is_zh());
        }

        if self.language.is_zh() {
            "復古桌面"
        } else {
            "RETRO DESKTOP"
        }
    }

    fn performance_focus_subtitle(&self) -> &'static str {
        if let Some(app_id) = self.performance_focus_app {
            return app_registry::descriptor(app_id).subtitle(self.language.is_zh());
        }

        if self.language.is_zh() {
            "桌面 shell / icon launcher / taskbar"
        } else {
            "desktop shell / icon launcher / taskbar"
        }
    }

    fn performance_recent_app_label(&self) -> &'static str {
        self.performance_focus_app
            .map(|app_id| app_registry::descriptor(app_id).title(false))
            .unwrap_or("HOME")
    }

    fn performance_focus_screen_label(&self) -> &'static str {
        if let Some(app_id) = self.performance_focus_app {
            return app_registry::descriptor(app_id).title(false);
        }
        "HOME"
    }

    fn performance_render_pipeline(&self) -> &'static str {
        self.performance_focus_app
            .map(|app_id| app_route(app_id).pipeline_label)
            .unwrap_or("DESKTOP + TASKBAR")
    }

    fn performance_render_cadence(&self) -> &'static str {
        self.performance_focus_app
            .map(|app_id| app_route(app_id).cadence_label)
            .unwrap_or("ui event redraw")
    }
}

pub fn boot_sequence(display: &mut Display, theme: ThemeMode, safe_boot: bool, touch_ready: bool) {
    let ui = palette(theme);
    for band in 0..15u16 {
        let tint = (band * 12) as u8;
        let fill = color::mix(ui.canvas, ui.indigo, tint / 2);
        display.fill_rect(0, band * 16, SCREEN_WIDTH, 16, fill);
    }

    display.fill_rect(20, 22, 280, 18, ui.panel_alt);
    display.stroke_rect(20, 22, 280, 18, 1, ui.cyan);
    display.text(30, 27, "MINIOS BIOS", ui.white, ui.panel_alt, 2);
    display.text(206, 28, "ROM CHECK 2026", ui.text_muted, ui.panel_alt, 1);

    display.fill_rect(20, 48, 280, 146, ui.panel);
    display.stroke_rect(20, 48, 280, 146, 1, ui.steel);

    let lines = if safe_boot {
        [
            "POST: DISPLAY ........................ OK",
            "POST: TOUCH CONTROLLER ............... BYPASS",
            "BOOT: SAFE MODE REQUEST .............. ACK",
            "LOAD: RECOVERY TOOLS ................. READY",
            "NEXT: OPEN MINIMAL SERVICE DESKTOP ...",
            "",
        ]
    } else if touch_ready {
        [
            "POST: DISPLAY ........................ OK",
            "POST: TOUCH CALIBRATION .............. OK",
            "LOAD: DESKTOP SHELL .................. READY",
            "LOAD: APP REGISTRY ................... READY",
            "LOAD: MEDIA INDEX .................... READY",
            "NEXT: ENTER RETRO DESKTOP ............",
        ]
    } else {
        [
            "POST: DISPLAY ........................ OK",
            "POST: TOUCH CALIBRATION .............. MISSING",
            "LOAD: INITIAL SETUP WIZARD ........... READY",
            "LOAD: SAFE DEFAULT THEME ............. READY",
            "WARN: DESKTOP ACCESS REQUIRES SETUP ..",
            "NEXT: ENTER TOUCH SETUP ..............",
        ]
    };

    for (index, line) in lines.iter().enumerate() {
        display.text(30, 58 + index as u16 * 18, line, ui.text, ui.panel, 1);
        delay_ms(40);
    }

    let progress_fill = if safe_boot {
        ui.rose
    } else if touch_ready {
        ui.cyan
    } else {
        ui.orange
    };
    display.fill_rect(30, 172, 260, 8, ui.shadow);
    display.stroke_rect(30, 172, 260, 8, 1, ui.steel);
    for step in 0..20u16 {
        let fill = 12 + step * 12;
        display.fill_rect(32, 174, fill, 4, progress_fill);
        delay_ms(35);
    }

    display.text(
        30,
        188,
        if safe_boot {
            "HANDOFF: SAFE MODE DESKTOP"
        } else if touch_ready {
            "HANDOFF: ICON DESKTOP"
        } else {
            "HANDOFF: TOUCH SETUP WIZARD"
        },
        ui.text_muted,
        ui.panel,
        1,
    );

    for column in 0..10u16 {
        let x = column * 32;
        display.fill_rect(x, 0, 16, 240, ui.canvas);
        delay_ms(12);
    }
    delay_ms(100);
}
