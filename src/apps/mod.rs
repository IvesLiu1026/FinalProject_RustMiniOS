mod album;
mod auto_battle;
mod game_center;
mod paint;
mod tap_rush;

pub use album::{AlbumAction, AlbumApp, AlbumRedraw, AlbumState};
pub use auto_battle::{AutoBattleAction, AutoBattleApp, AutoBattleRedraw};
pub use game_center::{GameCenterAction, GameCenterApp};
pub use paint::{PaintAction, PaintApp, PaintRedraw, PaintState};
pub use tap_rush::{TapRushAction, TapRushApp};

use crate::touch::TouchState;

fn touch_released_in_rect(touch: &TouchState, x: u16, y: u16, width: u16, height: u16) -> bool {
    if !touch.just_released || touch.dragging {
        return false;
    }

    let tap_x = ((touch.start_x as u32 + touch.release_x as u32) / 2) as u16;
    let tap_y = ((touch.start_y as u32 + touch.release_y as u32) / 2) as u16;
    tap_x >= x && tap_x < x.saturating_add(width) && tap_y >= y && tap_y < y.saturating_add(height)
}

fn touch_active_in_rect(touch: &TouchState, x: u16, y: u16, width: u16, height: u16) -> bool {
    touch.active
        && touch.x >= x
        && touch.x < x.saturating_add(width)
        && touch.y >= y
        && touch.y < y.saturating_add(height)
}
