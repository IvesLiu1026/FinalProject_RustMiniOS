use super::*;

impl MiniOs {
    pub(super) fn return_from_touch_calibration(&mut self) {
        self.switch_screen(self.touch_return_screen);
    }

    pub(super) fn enter_touch_calibration(&mut self, return_screen: Screen) {
        self.calibration_step = 0;
        self.calibration_raw_x = [0; 5];
        self.calibration_raw_y = [0; 5];
        self.touch_return_screen = return_screen;
        self.switch_screen(Screen::TouchCalibrate);
    }

    pub(super) fn commit_touch_calibration(&mut self, touch_driver: &mut Touch) -> bool {
        let tl = (self.calibration_raw_x[0], self.calibration_raw_y[0]);
        let tr = (self.calibration_raw_x[1], self.calibration_raw_y[1]);
        let center = (self.calibration_raw_x[2], self.calibration_raw_y[2]);
        let br = (self.calibration_raw_x[3], self.calibration_raw_y[3]);
        let bl = (self.calibration_raw_x[4], self.calibration_raw_y[4]);

        let x_span = abs_diff_u16(tl.0, tr.0)
            .max(abs_diff_u16(bl.0, br.0))
            .max(abs_diff_u16(tl.0, bl.0));
        let y_span = abs_diff_u16(tl.1, tr.1)
            .max(abs_diff_u16(bl.1, br.1))
            .max(abs_diff_u16(tl.1, bl.1));
        if x_span < 300 || y_span < 300 {
            return false;
        }

        let targets = [
            (28.0f32, 40.0f32),
            (292.0f32, 40.0f32),
            (160.0f32, 122.0f32),
            (292.0f32, 210.0f32),
            (28.0f32, 210.0f32),
        ];

        let calibration_points = [
            (tl.0 as f32, tl.1 as f32, targets[0].0, targets[0].1),
            (tr.0 as f32, tr.1 as f32, targets[1].0, targets[1].1),
            (center.0 as f32, center.1 as f32, targets[2].0, targets[2].1),
            (br.0 as f32, br.1 as f32, targets[3].0, targets[3].1),
            (bl.0 as f32, bl.1 as f32, targets[4].0, targets[4].1),
        ];

        let (ax, bx, cx) = match solve_affine_least_squares(&calibration_points, true) {
            Some(v) => v,
            None => return false,
        };
        let (ay, by, cy) = match solve_affine_least_squares(&calibration_points, false) {
            Some(v) => v,
            None => return false,
        };

        let mut worst_error = 0.0f32;
        for (raw_x, raw_y, target_x, target_y) in calibration_points {
            let px = ax * raw_x + bx * raw_y + cx;
            let py = ay * raw_x + by * raw_y + cy;
            let ex = (px - target_x).abs();
            let ey = (py - target_y).abs();
            worst_error = worst_error.max(ex.max(ey));
        }
        if worst_error > 24.0 {
            return false;
        }

        let calibration = TouchCalibration {
            x_min: 0,
            x_max: 4095,
            y_min: 0,
            y_max: 4095,
            swap_xy: false,
            invert_x: false,
            invert_y: false,
            valid: true,
            affine: true,
            ax,
            bx,
            cx,
            ay,
            by,
            cy,
        };

        touch_driver.set_calibration(calibration);
        self.touch_calibration = calibration;
        self.touch_ready = true;
        let _ = self.save_storage();
        true
    }
}

fn abs_diff_u16(a: u16, b: u16) -> u16 {
    a.abs_diff(b)
}

fn solve_affine_least_squares(
    points: &[(f32, f32, f32, f32)],
    solve_x: bool,
) -> Option<(f32, f32, f32)> {
    let mut s_xx = 0.0f32;
    let mut s_xy = 0.0f32;
    let mut s_yy = 0.0f32;
    let mut s_x = 0.0f32;
    let mut s_y = 0.0f32;
    let mut s_u = 0.0f32;
    let mut s_xu = 0.0f32;
    let mut s_yu = 0.0f32;
    let n = points.len() as f32;

    for &(raw_x, raw_y, target_x, target_y) in points {
        let u = if solve_x { target_x } else { target_y };
        s_xx += raw_x * raw_x;
        s_xy += raw_x * raw_y;
        s_yy += raw_y * raw_y;
        s_x += raw_x;
        s_y += raw_y;
        s_u += u;
        s_xu += raw_x * u;
        s_yu += raw_y * u;
    }

    let det = det3(s_xx, s_xy, s_x, s_xy, s_yy, s_y, s_x, s_y, n);
    if det.abs() < 1.0e-6 {
        return None;
    }

    let det_a = det3(s_xu, s_xy, s_x, s_yu, s_yy, s_y, s_u, s_y, n);
    let det_b = det3(s_xx, s_xu, s_x, s_xy, s_yu, s_y, s_x, s_u, n);
    let det_c = det3(s_xx, s_xy, s_xu, s_xy, s_yy, s_yu, s_x, s_y, s_u);

    Some((det_a / det, det_b / det, det_c / det))
}

fn det3(
    a11: f32,
    a12: f32,
    a13: f32,
    a21: f32,
    a22: f32,
    a23: f32,
    a31: f32,
    a32: f32,
    a33: f32,
) -> f32 {
    a11 * (a22 * a33 - a23 * a32) - a12 * (a21 * a33 - a23 * a31) + a13 * (a21 * a32 - a22 * a31)
}
