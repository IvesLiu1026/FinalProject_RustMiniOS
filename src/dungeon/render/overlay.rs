use super::super::*;

pub(super) fn draw_overlay(
    display: &mut Display,
    ui: &crate::display::Palette,
    title: &str,
    subtitle: &str,
    retry_label: &str,
    map_label: &str,
    accent: u16,
) {
    let glow = crate::display::color::mix(accent, ui.white, 84);
    const PANEL_Y: u16 = 80;
    display.fill_rect(
        44,
        PANEL_Y + 8,
        232,
        88,
        crate::display::color::mix(ui.shadow, accent, 26),
    );
    display.panel(52, PANEL_Y, 216, 88, ui.panel_alt, accent);
    display.stroke_rect(56, PANEL_Y + 4, 208, 80, 1, glow);
    display.centered_text(160, PANEL_Y + 14, title, ui.text, ui.panel_alt, 2);
    display.centered_text(160, PANEL_Y + 38, subtitle, ui.text_muted, ui.panel_alt, 1);

    display.panel(
        OVERLAY_RETRY_X,
        OVERLAY_RETRY_Y,
        OVERLAY_RETRY_W,
        OVERLAY_RETRY_H,
        ui.panel,
        accent,
    );
    display.centered_text(
        OVERLAY_RETRY_X + OVERLAY_RETRY_W / 2,
        OVERLAY_RETRY_Y + 6,
        retry_label,
        ui.text,
        ui.panel,
        1,
    );
    display.panel(
        OVERLAY_MAPS_X,
        OVERLAY_MAPS_Y,
        OVERLAY_MAPS_W,
        OVERLAY_MAPS_H,
        ui.panel,
        ui.cyan,
    );
    display.centered_text(
        OVERLAY_MAPS_X + OVERLAY_MAPS_W / 2,
        OVERLAY_MAPS_Y + 6,
        map_label,
        ui.text,
        ui.panel,
        1,
    );
}

pub(super) fn draw_intro_overlay(
    display: &mut Display,
    ui: &crate::display::Palette,
    map: &MapDef,
    zh_mode: bool,
) {
    let accent = match map.spawn_angle > 0.0 {
        true => ui.cyan,
        false => ui.orange,
    };
    let band = crate::display::color::mix(ui.panel_alt, accent, 78);
    display.fill_rect(
        34,
        66,
        252,
        80,
        crate::display::color::mix(ui.shadow, accent, 28),
    );
    display.panel(42, 58, 236, 74, ui.panel_alt, accent);
    display.fill_rect(54, 96, 212, 8, band);
    display.centered_text(
        160,
        72,
        if zh_mode {
            "任務部署中"
        } else {
            "MISSION DEPLOY"
        },
        ui.text,
        ui.panel_alt,
        2,
    );
    display.centered_text(
        160,
        98,
        if zh_mode { map.name_zh } else { map.name_en },
        ui.white,
        ui.panel_alt,
        2,
    );
    display.centered_text(
        160,
        120,
        if zh_mode {
            "點擊或按 K1 可略過"
        } else {
            "TAP OR PRESS K1 TO SKIP"
        },
        ui.text_muted,
        ui.panel_alt,
        1,
    );

    display.fill_rect(
        58,
        92,
        198,
        2,
        crate::display::color::mix(band, ui.white, 64),
    );
}
