use super::super::*;

pub(super) fn buffer_fill_rect(
    buffer: &mut [u16],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u16,
) {
    let x_end = (x + width).min(VIEW_W);
    let y_end = (y + height).min(VIEW_H_USIZE);
    for py in y..y_end {
        let row = py * VIEW_W;
        for px in x..x_end {
            buffer[row + px] = color;
        }
    }
}

pub(super) fn buffer_blend_rect(
    buffer: &mut [u16],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u16,
    alpha: u8,
) {
    let x_end = (x + width).min(VIEW_W);
    let y_end = (y + height).min(VIEW_H_USIZE);
    for py in y..y_end {
        let row = py * VIEW_W;
        for px in x..x_end {
            let idx = row + px;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

pub(super) fn buffer_stroke_rect(
    buffer: &mut [u16],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    thickness: usize,
    color: u16,
    alpha: u8,
) {
    buffer_blend_rect(buffer, x, y, width, thickness, color, alpha);
    buffer_blend_rect(
        buffer,
        x,
        y.saturating_add(height.saturating_sub(thickness)),
        width,
        thickness,
        color,
        alpha,
    );
    buffer_blend_rect(buffer, x, y, thickness, height, color, alpha);
    buffer_blend_rect(
        buffer,
        x.saturating_add(width.saturating_sub(thickness)),
        y,
        thickness,
        height,
        color,
        alpha,
    );
}

pub(super) fn buffer_blend_circle(
    buffer: &mut [u16],
    center_x: usize,
    center_y: usize,
    radius: i32,
    color: u16,
    alpha: u8,
) {
    if radius <= 0 {
        return;
    }

    let cx = center_x as i32;
    let cy = center_y as i32;

    for dy in -radius..=radius {
        let y = cy + dy;
        if !(0..VIEW_H as i32).contains(&y) {
            continue;
        }

        let dx = buffer_circle_dx(radius, dy);
        let start_x = (cx - dx).max(0);
        let end_x = (cx + dx).min(VIEW_W as i32 - 1);
        let row = y as usize * VIEW_W;
        for px in start_x..=end_x {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

pub(super) fn buffer_stroke_circle(
    buffer: &mut [u16],
    center_x: usize,
    center_y: usize,
    radius: i32,
    thickness: i32,
    color: u16,
    alpha: u8,
) {
    let outer = radius.max(0);
    let inner = (outer - thickness).max(0);
    let cx = center_x as i32;
    let cy = center_y as i32;

    for dy in -outer..=outer {
        let y = cy + dy;
        if !(0..VIEW_H as i32).contains(&y) {
            continue;
        }

        let outer_dx = buffer_circle_dx(outer, dy);
        let inner_dx = if dy.abs() <= inner {
            buffer_circle_dx(inner, dy)
        } else {
            -1
        };
        let row = y as usize * VIEW_W;
        let left_outer = (cx - outer_dx).max(0);
        let right_outer = (cx + outer_dx).min(VIEW_W as i32 - 1);

        if inner_dx < 0 {
            for px in left_outer..=right_outer {
                let idx = row + px as usize;
                buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
            }
            continue;
        }

        let left_inner = (cx - inner_dx).max(0);
        let right_inner = (cx + inner_dx).min(VIEW_W as i32 - 1);

        for px in left_outer..left_inner {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
        for px in (right_inner + 1)..=right_outer {
            let idx = row + px as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }
    }
}

pub(super) fn buffer_blend_line(
    buffer: &mut [u16],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u16,
    alpha: u8,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if (0..VIEW_W as i32).contains(&x0) && (0..VIEW_H as i32).contains(&y0) {
            let idx = y0 as usize * VIEW_W + x0 as usize;
            buffer[idx] = crate::display::color::mix(buffer[idx], color, alpha);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn buffer_circle_dx(radius: i32, dy: i32) -> i32 {
    let rr = radius * radius;
    let yy = dy * dy;
    let mut dx = radius;
    while dx > 0 && (dx * dx + yy) > rr {
        dx -= 1;
    }
    dx
}
