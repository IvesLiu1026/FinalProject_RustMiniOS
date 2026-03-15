#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderStrategy {
    Quality,
    Balanced,
    Performance,
}

impl RenderStrategy {
    pub const fn next(self) -> Self {
        match self {
            Self::Quality => Self::Balanced,
            Self::Balanced => Self::Performance,
            Self::Performance => Self::Quality,
        }
    }

    pub const fn wall_stride(self) -> usize {
        match self {
            Self::Quality => 1,
            Self::Balanced => 1,
            Self::Performance => 2,
        }
    }

    pub const fn floor_stride(self) -> usize {
        match self {
            Self::Quality => 1,
            Self::Balanced => 2,
            Self::Performance => 4,
        }
    }

    pub const fn ceiling_mix_alpha(self) -> u8 {
        match self {
            Self::Quality => 132,
            Self::Balanced => 120,
            Self::Performance => 108,
        }
    }

    pub const fn floor_mix_alpha(self) -> u8 {
        match self {
            Self::Quality => 68,
            Self::Balanced => 60,
            Self::Performance => 52,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Quality => "QUALITY",
            Self::Balanced => "BALANCED",
            Self::Performance => "PERFORMANCE",
        }
    }
}
