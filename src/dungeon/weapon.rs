use crate::display::Palette;

const FLASH_MS: u16 = 90;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WeaponKind {
    Pulse,
    Carbine,
    Scatter,
}

impl WeaponKind {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Pulse => Self::Carbine,
            Self::Carbine => Self::Scatter,
            Self::Scatter => Self::Pulse,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::Pulse => Self::Scatter,
            Self::Carbine => Self::Pulse,
            Self::Scatter => Self::Carbine,
        }
    }

    pub(crate) fn cooldown_ms(self) -> u16 {
        match self {
            Self::Pulse => 280,
            Self::Carbine => 165,
            Self::Scatter => 430,
        }
    }

    pub(crate) fn flash_ms(self) -> u16 {
        match self {
            Self::Pulse => FLASH_MS,
            Self::Carbine => 70,
            Self::Scatter => 120,
        }
    }

    pub(crate) fn label_en(self) -> &'static str {
        match self {
            Self::Pulse => "PULSE",
            Self::Carbine => "CARBINE",
            Self::Scatter => "SCATTER",
        }
    }

    pub(crate) fn label_zh(self) -> &'static str {
        match self {
            Self::Pulse => "脈衝槍",
            Self::Carbine => "卡賓槍",
            Self::Scatter => "散射炮",
        }
    }

    pub(crate) fn accent(self, ui: &Palette) -> u16 {
        match self {
            Self::Pulse => ui.cyan,
            Self::Carbine => ui.lime,
            Self::Scatter => ui.amber,
        }
    }
}
