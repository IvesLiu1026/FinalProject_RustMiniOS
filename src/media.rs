pub struct EmbeddedStill {
    pub label: &'static str,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub data: &'static [u8],
}

pub struct EmbeddedMotionClip {
    pub label: &'static str,
    pub width: u16,
    pub height: u16,
    pub scale: u16,
    pub frame_delay_ms: u16,
    pub frames: &'static [&'static [u8]],
}

impl EmbeddedMotionClip {
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn frame(&self, index: usize) -> &'static [u8] {
        self.frames[index % self.frames.len().max(1)]
    }
}

include!(concat!(env!("OUT_DIR"), "/generated_media.rs"));

pub fn stills() -> &'static [EmbeddedStill] {
    &STILL_IMAGES
}

pub fn motion_clips() -> &'static [EmbeddedMotionClip] {
    &MOTION_CLIPS
}
