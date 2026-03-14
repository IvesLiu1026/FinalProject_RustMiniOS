use crate::{font, font_zh};

pub const SCREEN_WIDTH: u16 = 320;
pub const SCREEN_HEIGHT: u16 = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: u16,
    pub panel: u16,
    pub panel_alt: u16,
    pub shadow: u16,
    pub text: u16,
    pub text_muted: u16,
    pub steel: u16,
    pub sky: u16,
    pub floor: u16,
    pub cyan: u16,
    pub orange: u16,
    pub rose: u16,
    pub lime: u16,
    pub amber: u16,
    pub indigo: u16,
    pub white: u16,
}

pub mod color {
    pub const fn rgb565(r: u8, g: u8, b: u8) -> u16 {
        (((r as u16) & 0xF8) << 8) | (((g as u16) & 0xFC) << 3) | ((b as u16) >> 3)
    }

    pub const WHITE: u16 = 0xFFFF;
    pub const MIDNIGHT: u16 = rgb565(10, 16, 30);
    pub const DEEP_BLUE: u16 = rgb565(18, 46, 92);
    pub const CYAN: u16 = rgb565(68, 214, 255);
    pub const ORANGE: u16 = rgb565(255, 158, 56);
    pub const ROSE: u16 = rgb565(255, 98, 129);
    pub const LIME: u16 = rgb565(154, 227, 92);
    pub const AMBER: u16 = rgb565(255, 202, 58);
    pub const INDIGO: u16 = rgb565(63, 81, 181);
    pub const PANEL: u16 = rgb565(18, 28, 51);
    pub const PANEL_ALT: u16 = rgb565(28, 42, 72);
    pub const SHADOW: u16 = rgb565(5, 9, 18);
    pub const STEEL: u16 = rgb565(124, 152, 184);
    pub const TEXT: u16 = rgb565(235, 244, 255);
    pub const TEXT_MUTED: u16 = rgb565(149, 167, 194);
    pub const FLOOR: u16 = rgb565(40, 34, 30);
    pub const PAPER: u16 = rgb565(240, 237, 229);
    pub const SAND: u16 = rgb565(220, 210, 194);
    pub const INK: u16 = rgb565(38, 42, 56);
    pub const SLATE: u16 = rgb565(101, 112, 128);
    pub const LIGHT_SKY: u16 = rgb565(148, 198, 255);
    pub const LIGHT_FLOOR: u16 = rgb565(205, 198, 187);

    pub const fn mix(a: u16, b: u16, t: u8) -> u16 {
        let t = t as u32;
        let inv = 255u32 - t;

        let ar = ((a >> 11) & 0x1F) as u32;
        let ag = ((a >> 5) & 0x3F) as u32;
        let ab = (a & 0x1F) as u32;

        let br = ((b >> 11) & 0x1F) as u32;
        let bg = ((b >> 5) & 0x3F) as u32;
        let bb = (b & 0x1F) as u32;

        let r = ((ar * inv) + (br * t)) / 255;
        let g = ((ag * inv) + (bg * t)) / 255;
        let bl = ((ab * inv) + (bb * t)) / 255;

        ((r as u16) << 11) | ((g as u16) << 5) | (bl as u16)
    }
}

pub fn palette(theme: ThemeMode) -> Palette {
    match theme {
        ThemeMode::Dark => Palette {
            canvas: color::MIDNIGHT,
            panel: color::PANEL,
            panel_alt: color::PANEL_ALT,
            shadow: color::SHADOW,
            text: color::TEXT,
            text_muted: color::TEXT_MUTED,
            steel: color::STEEL,
            sky: color::DEEP_BLUE,
            floor: color::FLOOR,
            cyan: color::CYAN,
            orange: color::ORANGE,
            rose: color::ROSE,
            lime: color::LIME,
            amber: color::AMBER,
            indigo: color::INDIGO,
            white: color::WHITE,
        },
        ThemeMode::Light => Palette {
            canvas: color::PAPER,
            panel: color::SAND,
            panel_alt: color::mix(color::PAPER, color::LIGHT_SKY, 35),
            shadow: color::mix(color::SLATE, color::SHADOW, 120),
            text: color::INK,
            text_muted: color::SLATE,
            steel: color::mix(color::SLATE, color::WHITE, 80),
            sky: color::LIGHT_SKY,
            floor: color::LIGHT_FLOOR,
            cyan: color::rgb565(31, 139, 224),
            orange: color::rgb565(219, 127, 26),
            rose: color::rgb565(201, 89, 120),
            lime: color::rgb565(88, 175, 72),
            amber: color::rgb565(215, 168, 44),
            indigo: color::rgb565(89, 102, 181),
            white: color::WHITE,
        },
    }
}

unsafe extern "C" {
    fn LCD_Init();
    fn LCD_SetTextColor(color: u16);
    fn LCD_FillRect(x: u16, y: u16, width: u16, height: u16);
    fn LCD_DrawRGBbuffer(image: *const ImageBuf);
}

pub struct Display;

#[repr(C)]
struct Point {
    x: u16,
    y: u16,
}

#[repr(C)]
struct ImageBuf {
    top_left: Point,
    width: u16,
    height: u16,
    data: *const u16,
}

impl Display {
    pub fn init() -> Self {
        unsafe {
            LCD_Init();
        }
        Self
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, color: u16) {
        if width == 0 || height == 0 {
            return;
        }

        unsafe {
            LCD_SetTextColor(color);
            LCD_FillRect(x, y, width, height);
        }
    }

    pub fn draw_rgb565(&mut self, x: u16, y: u16, width: u16, height: u16, data: &[u16]) {
        if width == 0 || height == 0 {
            return;
        }

        if data.len() < width as usize * height as usize {
            return;
        }

        let image = ImageBuf {
            top_left: Point { x, y },
            width,
            height,
            data: data.as_ptr(),
        };

        unsafe {
            LCD_DrawRGBbuffer(&image);
        }
    }

    pub fn stroke_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        thickness: u16,
        color: u16,
    ) {
        self.fill_rect(x, y, width, thickness, color);
        self.fill_rect(
            x,
            y.saturating_add(height.saturating_sub(thickness)),
            width,
            thickness,
            color,
        );
        self.fill_rect(x, y, thickness, height, color);
        self.fill_rect(
            x.saturating_add(width.saturating_sub(thickness)),
            y,
            thickness,
            height,
            color,
        );
    }

    pub fn panel(&mut self, x: u16, y: u16, width: u16, height: u16, fill: u16, accent: u16) {
        self.fill_rect(x + 3, y + 4, width, height, color::SHADOW);
        self.fill_rect(x, y, width, height, fill);
        self.stroke_rect(x, y, width, height, 2, accent);
        self.stroke_rect(
            x + 2,
            y + 2,
            width.saturating_sub(4),
            height.saturating_sub(4),
            1,
            color::WHITE,
        );
    }

    pub fn centered_text(
        &mut self,
        center_x: u16,
        y: u16,
        text: &str,
        fg: u16,
        bg: u16,
        scale: u16,
    ) {
        let width = self.measure_text(text, scale);
        let x = center_x.saturating_sub(width / 2);
        self.text(x, y, text, fg, bg, scale);
    }

    pub fn measure_text(&self, text: &str, scale: u16) -> u16 {
        text.chars()
            .map(|ch| glyph_advance(ch, scale))
            .fold(0u16, |acc, width| acc.saturating_add(width))
    }

    pub fn text(&mut self, mut x: u16, y: u16, text: &str, fg: u16, bg: u16, scale: u16) {
        for ch in text.chars() {
            self.glyph(x, y, ch, fg, bg, scale);
            x = x.saturating_add(glyph_advance(ch, scale));
        }
    }

    pub fn glyph(&mut self, x: u16, y: u16, ch: char, fg: u16, bg: u16, scale: u16) {
        if let Some(rows) = font_zh::glyph(ch) {
            for (row_index, row_bits) in rows.iter().enumerate() {
                for col in 0..12u16 {
                    let mask = 1u16 << (11 - col as u16);
                    let color = if (row_bits & mask) != 0 { fg } else { bg };
                    self.fill_rect(
                        x + col * scale,
                        y + (row_index as u16) * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        } else {
            let rows = font::glyph(ch);

            for (row_index, row_bits) in rows.iter().enumerate() {
                for col in 0..5u16 {
                    let mask = 1u8 << (4 - col as u8);
                    let color = if (row_bits & mask) != 0 { fg } else { bg };
                    self.fill_rect(
                        x + col * scale,
                        y + (row_index as u16) * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }
}

fn glyph_advance(ch: char, scale: u16) -> u16 {
    if ch.is_ascii() {
        6 * scale
    } else {
        13 * scale
    }
}

pub fn shade(color: u16, factor: u8) -> u16 {
    let factor = factor as u32;
    let red = (((color >> 11) & 0x1F) as u32 * factor / 255) as u16;
    let green = (((color >> 5) & 0x3F) as u32 * factor / 255) as u16;
    let blue = ((color & 0x1F) as u32 * factor / 255) as u16;
    (red << 11) | (green << 5) | blue
}
