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
        match self.screen {
            Screen::Home => {
                render_home(display, self.home_index, self.theme, self.language.is_zh())
            }
            Screen::Album => {
                if !full_refresh && self.album_redraw == Some(AlbumRedraw::MotionFrame) {
                    self.album.render_motion_frame(display);
                } else {
                    self.album
                        .render(display, self.theme, self.language.is_zh());
                }
                self.album_redraw = None;
            }
            Screen::GameCenter => {
                self.game_center
                    .render(display, self.theme, self.language.is_zh())
            }
            Screen::MapSelect => {
                render_map_select(display, self.map_index, self.theme, self.language.is_zh())
            }
            Screen::Settings => render_settings(
                display,
                self.theme,
                self.language.is_zh(),
                self.render_strategy,
                self.settings_index,
            ),
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
            Screen::DungeonCore => self.dungeon.render(
                display,
                touch,
                full_refresh,
                self.theme,
                self.language.is_zh(),
                self.fps_estimate,
                self.render_strategy,
            ),
            Screen::Paint => {
                if full_refresh {
                    self.paint
                        .render(display, self.theme, self.language.is_zh());
                } else if let Some(redraw) = self.paint_redraw {
                    self.paint
                        .render_partial(display, self.theme, self.language.is_zh(), redraw);
                }
                self.paint_redraw = None;
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
            }
            Screen::TapRush => self
                .tap_rush
                .render(display, self.theme, self.language.is_zh()),
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
        self.diagnostics_armed = false;
        self.diagnostics_notice = None;
        self.force_full_redraw = true;
    }
}

pub fn boot_sequence(display: &mut Display, theme: ThemeMode, safe_boot: bool) {
    let ui = palette(theme);
    for band in 0..12u16 {
        let tint = (band * 18) as u8;
        let fill = color::mix(ui.canvas, ui.indigo, tint);
        display.fill_rect(0, band * 20, SCREEN_WIDTH, 20, fill);
    }

    display.panel(18, 28, 284, 66, ui.panel, ui.cyan);
    display.centered_text(160, 42, "FINAL PROJECT", ui.text, ui.panel, 2);
    display.centered_text(160, 62, "RUST MINI OS", ui.white, ui.panel, 3);

    display.panel(34, 120, 252, 78, ui.panel_alt, ui.orange);
    display.centered_text(
        160,
        136,
        "TACTILE DUNGEON CONSOLE",
        ui.text,
        ui.panel_alt,
        2,
    );
    display.centered_text(
        160,
        162,
        if safe_boot {
            "SAFE MODE REQUESTED"
        } else {
            "BOOTING GRAPHICS CORE"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    for step in 0..18u16 {
        let fill = 12 + step * 13;
        display.fill_rect(48, 208, fill, 10, ui.cyan);
        display.fill_rect(48 + fill, 208, 224 - fill, 10, ui.panel);
        delay_ms(35);
    }
    delay_ms(160);
}
