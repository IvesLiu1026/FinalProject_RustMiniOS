use super::super::math::*;
use super::super::*;
use super::primitives::{buffer_blend_circle, buffer_fill_rect, buffer_stroke_circle};

pub(super) fn render_weapon(
    buffer: &mut [u16],
    dungeon: &DungeonApp,
    ui: &crate::display::Palette,
) {
    let recoil = if dungeon.muzzle_flash_ms > 0 {
        let flash_ms = dungeon.weapon.flash_ms().max(1) as u32;
        (((dungeon.muzzle_flash_ms as u32 * view_span(8) as u32) / flash_ms)
            .min(view_span(8) as u32)) as usize
    } else {
        0
    };
    let base_y = VIEW_H_USIZE
        .saturating_sub(view_span(26))
        .saturating_add(recoil);
    let center_x = VIEW_W / 2;
    let ring_radius = view_span(18).max(2) as i32;
    let ring_thickness = view_span(1) as i32;

    let accent = dungeon.weapon.accent(ui);
    buffer_blend_circle(
        buffer,
        center_x,
        base_y + view_span(14),
        ring_radius,
        crate::display::color::mix(ui.panel, accent, 54),
        118,
    );
    buffer_stroke_circle(
        buffer,
        center_x,
        base_y + view_span(14),
        ring_radius,
        ring_thickness,
        crate::display::color::mix(accent, ui.white, 40),
        170,
    );

    match dungeon.weapon {
        WeaponKind::Pulse => {
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(24)),
                base_y + view_span(10),
                view_span(48),
                view_span(12),
                ui.shadow,
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(18)),
                base_y + view_span(12),
                view_span(36),
                view_span(9),
                crate::display::color::mix(ui.panel_alt, ui.steel, 110),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(8)),
                base_y + view_span(5),
                view_span(16),
                view_span(8),
                crate::display::color::mix(ui.panel_alt, ui.white, 66),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(3)),
                base_y + view_span(1),
                view_span(6),
                view_span(6),
                ui.white,
            );
        }
        WeaponKind::Carbine => {
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(30)),
                base_y + view_span(11),
                view_span(60),
                view_span(10),
                ui.shadow,
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(26)),
                base_y + view_span(12),
                view_span(52),
                view_span(7),
                crate::display::color::mix(ui.panel_alt, ui.steel, 122),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(6)),
                base_y + view_span(6),
                view_span(20),
                view_span(6),
                crate::display::color::mix(ui.panel_alt, ui.white, 60),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(18)),
                base_y + view_span(16),
                view_span(10),
                view_span(5),
                crate::display::color::mix(ui.panel, ui.lime, 64),
            );
        }
        WeaponKind::Scatter => {
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(28)),
                base_y + view_span(10),
                view_span(56),
                view_span(14),
                ui.shadow,
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(16)),
                base_y + view_span(12),
                view_span(32),
                view_span(11),
                crate::display::color::mix(ui.panel_alt, ui.steel, 108),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(14)),
                base_y + view_span(4),
                view_span(28),
                view_span(10),
                crate::display::color::mix(ui.panel_alt, ui.white, 52),
            );
            buffer_fill_rect(
                buffer,
                center_x.saturating_sub(view_span(4)),
                base_y + view_span(1),
                view_span(8),
                view_span(5),
                ui.white,
            );
        }
    }

    for slot in 0..3usize {
        let slot_x = center_x.saturating_sub(view_span(12)) + slot * view_span(12);
        let active = slot
            == match dungeon.weapon {
                WeaponKind::Pulse => 0,
                WeaponKind::Carbine => 1,
                WeaponKind::Scatter => 2,
            };
        buffer_blend_circle(
            buffer,
            slot_x,
            base_y + view_span(30),
            view_span(4) as i32,
            if active {
                accent
            } else {
                crate::display::color::mix(ui.panel_alt, ui.steel, 100)
            },
            if active { 220 } else { 160 },
        );
    }

    if dungeon.muzzle_flash_ms > 0 {
        let flash = crate::display::color::mix(accent, ui.white, 178);
        let tip_y = base_y.saturating_sub(view_span(2));
        buffer_fill_rect(
            buffer,
            center_x.saturating_sub(view_span(2)),
            tip_y,
            view_span(4),
            view_span(4),
            flash,
        );
        buffer_fill_rect(
            buffer,
            center_x.saturating_sub(view_span(6)),
            tip_y + view_span(2),
            view_span(12),
            view_span(3),
            flash,
        );
        buffer_fill_rect(
            buffer,
            center_x.saturating_sub(view_span(10)),
            tip_y + view_span(4),
            view_span(20),
            view_span(2),
            accent,
        );
    }
}
