use core::fmt::Write;

use heapless::String;

use crate::display::{color, palette, Display, ThemeMode};
use crate::ui::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    render_nav_back,
};

use super::geometry::{arena_inner_rect, fill_rect_clipped, DirtyRegions, Rect};
use super::*;

const RESULT_BUTTON_GAP: u16 = 8;

impl AutoBattleApp {
    pub fn render(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        match self.state {
            BattleState::Profile => self.render_profile(display, theme, zh_mode),
            BattleState::StageSelect => self.render_stage_select(display, theme, zh_mode),
            BattleState::Running | BattleState::BossReward => {
                self.render_battle(display, theme, zh_mode);
            }
            BattleState::StageClear | BattleState::Defeat => {
                self.render_result(display, theme, zh_mode);
            }
        }
    }

    pub fn render_partial(
        &mut self,
        display: &mut Display,
        theme: ThemeMode,
        zh_mode: bool,
        redraw: AutoBattleRedraw,
    ) {
        if self.state != BattleState::Running {
            self.render(display, theme, zh_mode);
            return;
        }

        let ui = palette(theme);
        match redraw {
            AutoBattleRedraw::Full => self.render(display, theme, zh_mode),
            AutoBattleRedraw::Arena => self.render_running_arena(display, zh_mode, &ui),
        }
    }

    fn render_profile(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 118);
        draw_shell_window(display, ui.amber, &ui);
        draw_title_bar(
            display,
            if zh_mode {
                "定點獵手"
            } else {
                "STATION HUNTER"
            },
            if zh_mode {
                "hunter profile / permanent core"
            } else {
                "hunter profile / permanent core"
            },
            ui.amber,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.white, &ui);

        let mut xp_text: String<32> = String::new();
        let _ = write!(
            &mut xp_text,
            "{}/{}",
            self.profile.player_xp,
            Self::xp_to_next_level(self.profile.player_level)
        );
        let mut unlock_text: String<24> = String::new();
        let _ = write!(
            &mut unlock_text,
            "{} / {}",
            self.profile.unlocked_stage, STAGE_COUNT
        );
        draw_info_strip(
            display,
            20,
            44,
            128,
            if zh_mode { "獵手等級" } else { "HUNTER LV" },
            &format_u8(self.profile.player_level),
            ui.amber,
            &ui,
        );
        draw_info_strip(
            display,
            156,
            44,
            144,
            if zh_mode {
                "經驗進度"
            } else {
                "XP TO NEXT"
            },
            &xp_text,
            ui.cyan,
            &ui,
        );
        draw_info_strip(
            display,
            20,
            60,
            128,
            if zh_mode {
                "升級點數"
            } else {
                "UPGRADE PTS"
            },
            &format_u8(self.profile.upgrade_points),
            ui.lime,
            &ui,
        );
        draw_info_strip(
            display,
            156,
            60,
            144,
            if zh_mode { "解鎖關卡" } else { "UNLOCKED" },
            &unlock_text,
            ui.rose,
            &ui,
        );

        for (index, action) in ProfileAction::ALL[..4].iter().copied().enumerate() {
            let selected = self.profile_cursor == index;
            self.render_profile_card(display, action, selected, zh_mode, &ui);
        }

        display.panel(20, 176, 148, 32, ui.panel, ui.cyan);
        display.text(
            28,
            184,
            if zh_mode {
                "本機最高手數"
            } else {
                "BEST KILLS"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            28,
            196,
            &format_u16(self.profile.best_kills),
            ui.text,
            ui.panel,
            1,
        );

        let deploy_selected = self.profile_cursor == 4;
        let deploy_fill = if deploy_selected {
            ui.panel_alt
        } else {
            ui.panel
        };
        let deploy_border = if deploy_selected { ui.amber } else { ui.steel };
        display.panel(
            PROFILE_DEPLOY_X,
            PROFILE_DEPLOY_Y,
            PROFILE_DEPLOY_W,
            PROFILE_DEPLOY_H,
            deploy_fill,
            deploy_border,
        );
        display.centered_text(
            PROFILE_DEPLOY_X + PROFILE_DEPLOY_W / 2,
            PROFILE_DEPLOY_Y + 6,
            if zh_mode {
                "選關出擊"
            } else {
                "STAGE SELECT"
            },
            ui.text,
            deploy_fill,
            1,
        );

        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK 選能力  K1 投資或進入選關"
            } else {
                "K0/WK MOVE  K1 INVEST OR OPEN STAGES"
            },
            ui.amber,
            &ui,
        );
    }

    fn render_profile_card(
        &self,
        display: &mut Display,
        action: ProfileAction,
        selected: bool,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        let (x, y, w, h) = match action {
            ProfileAction::Attack => (
                PROFILE_LEFT_X,
                PROFILE_TOP_Y,
                PROFILE_CARD_W,
                PROFILE_CARD_H,
            ),
            ProfileAction::Vitality => (
                PROFILE_RIGHT_X,
                PROFILE_TOP_Y,
                PROFILE_CARD_W,
                PROFILE_CARD_H,
            ),
            ProfileAction::Trigger => (
                PROFILE_LEFT_X,
                PROFILE_BOTTOM_Y,
                PROFILE_CARD_W,
                PROFILE_CARD_H,
            ),
            ProfileAction::Thrusters => (
                PROFILE_RIGHT_X,
                PROFILE_BOTTOM_Y,
                PROFILE_CARD_W,
                PROFILE_CARD_H,
            ),
            ProfileAction::Deploy => return,
        };
        let accent = match action {
            ProfileAction::Attack => ui.rose,
            ProfileAction::Vitality => ui.lime,
            ProfileAction::Trigger => ui.amber,
            ProfileAction::Thrusters => ui.cyan,
            ProfileAction::Deploy => ui.amber,
        };
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let border = if selected { accent } else { ui.steel };
        display.panel(x, y, w, h, fill, border);
        display.text(
            x + 10,
            y + 8,
            if zh_mode {
                action.title_zh()
            } else {
                action.title_en()
            },
            ui.text,
            fill,
            1,
        );

        let (value, hint) = match action {
            ProfileAction::Attack => (
                format_u8(self.profile.base_attack),
                if zh_mode {
                    "基礎攻擊 +1"
                } else {
                    "BASE DMG +1"
                },
            ),
            ProfileAction::Vitality => (
                format_u8(self.profile.base_hp),
                if zh_mode {
                    "最大血量 +2"
                } else {
                    "MAX HP +2"
                },
            ),
            ProfileAction::Trigger => (
                format_u8(self.profile.base_fire_rate),
                if zh_mode {
                    "更快重新開火"
                } else {
                    "FASTER RE-FIRE"
                },
            ),
            ProfileAction::Thrusters => (
                format_u8(self.profile.base_move_speed),
                if zh_mode {
                    "移動速度更快"
                } else {
                    "MOVE SPEED +"
                },
            ),
            ProfileAction::Deploy => unreachable!(),
        };
        display.text(x + 10, y + 22, hint, ui.text_muted, fill, 1);
        display.fill_rect(x + w - 34, y + 8, 22, 18, color::mix(fill, accent, 24));
        display.stroke_rect(x + w - 34, y + 8, 22, 18, 1, accent);
        display.centered_text(
            x + w - 23,
            y + 12,
            &value,
            ui.text,
            color::mix(fill, accent, 24),
            1,
        );
    }

    fn render_stage_select(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 84);
        draw_shell_window(display, ui.cyan, &ui);
        draw_title_bar(
            display,
            if zh_mode {
                "選擇關卡"
            } else {
                "STAGE SELECT"
            },
            if zh_mode {
                "5 關逐步解鎖 / boss 每 10 waves"
            } else {
                "5 stages / bosses every 10 waves"
            },
            ui.cyan,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.white, &ui);

        draw_info_strip(
            display,
            20,
            42,
            132,
            if zh_mode { "玩家等級" } else { "PLAYER LV" },
            &format_u8(self.profile.player_level),
            ui.amber,
            &ui,
        );
        draw_info_strip(
            display,
            164,
            42,
            136,
            if zh_mode { "已解鎖" } else { "UNLOCKED" },
            &format_u8(self.profile.unlocked_stage),
            ui.lime,
            &ui,
        );

        for index in 0..STAGE_COUNT {
            self.render_stage_card(
                display,
                index,
                index == self.stage_select_index,
                zh_mode,
                &ui,
            );
        }

        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK 選關  K1 進入  BACK 返回角色頁"
            } else {
                "K0/WK SELECT  K1 DEPLOY  BACK TO PROFILE"
            },
            ui.cyan,
            &ui,
        );
    }

    fn render_stage_card(
        &self,
        display: &mut Display,
        stage_index: usize,
        selected: bool,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        let stage = self.stage_def(stage_index);
        let y = STAGE_CARD_Y + stage_index as u16 * (STAGE_CARD_H + STAGE_CARD_GAP);
        let unlocked = self.is_stage_unlocked(stage_index);
        let fill = if selected { ui.panel_alt } else { ui.panel };
        let accent = if unlocked { ui.amber } else { ui.steel };
        display.panel(STAGE_CARD_X, y, STAGE_CARD_W, STAGE_CARD_H, fill, accent);
        display.text(
            STAGE_CARD_X + 10,
            y + 7,
            if zh_mode {
                stage.title_zh
            } else {
                stage.title_en
            },
            if unlocked { ui.text } else { ui.text_muted },
            fill,
            1,
        );
        display.text(
            STAGE_CARD_X + 10,
            y + 18,
            if zh_mode {
                stage.note_zh
            } else {
                stage.note_en
            },
            ui.text_muted,
            fill,
            1,
        );

        let right_x = STAGE_CARD_X + STAGE_CARD_W - 68;
        if unlocked {
            let mut badge: String<20> = String::new();
            let _ = write!(
                &mut badge,
                "W{} C{}",
                self.stage_best_wave(stage_index),
                self.stage_clear_count(stage_index)
            );
            display.text(right_x, y + 7, &badge, ui.text, fill, 1);
            display.text(
                right_x + 36,
                y + 18,
                &format_u16(self.stage_best_kills(stage_index)),
                ui.text_muted,
                fill,
                1,
            );
            let best_wave = self.stage_best_wave(stage_index);
            for boss_idx in 0..3u16 {
                let cleared = best_wave >= (boss_idx as u8 + 1) * BOSS_INTERVAL;
                let pip_x = right_x + boss_idx * 12;
                let pip_fill = if cleared {
                    color::mix(fill, ui.rose, 36)
                } else {
                    color::mix(fill, ui.shadow, 40)
                };
                let pip_border = if cleared { ui.rose } else { ui.steel };
                display.fill_rect(pip_x, y + 20, 8, 6, pip_fill);
                display.stroke_rect(pip_x, y + 20, 8, 6, 1, pip_border);
            }
        } else {
            display.fill_rect(right_x, y + 7, 46, 12, color::mix(fill, ui.shadow, 40));
            display.stroke_rect(right_x, y + 7, 46, 12, 1, ui.steel);
            display.centered_text(
                right_x + 23,
                y + 10,
                "LOCK",
                ui.text_muted,
                color::mix(fill, ui.shadow, 40),
                1,
            );
        }
    }

    fn render_result(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        let cleared = self.state == BattleState::StageClear;
        let accent = if cleared { ui.lime } else { ui.rose };
        let stage_index = self.result_summary.stage.saturating_sub(1) as usize;
        let stage = self.stage_def(stage_index.min(STAGE_COUNT - 1));
        let best_wave = self.stage_best_wave(stage_index.min(STAGE_COUNT - 1));
        let best_kills = self.stage_best_kills(stage_index.min(STAGE_COUNT - 1));
        let clear_count = self.stage_clear_count(stage_index.min(STAGE_COUNT - 1));
        let result_score = station_hunter_result_score(&self.result_summary);
        let result_grade = station_hunter_result_grade(result_score);
        draw_gradient_background(display, theme, 106);
        draw_shell_window(display, accent, &ui);
        draw_title_bar(
            display,
            if cleared {
                if zh_mode {
                    "關卡完成"
                } else {
                    "MISSION CLEAR"
                }
            } else if zh_mode {
                "獵手倒下"
            } else {
                "SYSTEM FAIL"
            },
            if cleared {
                if zh_mode {
                    "永久成長已記錄"
                } else {
                    "permanent growth recorded"
                }
            } else if zh_mode {
                "這一局已結算，可立即重試"
            } else {
                "run ended / retry or return"
            },
            accent,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.white, &ui);

        display.panel(20, 44, 280, 18, ui.panel_alt, accent);
        display.text(
            28,
            50,
            if zh_mode {
                stage.title_zh
            } else {
                stage.title_en
            },
            ui.text,
            ui.panel_alt,
            1,
        );
        display.fill_rect(244, 47, 48, 12, color::mix(ui.panel, accent, 22));
        display.stroke_rect(244, 47, 48, 12, 1, accent);
        display.centered_text(
            268,
            50,
            result_grade,
            ui.white,
            color::mix(ui.panel, accent, 22),
            1,
        );

        draw_info_strip(
            display,
            20,
            66,
            132,
            if zh_mode { "關卡" } else { "STAGE" },
            &format_u8(self.result_summary.stage),
            accent,
            &ui,
        );
        draw_info_strip(
            display,
            164,
            66,
            136,
            if zh_mode { "波次" } else { "WAVE" },
            &format_u8(self.result_summary.wave_reached),
            ui.cyan,
            &ui,
        );

        display.panel(24, 90, 272, 86, ui.panel, accent);
        display.text(
            36,
            100,
            if zh_mode { "本局擊殺" } else { "RUN KILLS" },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            140,
            100,
            &format_u16(self.result_summary.kills),
            ui.text,
            ui.panel,
            1,
        );
        display.text(
            36,
            114,
            if zh_mode { "永久經驗" } else { "PERMA XP" },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            140,
            114,
            &format_u16(self.result_summary.xp_gain),
            ui.text,
            ui.panel,
            1,
        );
        display.text(
            36,
            128,
            if zh_mode {
                "升級成長"
            } else {
                "LEVEL GAIN"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            140,
            128,
            &format_u8(self.result_summary.level_gained),
            ui.text,
            ui.panel,
            1,
        );
        display.text(
            36,
            142,
            if zh_mode {
                "升級點數"
            } else {
                "UPGRADE PTS"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            140,
            142,
            &format_u8(self.result_summary.upgrade_points_gain),
            ui.text,
            ui.panel,
            1,
        );
        display.text(
            196,
            100,
            if zh_mode { "分數" } else { "SCORE" },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(236, 100, &format_u16(result_score), accent, ui.panel, 1);
        let mut best_line: String<24> = String::new();
        let _ = write!(
            &mut best_line,
            "{} {} / {}",
            if zh_mode { "紀錄" } else { "BEST" },
            best_wave,
            best_kills
        );
        display.text(196, 114, &best_line, ui.text, ui.panel, 1);
        let mut clear_line: String<24> = String::new();
        let _ = write!(
            &mut clear_line,
            "{} {}",
            if zh_mode { "通關次數" } else { "CLEARS" },
            clear_count
        );
        display.text(196, 128, &clear_line, ui.text_muted, ui.panel, 1);
        if let Some(unlocked) = self.result_summary.unlocked_stage {
            let mut unlock_text: String<24> = String::new();
            let _ = write!(
                &mut unlock_text,
                "{} {}",
                if zh_mode { "解鎖關卡" } else { "UNLOCKED" },
                unlocked
            );
            display.text(196, 142, &unlock_text, accent, ui.panel, 1);
        }

        let labels = if cleared {
            (
                if zh_mode { "角色頁" } else { "PROFILE" },
                if zh_mode { "關卡列表" } else { "STAGES" },
            )
        } else {
            (
                if zh_mode { "再試一次" } else { "RETRY" },
                if zh_mode { "回角色頁" } else { "PROFILE" },
            )
        };
        for index in 0..2 {
            let y = 184 + index as u16 * (RESULT_BUTTON_H + RESULT_BUTTON_GAP);
            let selected = self.result_choice == index;
            let fill = if selected { ui.panel_alt } else { ui.panel };
            let border = if selected { accent } else { ui.steel };
            display.panel(
                RESULT_BUTTON_X,
                y,
                RESULT_BUTTON_W,
                RESULT_BUTTON_H,
                fill,
                border,
            );
            display.centered_text(
                RESULT_BUTTON_X + RESULT_BUTTON_W / 2,
                y + 5,
                if index == 0 { labels.0 } else { labels.1 },
                ui.text,
                fill,
                1,
            );
        }
    }

    fn render_battle(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 104);

        self.render_header(display, zh_mode, &ui);
        self.render_arena_region(display, zh_mode, &ui);

        if self.state == BattleState::BossReward {
            self.render_reward_overlay(display, zh_mode, &ui);
            self.render_footer(display, zh_mode, &ui);
        }
    }

    fn render_header(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.panel(12, 8, 176, 24, ui.panel, ui.amber);
        render_nav_back(display, zh_mode, ui.white, ui);
        display.text(
            84,
            16,
            if zh_mode {
                "定點獵手"
            } else {
                "STATION HUNTER"
            },
            ui.text,
            ui.panel,
            1,
        );

        let chip_accent = match self.wave_tracker.kind {
            WaveKind::Standard => ui.cyan,
            WaveKind::Pressure => ui.orange,
            WaveKind::Elite => ui.amber,
            WaveKind::Boss => ui.rose,
        };
        let chip_label = match (zh_mode, self.wave_tracker.kind) {
            (true, WaveKind::Standard) => "普",
            (true, WaveKind::Pressure) => "壓",
            (true, WaveKind::Elite) => "菁",
            (true, WaveKind::Boss) => "B",
            (false, WaveKind::Standard) => "STD",
            (false, WaveKind::Pressure) => "PRS",
            (false, WaveKind::Elite) => "ELT",
            (false, WaveKind::Boss) => "BOSS",
        };
        let mut stage_text: String<18> = String::new();
        let _ = write!(
            &mut stage_text,
            "S{} W{}",
            self.current_stage, self.wave_tracker.wave
        );
        display.panel(196, 8, 112, 24, ui.panel, chip_accent);
        display.text(204, 16, &stage_text, ui.text, ui.panel, 1);
        display.fill_rect(264, 12, 34, 12, color::mix(ui.panel_alt, chip_accent, 28));
        display.stroke_rect(264, 12, 34, 12, 1, chip_accent);
        display.centered_text(
            281,
            15,
            chip_label,
            ui.text,
            color::mix(ui.panel_alt, chip_accent, 28),
            1,
        );
    }

    fn render_footer(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        draw_footer_hint(
            display,
            if zh_mode {
                "K0/WK 選擇  K1 確認  點卡片也可"
            } else {
                "K0/WK PICK  K1 CONFIRM  OR TAP A CARD"
            },
            ui.amber,
            ui,
        );
    }

    fn render_arena_region(
        &mut self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.panel(ARENA_X, ARENA_Y, ARENA_W, ARENA_H, ui.panel, ui.cyan);
        self.render_arena_background(display, ui);
        self.render_arena_dynamic(display, zh_mode, ui);
        self.last_arena_frame = self.capture_arena_frame();
        self.arena_frame_valid = self.state == BattleState::Running;
    }

    fn render_running_arena(
        &mut self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        if self.state != BattleState::Running || !self.arena_frame_valid {
            self.render_arena_region(display, zh_mode, ui);
            return;
        }

        let current = self.capture_arena_frame();
        let mut dirty = DirtyRegions::new();
        self.last_arena_frame.collect_dirty_regions(&mut dirty);
        current.collect_dirty_regions(&mut dirty);

        for rect in dirty.as_slice() {
            self.render_arena_background_rect(display, ui, *rect);
        }
        self.render_arena_dynamic(display, zh_mode, ui);
        self.last_arena_frame = current;
        self.arena_frame_valid = true;
    }

    fn render_arena_background(&self, display: &mut Display, ui: &crate::display::Palette) {
        let floor_top = color::mix(ui.canvas, ui.panel_alt, 104);
        let floor_bottom = color::mix(ui.panel_alt, ui.indigo, 54);
        display.fill_rect(
            ARENA_INNER_X,
            ARENA_INNER_Y,
            ARENA_INNER_W,
            ARENA_INNER_H,
            floor_top,
        );

        for band in 0..7u16 {
            let shade = color::mix(floor_top, floor_bottom, (band * 28) as u8);
            let y = ARENA_Y + 8 + band * 24;
            display.fill_rect(ARENA_X + 6, y, ARENA_W - 12, 18, shade);
        }
        for lane in 0..4u16 {
            let y = ARENA_Y + 24 + lane * 36;
            display.fill_rect(
                ARENA_X + 8,
                y,
                ARENA_W - 16,
                1,
                color::mix(ui.white, floor_bottom, 24),
            );
        }
        for lane in 0..3u16 {
            let x = ARENA_X + 54 + lane * 50;
            display.fill_rect(
                x,
                ARENA_Y + 8,
                1,
                ARENA_H - 16,
                color::mix(ui.white, floor_bottom, 28),
            );
        }
    }

    fn render_arena_background_rect(
        &self,
        display: &mut Display,
        ui: &crate::display::Palette,
        clip: Rect,
    ) {
        let floor_top = color::mix(ui.canvas, ui.panel_alt, 104);
        let floor_bottom = color::mix(ui.panel_alt, ui.indigo, 54);
        fill_rect_clipped(display, clip, arena_inner_rect(), floor_top);

        for band in 0..7u16 {
            let shade = color::mix(floor_top, floor_bottom, (band * 28) as u8);
            fill_rect_clipped(
                display,
                clip,
                Rect {
                    x: (ARENA_X + 6) as i16,
                    y: (ARENA_Y + 8 + band * 24) as i16,
                    w: (ARENA_W - 12) as i16,
                    h: 18,
                },
                shade,
            );
        }
        for lane in 0..4u16 {
            fill_rect_clipped(
                display,
                clip,
                Rect {
                    x: (ARENA_X + 8) as i16,
                    y: (ARENA_Y + 24 + lane * 36) as i16,
                    w: (ARENA_W - 16) as i16,
                    h: 1,
                },
                color::mix(ui.white, floor_bottom, 24),
            );
        }
        for lane in 0..3u16 {
            fill_rect_clipped(
                display,
                clip,
                Rect {
                    x: (ARENA_X + 54 + lane * 50) as i16,
                    y: (ARENA_Y + 8) as i16,
                    w: 1,
                    h: (ARENA_H - 16) as i16,
                },
                color::mix(ui.white, floor_bottom, 28),
            );
        }
    }

    fn render_arena_dynamic(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        if self.moving {
            let tx = ARENA_X + self.target_x as u16;
            let ty = ARENA_Y + self.target_y as u16;
            display.stroke_rect(
                tx.saturating_sub(6),
                ty.saturating_sub(6),
                12,
                12,
                1,
                ui.cyan,
            );
        }

        for projectile in &self.projectiles {
            if !projectile.active {
                continue;
            }
            let glow = if projectile.from_enemy {
                color::mix(ui.rose, ui.white, 82)
            } else {
                color::mix(ui.amber, ui.white, 82)
            };
            let core = if projectile.from_enemy {
                ui.rose
            } else {
                ui.amber
            };
            let px = ARENA_X as i16 + projectile.x as i16;
            let py = ARENA_Y as i16 + projectile.y as i16;
            let tail_x = px - (projectile.vx * 18.0) as i16;
            let tail_y = py - (projectile.vy * 18.0) as i16;
            display.fill_rect(
                tail_x.min(px).saturating_sub(1) as u16,
                tail_y.min(py).saturating_sub(1) as u16,
                (tail_x.abs_diff(px) + 2).max(2),
                (tail_y.abs_diff(py) + 2).max(2),
                glow,
            );
            display.fill_rect(
                px.saturating_sub(2) as u16,
                py.saturating_sub(2) as u16,
                4,
                4,
                core,
            );
        }

        for enemy in &self.enemies {
            if !enemy.active {
                continue;
            }
            let ex = ARENA_X as i16 + enemy.x as i16;
            let ey = ARENA_Y as i16 + enemy.y as i16;
            let size = enemy.size as i16;
            display.fill_rect(
                ex.saturating_sub(size / 2 + 2) as u16,
                ey.saturating_add(size / 2 - 1) as u16,
                size as u16 + 4,
                3,
                color::mix(ui.shadow, ui.rose, 24),
            );

            let base_fill = match enemy.kind {
                EnemyKind::Runner => {
                    color::mix(ui.orange, ui.amber, ((enemy.phase >> 3) & 0x1F) as u8)
                }
                EnemyKind::Shooter => {
                    color::mix(ui.indigo, ui.cyan, ((enemy.phase >> 4) & 0x1F) as u8)
                }
                EnemyKind::Bruiser => {
                    color::mix(ui.rose, ui.orange, ((enemy.phase >> 5) & 0x1F) as u8)
                }
                EnemyKind::Dasher => {
                    color::mix(ui.cyan, ui.white, ((enemy.phase >> 3) & 0x1F) as u8)
                }
                EnemyKind::Summoner => {
                    color::mix(ui.indigo, ui.lime, ((enemy.phase >> 4) & 0x1F) as u8)
                }
                EnemyKind::BossRam => color::mix(ui.rose, ui.orange, 120),
                EnemyKind::BossBurst => color::mix(ui.indigo, ui.white, 84),
                EnemyKind::BossNest => color::mix(ui.lime, ui.indigo, 84),
                EnemyKind::BossRing => color::mix(ui.cyan, ui.amber, 92),
            };
            let fill = if enemy.flash_ms > 0 {
                color::mix(base_fill, ui.white, 170)
            } else {
                base_fill
            };
            display.fill_rect(
                ex.saturating_sub(size / 2) as u16,
                ey.saturating_sub(size / 2) as u16,
                size as u16,
                size as u16,
                fill,
            );
            display.stroke_rect(
                ex.saturating_sub(size / 2) as u16,
                ey.saturating_sub(size / 2) as u16,
                size as u16,
                size as u16,
                1,
                if enemy.kind.is_boss() {
                    ui.white
                } else {
                    ui.text
                },
            );

            match enemy.kind {
                EnemyKind::Runner => {
                    display.fill_rect(
                        ex.saturating_sub(2) as u16,
                        ey.saturating_sub(size / 2 + 2) as u16,
                        4,
                        2,
                        ui.white,
                    );
                }
                EnemyKind::Shooter => {
                    display.fill_rect(
                        ex.saturating_sub(1) as u16,
                        ey.saturating_sub(1) as u16,
                        2,
                        2,
                        ui.white,
                    );
                    display.fill_rect(
                        ex.saturating_sub(5) as u16,
                        ey.saturating_sub(1) as u16,
                        3,
                        2,
                        ui.cyan,
                    );
                }
                EnemyKind::Bruiser => {
                    display.fill_rect(
                        ex.saturating_sub(4) as u16,
                        ey.saturating_sub(size / 2 + 2) as u16,
                        8,
                        2,
                        ui.orange,
                    );
                }
                EnemyKind::Dasher => {
                    display.fill_rect(
                        ex.saturating_sub(5) as u16,
                        ey.saturating_sub(5) as u16,
                        3,
                        3,
                        ui.white,
                    );
                    display.fill_rect(
                        ex.saturating_add(2) as u16,
                        ey.saturating_add(2) as u16,
                        3,
                        3,
                        ui.cyan,
                    );
                }
                EnemyKind::Summoner => {
                    display.fill_rect(
                        ex.saturating_sub(1) as u16,
                        ey.saturating_sub(5) as u16,
                        2,
                        10,
                        ui.lime,
                    );
                    display.fill_rect(
                        ex.saturating_sub(5) as u16,
                        ey.saturating_sub(1) as u16,
                        10,
                        2,
                        ui.lime,
                    );
                }
                EnemyKind::BossRam => {
                    display.fill_rect(
                        ex.saturating_sub(6) as u16,
                        ey.saturating_sub(2) as u16,
                        12,
                        4,
                        ui.white,
                    );
                }
                EnemyKind::BossBurst => {
                    display.fill_rect(
                        ex.saturating_sub(1) as u16,
                        ey.saturating_sub(7) as u16,
                        2,
                        14,
                        ui.white,
                    );
                    display.fill_rect(
                        ex.saturating_sub(7) as u16,
                        ey.saturating_sub(1) as u16,
                        14,
                        2,
                        ui.cyan,
                    );
                }
                EnemyKind::BossNest => {
                    display.fill_rect(
                        ex.saturating_sub(5) as u16,
                        ey.saturating_sub(5) as u16,
                        10,
                        10,
                        ui.lime,
                    );
                    display.fill_rect(
                        ex.saturating_sub(2) as u16,
                        ey.saturating_sub(2) as u16,
                        4,
                        4,
                        ui.indigo,
                    );
                }
                EnemyKind::BossRing => {
                    display.stroke_rect(
                        ex.saturating_sub(7) as u16,
                        ey.saturating_sub(7) as u16,
                        14,
                        14,
                        1,
                        ui.white,
                    );
                    display.fill_rect(
                        ex.saturating_sub(2) as u16,
                        ey.saturating_sub(2) as u16,
                        4,
                        4,
                        ui.amber,
                    );
                }
            }

            let hp_w =
                ((enemy.hp.max(0) as u16 * (size as u16 + 2)) / enemy.max_hp.max(1) as u16).max(1);
            display.fill_rect(
                ex.saturating_sub(size / 2) as u16,
                ey.saturating_sub(size / 2 + 6) as u16,
                size as u16 + 2,
                2,
                ui.shadow,
            );
            display.fill_rect(
                ex.saturating_sub(size / 2) as u16,
                ey.saturating_sub(size / 2 + 6) as u16,
                hp_w,
                2,
                color::mix(ui.rose, ui.white, 88),
            );
        }

        for pickup in &self.pickups {
            if !pickup.active {
                continue;
            }
            let px = ARENA_X as i16 + pickup.x as i16;
            let py = ARENA_Y as i16 + pickup.y as i16;
            let size = pickup.size as i16;
            let fill = color::mix(ui.lime, ui.white, 92);
            display.fill_rect(
                px.saturating_sub(size / 2) as u16,
                py.saturating_sub(size / 2) as u16,
                size as u16,
                size as u16,
                fill,
            );
            display.stroke_rect(
                px.saturating_sub(size / 2) as u16,
                py.saturating_sub(size / 2) as u16,
                size as u16,
                size as u16,
                1,
                ui.white,
            );
            display.fill_rect(
                px.saturating_sub(1) as u16,
                py.saturating_sub(4) as u16,
                2,
                8,
                ui.white,
            );
            display.fill_rect(
                px.saturating_sub(4) as u16,
                py.saturating_sub(1) as u16,
                8,
                2,
                ui.white,
            );
        }

        if !self.moving {
            if let Some((nx, ny)) = self.nearest_enemy_position() {
                let ex = ARENA_X as i16 + nx as i16;
                let ey = ARENA_Y as i16 + ny as i16;
                display.stroke_rect(
                    ex.saturating_sub(9) as u16,
                    ey.saturating_sub(9) as u16,
                    18,
                    18,
                    1,
                    ui.amber,
                );
                let px = ARENA_X as i16 + self.player_x as i16;
                let py = ARENA_Y as i16 + self.player_y as i16;
                display.fill_rect(
                    px.min(ex) as u16,
                    py as u16,
                    px.abs_diff(ex).max(1),
                    1,
                    ui.amber,
                );
                display.fill_rect(
                    ex as u16,
                    py.min(ey) as u16,
                    1,
                    py.abs_diff(ey).max(1),
                    ui.amber,
                );
            }
        }

        let px = ARENA_X as i16 + self.player_x as i16;
        let py = ARENA_Y as i16 + self.player_y as i16;
        if self.weapon_flash_ms > 0 {
            let flash = color::mix(ui.amber, ui.white, 140);
            display.fill_rect(
                px.saturating_sub(16) as u16,
                py.saturating_sub(1) as u16,
                5,
                2,
                flash,
            );
            display.fill_rect(
                px.saturating_add(11) as u16,
                py.saturating_sub(1) as u16,
                5,
                2,
                flash,
            );
            display.fill_rect(
                px.saturating_sub(1) as u16,
                py.saturating_sub(16) as u16,
                2,
                5,
                flash,
            );
            display.fill_rect(
                px.saturating_sub(1) as u16,
                py.saturating_add(11) as u16,
                2,
                5,
                flash,
            );
        }
        if self.heal_flash_ms > 0 {
            let heal = color::mix(ui.lime, ui.white, 140);
            display.stroke_rect(
                px.saturating_sub(12) as u16,
                py.saturating_sub(12) as u16,
                24,
                24,
                1,
                heal,
            );
            display.fill_rect(
                px.saturating_sub(1) as u16,
                py.saturating_sub(13) as u16,
                2,
                6,
                heal,
            );
            display.fill_rect(
                px.saturating_sub(1) as u16,
                py.saturating_add(8) as u16,
                2,
                6,
                heal,
            );
        }
        if self.damage_flash_ms > 0 {
            let warn = color::mix(ui.rose, ui.white, 104);
            display.stroke_rect(
                px.saturating_sub(14) as u16,
                py.saturating_sub(14) as u16,
                28,
                28,
                1,
                warn,
            );
        }
        let player_fill = if self.hit_invuln_ms > 0 {
            color::mix(ui.cyan, ui.white, 156)
        } else {
            color::mix(ui.cyan, ui.white, 78)
        };
        display.fill_rect(
            px.saturating_sub(6) as u16,
            py.saturating_add(5) as u16,
            12,
            3,
            color::mix(ui.shadow, ui.cyan, 28),
        );
        display.fill_rect(
            px.saturating_sub(PLAYER_SIZE / 2) as u16,
            py.saturating_sub(PLAYER_SIZE / 2) as u16,
            PLAYER_SIZE as u16,
            PLAYER_SIZE as u16,
            player_fill,
        );
        display.stroke_rect(
            px.saturating_sub(PLAYER_SIZE / 2) as u16,
            py.saturating_sub(PLAYER_SIZE / 2) as u16,
            PLAYER_SIZE as u16,
            PLAYER_SIZE as u16,
            1,
            ui.white,
        );
        display.fill_rect(
            px.saturating_sub(1) as u16,
            py.saturating_sub(10) as u16,
            2,
            4,
            ui.white,
        );
        display.fill_rect(
            px.saturating_sub(10) as u16,
            py.saturating_sub(1) as u16,
            4,
            2,
            ui.white,
        );
        display.fill_rect(
            px.saturating_add(6) as u16,
            py.saturating_sub(1) as u16,
            4,
            2,
            ui.white,
        );

        if !self.moving {
            display.stroke_rect(
                px.saturating_sub(12) as u16,
                py.saturating_sub(12) as u16,
                24,
                24,
                1,
                ui.amber,
            );
        }

        self.render_player_status_bars(display, ui, px, py);

        if self.banner_active() {
            self.render_wave_banner(display, zh_mode, ui);
        }
    }

    fn render_player_status_bars(
        &self,
        display: &mut Display,
        ui: &crate::display::Palette,
        px: i16,
        py: i16,
    ) {
        let bar_x = px.saturating_sub(12) as u16;
        let hp_y = py.saturating_sub(18) as u16;
        let xp_y = py.saturating_sub(13) as u16;
        let bar_w = 24u16;
        let hp_fill =
            ((self.health.max(0) as u32 * bar_w as u32) / self.max_health.max(1) as u32) as u16;
        let xp_target = Self::xp_to_next_level(self.profile.player_level).max(1) as u32;
        let xp_fill = ((self.profile.player_xp.min(xp_target as u16) as u32 * bar_w as u32)
            / xp_target) as u16;

        display.fill_rect(
            bar_x,
            hp_y,
            bar_w,
            3,
            color::mix(ui.shadow, ui.panel_alt, 34),
        );
        if hp_fill > 0 {
            display.fill_rect(
                bar_x,
                hp_y,
                hp_fill.min(bar_w),
                3,
                color::mix(ui.lime, ui.white, 92),
            );
        }
        display.fill_rect(bar_x, xp_y, bar_w, 3, color::mix(ui.shadow, ui.indigo, 34));
        if xp_fill > 0 {
            display.fill_rect(
                bar_x,
                xp_y,
                xp_fill.min(bar_w),
                3,
                color::mix(ui.cyan, ui.white, 84),
            );
        }
    }

    fn render_reward_overlay(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.fill_rect(20, 62, 280, 142, color::mix(ui.shadow, ui.orange, 24));
        display.panel(28, 54, 264, 142, ui.panel_alt, ui.orange);
        display.centered_text(
            160,
            66,
            if self.wave_tracker.wave % BOSS_INTERVAL == 0 {
                if zh_mode {
                    "Boss 核心獎勵"
                } else {
                    "BOSS CORE REWARD"
                }
            } else if zh_mode {
                "Wave 升級"
            } else {
                "WAVE UPGRADE"
            },
            ui.text,
            ui.panel_alt,
            2,
        );
        display.centered_text(
            160,
            82,
            if zh_mode {
                "選一個 buff，下一波會放一個醫療包"
            } else {
                "PICK A BUFF, NEXT WAVE GETS A MED KIT"
            },
            ui.text_muted,
            ui.panel_alt,
            1,
        );

        for index in 0..LEVEL_UP_CHOICES {
            let buff = self.reward_choices[index];
            let y = BUFF_Y + index as u16 * (BUFF_H + BUFF_GAP);
            let selected = self.selected_choice == index;
            let accent = buff.accent(ui);
            let fill = if selected { ui.panel } else { ui.panel_alt };
            let border = if selected { accent } else { ui.steel };
            display.panel(BUFF_X, y, BUFF_W, BUFF_H, fill, border);
            display.text(
                BUFF_X + 10,
                y + 8,
                if zh_mode {
                    buff.title_zh()
                } else {
                    buff.title_en()
                },
                ui.text,
                fill,
                1,
            );
            display.text(
                BUFF_X + 10,
                y + 22,
                if zh_mode {
                    buff.desc_zh()
                } else {
                    buff.desc_en()
                },
                ui.text_muted,
                fill,
                1,
            );
            display.panel(BUFF_X + 218, y + 8, 32, 18, fill, accent);
            display.centered_text(BUFF_X + 234, y + 13, "PICK", ui.text, fill, 1);
        }
    }

    fn render_wave_banner(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        let accent = if self.wave_tracker.kind == WaveKind::Boss {
            ui.rose
        } else {
            match self.wave_tracker.kind {
                WaveKind::Standard => ui.cyan,
                WaveKind::Pressure => ui.orange,
                WaveKind::Elite => ui.amber,
                WaveKind::Boss => ui.rose,
            }
        };
        let fill = color::mix(ui.panel_alt, accent, 18);
        let title_y = ARENA_Y + 16;
        let subtitle_y = ARENA_Y + 28;
        display.fill_rect(
            ARENA_X + 14,
            ARENA_Y + 10,
            156,
            30,
            color::mix(ui.shadow, accent, 12),
        );
        display.panel(ARENA_X + 18, ARENA_Y + 12, 148, 24, fill, accent);

        let title = if self.wave_tracker.kind == WaveKind::Boss {
            if zh_mode {
                "Boss 波"
            } else {
                "BOSS WAVE"
            }
        } else {
            if zh_mode {
                "下一波開始"
            } else {
                "WAVE START"
            }
        };
        let subtitle = if self.wave_tracker.kind == WaveKind::Boss {
            if let Some(kind) = self.active_boss_kind() {
                if zh_mode {
                    kind.title_zh()
                } else {
                    kind.title_en()
                }
            } else {
                if zh_mode {
                    "核心入場"
                } else {
                    "CORE INBOUND"
                }
            }
        } else {
            if zh_mode {
                self.wave_tracker.kind.title_zh()
            } else {
                self.wave_tracker.kind.title_en()
            }
        };
        display.text(ARENA_X + 28, title_y, title, ui.text, fill, 1);
        display.text(ARENA_X + 28, subtitle_y, subtitle, ui.text_muted, fill, 1);

        let mut wave_text: String<12> = String::new();
        let _ = write!(&mut wave_text, "W{}", self.wave_tracker.wave);
        display.fill_rect(
            ARENA_X + 128,
            ARENA_Y + 17,
            28,
            12,
            color::mix(fill, accent, 24),
        );
        display.stroke_rect(ARENA_X + 128, ARENA_Y + 17, 28, 12, 1, accent);
        display.centered_text(
            ARENA_X + 142,
            ARENA_Y + 20,
            &wave_text,
            ui.text,
            color::mix(fill, accent, 24),
            1,
        );
    }
}

impl BuffKind {
    fn accent(self, ui: &crate::display::Palette) -> u16 {
        match self {
            Self::MultiShot => ui.cyan,
            Self::VitalCore => ui.lime,
            Self::Impact => ui.rose,
            Self::QuickTrigger => ui.amber,
            Self::Velocity => ui.orange,
            Self::Thrusters => ui.cyan,
            Self::PhaseRound => ui.white,
            Self::LongBarrel => ui.indigo,
            Self::GuardShell => ui.lime,
        }
    }
}

fn format_u8(value: u8) -> String<8> {
    let mut out = String::<8>::new();
    let _ = write!(&mut out, "{}", value);
    out
}

fn format_u16(value: u16) -> String<12> {
    let mut out = String::<12>::new();
    let _ = write!(&mut out, "{}", value);
    out
}

fn station_hunter_result_score(summary: &ResultSummary) -> u16 {
    let score = summary.kills as u32 * 7
        + summary.wave_reached as u32 * 12
        + summary.xp_gain as u32 * 2
        + summary.level_gained as u32 * 36
        + summary.upgrade_points_gain as u32 * 28;
    score.min(u16::MAX as u32) as u16
}

fn station_hunter_result_grade(score: u16) -> &'static str {
    match score {
        720..=u16::MAX => "S",
        560..=719 => "A",
        420..=559 => "B",
        300..=419 => "C",
        _ => "D",
    }
}
