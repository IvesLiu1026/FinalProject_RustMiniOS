use crate::board::ButtonSnapshot;
use crate::companion::{self, CompanionError, CompanionState};
use crate::display::{color, palette, Display, ThemeMode};
use crate::media;
use crate::touch::TouchState;
use crate::ui::{
    draw_footer_hint, draw_gradient_background, draw_info_strip, draw_shell_window, draw_title_bar,
    render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const SOURCE_CHIP_X: u16 = 72;
const SOURCE_CHIP_Y: u16 = 40;
const SOURCE_CHIP_W: u16 = 100;

const PREVIEW_PANEL_X: u16 = 18;
const PREVIEW_PANEL_Y: u16 = 58;
const PREVIEW_PANEL_W: u16 = 182;
const PREVIEW_PANEL_H: u16 = 136;

const PREVIEW_BOX_X: u16 = 27;
const PREVIEW_BOX_Y: u16 = 66;
const PREVIEW_BOX_W: u16 = 164;
const PREVIEW_BOX_H: u16 = 120;

const INFO_PANEL_X: u16 = 208;
const INFO_PANEL_Y: u16 = 58;
const INFO_PANEL_W: u16 = 94;
const INFO_PANEL_H: u16 = 136;

const META_LABEL_X: u16 = 214;
const META_LABEL_Y: u16 = 78;
const META_LABEL_W: u16 = 82;
const META_LABEL_H: u16 = 22;
const META_COUNT_Y: u16 = 106;
const META_KIND_Y: u16 = 124;
const META_STATUS_Y: u16 = 142;

const PREV_BUTTON_X: u16 = 218;
const PREV_BUTTON_Y: u16 = 162;
const PREV_BUTTON_W: u16 = 74;
const PREV_BUTTON_H: u16 = 14;

const NEXT_BUTTON_X: u16 = 218;
const NEXT_BUTTON_Y: u16 = 180;
const NEXT_BUTTON_W: u16 = 74;
const NEXT_BUTTON_H: u16 = 14;

const TAB_STILLS_X: u16 = 190;
const TAB_MOTION_X: u16 = 248;
const TAB_Y: u16 = 40;
const TAB_W: u16 = 52;
const TAB_H: u16 = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlbumTab {
    Stills,
    Motion,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AlbumSource {
    Companion,
    Embedded,
    Waiting,
}

#[derive(Clone, Copy)]
pub struct AlbumState {
    pub motion_tab: bool,
    pub still_index: u16,
    pub motion_index: u16,
    pub playing: bool,
}

pub enum AlbumAction {
    Stay,
    ExitHome,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlbumRedraw {
    Full,
    MotionFrame,
}

pub struct AlbumApp {
    tab: AlbumTab,
    source: AlbumSource,
    still_index: usize,
    motion_index: usize,
    motion_frame: usize,
    motion_timer_ms: u16,
    playing: bool,
    redraw_pending: Option<AlbumRedraw>,
}

impl AlbumApp {
    pub const fn new() -> Self {
        Self {
            tab: AlbumTab::Stills,
            source: AlbumSource::Waiting,
            still_index: 0,
            motion_index: 0,
            motion_frame: 0,
            motion_timer_ms: 0,
            playing: true,
            redraw_pending: None,
        }
    }

    pub fn update(
        &mut self,
        input: &ButtonSnapshot,
        touch: &TouchState,
        dt_ms: u32,
    ) -> AlbumAction {
        self.refresh_source();
        self.normalize_indices();

        if input.home_chord()
            || touch_released_in_rect(touch, NAV_BACK_X, NAV_BACK_Y, NAV_BACK_W, NAV_BACK_H)
        {
            return AlbumAction::ExitHome;
        }

        if input.k1_just_pressed {
            self.toggle_tab();
        }

        if input.k0_just_pressed {
            self.previous_item();
        }

        if input.wkup_just_pressed {
            self.next_item();
        }

        if touch_released_in_rect(touch, TAB_STILLS_X, TAB_Y, TAB_W, TAB_H) {
            self.set_tab(AlbumTab::Stills);
        } else if touch_released_in_rect(touch, TAB_MOTION_X, TAB_Y, TAB_W, TAB_H) {
            self.set_tab(AlbumTab::Motion);
        } else if touch_released_in_rect(
            touch,
            PREV_BUTTON_X,
            PREV_BUTTON_Y,
            PREV_BUTTON_W,
            PREV_BUTTON_H,
        ) {
            self.previous_item();
        } else if touch_released_in_rect(
            touch,
            NEXT_BUTTON_X,
            NEXT_BUTTON_Y,
            NEXT_BUTTON_W,
            NEXT_BUTTON_H,
        ) {
            self.next_item();
        } else if touch_released_in_rect(
            touch,
            PREVIEW_PANEL_X,
            PREVIEW_PANEL_Y,
            PREVIEW_PANEL_W,
            PREVIEW_PANEL_H,
        ) {
            if matches!(self.tab, AlbumTab::Motion) && self.motion_count() > 0 {
                self.playing = !self.playing;
                self.request_full_redraw();
            }
        }

        if matches!(self.tab, AlbumTab::Motion) {
            if let Some(frame_delay_ms) = self.current_motion_delay_ms() {
                let frame_count = self.current_motion_frame_count();
                if self.playing && frame_count > 1 {
                    self.motion_timer_ms = self.motion_timer_ms.saturating_add(dt_ms as u16);
                    while self.motion_timer_ms >= frame_delay_ms.max(1) {
                        self.motion_timer_ms =
                            self.motion_timer_ms.saturating_sub(frame_delay_ms.max(1));
                        self.motion_frame = (self.motion_frame + 1) % frame_count;
                        self.request_motion_frame_redraw();
                    }
                }
            }
        }

        self.ensure_current_media_loaded();

        AlbumAction::Stay
    }

    pub fn take_redraw_request(&mut self) -> Option<AlbumRedraw> {
        let redraw = self.redraw_pending;
        self.redraw_pending = None;
        redraw
    }

    pub fn snapshot(&self) -> AlbumState {
        AlbumState {
            motion_tab: matches!(self.tab, AlbumTab::Motion),
            still_index: self.still_index.min(u16::MAX as usize) as u16,
            motion_index: self.motion_index.min(u16::MAX as usize) as u16,
            playing: self.playing,
        }
    }

    pub fn restore(&mut self, state: AlbumState) {
        self.tab = if state.motion_tab {
            AlbumTab::Motion
        } else {
            AlbumTab::Stills
        };
        self.still_index = state.still_index as usize;
        self.motion_index = state.motion_index as usize;
        self.motion_frame = 0;
        self.motion_timer_ms = 0;
        self.playing = state.playing;
        self.redraw_pending = Some(AlbumRedraw::Full);
    }

    pub fn render(&self, display: &mut Display, theme: ThemeMode, zh_mode: bool) {
        let ui = palette(theme);
        draw_gradient_background(display, theme, 18);

        draw_shell_window(display, ui.cyan, &ui);
        draw_title_bar(
            display,
            if zh_mode { "相簿" } else { "ALBUM" },
            if zh_mode {
                "stills / motion / built-in media"
            } else {
                "stills / motion / built-in media"
            },
            ui.cyan,
            &ui,
        );
        render_nav_back(display, zh_mode, ui.orange, &ui);
        self.render_source_chip(display, zh_mode, &ui);
        self.render_preview_frame(display, &ui);
        self.render_info_panel(display, zh_mode, &ui);

        self.render_tab_chip(
            display,
            TAB_STILLS_X,
            if matches!(self.tab, AlbumTab::Stills) {
                ui.cyan
            } else {
                ui.steel
            },
            if zh_mode { "圖片" } else { "STILL" },
            &ui,
        );
        self.render_tab_chip(
            display,
            TAB_MOTION_X,
            if matches!(self.tab, AlbumTab::Motion) {
                ui.rose
            } else {
                ui.steel
            },
            if zh_mode { "動畫" } else { "MOTION" },
            &ui,
        );

        match self.tab {
            AlbumTab::Stills => {
                self.render_stills(display, zh_mode, &ui);
            }
            AlbumTab::Motion => {
                self.render_motion(display, zh_mode, &ui);
            }
        }

        display.panel(
            PREV_BUTTON_X,
            PREV_BUTTON_Y,
            PREV_BUTTON_W,
            PREV_BUTTON_H,
            ui.panel_alt,
            ui.orange,
        );
        display.centered_text(
            PREV_BUTTON_X + PREV_BUTTON_W / 2,
            PREV_BUTTON_Y + 4,
            if zh_mode { "上一個" } else { "PREV" },
            ui.text,
            ui.panel_alt,
            1,
        );
        display.panel(
            NEXT_BUTTON_X,
            NEXT_BUTTON_Y,
            NEXT_BUTTON_W,
            NEXT_BUTTON_H,
            ui.panel_alt,
            ui.cyan,
        );
        display.centered_text(
            NEXT_BUTTON_X + NEXT_BUTTON_W / 2,
            NEXT_BUTTON_Y + 4,
            if zh_mode { "下一個" } else { "NEXT" },
            ui.text,
            ui.panel_alt,
            1,
        );

        if matches!(self.source, AlbumSource::Companion) {
            let mut footer = heapless::String::<48>::new();
            let _ = core::fmt::write(
                &mut footer,
                format_args!("MAC LINK / USART3 @ {}", companion::baud_rate()),
            );
            draw_footer_hint(display, &footer, ui.cyan, &ui);
        } else {
            draw_footer_hint(
                display,
                if zh_mode {
                    "K0/WK 切換  K1 切頁  點預覽可暫停動畫"
                } else {
                    "K0/WK SWITCH  K1 TAB  TAP PREVIEW TO PAUSE"
                },
                ui.amber,
                &ui,
            );
        }
    }

    pub fn render_motion_frame(&self, display: &mut Display) {
        if !matches!(self.tab, AlbumTab::Motion) {
            return;
        }
        match self.source {
            AlbumSource::Companion => {
                if let Some(clip) = companion::link().motion_clip(self.motion_index) {
                    if let Some((draw_x, draw_y, scale)) =
                        media_layout(clip.width, clip.height, clip.scale)
                    {
                        companion::link().draw_cached_frame_scaled(display, draw_x, draw_y, scale);
                    }
                }
            }
            AlbumSource::Embedded => {
                if let Some(clip) = self.current_embedded_clip() {
                    self.draw_embedded_clip(display, clip);
                }
            }
            AlbumSource::Waiting => {}
        }
    }

    fn render_tab_chip(
        &self,
        display: &mut Display,
        x: u16,
        accent: u16,
        label: &str,
        ui: &crate::display::Palette,
    ) {
        display.panel(x, TAB_Y, TAB_W, TAB_H, ui.panel_alt, accent);
        display.centered_text(x + TAB_W / 2, TAB_Y + 4, label, ui.text, ui.panel_alt, 1);
    }

    fn render_preview_frame(&self, display: &mut Display, ui: &crate::display::Palette) {
        display.panel(
            PREVIEW_PANEL_X,
            PREVIEW_PANEL_Y,
            PREVIEW_PANEL_W,
            PREVIEW_PANEL_H,
            ui.panel,
            ui.white,
        );
        display.fill_rect(
            PREVIEW_PANEL_X + 10,
            PREVIEW_PANEL_Y + 8,
            54,
            14,
            color::mix(ui.panel_alt, ui.cyan, 26),
        );
        display.stroke_rect(
            PREVIEW_PANEL_X + 10,
            PREVIEW_PANEL_Y + 8,
            54,
            14,
            1,
            ui.cyan,
        );
        display.text(
            PREVIEW_PANEL_X + 18,
            PREVIEW_PANEL_Y + 12,
            if matches!(self.tab, AlbumTab::Motion) {
                "MOTION"
            } else {
                "PHOTO"
            },
            ui.text,
            color::mix(ui.panel_alt, ui.cyan, 26),
            1,
        );
        display.fill_rect(
            PREVIEW_BOX_X,
            PREVIEW_BOX_Y,
            PREVIEW_BOX_W,
            PREVIEW_BOX_H,
            color::mix(ui.panel_alt, ui.canvas, 18),
        );
        display.stroke_rect(
            PREVIEW_BOX_X,
            PREVIEW_BOX_Y,
            PREVIEW_BOX_W,
            PREVIEW_BOX_H,
            1,
            ui.steel,
        );
        display.fill_rect(
            PREVIEW_BOX_X + 2,
            PREVIEW_BOX_Y + 2,
            PREVIEW_BOX_W - 4,
            4,
            color::mix(ui.white, ui.cyan, 12),
        );
        draw_album_preview_decor(
            display,
            PREVIEW_PANEL_X + PREVIEW_PANEL_W - 38,
            PREVIEW_PANEL_Y + 8,
            self.tab,
            ui,
        );
        if matches!(self.tab, AlbumTab::Motion) {
            draw_film_ticks(
                display,
                PREVIEW_BOX_X + 4,
                PREVIEW_BOX_Y + PREVIEW_BOX_H - 10,
                PREVIEW_BOX_W - 8,
                ui,
            );
        } else {
            draw_polaroid_corner(
                display,
                PREVIEW_BOX_X + PREVIEW_BOX_W - 24,
                PREVIEW_BOX_Y + PREVIEW_BOX_H - 18,
                ui,
            );
        }
    }

    fn render_info_panel(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.panel(
            INFO_PANEL_X,
            INFO_PANEL_Y,
            INFO_PANEL_W,
            INFO_PANEL_H,
            ui.panel,
            ui.steel,
        );
        display.text(
            INFO_PANEL_X + 10,
            INFO_PANEL_Y + 8,
            if zh_mode {
                "媒體資訊"
            } else {
                "MEDIA INFO"
            },
            ui.text,
            ui.panel,
            1,
        );
        draw_info_panel_icon(display, INFO_PANEL_X + 56, INFO_PANEL_Y + 8, self.tab, ui);
        let source_label = match self.source {
            AlbumSource::Companion => "MAC",
            AlbumSource::Embedded => "ROM",
            AlbumSource::Waiting => "WAIT",
        };
        let source_accent = match self.source {
            AlbumSource::Companion => ui.cyan,
            AlbumSource::Embedded => ui.amber,
            AlbumSource::Waiting => ui.rose,
        };
        display.fill_rect(INFO_PANEL_X + 12, INFO_PANEL_Y + 28, 8, 8, source_accent);
        display.stroke_rect(INFO_PANEL_X + 12, INFO_PANEL_Y + 28, 8, 8, 1, ui.white);
        display.text(
            INFO_PANEL_X + 26,
            INFO_PANEL_Y + 29,
            if zh_mode { "來源" } else { "SRC" },
            ui.text_muted,
            ui.panel,
            1,
        );
        display.text(
            INFO_PANEL_X + 62,
            INFO_PANEL_Y + 29,
            source_label,
            source_accent,
            ui.panel,
            1,
        );
        display.text(
            INFO_PANEL_X + 12,
            INFO_PANEL_Y + INFO_PANEL_H - 24,
            if matches!(self.tab, AlbumTab::Motion) {
                if zh_mode {
                    "點預覽暫停"
                } else {
                    "TAP TO PAUSE"
                }
            } else if zh_mode {
                "像素相紙檢視"
            } else {
                "PIXEL PHOTO VIEW"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
    }

    fn render_caption(
        &self,
        display: &mut Display,
        label: &str,
        index: usize,
        total: usize,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.panel(
            META_LABEL_X,
            META_LABEL_Y,
            META_LABEL_W,
            META_LABEL_H,
            ui.panel_alt,
            ui.cyan,
        );
        display.centered_text(
            META_LABEL_X + META_LABEL_W / 2,
            META_LABEL_Y + 7,
            label,
            ui.text,
            ui.panel_alt,
            1,
        );

        let mut count_text = heapless::String::<24>::new();
        let _ = core::fmt::write(&mut count_text, format_args!("{}/{}", index, total.max(1)));
        draw_info_strip(
            display,
            META_LABEL_X,
            META_COUNT_Y,
            META_LABEL_W,
            if zh_mode { "項目" } else { "ITEM" },
            &count_text,
            ui.white,
            ui,
        );
        draw_info_strip(
            display,
            META_LABEL_X,
            META_KIND_Y,
            META_LABEL_W,
            if zh_mode { "類型" } else { "TYPE" },
            if matches!(self.tab, AlbumTab::Motion) {
                if zh_mode {
                    "動畫"
                } else {
                    "MOTION"
                }
            } else if zh_mode {
                "圖片"
            } else {
                "STILL"
            },
            ui.cyan,
            ui,
        );
    }

    fn render_empty(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.panel(
            PREVIEW_BOX_X + 12,
            PREVIEW_BOX_Y + 28,
            PREVIEW_BOX_W - 24,
            60,
            ui.panel_alt,
            ui.rose,
        );
        display.centered_text(
            PREVIEW_BOX_X + PREVIEW_BOX_W / 2,
            PREVIEW_BOX_Y + 44,
            self.empty_title(zh_mode),
            ui.text,
            ui.panel_alt,
            1,
        );
        display.centered_text(
            PREVIEW_BOX_X + PREVIEW_BOX_W / 2,
            PREVIEW_BOX_Y + 68,
            self.empty_subtitle(zh_mode),
            ui.text_muted,
            ui.panel_alt,
            1,
        );
    }

    fn set_tab(&mut self, tab: AlbumTab) {
        if self.tab == tab {
            return;
        }
        self.tab = tab;
        self.motion_frame = 0;
        self.motion_timer_ms = 0;
        self.request_full_redraw();
    }

    fn toggle_tab(&mut self) {
        self.set_tab(match self.tab {
            AlbumTab::Stills => AlbumTab::Motion,
            AlbumTab::Motion => AlbumTab::Stills,
        });
    }

    fn previous_item(&mut self) {
        match self.tab {
            AlbumTab::Stills => {
                let total = self.still_count();
                if total > 0 {
                    self.still_index = (self.still_index + total - 1) % total;
                    self.request_full_redraw();
                }
            }
            AlbumTab::Motion => {
                let total = self.motion_count();
                if total > 0 {
                    self.motion_index = (self.motion_index + total - 1) % total;
                    self.motion_frame = 0;
                    self.motion_timer_ms = 0;
                    self.request_full_redraw();
                }
            }
        }
    }

    fn next_item(&mut self) {
        match self.tab {
            AlbumTab::Stills => {
                let total = self.still_count();
                if total > 0 {
                    self.still_index = (self.still_index + 1) % total;
                    self.request_full_redraw();
                }
            }
            AlbumTab::Motion => {
                let total = self.motion_count();
                if total > 0 {
                    self.motion_index = (self.motion_index + 1) % total;
                    self.motion_frame = 0;
                    self.motion_timer_ms = 0;
                    self.request_full_redraw();
                }
            }
        }
    }

    fn request_full_redraw(&mut self) {
        self.redraw_pending = Some(AlbumRedraw::Full);
    }

    fn request_motion_frame_redraw(&mut self) {
        if self.redraw_pending != Some(AlbumRedraw::Full) {
            self.redraw_pending = Some(AlbumRedraw::MotionFrame);
        }
    }

    fn refresh_source(&mut self) {
        let next_source = if !media::stills().is_empty() || !media::motion_clips().is_empty() {
            AlbumSource::Embedded
        } else {
            let companion = companion::link();
            companion.tick();
            if matches!(companion.state(), CompanionState::Ready)
                && (companion.still_count() > 0 || companion.motion_count() > 0)
            {
                AlbumSource::Companion
            } else {
                AlbumSource::Waiting
            }
        };

        if self.source != next_source {
            self.source = next_source;
            self.motion_timer_ms = 0;
            self.motion_frame = 0;
            self.request_full_redraw();
        }
    }

    fn normalize_indices(&mut self) {
        let still_count = self.still_count();
        self.still_index = if still_count == 0 {
            0
        } else {
            self.still_index % still_count
        };

        let motion_count = self.motion_count();
        self.motion_index = if motion_count == 0 {
            0
        } else {
            self.motion_index % motion_count
        };

        let frame_count = self.current_motion_frame_count();
        self.motion_frame = if frame_count == 0 {
            0
        } else {
            self.motion_frame % frame_count
        };
    }

    fn ensure_current_media_loaded(&mut self) {
        if !matches!(self.source, AlbumSource::Companion) {
            return;
        }

        let companion = companion::link();
        if !matches!(companion.state(), CompanionState::Ready) {
            return;
        }

        match self.tab {
            AlbumTab::Stills => {
                if self.still_count() == 0 {
                    return;
                }
                if companion.cached_still_index() != Some(self.still_index)
                    && companion.fetch_still(self.still_index)
                {
                    self.request_full_redraw();
                }
            }
            AlbumTab::Motion => {
                if self.motion_count() == 0 {
                    return;
                }
                if companion.cached_motion_frame() != Some((self.motion_index, self.motion_frame))
                    && companion.fetch_motion_frame(self.motion_index, self.motion_frame)
                {
                    if self.redraw_pending == Some(AlbumRedraw::MotionFrame) {
                        self.request_motion_frame_redraw();
                    } else {
                        self.request_full_redraw();
                    }
                }
            }
        }
    }

    fn render_stills(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        match self.source {
            AlbumSource::Companion => {
                let companion = companion::link();
                if let Some(still) = companion.still(self.still_index) {
                    if let Some((draw_x, draw_y, scale)) =
                        media_layout(still.width, still.height, still.scale)
                    {
                        companion.draw_cached_frame_scaled(display, draw_x, draw_y, scale);
                    }
                    self.render_caption(
                        display,
                        still.label.as_str(),
                        self.still_index + 1,
                        companion.still_count(),
                        zh_mode,
                        ui,
                    );
                    self.render_still_status(display, zh_mode, ui);
                } else {
                    self.render_empty(display, zh_mode, ui);
                }
            }
            AlbumSource::Embedded => {
                if let Some(still) = self.current_embedded_still() {
                    self.draw_embedded_still(display, still);
                    self.render_caption(
                        display,
                        still.label,
                        self.still_index + 1,
                        media::stills().len(),
                        zh_mode,
                        ui,
                    );
                    self.render_still_status(display, zh_mode, ui);
                } else {
                    self.render_empty(display, zh_mode, ui);
                }
            }
            AlbumSource::Waiting => self.render_empty(display, zh_mode, ui),
        }
    }

    fn render_motion(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        match self.source {
            AlbumSource::Companion => {
                let companion = companion::link();
                if let Some(clip) = companion.motion_clip(self.motion_index) {
                    if let Some((draw_x, draw_y, scale)) =
                        media_layout(clip.width, clip.height, clip.scale)
                    {
                        companion.draw_cached_frame_scaled(display, draw_x, draw_y, scale);
                    }
                    self.render_caption(
                        display,
                        clip.label.as_str(),
                        self.motion_index + 1,
                        companion.motion_count(),
                        zh_mode,
                        ui,
                    );
                    self.render_motion_status(display, zh_mode, ui);
                } else {
                    self.render_empty(display, zh_mode, ui);
                }
            }
            AlbumSource::Embedded => {
                if let Some(clip) = self.current_embedded_clip() {
                    self.draw_embedded_clip(display, clip);
                    self.render_caption(
                        display,
                        clip.label,
                        self.motion_index + 1,
                        media::motion_clips().len(),
                        zh_mode,
                        ui,
                    );
                    self.render_motion_status(display, zh_mode, ui);
                } else {
                    self.render_empty(display, zh_mode, ui);
                }
            }
            AlbumSource::Waiting => self.render_empty(display, zh_mode, ui),
        }
    }

    fn render_motion_status(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        draw_info_strip(
            display,
            META_LABEL_X,
            META_STATUS_Y,
            META_LABEL_W,
            if zh_mode { "狀態" } else { "STATE" },
            if self.playing {
                if zh_mode {
                    "播放中"
                } else {
                    "PLAYING"
                }
            } else if zh_mode {
                "已暫停"
            } else {
                "PAUSED"
            },
            ui.rose,
            ui,
        );
    }

    fn render_still_status(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        draw_info_strip(
            display,
            META_LABEL_X,
            META_STATUS_Y,
            META_LABEL_W,
            if zh_mode { "狀態" } else { "STATE" },
            if zh_mode { "檢視中" } else { "VIEWING" },
            ui.lime,
            ui,
        );
    }

    fn render_source_chip(
        &self,
        display: &mut Display,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        let (accent, label) = match self.source {
            AlbumSource::Companion => (ui.cyan, if zh_mode { "MAC 連線" } else { "MAC LINK" }),
            AlbumSource::Embedded => (ui.orange, if zh_mode { "內建媒體" } else { "EMBEDDED" }),
            AlbumSource::Waiting => (ui.rose, if zh_mode { "等待連線" } else { "WAIT LINK" }),
        };
        display.panel(
            SOURCE_CHIP_X,
            SOURCE_CHIP_Y,
            SOURCE_CHIP_W,
            12,
            ui.panel_alt,
            accent,
        );
        display.centered_text(
            SOURCE_CHIP_X + SOURCE_CHIP_W / 2,
            SOURCE_CHIP_Y + 3,
            label,
            ui.text,
            ui.panel_alt,
            1,
        );
    }

    fn draw_embedded_still(&self, display: &mut Display, still: &'static media::EmbeddedStill) {
        if let Some((draw_x, draw_y, scale)) = media_layout(still.width, still.height, still.scale)
        {
            display.draw_rgb565_scaled_bytes(
                draw_x,
                draw_y,
                still.width,
                still.height,
                scale,
                still.data,
            );
        }
    }

    fn draw_embedded_clip(&self, display: &mut Display, clip: &'static media::EmbeddedMotionClip) {
        if let Some((draw_x, draw_y, scale)) = media_layout(clip.width, clip.height, clip.scale) {
            display.draw_rgb565_scaled_bytes(
                draw_x,
                draw_y,
                clip.width,
                clip.height,
                scale,
                clip.frame(self.motion_frame),
            );
        }
    }

    fn empty_title(&self, zh_mode: bool) -> &'static str {
        if matches!(self.source, AlbumSource::Waiting) {
            if zh_mode {
                "請連接 Mac Companion"
            } else {
                "CONNECT MAC COMPANION"
            }
        } else if zh_mode {
            "目前沒有媒體"
        } else {
            "NO MEDIA READY"
        }
    }

    fn empty_subtitle(&self, zh_mode: bool) -> &'static str {
        if matches!(self.source, AlbumSource::Waiting) {
            match companion::link().last_error() {
                Some(CompanionError::Protocol) => {
                    if zh_mode {
                        "協定不相容，請重啟 companion"
                    } else {
                        "PROTOCOL MISMATCH - RESTART COMPANION"
                    }
                }
                Some(CompanionError::FrameTooLarge) => {
                    if zh_mode {
                        "影格太大，請重轉素材"
                    } else {
                        "FRAME TOO LARGE - REPROCESS MEDIA"
                    }
                }
                _ => {
                    if zh_mode {
                        "USART3 PC10/PC11，Mac 端執行 companion"
                    } else {
                        "USART3 PC10/PC11 - RUN THE MAC COMPANION"
                    }
                }
            }
        } else if zh_mode {
            "先執行媒體轉檔腳本"
        } else {
            "RUN THE MEDIA PREPROCESSOR FIRST"
        }
    }

    fn still_count(&self) -> usize {
        match self.source {
            AlbumSource::Companion => companion::link().still_count(),
            AlbumSource::Embedded => media::stills().len(),
            AlbumSource::Waiting => 0,
        }
    }

    fn motion_count(&self) -> usize {
        match self.source {
            AlbumSource::Companion => companion::link().motion_count(),
            AlbumSource::Embedded => media::motion_clips().len(),
            AlbumSource::Waiting => 0,
        }
    }

    fn current_motion_frame_count(&self) -> usize {
        match self.source {
            AlbumSource::Companion => companion::link()
                .motion_clip(self.motion_index)
                .map(|clip| clip.frame_count as usize)
                .unwrap_or(0),
            AlbumSource::Embedded => self
                .current_embedded_clip()
                .map(|clip| clip.frame_count())
                .unwrap_or(0),
            AlbumSource::Waiting => 0,
        }
    }

    fn current_motion_delay_ms(&self) -> Option<u16> {
        match self.source {
            AlbumSource::Companion => companion::link()
                .motion_clip(self.motion_index)
                .map(|clip| clip.frame_delay_ms),
            AlbumSource::Embedded => self.current_embedded_clip().map(|clip| clip.frame_delay_ms),
            AlbumSource::Waiting => None,
        }
    }

    fn current_embedded_still(&self) -> Option<&'static media::EmbeddedStill> {
        media::stills().get(self.still_index)
    }

    fn current_embedded_clip(&self) -> Option<&'static media::EmbeddedMotionClip> {
        media::motion_clips().get(self.motion_index)
    }
}

fn media_layout(width: u16, height: u16, max_scale: u16) -> Option<(u16, u16, u16)> {
    if width == 0 || height == 0 || max_scale == 0 {
        return None;
    }

    let scale_x = PREVIEW_BOX_W / width;
    let scale_y = PREVIEW_BOX_H / height;
    let scale = scale_x.min(scale_y).min(max_scale);
    if scale == 0 {
        return None;
    }

    let render_w = width.saturating_mul(scale);
    let render_h = height.saturating_mul(scale);
    let draw_x = PREVIEW_BOX_X + (PREVIEW_BOX_W.saturating_sub(render_w)) / 2;
    let draw_y = PREVIEW_BOX_Y + (PREVIEW_BOX_H.saturating_sub(render_h)) / 2;
    Some((draw_x, draw_y, scale))
}

fn draw_album_preview_decor(
    display: &mut Display,
    x: u16,
    y: u16,
    tab: AlbumTab,
    ui: &crate::display::Palette,
) {
    let fill = if matches!(tab, AlbumTab::Motion) {
        color::mix(ui.panel_alt, ui.rose, 28)
    } else {
        color::mix(ui.panel_alt, ui.amber, 28)
    };
    let accent = if matches!(tab, AlbumTab::Motion) {
        ui.rose
    } else {
        ui.amber
    };
    display.fill_rect(x, y, 24, 14, fill);
    display.stroke_rect(x, y, 24, 14, 1, accent);
    if matches!(tab, AlbumTab::Motion) {
        display.fill_rect(x + 4, y + 3, 14, 8, ui.text);
        display.fill_rect(x + 8, y + 5, 4, 4, fill);
        display.fill_rect(x + 2, y + 4, 1, 2, ui.white);
        display.fill_rect(x + 2, y + 8, 1, 2, ui.white);
        display.fill_rect(x + 20, y + 4, 1, 2, ui.white);
        display.fill_rect(x + 20, y + 8, 1, 2, ui.white);
    } else {
        display.fill_rect(x + 4, y + 2, 16, 10, ui.white);
        display.stroke_rect(x + 4, y + 2, 16, 10, 1, accent);
        display.fill_rect(x + 6, y + 4, 10, 5, color::mix(ui.cyan, ui.white, 80));
        display.fill_rect(x + 8, y + 5, 2, 2, ui.amber);
        display.fill_rect(x + 13, y + 5, 2, 2, ui.lime);
    }
}

fn draw_film_ticks(
    display: &mut Display,
    x: u16,
    y: u16,
    width: u16,
    ui: &crate::display::Palette,
) {
    let mut tick_x = x;
    while tick_x + 4 < x + width {
        display.fill_rect(tick_x, y, 3, 3, ui.white);
        tick_x += 8;
    }
}

fn draw_polaroid_corner(display: &mut Display, x: u16, y: u16, ui: &crate::display::Palette) {
    display.fill_rect(x, y, 14, 10, ui.white);
    display.stroke_rect(x, y, 14, 10, 1, ui.steel);
    display.fill_rect(x + 2, y + 2, 9, 4, color::mix(ui.cyan, ui.white, 76));
    display.fill_rect(x + 4, y + 7, 6, 1, ui.rose);
}

fn draw_info_panel_icon(
    display: &mut Display,
    x: u16,
    y: u16,
    tab: AlbumTab,
    ui: &crate::display::Palette,
) {
    let fill = if matches!(tab, AlbumTab::Motion) {
        color::mix(ui.panel_alt, ui.rose, 24)
    } else {
        color::mix(ui.panel_alt, ui.amber, 24)
    };
    let accent = if matches!(tab, AlbumTab::Motion) {
        ui.rose
    } else {
        ui.amber
    };
    display.fill_rect(x, y, 26, 16, fill);
    display.stroke_rect(x, y, 26, 16, 1, accent);
    if matches!(tab, AlbumTab::Motion) {
        display.fill_rect(x + 5, y + 4, 12, 8, ui.text);
        display.fill_rect(x + 9, y + 6, 4, 4, fill);
        display.fill_rect(x + 3, y + 5, 1, 2, ui.white);
        display.fill_rect(x + 3, y + 9, 1, 2, ui.white);
        display.fill_rect(x + 18, y + 5, 1, 2, ui.white);
        display.fill_rect(x + 18, y + 9, 1, 2, ui.white);
    } else {
        display.fill_rect(x + 4, y + 3, 15, 10, ui.white);
        display.stroke_rect(x + 4, y + 3, 15, 10, 1, accent);
        display.fill_rect(x + 6, y + 5, 10, 5, color::mix(ui.cyan, ui.white, 80));
        display.fill_rect(x + 8, y + 6, 2, 2, ui.amber);
        display.fill_rect(x + 12, y + 6, 2, 2, ui.lime);
    }
}
