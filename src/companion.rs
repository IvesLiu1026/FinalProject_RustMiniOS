use core::str;

use stm32f4::stm32f407::{self, gpioa};

use crate::board::millis;
use crate::display::Display;

const MAX_STILLS: usize = 16;
const MAX_MOTION: usize = 8;
const MAX_LABEL_LEN: usize = 24;
const MAX_LINE_LEN: usize = 96;
#[cfg(minios_mac_companion)]
const COMPANION_ENABLED: bool = true;
#[cfg(not(minios_mac_companion))]
const COMPANION_ENABLED: bool = false;
const FRAME_BUFFER_CAPACITY: usize = if COMPANION_ENABLED { 120 * 90 * 2 } else { 0 };
const CATALOG_RETRY_MS: u32 = 1_500;
const LINE_TIMEOUT_MS: u32 = 220;
const FRAME_TIMEOUT_MS: u32 = 1_500;
const PCLK1_HZ: u32 = 42_000_000;
const COMPANION_BAUD_HZ: u32 = 921_600;

const USART_SR_FE: u16 = 1 << 1;
const USART_SR_NE: u16 = 1 << 2;
const USART_SR_ORE: u16 = 1 << 3;
const USART_SR_RXNE: u16 = 1 << 5;
const USART_SR_TXE: u16 = 1 << 7;

const USART_CR1_RE: u16 = 1 << 2;
const USART_CR1_TE: u16 = 1 << 3;
const USART_CR1_UE: u16 = 1 << 13;

type Gpio = gpioa::RegisterBlock;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompanionState {
    Waiting,
    Ready,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompanionError {
    Timeout,
    Protocol,
    FrameTooLarge,
}

#[derive(Clone, Copy)]
pub struct CompactLabel {
    bytes: [u8; MAX_LABEL_LEN],
    len: u8,
}

impl CompactLabel {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; MAX_LABEL_LEN],
            len: 0,
        }
    }

    fn write_from(&mut self, value: &str) {
        self.len = 0;
        for byte in self.bytes.iter_mut() {
            *byte = 0;
        }

        for (index, byte) in value.as_bytes().iter().enumerate() {
            if index >= MAX_LABEL_LEN {
                break;
            }
            self.bytes[index] = *byte;
            self.len = (index + 1) as u8;
        }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("")
    }
}

#[derive(Clone, Copy)]
pub struct CompanionStillMeta {
    pub label: CompactLabel,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub valid: bool,
}

impl CompanionStillMeta {
    pub const fn empty() -> Self {
        Self {
            label: CompactLabel::empty(),
            width: 0,
            height: 0,
            scale: 0,
            valid: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CompanionMotionMeta {
    pub label: CompactLabel,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub frame_delay_ms: u16,
    pub frame_count: u16,
    pub valid: bool,
}

impl CompanionMotionMeta {
    pub const fn empty() -> Self {
        Self {
            label: CompactLabel::empty(),
            width: 0,
            height: 0,
            scale: 0,
            frame_delay_ms: 0,
            frame_count: 0,
            valid: false,
        }
    }
}

pub struct CompanionLink {
    initialized: bool,
    state: CompanionState,
    last_error_code: u8,
    last_attempt_ms: u32,
    still_count: usize,
    motion_count: usize,
    stills: [CompanionStillMeta; MAX_STILLS],
    motion: [CompanionMotionMeta; MAX_MOTION],
    cached_kind: u8,
    cached_item_index: usize,
    cached_frame_index: usize,
    cached_width: u16,
    cached_height: u16,
    cached_scale: u16,
    cached_len: usize,
    frame_buffer: [u8; FRAME_BUFFER_CAPACITY],
}

impl CompanionLink {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            state: CompanionState::Waiting,
            last_error_code: 0,
            last_attempt_ms: 0,
            still_count: 0,
            motion_count: 0,
            stills: [CompanionStillMeta::empty(); MAX_STILLS],
            motion: [CompanionMotionMeta::empty(); MAX_MOTION],
            cached_kind: 0,
            cached_item_index: 0,
            cached_frame_index: 0,
            cached_width: 0,
            cached_height: 0,
            cached_scale: 0,
            cached_len: 0,
            frame_buffer: [0; FRAME_BUFFER_CAPACITY],
        }
    }

    pub fn tick(&mut self) {
        if !COMPANION_ENABLED {
            return;
        }
        self.ensure_uart_ready();
        if matches!(self.state, CompanionState::Ready) {
            return;
        }

        let now = millis();
        if now.wrapping_sub(self.last_attempt_ms) < CATALOG_RETRY_MS {
            return;
        }

        self.last_attempt_ms = now;
        let _ = self.refresh_catalog();
    }

    pub fn state(&self) -> CompanionState {
        self.state
    }

    pub fn last_error(&self) -> Option<CompanionError> {
        match self.last_error_code {
            1 => Some(CompanionError::Timeout),
            2 => Some(CompanionError::Protocol),
            3 => Some(CompanionError::FrameTooLarge),
            _ => None,
        }
    }

    pub fn still_count(&self) -> usize {
        self.still_count
    }

    pub fn motion_count(&self) -> usize {
        self.motion_count
    }

    pub fn still(&self, index: usize) -> Option<&CompanionStillMeta> {
        self.stills.get(index).filter(|entry| entry.valid)
    }

    pub fn motion_clip(&self, index: usize) -> Option<&CompanionMotionMeta> {
        self.motion.get(index).filter(|entry| entry.valid)
    }

    pub fn cached_still_index(&self) -> Option<usize> {
        if self.cached_kind == 1 {
            Some(self.cached_item_index)
        } else {
            None
        }
    }

    pub fn cached_motion_frame(&self) -> Option<(usize, usize)> {
        if self.cached_kind == 2 {
            Some((self.cached_item_index, self.cached_frame_index))
        } else {
            None
        }
    }

    pub fn fetch_still(&mut self, index: usize) -> bool {
        if !COMPANION_ENABLED {
            return false;
        }
        if self.still(index).is_none() {
            return false;
        }

        let mut line = [0u8; MAX_LINE_LEN];
        let mut request = heapless::String::<24>::new();
        let _ = core::fmt::write(&mut request, format_args!("GET|S|{index}\n"));
        self.flush_rx();
        self.write_bytes(request.as_bytes());

        let Ok(header) = self.read_line(FRAME_TIMEOUT_MS, &mut line) else {
            self.invalidate(CompanionError::Timeout);
            return false;
        };

        let mut parts = header.split('|');
        let valid = matches!(parts.next(), Some("FRAME"))
            && matches!(parts.next(), Some("S"))
            && parse_usize(parts.next()) == Some(index)
            && self.load_frame_payload(
                parse_u16(parts.next()),
                parse_u16(parts.next()),
                parse_u16(parts.next()),
                parse_usize(parts.next()),
            );
        if !valid {
            self.invalidate(CompanionError::Protocol);
            return false;
        }

        self.cached_kind = 1;
        self.cached_item_index = index;
        self.cached_frame_index = 0;
        true
    }

    pub fn fetch_motion_frame(&mut self, clip_index: usize, frame_index: usize) -> bool {
        if !COMPANION_ENABLED {
            return false;
        }
        if self.motion_clip(clip_index).is_none() {
            return false;
        }

        let mut line = [0u8; MAX_LINE_LEN];
        let mut request = heapless::String::<32>::new();
        let _ = core::fmt::write(
            &mut request,
            format_args!("GET|M|{clip_index}|{frame_index}\n"),
        );
        self.flush_rx();
        self.write_bytes(request.as_bytes());

        let Ok(header) = self.read_line(FRAME_TIMEOUT_MS, &mut line) else {
            self.invalidate(CompanionError::Timeout);
            return false;
        };

        let mut parts = header.split('|');
        let valid = matches!(parts.next(), Some("FRAME"))
            && matches!(parts.next(), Some("M"))
            && parse_usize(parts.next()) == Some(clip_index)
            && parse_usize(parts.next()) == Some(frame_index)
            && self.load_frame_payload(
                parse_u16(parts.next()),
                parse_u16(parts.next()),
                parse_u16(parts.next()),
                parse_usize(parts.next()),
            );
        if !valid {
            self.invalidate(CompanionError::Protocol);
            return false;
        }

        self.cached_kind = 2;
        self.cached_item_index = clip_index;
        self.cached_frame_index = frame_index;
        true
    }

    pub fn draw_cached_frame(&self, display: &mut Display, x: u16, y: u16) {
        if !COMPANION_ENABLED || self.cached_len == 0 {
            return;
        }

        display.draw_rgb565_scaled_bytes(
            x,
            y,
            self.cached_width,
            self.cached_height,
            self.cached_scale,
            &self.frame_buffer[..self.cached_len],
        );
    }

    fn refresh_catalog(&mut self) -> Result<(), CompanionError> {
        if !COMPANION_ENABLED {
            return Err(CompanionError::Protocol);
        }
        self.flush_rx();
        self.write_bytes(b"HELLO|1\n");

        let mut line = [0u8; MAX_LINE_LEN];
        let ready = self.read_line(LINE_TIMEOUT_MS, &mut line)?;
        let mut ready_parts = ready.split('|');
        if !matches!(ready_parts.next(), Some("READY")) {
            return Err(CompanionError::Protocol);
        }

        let reported_stills = parse_usize(ready_parts.next()).ok_or(CompanionError::Protocol)?;
        let reported_motion = parse_usize(ready_parts.next()).ok_or(CompanionError::Protocol)?;

        self.clear_catalog();

        for _ in 0..reported_stills.saturating_add(reported_motion) {
            let entry = self.read_line(LINE_TIMEOUT_MS, &mut line)?;
            self.parse_catalog_entry(entry)?;
        }

        let end = self.read_line(LINE_TIMEOUT_MS, &mut line)?;
        if end != "END" {
            return Err(CompanionError::Protocol);
        }

        self.still_count = reported_stills.min(MAX_STILLS);
        self.motion_count = reported_motion.min(MAX_MOTION);
        self.state = CompanionState::Ready;
        self.last_error_code = 0;
        Ok(())
    }

    fn parse_catalog_entry(&mut self, entry: &str) -> Result<(), CompanionError> {
        let mut parts = entry.split('|');
        match parts.next() {
            Some("S") => {
                let index = parse_usize(parts.next()).ok_or(CompanionError::Protocol)?;
                let width = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let height = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let scale = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let label = parts.next().ok_or(CompanionError::Protocol)?;
                if let Some(slot) = self.stills.get_mut(index) {
                    slot.width = width;
                    slot.height = height;
                    slot.scale = scale.max(1);
                    slot.valid = true;
                    slot.label.write_from(label);
                }
            }
            Some("M") => {
                let index = parse_usize(parts.next()).ok_or(CompanionError::Protocol)?;
                let width = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let height = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let scale = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let frame_delay_ms = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let frame_count = parse_u16(parts.next()).ok_or(CompanionError::Protocol)?;
                let label = parts.next().ok_or(CompanionError::Protocol)?;
                if let Some(slot) = self.motion.get_mut(index) {
                    slot.width = width;
                    slot.height = height;
                    slot.scale = scale.max(1);
                    slot.frame_delay_ms = frame_delay_ms.max(1);
                    slot.frame_count = frame_count.max(1);
                    slot.valid = true;
                    slot.label.write_from(label);
                }
            }
            _ => return Err(CompanionError::Protocol),
        }

        Ok(())
    }

    fn load_frame_payload(
        &mut self,
        width: Option<u16>,
        height: Option<u16>,
        scale: Option<u16>,
        len: Option<usize>,
    ) -> bool {
        let Some(width) = width else {
            return false;
        };
        let Some(height) = height else {
            return false;
        };
        let Some(scale) = scale else {
            return false;
        };
        let Some(len) = len else {
            return false;
        };

        if len == 0 || len > FRAME_BUFFER_CAPACITY {
            self.invalidate(CompanionError::FrameTooLarge);
            return false;
        }

        let start = millis();
        for index in 0..len {
            match self.read_byte(start, FRAME_TIMEOUT_MS) {
                Ok(byte) => self.frame_buffer[index] = byte,
                Err(error) => {
                    self.invalidate(error);
                    return false;
                }
            }
        }

        self.cached_width = width;
        self.cached_height = height;
        self.cached_scale = scale.max(1);
        self.cached_len = len;
        self.state = CompanionState::Ready;
        self.last_error_code = 0;
        true
    }

    fn ensure_uart_ready(&mut self) {
        if self.initialized {
            return;
        }

        let rcc = unsafe { &*stm32f407::RCC::ptr() };
        rcc.ahb1enr().modify(|_, w| w.gpiocen().set_bit());
        rcc.apb1enr().modify(|_, w| w.usart3en().set_bit());
        let _ = rcc.apb1enr().read().bits();

        let gpio = gpioc();
        gpio.moder().modify(|r, w| unsafe {
            let mut bits = r.bits();
            bits &= !((0x3 << (10 * 2)) | (0x3 << (11 * 2)));
            bits |= (0x2 << (10 * 2)) | (0x2 << (11 * 2));
            w.bits(bits)
        });
        gpio.otyper()
            .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 10) | (1 << 11))) });
        gpio.ospeedr().modify(|r, w| unsafe {
            let mut bits = r.bits();
            bits &= !((0x3 << (10 * 2)) | (0x3 << (11 * 2)));
            bits |= (0x3 << (10 * 2)) | (0x3 << (11 * 2));
            w.bits(bits)
        });
        gpio.pupdr().modify(|r, w| unsafe {
            let mut bits = r.bits();
            bits &= !((0x3 << (10 * 2)) | (0x3 << (11 * 2)));
            bits |= 0x1 << (11 * 2);
            w.bits(bits)
        });
        gpio.afrh().modify(|r, w| unsafe {
            let mut bits = r.bits();
            bits &= !((0xF << 8) | (0xF << 12));
            bits |= (0x7 << 8) | (0x7 << 12);
            w.bits(bits)
        });

        let usart = usart3();
        usart.cr1().write(|w| unsafe { w.bits(0) });
        let divider = ((PCLK1_HZ + (COMPANION_BAUD_HZ / 2)) / COMPANION_BAUD_HZ).max(1) as u16;
        usart.brr().write(|w| unsafe { w.bits(divider) });
        usart.cr2().write(|w| unsafe { w.bits(0) });
        usart.cr3().write(|w| unsafe { w.bits(0) });
        usart
            .cr1()
            .write(|w| unsafe { w.bits(USART_CR1_RE | USART_CR1_TE | USART_CR1_UE) });

        self.initialized = true;
        self.flush_rx();
    }

    fn invalidate(&mut self, error: CompanionError) {
        self.state = CompanionState::Error;
        self.last_error_code = match error {
            CompanionError::Timeout => 1,
            CompanionError::Protocol => 2,
            CompanionError::FrameTooLarge => 3,
        };
        self.cached_kind = 0;
        self.cached_item_index = 0;
        self.cached_frame_index = 0;
        self.cached_len = 0;
    }

    fn clear_catalog(&mut self) {
        self.still_count = 0;
        self.motion_count = 0;
        self.cached_kind = 0;
        self.cached_item_index = 0;
        self.cached_frame_index = 0;
        self.cached_len = 0;
        for slot in self.stills.iter_mut() {
            *slot = CompanionStillMeta::empty();
        }
        for slot in self.motion.iter_mut() {
            *slot = CompanionMotionMeta::empty();
        }
    }

    fn write_bytes(&self, bytes: &[u8]) {
        let usart = usart3();
        for byte in bytes {
            while (usart.sr().read().bits() & USART_SR_TXE) == 0 {}
            usart.dr().write(|w| unsafe { w.bits(*byte as u16) });
        }
    }

    fn flush_rx(&self) {
        let usart = usart3();
        loop {
            let sr = usart.sr().read().bits();
            if (sr & (USART_SR_RXNE | USART_SR_FE | USART_SR_NE | USART_SR_ORE)) == 0 {
                break;
            }
            let _ = usart.dr().read().bits();
        }
    }

    fn read_line<'a, const N: usize>(
        &self,
        timeout_ms: u32,
        buffer: &'a mut [u8; N],
    ) -> Result<&'a str, CompanionError> {
        let start = millis();
        let mut len = 0usize;

        loop {
            let byte = self.read_byte(start, timeout_ms)?;
            match byte {
                b'\n' => break,
                b'\r' => continue,
                _ => {
                    if len >= N {
                        return Err(CompanionError::Protocol);
                    }
                    buffer[len] = byte;
                    len += 1;
                }
            }
        }

        str::from_utf8(&buffer[..len]).map_err(|_| CompanionError::Protocol)
    }

    fn read_byte(&self, start_ms: u32, timeout_ms: u32) -> Result<u8, CompanionError> {
        let usart = usart3();
        loop {
            let sr = usart.sr().read().bits();
            if (sr & (USART_SR_FE | USART_SR_NE | USART_SR_ORE)) != 0 {
                let _ = usart.dr().read().bits();
            } else if (sr & USART_SR_RXNE) != 0 {
                return Ok((usart.dr().read().bits() & 0xFF) as u8);
            }

            if millis().wrapping_sub(start_ms) >= timeout_ms {
                return Err(CompanionError::Timeout);
            }
        }
    }
}

static mut COMPANION_LINK: CompanionLink = CompanionLink::new();

#[allow(static_mut_refs)]
pub fn link() -> &'static mut CompanionLink {
    unsafe { &mut COMPANION_LINK }
}

pub fn baud_rate() -> u32 {
    COMPANION_BAUD_HZ
}

fn parse_usize(raw: Option<&str>) -> Option<usize> {
    raw.and_then(|value| value.parse::<usize>().ok())
}

fn parse_u16(raw: Option<&str>) -> Option<u16> {
    raw.and_then(|value| value.parse::<u16>().ok())
}

fn gpioc() -> &'static Gpio {
    unsafe { &*stm32f407::GPIOC::ptr().cast::<Gpio>() }
}

fn usart3() -> &'static stm32f407::usart3::RegisterBlock {
    unsafe { &*stm32f407::USART3::ptr() }
}
