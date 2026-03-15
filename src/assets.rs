pub const TEX_SIZE: usize = 64;
pub const SPRITE_SIZE: usize = 24;

pub enum TextureId {
    WallLight,
    WallMid,
    WallDark,
    DoorDark,
    WindowDark,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemySprite {
    Imp,
    Bat,
    Hound,
}

pub struct SpriteAsset {
    pub rgb565: &'static [u8],
    pub mask: &'static [u8],
    pub width: usize,
    pub height: usize,
}

const WALL_LIGHT: &[u8] = include_bytes!("../assets/converted/textures/wall_light.rgb565");
const WALL_MID: &[u8] = include_bytes!("../assets/converted/textures/wall_mid.rgb565");
const WALL_DARK: &[u8] = include_bytes!("../assets/converted/textures/wall_dark.rgb565");
const DOOR_DARK: &[u8] = include_bytes!("../assets/converted/textures/door_dark.rgb565");
const WINDOW_DARK: &[u8] = include_bytes!("../assets/converted/textures/window_dark.rgb565");

const IMP_RGB565: &[u8] = include_bytes!("../assets/converted/sprites/imp.rgb565");
const IMP_MASK: &[u8] = include_bytes!("../assets/converted/sprites/imp.mask");
const BAT_RGB565: &[u8] = include_bytes!("../assets/converted/sprites/bat.rgb565");
const BAT_MASK: &[u8] = include_bytes!("../assets/converted/sprites/bat.mask");
const HOUND_RGB565: &[u8] = include_bytes!("../assets/converted/sprites/hound.rgb565");
const HOUND_MASK: &[u8] = include_bytes!("../assets/converted/sprites/hound.mask");

pub fn texture(id: TextureId) -> &'static [u8] {
    match id {
        TextureId::WallLight => WALL_LIGHT,
        TextureId::WallMid => WALL_MID,
        TextureId::WallDark => WALL_DARK,
        TextureId::DoorDark => DOOR_DARK,
        TextureId::WindowDark => WINDOW_DARK,
    }
}

pub fn enemy_sprite(id: EnemySprite) -> SpriteAsset {
    match id {
        EnemySprite::Imp => SpriteAsset {
            rgb565: IMP_RGB565,
            mask: IMP_MASK,
            width: SPRITE_SIZE,
            height: SPRITE_SIZE,
        },
        EnemySprite::Bat => SpriteAsset {
            rgb565: BAT_RGB565,
            mask: BAT_MASK,
            width: SPRITE_SIZE,
            height: SPRITE_SIZE,
        },
        EnemySprite::Hound => SpriteAsset {
            rgb565: HOUND_RGB565,
            mask: HOUND_MASK,
            width: SPRITE_SIZE,
            height: SPRITE_SIZE,
        },
    }
}

pub fn texture_sample(texture_data: &[u8], x: usize, y: usize) -> u16 {
    let idx = (y * TEX_SIZE + x) * 2;
    u16::from_le_bytes([texture_data[idx], texture_data[idx + 1]])
}

pub fn sprite_sample(asset: &SpriteAsset, x: usize, y: usize) -> Option<u16> {
    let idx = y * asset.width + x;
    if asset.mask[idx] < 16 {
        return None;
    }
    let byte = idx * 2;
    Some(u16::from_le_bytes([
        asset.rgb565[byte],
        asset.rgb565[byte + 1],
    ]))
}
