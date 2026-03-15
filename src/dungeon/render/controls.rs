use super::super::math::*;
use super::super::*;
use super::primitives::{buffer_blend_circle, buffer_blend_rect, buffer_stroke_circle};

pub(super) fn render_touch_controls(
    buffer: &mut [u16],
    touch: &TouchState,
    mode: TouchMode,
    ui: &crate::display::Palette,
) {
    let control_fill = crate::display::color::mix(ui.panel_alt, ui.cyan, 54);
    let control_core = crate::display::color::mix(ui.canvas, ui.panel, 112);
    let fire_fill = crate::display::color::mix(ui.panel_alt, ui.amber, 58);
    let fire_core = crate::display::color::mix(ui.canvas, ui.panel, 112);
    let control_cx = view_px(CONTROL_CENTER_X);
    let control_cy = view_screen_y(CONTROL_CENTER_Y);
    let fire_cx = view_px(FIRE_CENTER_X);
    let fire_cy = view_screen_y(FIRE_CENTER_Y);
    let control_ring = view_span(CONTROL_RING_RADIUS).max(2) as i32;
    let control_base = view_span(CONTROL_BASE_RADIUS).max(2) as i32;
    let control_input = view_span(CONTROL_INPUT_RADIUS as u16).max(2) as i32;
    let control_knob = view_span(CONTROL_KNOB_RADIUS as u16).max(2) as i32;
    let fire_ring = view_span(FIRE_RING_RADIUS).max(2) as i32;
    let fire_base = view_span(FIRE_BASE_RADIUS).max(2) as i32;
    let fire_knob = view_span(FIRE_KNOB_RADIUS as u16).max(2) as i32;

    buffer_blend_circle(
        buffer,
        control_cx + view_span(2),
        control_cy + view_span(2),
        control_ring,
        crate::display::color::mix(ui.shadow, ui.indigo, 40),
        112,
    );
    buffer_blend_circle(
        buffer,
        control_cx,
        control_cy,
        control_ring,
        control_fill,
        120,
    );
    buffer_blend_circle(
        buffer,
        control_cx,
        control_cy,
        control_base,
        control_core,
        130,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        control_ring,
        view_span(2) as i32,
        ui.cyan,
        220,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        control_base,
        view_span(1) as i32,
        crate::display::color::mix(ui.steel, ui.white, 62),
        200,
    );
    buffer_stroke_circle(
        buffer,
        control_cx,
        control_cy,
        control_input,
        view_span(1) as i32,
        crate::display::color::mix(ui.steel, ui.cyan, 96),
        180,
    );
    buffer_blend_rect(
        buffer,
        control_cx.saturating_sub(view_span(1)),
        control_cy.saturating_sub(view_span(12)),
        view_span(2),
        view_span(24),
        ui.steel,
        150,
    );
    buffer_blend_rect(
        buffer,
        control_cx.saturating_sub(view_span(12)),
        control_cy.saturating_sub(view_span(1)),
        view_span(24),
        view_span(2),
        ui.steel,
        150,
    );

    buffer_blend_circle(
        buffer,
        fire_cx + view_span(2),
        fire_cy + view_span(2),
        fire_ring,
        crate::display::color::mix(ui.shadow, ui.orange, 30),
        104,
    );
    buffer_blend_circle(buffer, fire_cx, fire_cy, fire_ring, fire_fill, 122);
    buffer_blend_circle(buffer, fire_cx, fire_cy, fire_base, fire_core, 132);
    buffer_stroke_circle(
        buffer,
        fire_cx,
        fire_cy,
        fire_ring,
        view_span(2) as i32,
        ui.amber,
        220,
    );
    buffer_stroke_circle(
        buffer,
        fire_cx,
        fire_cy,
        fire_base,
        view_span(1) as i32,
        crate::display::color::mix(ui.steel, ui.white, 72),
        200,
    );

    let (control_dx, control_dy) = if touch.active && mode == TouchMode::Control {
        clamp_circle_delta(
            touch.x as f32 - CONTROL_CENTER_X as f32,
            touch.y as f32 - CONTROL_CENTER_Y as f32,
            CONTROL_INPUT_RADIUS,
        )
    } else {
        (0.0, 0.0)
    };
    let control_knob_x = round_to_usize(view_f(CONTROL_CENTER_X as f32 + control_dx));
    let control_knob_y =
        round_to_usize(view_f(CONTROL_CENTER_Y as f32 + control_dy - VIEW_Y as f32));
    buffer_blend_circle(
        buffer,
        control_knob_x + view_span(1),
        control_knob_y + view_span(1),
        control_knob + view_span(1) as i32,
        crate::display::color::mix(ui.shadow, ui.cyan, 56),
        145,
    );
    buffer_blend_circle(
        buffer,
        control_knob_x,
        control_knob_y,
        control_knob,
        crate::display::color::mix(ui.panel_alt, ui.cyan, 142),
        220,
    );
    buffer_stroke_circle(
        buffer,
        control_knob_x,
        control_knob_y,
        control_knob,
        view_span(2) as i32,
        ui.white,
        220,
    );

    let (fire_dx, fire_dy) = if touch.active && mode == TouchMode::Fire {
        clamp_circle_delta(
            touch.x as f32 - FIRE_CENTER_X as f32,
            touch.y as f32 - FIRE_CENTER_Y as f32,
            FIRE_INPUT_RADIUS,
        )
    } else {
        (0.0, 0.0)
    };
    let fire_hot = touch.active && mode == TouchMode::Fire;
    let fire_knob_x = round_to_usize(view_f(FIRE_CENTER_X as f32 + fire_dx));
    let fire_knob_y = round_to_usize(view_f(FIRE_CENTER_Y as f32 + fire_dy - VIEW_Y as f32));
    buffer_blend_circle(
        buffer,
        fire_knob_x + view_span(1),
        fire_knob_y + view_span(1),
        fire_knob + view_span(1) as i32,
        crate::display::color::mix(ui.shadow, ui.orange, 64),
        152,
    );
    buffer_blend_circle(
        buffer,
        fire_knob_x,
        fire_knob_y,
        fire_knob,
        if fire_hot {
            crate::display::color::mix(ui.orange, ui.white, 140)
        } else {
            crate::display::color::mix(ui.panel_alt, ui.amber, 138)
        },
        if fire_hot { 236 } else { 214 },
    );
    buffer_stroke_circle(
        buffer,
        fire_knob_x,
        fire_knob_y,
        fire_knob,
        view_span(2) as i32,
        ui.white,
        220,
    );

    let glyph = if fire_hot {
        crate::display::color::mix(ui.orange, ui.white, 180)
    } else {
        crate::display::color::mix(ui.amber, ui.white, 90)
    };
    buffer_blend_rect(
        buffer,
        fire_cx.saturating_sub(view_span(1)),
        fire_cy.saturating_sub(view_span(8)),
        view_span(2),
        view_span(16),
        glyph,
        220,
    );
    buffer_blend_rect(
        buffer,
        fire_cx.saturating_sub(view_span(8)),
        fire_cy.saturating_sub(view_span(1)),
        view_span(16),
        view_span(2),
        glyph,
        220,
    );
}
