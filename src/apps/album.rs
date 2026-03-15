use crate::board::ButtonSnapshot;
use crate::companion::{self, CompanionError, CompanionState};
use crate::display::{palette, Display, ThemeMode};
use crate::media;
use crate::touch::TouchState;
use crate::ui::{
    draw_gradient_background, render_nav_back, NAV_BACK_H, NAV_BACK_W, NAV_BACK_X, NAV_BACK_Y,
};

use super::touch_released_in_rect;

const MEDIA_PANEL_X: u16 = 20;
const MEDIA_PANEL_Y: u16 = 44;
const MEDIA_PANEL_W: u16 = 280;
const MEDIA_PANEL_H: u16 = 186;

const TAB_STILLS_X: u16 = 182;
const TAB_MOTION_X: u16 = 242;
const TAB_Y: u16 = 14;
const TAB_W: u16 = 52;
const TAB_H: u16 = 16;

const MEDIA_RENDER_X: u16 = 40;
const MEDIA_RENDER_Y: u16 = 46;

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
        } else if touch_released_in_rect(touch, 18, 214, 58, 12) {
            self.previous_item();
        } else if touch_released_in_rect(touch, 244, 214, 58, 12) {
            self.next_item();
        } else if touch_released_in_rect(
            touch,
            MEDIA_PANEL_X,
            MEDIA_PANEL_Y,
            MEDIA_PANEL_W,
            MEDIA_PANEL_H,
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

        display.panel(14, 10, 292, 28, ui.panel, ui.cyan);
        render_nav_back(display, zh_mode, ui.orange, &ui);
        display.text(
            74,
            18,
            if zh_mode { "相簿" } else { "ALBUM" },
            ui.text,
            ui.panel,
            2,
        );
        display.text(
            86,
            20,
            if zh_mode {
                "靜態圖與動態片段"
            } else {
                "STILLS + MOTION CLIPS"
            },
            ui.text_muted,
            ui.panel,
            1,
        );
        self.render_source_chip(display, zh_mode, &ui);

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

        display.panel(
            MEDIA_PANEL_X,
            MEDIA_PANEL_Y,
            MEDIA_PANEL_W,
            MEDIA_PANEL_H,
            ui.panel,
            ui.white,
        );

        match self.tab {
            AlbumTab::Stills => {
                self.render_stills(display, zh_mode, &ui);
            }
            AlbumTab::Motion => {
                self.render_motion(display, zh_mode, &ui);
            }
        }

        display.panel(20, 214, 58, 12, ui.panel_alt, ui.orange);
        display.centered_text(
            49,
            217,
            if zh_mode { "上一個" } else { "PREV" },
            ui.text,
            ui.panel_alt,
            1,
        );
        display.panel(242, 214, 58, 12, ui.panel_alt, ui.cyan);
        display.centered_text(
            271,
            217,
            if zh_mode { "下一個" } else { "NEXT" },
            ui.text,
            ui.panel_alt,
            1,
        );

        display.panel(18, 230, 284, 10, ui.panel, ui.amber);
        if matches!(self.source, AlbumSource::Companion) {
            let mut footer = heapless::String::<48>::new();
            let _ = core::fmt::write(
                &mut footer,
                format_args!("MAC COMPANION / USART3 @ {}", companion::baud_rate()),
            );
            display.text(24, 232, &footer, ui.text_muted, ui.panel, 1);
        } else {
            display.text(
                24,
                232,
                if zh_mode {
                    "K0/WK 切換  K1 切頁  點畫面可暫停動畫"
                } else {
                    "K0/WK SWITCH  K1 TAB  TAP MEDIA TO PAUSE MOTION"
                },
                ui.text_muted,
                ui.panel,
                1,
            );
        }
    }

    pub fn render_motion_frame(&self, display: &mut Display) {
        if !matches!(self.tab, AlbumTab::Motion) {
            return;
        }
        match self.source {
            AlbumSource::Companion => {
                companion::link().draw_cached_frame(display, MEDIA_RENDER_X, MEDIA_RENDER_Y);
            }
            AlbumSource::Embedded => {
                if let Some(clip) = self.current_embedded_clip() {
                    media::draw_clip_frame_centered(
                        display,
                        clip,
                        self.motion_frame,
                        MEDIA_RENDER_X,
                        MEDIA_RENDER_Y,
                    );
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

    fn render_caption(
        &self,
        display: &mut Display,
        label: &str,
        index: usize,
        total: usize,
        zh_mode: bool,
        ui: &crate::display::Palette,
    ) {
        display.panel(30, 206, 154, 14, ui.panel_alt, ui.cyan);
        display.text(38, 210, label, ui.text, ui.panel_alt, 1);

        let mut count_text = heapless::String::<24>::new();
        let _ = core::fmt::write(
            &mut count_text,
            format_args!(
                "{} {}/{}",
                if zh_mode { "項目" } else { "ITEM" },
                index,
                total.max(1)
            ),
        );
        display.panel(196, 44, 92, 14, ui.panel_alt, ui.white);
        display.centered_text(242, 48, &count_text, ui.text, ui.panel_alt, 1);
    }

    fn render_empty(&self, display: &mut Display, zh_mode: bool, ui: &crate::display::Palette) {
        display.panel(58, 102, 204, 60, ui.panel_alt, ui.rose);
        display.centered_text(
            160,
            118,
            self.empty_title(zh_mode),
            ui.text,
            ui.panel_alt,
            2,
        );
        display.centered_text(
            160,
            142,
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
                    companion.draw_cached_frame(display, MEDIA_RENDER_X, MEDIA_RENDER_Y);
                    self.render_caption(
                        display,
                        still.label.as_str(),
                        self.still_index + 1,
                        companion.still_count(),
                        zh_mode,
                        ui,
                    );
                } else {
                    self.render_empty(display, zh_mode, ui);
                }
            }
            AlbumSource::Embedded => {
                if let Some(still) = self.current_embedded_still() {
                    media::draw_still_centered(display, still, MEDIA_RENDER_X, MEDIA_RENDER_Y);
                    self.render_caption(
                        display,
                        still.label,
                        self.still_index + 1,
                        media::stills().len(),
                        zh_mode,
                        ui,
                    );
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
                    companion.draw_cached_frame(display, MEDIA_RENDER_X, MEDIA_RENDER_Y);
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
                    media::draw_clip_frame_centered(
                        display,
                        clip,
                        self.motion_frame,
                        MEDIA_RENDER_X,
                        MEDIA_RENDER_Y,
                    );
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
        display.panel(198, 206, 92, 14, ui.panel_alt, ui.rose);
        display.centered_text(
            244,
            210,
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
            ui.text,
            ui.panel_alt,
            1,
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
        display.panel(14, 40, 102, 12, ui.panel_alt, accent);
        display.centered_text(65, 43, label, ui.text, ui.panel_alt, 1);
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
