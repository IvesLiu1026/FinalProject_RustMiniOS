use std::path::PathBuf;

use minios_host_checks::app_registry::{self, AppId};
use minios_host_checks::display::ThemeMode;
use minios_host_checks::dungeon::RenderStrategy;
use minios_host_checks::media_manifest;
use minios_host_checks::storage::{
    PersistedAppData, PersistedPseudoRacerData, PersistedState, PersistedStationHunterData,
    PersistedSystemSettings,
};
use minios_host_checks::storage_codec;
use minios_host_checks::touch::TouchCalibration;

fn sample_state() -> PersistedState {
    let mut paint_pixels = [0u8; storage_codec::PAINT_STORAGE_BYTES];
    paint_pixels[0] = 1;
    paint_pixels[17] = 3;
    paint_pixels[storage_codec::PAINT_STORAGE_BYTES - 1] = 9;

    PersistedState {
        system: PersistedSystemSettings {
            theme: ThemeMode::Light,
            language_zh: true,
            render_strategy: RenderStrategy::Performance,
            touch_ready: true,
            touch_calibration: TouchCalibration {
                x_min: 101,
                x_max: 4010,
                y_min: 202,
                y_max: 3901,
                swap_xy: false,
                invert_x: true,
                invert_y: true,
                valid: true,
                affine: true,
                ax: 0.25,
                bx: -0.5,
                cx: 12.0,
                ay: 1.25,
                by: 0.75,
                cy: -4.0,
            },
        },
        apps: PersistedAppData {
            recent_app: Some(AppId::AutoBattle),
            album_motion_tab: true,
            album_still_index: 2,
            album_motion_index: 1,
            album_playing: false,
            paint_selected_color: 7,
            paint_pixels,
            station_hunter: PersistedStationHunterData {
                selected_stage: 3,
                player_level: 4,
                player_xp: 28,
                upgrade_points: 2,
                unlocked_stage: 4,
                base_attack: 1,
                base_hp: 2,
                base_fire_rate: 1,
                base_move_speed: 3,
                best_kills: 42,
                stage_best_wave: [30, 18, 9, 0, 0],
                stage_best_kills: [44, 36, 12, 0, 0],
                stage_clear_count: [1, 1, 0, 0, 0],
            },
            pseudo_racer: PersistedPseudoRacerData {
                selected_track: 2,
                best_time_ms: [14_230, 17_640, 19_880],
            },
            tap_rush_best_score: 99,
        },
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("host_checks lives inside repo")
        .to_path_buf()
}

#[test]
fn storage_round_trip_preserves_persisted_state() {
    let state = sample_state();
    let bytes = storage_codec::encode(&state);
    let decoded = storage_codec::decode(&bytes).expect("encoded state should decode");

    assert_eq!(decoded, state);
}

#[test]
fn storage_detects_checksum_corruption() {
    let state = sample_state();
    let mut bytes = storage_codec::encode(&state);
    bytes[64] ^= 0x5A;

    let status = storage_codec::inspect_bytes(&bytes);
    assert!(status.found_magic);
    assert!(!status.valid_record);
    assert!(!status.checksum_ok);
    assert!(storage_codec::decode(&bytes).is_none());
}

#[test]
fn storage_defaults_report_no_app_saves() {
    let default_apps = storage_codec::default_app_data();

    assert_eq!(default_apps.recent_app, None);
    assert_eq!(default_apps.paint_selected_color, 1);
    assert!(default_apps.album_playing);
    assert_eq!(default_apps.station_hunter.best_kills, 0);
    assert_eq!(default_apps.station_hunter.player_level, 1);
    assert_eq!(default_apps.station_hunter.unlocked_stage, 1);
    assert_eq!(default_apps.pseudo_racer.selected_track, 0);
    assert_eq!(default_apps.pseudo_racer.best_time_ms, [0; 3]);
    assert_eq!(default_apps.tap_rush_best_score, 0);
}

#[test]
fn app_registry_slots_and_groupings_are_consistent() {
    assert_eq!(app_registry::home_apps().len(), 4);
    assert_eq!(app_registry::game_center_apps().len(), 4);

    assert_eq!(app_registry::home_slot_for_app(AppId::Album), 0);
    assert_eq!(app_registry::home_slot_for_app(AppId::GameCenter), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::DungeonCore), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::AutoBattle), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::TapRush), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::PseudoRacer), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::GraphicsLab), 1);
    assert_eq!(app_registry::home_slot_for_app(AppId::Paint), 2);
    assert_eq!(app_registry::home_slot_for_app(AppId::Settings), 3);

    assert_eq!(app_registry::game_center_slot_for_app(AppId::DungeonCore), Some(0));
    assert_eq!(app_registry::game_center_slot_for_app(AppId::AutoBattle), Some(1));
    assert_eq!(app_registry::game_center_slot_for_app(AppId::PseudoRacer), Some(2));
    assert_eq!(app_registry::game_center_slot_for_app(AppId::GraphicsLab), Some(3));
    assert_eq!(app_registry::game_center_slot_for_app(AppId::TapRush), None);
    assert_eq!(app_registry::game_center_slot_for_app(AppId::Album), None);
}

#[test]
fn app_registry_descriptors_stay_bilingual_and_non_empty() {
    for app_id in [
        AppId::Album,
        AppId::GameCenter,
        AppId::Paint,
        AppId::Settings,
        AppId::DungeonCore,
        AppId::AutoBattle,
        AppId::TapRush,
        AppId::PseudoRacer,
        AppId::GraphicsLab,
    ] {
        let descriptor = app_registry::descriptor(app_id);
        assert!(!descriptor.title(false).is_empty());
        assert!(!descriptor.title(true).is_empty());
        assert!(!descriptor.subtitle(false).is_empty());
        assert!(!descriptor.subtitle(true).is_empty());
    }
}

#[test]
fn media_manifests_match_firmware_assets() {
    let repo_root = repo_root();
    let manifests_root = repo_root.join("assets/test_media/converted/manifests");
    let stills_root = repo_root.join("assets/test_media/converted/firmware/stills");
    let motion_root = repo_root.join("assets/test_media/converted/firmware/motion");

    let stills = media_manifest::collect_stills(&stills_root, &manifests_root);
    let clips = media_manifest::collect_motion_clips(&motion_root, &manifests_root);

    assert!(!stills.is_empty(), "expected at least one still asset");
    assert!(!clips.is_empty(), "expected at least one motion clip");

    for still in stills {
        assert!(still.path.is_file(), "missing still asset {:?}", still.path);
        assert!(still.width > 0 && still.height > 0);
        assert!(still.scale > 0);
        assert!(!still.label.is_empty());
        assert!(!still.key.is_empty());
    }

    for clip in clips {
        assert!(clip.width > 0 && clip.height > 0);
        assert!(clip.scale > 0);
        assert!(clip.frame_delay_ms > 0);
        assert!(!clip.frames.is_empty(), "clip {} has no frames", clip.key);
        assert!(!clip.symbol.is_empty());
        assert!(!clip.label.is_empty());
        if let Some(frames_kept) = clip.manifest.get("frames_kept").and_then(|v| v.parse::<usize>().ok()) {
            assert_eq!(clip.frames.len(), frames_kept, "clip {} frame count mismatch", clip.key);
        }
        for frame in &clip.frames {
            assert!(frame.is_file(), "missing motion frame {:?}", frame);
        }
    }
}
