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
    PseudoRacer,
    GraphicsLab,
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
pub enum AppIcon {
    Album,
    GameCenter,
    Paint,
    Settings,
    Dungeon,
    Hunter,
    TapRush,
    Racer,
    Lab,
}

#[derive(Clone, Copy)]
pub struct AppDescriptor {
    pub accent: AppAccent,
    pub icon: AppIcon,
    pub desktop_label_en: &'static str,
    pub desktop_label_zh: &'static str,
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

    pub fn desktop_label(self, zh_mode: bool) -> &'static str {
        if zh_mode {
            self.desktop_label_zh
        } else {
            self.desktop_label_en
        }
    }
}

const HOME_APPS: [AppId; 4] = [
    AppId::Album,
    AppId::GameCenter,
    AppId::Paint,
    AppId::Settings,
];

const GAME_CENTER_APPS: [AppId; 5] = [
    AppId::DungeonCore,
    AppId::AutoBattle,
    AppId::PseudoRacer,
    AppId::GraphicsLab,
    AppId::TapRush,
];

pub fn home_apps() -> &'static [AppId] {
    &HOME_APPS
}

pub fn game_center_apps() -> &'static [AppId] {
    &GAME_CENTER_APPS
}

pub fn home_slot_for_app(app_id: AppId) -> usize {
    match app_id {
        AppId::Album => 0,
        AppId::GameCenter
        | AppId::DungeonCore
        | AppId::AutoBattle
        | AppId::TapRush
        | AppId::PseudoRacer
        | AppId::GraphicsLab => 1,
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
            icon: AppIcon::Album,
            desktop_label_en: "MY ALBUM",
            desktop_label_zh: "我的相簿",
            title_en: "ALBUM",
            title_zh: "相簿",
            subtitle_en: "STILLS + MOTION",
            subtitle_zh: "圖片與動態片段",
        },
        AppId::GameCenter => AppDescriptor {
            accent: AppAccent::Rose,
            icon: AppIcon::GameCenter,
            desktop_label_en: "GAMES",
            desktop_label_zh: "遊戲集",
            title_en: "GAME CENTER",
            title_zh: "遊戲中心",
            subtitle_en: "DUNGEON HEADLINER",
            subtitle_zh: "主打 dungeon",
        },
        AppId::Paint => AppDescriptor {
            accent: AppAccent::Lime,
            icon: AppIcon::Paint,
            desktop_label_en: "PAINT",
            desktop_label_zh: "小畫家",
            title_en: "PIXEL PAINT",
            title_zh: "像素畫板",
            subtitle_en: "RETRO DRAW PAD",
            subtitle_zh: "復古小畫家",
        },
        AppId::Settings => AppDescriptor {
            accent: AppAccent::Orange,
            icon: AppIcon::Settings,
            desktop_label_en: "CONTROL",
            desktop_label_zh: "控制台",
            title_en: "SETTINGS",
            title_zh: "系統設定",
            subtitle_en: "THEME + UTILITIES",
            subtitle_zh: "主題與工具",
        },
        AppId::DungeonCore => AppDescriptor {
            accent: AppAccent::Cyan,
            icon: AppIcon::Dungeon,
            desktop_label_en: "DUNGEON",
            desktop_label_zh: "地城",
            title_en: "DUNGEON CORE",
            title_zh: "地城核心",
            subtitle_en: "MULTI-MAP 3D RAYCAST ADVENTURE",
            subtitle_zh: "多地圖 3D 地城，主打內容",
        },
        AppId::AutoBattle => AppDescriptor {
            accent: AppAccent::Amber,
            icon: AppIcon::Hunter,
            desktop_label_en: "STATION",
            desktop_label_zh: "定點獵手",
            title_en: "STATION HUNTER",
            title_zh: "定點獵手",
            subtitle_en: "STOP TO LOCK, STAGES + BOSSES + BUILDS",
            subtitle_zh: "停下鎖定射擊，含關卡與頭目",
        },
        AppId::TapRush => AppDescriptor {
            accent: AppAccent::Cyan,
            icon: AppIcon::TapRush,
            desktop_label_en: "TAP RUSH",
            desktop_label_zh: "衝刺",
            title_en: "TAP RUSH",
            title_zh: "點擊衝刺",
            subtitle_en: "REACTION MICROGAME WITH FAST ROUNDS",
            subtitle_zh: "反應小遊戲，快打快拿分",
        },
        AppId::PseudoRacer => AppDescriptor {
            accent: AppAccent::Orange,
            icon: AppIcon::Racer,
            desktop_label_en: "RACER",
            desktop_label_zh: "賽車",
            title_en: "PSEUDO RACER",
            title_zh: "假 3D 賽車",
            subtitle_en: "SCANLINE ROAD / CHECKPOINT RUN",
            subtitle_zh: "假 3D 道路與檢查點衝刺",
        },
        AppId::GraphicsLab => AppDescriptor {
            accent: AppAccent::Lime,
            icon: AppIcon::Lab,
            desktop_label_en: "GRAPH LAB",
            desktop_label_zh: "圖學實驗",
            title_en: "GRAPHICS LAB",
            title_zh: "圖學實驗室",
            subtitle_en: "STARFIELD / PLASMA / FIRE / 3D",
            subtitle_zh: "星空 / 電漿 / 火焰 / 3D 線框",
        },
    }
}
