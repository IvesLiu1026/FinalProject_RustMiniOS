use core::fmt::Write;

use heapless::String;

use crate::display::{color, palette, Display, ThemeMode};
use crate::ui::{draw_gradient_background, render_nav_back};

use super::geometry::{arena_inner_rect, fill_rect_clipped, DirtyRegions, Rect};
use super::*;

impl AutoBattleApp {
    pub fn render(&mut self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 104);

        self.render_header(display, zh_mode, &ui);
        self.render_arena_region(display, &ui);
        self.render_panel_region(display, zh_mode, &ui);

        if self.state == BattleState::LevelUp {
            self.render_level_up_overlay(display, zh_mode, &ui);
        } else if !matches!(self.state, BattleState::Running) {
            self.render_overlay(display, zh_mode, &ui);
        }

        if !matches!(self.state, BattleState::Running) {
            self.render_footer(display, zh_mode, &ui);
        }
    }

    pub fn render_partial(
        &mut self,
        display: &mut Display,
        theme: ThemeMode,
        zh_mode: bool,
        redraw: AutoBattleRedraw,
    ) {
        let ui = palette(theme);
        match redraw {
            AutoBattleRedraw::Full => self.render(display, theme, zh_mode),
            AutoBattleRedraw::Arena => {
                self.render_running_arena(display, &ui);
                self.render_panel_region(display, zh_mode, &ui);
            }
            AutoBattleRedraw::ArenaAndPanel => {
                self.render_running_arena(display, &ui);
                self.render_panel_region(display, zh_mode, &ui);
            }
            AutoBattleRedraw::Overlay => self.render_overlay_region(display, zh_mode, &ui),
        }
    }

    fn render_header(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.panel(12, 8, 152, 24, ui.panel, ui.orange);
        render_nav_back(display, zh_mode, ui.white, &ui);
        display.text(
            74,
            16,
            if zh_mode {
                "自動獵手"
            } else {
                "AUTO HUNTER"
            },
            ui.text,
            ui.panel,
            1,
        );
    }

    fn render_footer(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.panel(18, 226, 284, 12, ui.panel, ui.white);
        display.text(
            24,
            228,
            match self.state {
                BattleState::LevelUp => {
                    if zh_mode {
                        "K0/WK 選 buff，K1 確認，也可以直接點卡片"
                    } else {
                        "K0/WK TO PICK, K1 TO CONFIRM, OR TAP A CARD"
                    }
                }
                _ => {
                    if zh_mode {
                        "拖曳競技場移動，放手自動開火，K0 或返回離開"
                    } else {
                        "DRAG TO MOVE, RELEASE TO AUTO-FIRE, K0 OR BACK TO EXIT"
                    }
                }
            },
            ui.text_muted,
            ui.panel,
            1,
        );
    }

    fn render_arena_region(&mut self, display: &mut Display, ui: &crate::display::Palette) {
        display.panel(ARENA_X, ARENA_Y, ARENA_W, ARENA_H, ui.panel, ui.cyan);
        self.render_arena_background(display, &ui);
        self.render_arena_dynamic(display, &ui);
        self.last_arena_frame = self.capture_arena_frame();
        self.arena_frame_valid = matches!(self.state, BattleState::Running);
    }

    fn render_panel_region(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        self.render_panel(display, zh_mode, &ui);
    }

    fn render_overlay_region(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        if self.state == BattleState::LevelUp {
            self.render_level_up_overlay(display, zh_mode, ui);
        } else if !matches!(self.state, BattleState::Running) {
            self.render_overlay(display, zh_mode, ui);
        }
    }

    fn render_running_arena(&mut self, display: &mut Display, ui: &crate::display::Palette) {
        if !matches!(self.state, BattleState::Running) || !self.arena_frame_valid {
            self.render_arena_region(display, ui);
            return;
        }

        let current = self.capture_arena_frame();
        let mut dirty = DirtyRegions::new();
        self.last_arena_frame.collect_dirty_regions(&mut dirty);
        current.collect_dirty_regions(&mut dirty);

        for rect in dirty.as_slice() {
            self.render_arena_background_rect(display, ui, *rect);
        }
        self.render_arena_dynamic(display, ui);
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

    fn render_arena_dynamic(&self, display: &mut Display, ui: &crate::display::Palette) {
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
                match enemy.kind {
                    EnemyKind::Runner => ui.white,
                    EnemyKind::Shooter => ui.cyan,
                    EnemyKind::Bruiser => ui.orange,
                    EnemyKind::Dasher => ui.white,
                    EnemyKind::Summoner => ui.lime,
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
    }

    fn render_overlay(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        let (title, subtitle, accent, action) = match self.state {
            BattleState::Ready => (
                if zh_mode { "準備開始" } else { "READY" },
                if zh_mode {
                    "進場後先閃躲，停下來就會自動射擊"
                } else {
                    "DODGE FIRST, THEN STOP TO AUTO-FIRE"
                },
                ui.amber,
                if zh_mode { "開始" } else { "START" },
            ),
            BattleState::Victory => (
                if zh_mode { "清場完成" } else { "CLEAR" },
                if zh_mode {
                    "你撐住了，按下重新開始再打一次"
                } else {
                    "YOU SURVIVED THE WAVE. PLAY AGAIN?"
                },
                ui.lime,
                if zh_mode { "再來一局" } else { "REPLAY" },
            ),
            BattleState::Defeat => (
                if zh_mode { "獵手倒下" } else { "TRY AGAIN" },
                if zh_mode {
                    "下一次先拉開距離再停下來射"
                } else {
                    "CREATE SPACE, THEN PLANT AND FIRE"
                },
                ui.rose,
                if zh_mode { "重試" } else { "RETRY" },
            ),
            BattleState::Running | BattleState::LevelUp => return,
        };

        let mut best_text: String<24> = String::new();
        let _ = write!(
            &mut best_text,
            "{} {}",
            if zh_mode {
                "最佳擊殺"
            } else {
                "BEST KILLS"
            },
            self.best_kills
        );

        display.fill_rect(34, 90, 252, 86, color::mix(ui.shadow, accent, 26));
        display.panel(42, 82, 236, 86, ui.panel_alt, accent);
        display.centered_text(160, 98, title, ui.text, ui.panel_alt, 2);
        display.centered_text(160, 122, subtitle, ui.text_muted, ui.panel_alt, 1);
        display.centered_text(160, 142, &best_text, ui.text_muted, ui.panel_alt, 1);
        display.panel(
            OVERLAY_ACTION_X,
            OVERLAY_ACTION_Y,
            OVERLAY_ACTION_W,
            OVERLAY_ACTION_H,
            ui.panel,
            accent,
        );
        display.centered_text(
            OVERLAY_ACTION_X + OVERLAY_ACTION_W / 2,
            OVERLAY_ACTION_Y + 6,
            action,
            ui.text,
            ui.panel,
            1,
        );
    }

    fn render_level_up_overlay(
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
            if zh_mode { "升級選擇" } else { "LEVEL UP" },
            ui.text,
            ui.panel_alt,
            2,
        );
        display.centered_text(
            160,
            82,
            if zh_mode {
                "從三個 buff 中選一個"
            } else {
                "PICK ONE RANDOM BUFF"
            },
            ui.text_muted,
            ui.panel_alt,
            1,
        );

        for index in 0..LEVEL_UP_CHOICES {
            let buff = self.level_up_choices[index];
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

    fn render_panel(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        let shell = color::mix(ui.panel_alt, ui.shadow, 42);
        display.fill_rect(
            PANEL_X + 2,
            PANEL_Y + 2,
            PANEL_W,
            PANEL_H,
            color::mix(ui.shadow, ui.cyan, 14),
        );
        display.panel(PANEL_X, PANEL_Y, PANEL_W, PANEL_H, shell, ui.cyan);

        let hp_bar_w = PANEL_W - 28;
        let hp_fill = ((self.health.max(0) as u16 * hp_bar_w) / self.max_health.max(1) as u16)
            .max(1)
            .min(hp_bar_w);
        let progress_goal = self.next_level_kills.min(TARGET_KILLS);
        let progress_start = progress_goal.saturating_sub(KILLS_PER_LEVEL);
        let progress_now = self.kills.saturating_sub(progress_start);
        let progress_cap = progress_goal.saturating_sub(progress_start).max(1);
        let xp_fill = ((progress_now as u32 * hp_bar_w as u32) / progress_cap as u32) as u16;

        let mut top_text: String<24> = String::new();
        let _ = write!(
            &mut top_text,
            "LV{}  {}/{}",
            self.level, self.kills, TARGET_KILLS
        );
        let mut best_text: String<16> = String::new();
        let _ = write!(&mut best_text, "B{}", self.best_kills);
        display.text(PANEL_X + 8, PANEL_Y + 6, &top_text, ui.text, shell, 1);
        display.text(
            PANEL_X + 74,
            PANEL_Y + 6,
            &best_text,
            ui.text_muted,
            shell,
            1,
        );
        display.text(
            PANEL_X + 8,
            PANEL_Y + 18,
            if zh_mode { "血量" } else { "HP" },
            ui.text_muted,
            shell,
            1,
        );
        display.fill_rect(PANEL_X + 24, PANEL_Y + 18, hp_bar_w, 4, ui.shadow);
        display.fill_rect(
            PANEL_X + 24,
            PANEL_Y + 18,
            hp_fill,
            4,
            if self.health <= 2 { ui.rose } else { ui.lime },
        );
        display.text(
            PANEL_X + 8,
            PANEL_Y + 28,
            if zh_mode { "經驗" } else { "XP" },
            ui.text_muted,
            shell,
            1,
        );
        display.fill_rect(PANEL_X + 24, PANEL_Y + 28, hp_bar_w, 4, ui.shadow);
        display.fill_rect(PANEL_X + 24, PANEL_Y + 28, xp_fill.max(1), 4, ui.amber);
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
