use crate::assets::EnemySprite;

pub(crate) const MAP_W: usize = 12;
pub(crate) const MAP_H: usize = 12;
pub(crate) const MAX_ENEMIES: usize = 6;
pub(crate) const MAX_PICKUPS: usize = 6;

#[derive(Clone, Copy)]
pub(crate) struct EnemySpawn {
    pub(crate) kind: EnemySprite,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct PickupSpawn {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) amount: i16,
}

#[derive(Clone, Copy)]
pub(crate) struct MapDef {
    pub(crate) name_en: &'static str,
    pub(crate) name_zh: &'static str,
    pub(crate) layout: [[u8; MAP_W]; MAP_H],
    pub(crate) spawn_x: f32,
    pub(crate) spawn_y: f32,
    pub(crate) spawn_angle: f32,
    pub(crate) enemies: &'static [EnemySpawn],
    pub(crate) pickups: &'static [PickupSpawn],
}

const MAP0_ENEMIES: [EnemySpawn; 3] = [
    EnemySpawn {
        kind: EnemySprite::Bat,
        x: 5.5,
        y: 2.5,
    },
    EnemySpawn {
        kind: EnemySprite::Imp,
        x: 9.2,
        y: 8.4,
    },
    EnemySpawn {
        kind: EnemySprite::Hound,
        x: 7.3,
        y: 5.4,
    },
];

const MAP0_PICKUPS: [PickupSpawn; 2] = [
    PickupSpawn {
        x: 3.5,
        y: 3.5,
        amount: 28,
    },
    PickupSpawn {
        x: 8.5,
        y: 9.4,
        amount: 22,
    },
];

const MAP1_ENEMIES: [EnemySpawn; 4] = [
    EnemySpawn {
        kind: EnemySprite::Hound,
        x: 9.5,
        y: 2.5,
    },
    EnemySpawn {
        kind: EnemySprite::Bat,
        x: 8.1,
        y: 4.9,
    },
    EnemySpawn {
        kind: EnemySprite::Imp,
        x: 5.3,
        y: 8.5,
    },
    EnemySpawn {
        kind: EnemySprite::Bat,
        x: 2.7,
        y: 8.1,
    },
];

const MAP1_PICKUPS: [PickupSpawn; 2] = [
    PickupSpawn {
        x: 4.2,
        y: 1.8,
        amount: 24,
    },
    PickupSpawn {
        x: 8.8,
        y: 8.8,
        amount: 30,
    },
];

const MAP2_ENEMIES: [EnemySpawn; 4] = [
    EnemySpawn {
        kind: EnemySprite::Imp,
        x: 8.5,
        y: 2.5,
    },
    EnemySpawn {
        kind: EnemySprite::Hound,
        x: 9.5,
        y: 7.5,
    },
    EnemySpawn {
        kind: EnemySprite::Bat,
        x: 4.4,
        y: 7.1,
    },
    EnemySpawn {
        kind: EnemySprite::Imp,
        x: 7.2,
        y: 9.2,
    },
];

const MAP2_PICKUPS: [PickupSpawn; 3] = [
    PickupSpawn {
        x: 2.4,
        y: 4.6,
        amount: 24,
    },
    PickupSpawn {
        x: 8.6,
        y: 8.4,
        amount: 26,
    },
    PickupSpawn {
        x: 10.1,
        y: 2.1,
        amount: 18,
    },
];

pub(crate) const MAPS: [MapDef; 3] = [
    MapDef {
        name_en: "SUNLIT RUINS",
        name_zh: "日耀遺跡",
        layout: [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 1],
            [1, 0, 5, 0, 2, 0, 1, 0, 2, 5, 0, 1],
            [1, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 1],
            [1, 0, 2, 0, 0, 0, 1, 1, 1, 0, 0, 1],
            [1, 0, 2, 0, 5, 0, 0, 0, 1, 0, 0, 1],
            [1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 5, 1],
            [1, 0, 1, 0, 2, 0, 1, 0, 2, 0, 0, 1],
            [1, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 1],
            [1, 0, 2, 0, 0, 0, 5, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ],
        spawn_x: 1.5,
        spawn_y: 1.5,
        spawn_angle: 0.15,
        enemies: &MAP0_ENEMIES,
        pickups: &MAP0_PICKUPS,
    },
    MapDef {
        name_en: "EMBER FORGE",
        name_zh: "餘燼熔爐",
        layout: [
            [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            [2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 2],
            [2, 0, 3, 0, 2, 0, 2, 2, 2, 0, 0, 2],
            [2, 0, 2, 0, 2, 0, 0, 0, 2, 0, 5, 2],
            [2, 0, 2, 0, 2, 2, 2, 0, 2, 0, 0, 2],
            [2, 0, 2, 0, 0, 0, 2, 0, 2, 0, 0, 2],
            [2, 0, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2],
            [2, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 2],
            [2, 0, 5, 0, 2, 0, 2, 2, 2, 0, 0, 2],
            [2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 2],
            [2, 0, 0, 0, 2, 2, 2, 0, 0, 0, 0, 2],
            [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        ],
        spawn_x: 1.5,
        spawn_y: 9.5,
        spawn_angle: -0.7,
        enemies: &MAP1_ENEMIES,
        pickups: &MAP1_PICKUPS,
    },
    MapDef {
        name_en: "NIGHT CRYPT",
        name_zh: "夜影墓穴",
        layout: [
            [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
            [3, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 3],
            [3, 0, 3, 3, 3, 0, 3, 0, 3, 0, 0, 3],
            [3, 0, 0, 0, 3, 0, 3, 0, 3, 0, 0, 3],
            [3, 5, 3, 0, 3, 0, 3, 0, 3, 3, 0, 3],
            [3, 0, 3, 0, 0, 0, 3, 0, 0, 3, 0, 3],
            [3, 0, 3, 3, 3, 0, 3, 3, 0, 3, 0, 3],
            [3, 0, 0, 0, 3, 0, 0, 0, 0, 3, 0, 3],
            [3, 0, 3, 0, 3, 3, 3, 0, 5, 3, 0, 3],
            [3, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 3],
            [3, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 3],
            [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
        ],
        spawn_x: 1.5,
        spawn_y: 1.5,
        spawn_angle: 0.3,
        enemies: &MAP2_ENEMIES,
        pickups: &MAP2_PICKUPS,
    },
];
