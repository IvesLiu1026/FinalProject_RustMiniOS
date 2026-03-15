use crate::display::Palette;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppId {
    Album,
    GameCenter,
    Paint,
    Settings,
    DungeonCore,
    AutoBattle,
    TapRush,
}

#[derive(Clone, Copy)]
pub enum AppAccent {
    Cyan,
    Rose,
    Lime,
    Orange,
    Amber,
}

impl AppAccent {
    pub fn resolve(self, ui: &Palette) -> u16 {
        match self {
            Self::Cyan => ui.cyan,
            Self::Rose => ui.rose,
            Self::Lime => ui.lime,
            Self::Orange => ui.orange,
            Self::Amber => ui.amber,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppDescriptor {
    pub accent: AppAccent,
    pub title_en: &'static str,
    pub title_zh: &'static str,
    pub subtitle_en: &'static str,
    pub subtitle_zh: &'static str,
}

impl AppDescriptor {
    pub fn title(self, zh_mode: bool) -> &'static str {
        if zh_mode {
            self.title_zh
        } else {
            self.title_en
        }
    }

    pub fn subtitle(self, zh_mode: bool) -> &'static str {
        if zh_mode {
            self.subtitle_zh
        } else {
            self.subtitle_en
        }
    }
}

const HOME_APPS: [AppId; 4] = [
    AppId::Album,
    AppId::GameCenter,
    AppId::Paint,
    AppId::Settings,
];

const GAME_CENTER_APPS: [AppId; 3] = [AppId::DungeonCore, AppId::AutoBattle, AppId::TapRush];

pub fn home_apps() -> &'static [AppId] {
    &HOME_APPS
}

pub fn game_center_apps() -> &'static [AppId] {
    &GAME_CENTER_APPS
}

pub fn home_slot_for_app(app_id: AppId) -> usize {
    match app_id {
        AppId::Album => 0,
        AppId::GameCenter | AppId::DungeonCore | AppId::AutoBattle | AppId::TapRush => 1,
        AppId::Paint => 2,
        AppId::Settings => 3,
    }
}

pub fn game_center_slot_for_app(app_id: AppId) -> Option<usize> {
    game_center_apps()
        .iter()
        .position(|candidate| *candidate == app_id)
}

pub fn descriptor(app_id: AppId) -> AppDescriptor {
    match app_id {
        AppId::Album => AppDescriptor {
            accent: AppAccent::Cyan,
            title_en: "ALBUM",
            title_zh: "相簿",
            subtitle_en: "STILLS + MOTION",
            subtitle_zh: "圖片與動態片段",
        },
        AppId::GameCenter => AppDescriptor {
            accent: AppAccent::Rose,
            title_en: "GAME CENTER",
            title_zh: "遊戲中心",
            subtitle_en: "DUNGEON HEADLINER",
            subtitle_zh: "主打 dungeon",
        },
        AppId::Paint => AppDescriptor {
            accent: AppAccent::Lime,
            title_en: "PIXEL PAINT",
            title_zh: "像素畫板",
            subtitle_en: "RETRO DRAW PAD",
            subtitle_zh: "復古小畫家",
        },
        AppId::Settings => AppDescriptor {
            accent: AppAccent::Orange,
            title_en: "SETTINGS",
            title_zh: "系統設定",
            subtitle_en: "THEME + UTILITIES",
            subtitle_zh: "主題與工具",
        },
        AppId::DungeonCore => AppDescriptor {
            accent: AppAccent::Cyan,
            title_en: "DUNGEON CORE",
            title_zh: "地城核心",
            subtitle_en: "MULTI-MAP 3D RAYCAST ADVENTURE",
            subtitle_zh: "多地圖 3D 地城，主打內容",
        },
        AppId::AutoBattle => AppDescriptor {
            accent: AppAccent::Amber,
            title_en: "AUTO HUNTER",
            title_zh: "自動獵手",
            subtitle_en: "STOP TO AUTO-FIRE THE NEAREST ENEMY",
            subtitle_zh: "停下來自動射最近敵人",
        },
        AppId::TapRush => AppDescriptor {
            accent: AppAccent::Cyan,
            title_en: "TAP RUSH",
            title_zh: "點擊衝刺",
            subtitle_en: "REACTION MICROGAME WITH FAST ROUNDS",
            subtitle_zh: "反應小遊戲，快打快拿分",
        },
    }
}
