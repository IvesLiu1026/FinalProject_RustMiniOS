mod album;
mod auto_battle;
mod game_center;
mod graphics_lab;
mod paint;
mod pseudo_racer;
mod tap_rush;

pub use album::{AlbumAction, AlbumApp, AlbumRedraw, AlbumState};
pub use auto_battle::{AutoBattleAction, AutoBattleApp, AutoBattleRedraw};
pub use game_center::{GameCenterAction, GameCenterApp};
pub use graphics_lab::{GraphicsLabAction, GraphicsLabApp};
pub use paint::{PaintAction, PaintApp, PaintRedraw, PaintState};
pub use pseudo_racer::{PseudoRacerAction, PseudoRacerApp};
pub use tap_rush::{TapRushAction, TapRushApp};

use crate::touch::TouchState;

fn touch_released_in_rect(touch: &TouchState, x: u16, y: u16, width: u16, height: u16) -> bool {
    if !touch.just_released {
        return false;
    }

    if touch.dragging {
        return point_in_rect_with_slop(touch.start_x, touch.start_y, x, y, width, height)
            && point_in_rect_with_slop(touch.release_x, touch.release_y, x, y, width, height);
    }

    let tap_x = ((touch.start_x as u32 + touch.release_x as u32) / 2) as u16;
    let tap_y = ((touch.start_y as u32 + touch.release_y as u32) / 2) as u16;
    point_in_rect_with_slop(tap_x, tap_y, x, y, width, height)
}

fn touch_active_in_rect(touch: &TouchState, x: u16, y: u16, width: u16, height: u16) -> bool {
    touch.active
        && touch.x >= x
        && touch.x < x.saturating_add(width)
        && touch.y >= y
        && touch.y < y.saturating_add(height)
}

fn point_in_rect_with_slop(px: u16, py: u16, x: u16, y: u16, width: u16, height: u16) -> bool {
    let slop = 10u16;
    let left = x.saturating_sub(slop);
    let top = y.saturating_sub(slop);
    let right = x.saturating_add(width).saturating_add(slop);
    let bottom = y.saturating_add(height).saturating_add(slop);
    px >= left && px < right && py >= top && py < bottom
}
