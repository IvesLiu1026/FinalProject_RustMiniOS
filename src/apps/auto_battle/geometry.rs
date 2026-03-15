use crate::display::Display;

use super::{
    ARENA_INNER_H, ARENA_INNER_W, ARENA_INNER_X, ARENA_INNER_Y, MAX_ARENA_DIRTY_RECTS, MAX_ENEMIES,
    MAX_PROJECTILES,
};

#[derive(Clone, Copy)]
pub(super) struct Rect {
    pub(super) x: i16,
    pub(super) y: i16,
    pub(super) w: i16,
    pub(super) h: i16,
}

impl Rect {
    pub(super) const fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }
    }

    pub(super) const fn is_empty(self) -> bool {
        self.w <= 0 || self.h <= 0
    }

    fn right(self) -> i16 {
        self.x + self.w
    }

    fn bottom(self) -> i16 {
        self.y + self.h
    }

    fn intersects(self, other: Self) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    fn touches(self, other: Self) -> bool {
        self.intersects(other)
            || (!self.is_empty()
                && !other.is_empty()
                && self.x <= other.right() + 1
                && self.right() + 1 >= other.x
                && self.y <= other.bottom() + 1
                && self.bottom() + 1 >= other.y)
    }

    pub(super) fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            x: left,
            y: top,
            w: right - left,
            h: bottom - top,
        }
    }

    pub(super) fn intersect(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= left || bottom <= top {
            Self::empty()
        } else {
            Self {
                x: left,
                y: top,
                w: right - left,
                h: bottom - top,
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct DirtyRegions {
    rects: [Rect; MAX_ARENA_DIRTY_RECTS],
    len: usize,
}

impl DirtyRegions {
    pub(super) const fn new() -> Self {
        Self {
            rects: [Rect::empty(); MAX_ARENA_DIRTY_RECTS],
            len: 0,
        }
    }

    pub(super) fn add(&mut self, mut rect: Rect) {
        rect = rect.intersect(arena_inner_rect());
        if rect.is_empty() {
            return;
        }

        let mut index = 0usize;
        while index < self.len {
            if self.rects[index].touches(rect) {
                rect = self.rects[index].union(rect);
                self.rects[index] = self.rects[self.len - 1];
                self.len -= 1;
                index = 0;
                continue;
            }
            index += 1;
        }

        if self.len < MAX_ARENA_DIRTY_RECTS {
            self.rects[self.len] = rect;
            self.len += 1;
        } else if self.len > 0 {
            self.rects[0] = self.rects[0].union(rect);
        }
    }

    pub(super) fn as_slice(&self) -> &[Rect] {
        &self.rects[..self.len]
    }
}

#[derive(Clone, Copy)]
pub(super) struct EnemyFrame {
    pub(super) active: bool,
    pub(super) x: i16,
    pub(super) y: i16,
    pub(super) size: i16,
}

impl EnemyFrame {
    pub(super) const fn empty() -> Self {
        Self {
            active: false,
            x: 0,
            y: 0,
            size: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ProjectileFrame {
    pub(super) active: bool,
    pub(super) x: i16,
    pub(super) y: i16,
    pub(super) tail_x: i16,
    pub(super) tail_y: i16,
}

impl ProjectileFrame {
    pub(super) const fn empty() -> Self {
        Self {
            active: false,
            x: 0,
            y: 0,
            tail_x: 0,
            tail_y: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ArenaFrame {
    pub(super) player_x: i16,
    pub(super) player_y: i16,
    pub(super) target_x: i16,
    pub(super) target_y: i16,
    pub(super) moving: bool,
    pub(super) nearest_enemy: Option<(i16, i16)>,
    pub(super) enemies: [EnemyFrame; MAX_ENEMIES],
    pub(super) projectiles: [ProjectileFrame; MAX_PROJECTILES],
}

impl ArenaFrame {
    pub(super) const fn empty() -> Self {
        Self {
            player_x: 0,
            player_y: 0,
            target_x: 0,
            target_y: 0,
            moving: false,
            nearest_enemy: None,
            enemies: [EnemyFrame::empty(); MAX_ENEMIES],
            projectiles: [ProjectileFrame::empty(); MAX_PROJECTILES],
        }
    }

    pub(super) fn collect_dirty_regions(&self, regions: &mut DirtyRegions) {
        regions.add(player_rect(self.player_x, self.player_y));
        if self.moving {
            regions.add(target_rect(self.target_x, self.target_y));
        } else {
            regions.add(nearest_indicator_rect(
                self.player_x,
                self.player_y,
                self.nearest_enemy,
            ));
        }

        for enemy in self.enemies {
            if enemy.active {
                regions.add(enemy_rect(enemy));
            }
        }

        for projectile in self.projectiles {
            if projectile.active {
                regions.add(projectile_rect(projectile));
            }
        }
    }
}

pub(super) fn arena_inner_rect() -> Rect {
    Rect {
        x: ARENA_INNER_X as i16,
        y: ARENA_INNER_Y as i16,
        w: ARENA_INNER_W as i16,
        h: ARENA_INNER_H as i16,
    }
}

pub(super) fn fill_rect_clipped(display: &mut Display, clip: Rect, target: Rect, color: u16) {
    let intersection = clip.intersect(target);
    if intersection.is_empty() {
        return;
    }
    display.fill_rect(
        intersection.x as u16,
        intersection.y as u16,
        intersection.w as u16,
        intersection.h as u16,
        color,
    );
}

pub(super) fn player_rect(px: i16, py: i16) -> Rect {
    Rect {
        x: px - 12,
        y: py - 12,
        w: 24,
        h: 24,
    }
}

pub(super) fn target_rect(tx: i16, ty: i16) -> Rect {
    Rect {
        x: tx - 6,
        y: ty - 6,
        w: 12,
        h: 12,
    }
}

pub(super) fn enemy_rect(enemy: EnemyFrame) -> Rect {
    let half = enemy.size / 2;
    Rect {
        x: enemy.x - half - 2,
        y: enemy.y - half - 6,
        w: enemy.size + 4,
        h: enemy.size + 11,
    }
}

pub(super) fn projectile_rect(projectile: ProjectileFrame) -> Rect {
    let left = projectile.x.min(projectile.tail_x) - 2;
    let top = projectile.y.min(projectile.tail_y) - 2;
    let right = projectile.x.max(projectile.tail_x) + 2;
    let bottom = projectile.y.max(projectile.tail_y) + 2;
    Rect {
        x: left,
        y: top,
        w: right - left,
        h: bottom - top,
    }
}

pub(super) fn nearest_indicator_rect(px: i16, py: i16, enemy: Option<(i16, i16)>) -> Rect {
    let Some((ex, ey)) = enemy else {
        return Rect::empty();
    };

    let target = Rect {
        x: ex - 9,
        y: ey - 9,
        w: 18,
        h: 18,
    };
    let horizontal = Rect {
        x: px.min(ex),
        y: py,
        w: px.max(ex) - px.min(ex) + 1,
        h: 1,
    };
    let vertical = Rect {
        x: ex,
        y: py.min(ey),
        w: 1,
        h: py.max(ey) - py.min(ey) + 1,
    };
    target.union(horizontal).union(vertical)
}
