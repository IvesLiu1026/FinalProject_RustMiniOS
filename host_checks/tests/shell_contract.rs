use minios_host_checks::app_registry::AppId;
use minios_host_checks::shell_contract::{
    app_route, reduce_dungeon_host, reduce_game_center, reduce_map_select, DungeonHostIntent,
    DungeonHostSignals, GameCenterIntent, HostedAppNavigation, MapSelectIntent, Screen,
};

#[test]
fn app_routes_match_shell_entry_and_exit_expectations() {
    let cases = [
        (AppId::Album, Screen::Album, Screen::Home, true),
        (AppId::GameCenter, Screen::GameCenter, Screen::Home, false),
        (AppId::Paint, Screen::Paint, Screen::Home, true),
        (AppId::Settings, Screen::Settings, Screen::Home, false),
        (AppId::DungeonCore, Screen::MapSelect, Screen::GameCenter, true),
        (AppId::AutoBattle, Screen::AutoBattle, Screen::GameCenter, true),
        (AppId::TapRush, Screen::TapRush, Screen::GameCenter, true),
        (AppId::PseudoRacer, Screen::PseudoRacer, Screen::GameCenter, true),
        (AppId::GraphicsLab, Screen::GraphicsLab, Screen::GameCenter, true),
    ];

    for (app_id, entry_screen, exit_screen, track_performance) in cases {
        let route = app_route(app_id);
        assert_eq!(route.entry_screen, entry_screen);
        assert_eq!(route.exit_screen, exit_screen);
        assert_eq!(route.track_performance, track_performance);
        assert!(!route.pipeline_label.is_empty());
        assert!(!route.cadence_label.is_empty());
    }
}

#[test]
fn screen_labels_stay_bilingual_and_non_empty() {
    for screen in [
        Screen::Home,
        Screen::Album,
        Screen::GameCenter,
        Screen::MapSelect,
        Screen::Settings,
        Screen::PerformanceConsole,
        Screen::Benchmark,
        Screen::About,
        Screen::Diagnostics,
        Screen::SafeMode,
        Screen::TouchCalibrate,
        Screen::ControlRoom,
        Screen::DungeonCore,
        Screen::AutoBattle,
        Screen::Paint,
        Screen::TapRush,
        Screen::PseudoRacer,
        Screen::GraphicsLab,
    ] {
        assert!(!screen.label(false).is_empty());
        assert!(!screen.label(true).is_empty());
    }
}

#[test]
fn map_select_wraps_with_prev_and_next_intents() {
    let prev = reduce_map_select(0, 3, MapSelectIntent::Previous);
    assert_eq!(prev.next_map_index, 2);
    assert!(prev.dirty);
    assert_eq!(prev.navigation, HostedAppNavigation::Stay);
    assert!(!prev.prepare_dungeon_launch);

    let next = reduce_map_select(2, 3, MapSelectIntent::Next);
    assert_eq!(next.next_map_index, 0);
    assert!(next.dirty);
    assert_eq!(next.navigation, HostedAppNavigation::Stay);
    assert!(!next.prepare_dungeon_launch);
}

#[test]
fn map_select_selects_new_map_before_launching() {
    let select_new = reduce_map_select(0, 4, MapSelectIntent::SelectMap(2));
    assert_eq!(select_new.next_map_index, 2);
    assert!(select_new.dirty);
    assert_eq!(select_new.navigation, HostedAppNavigation::Stay);
    assert!(!select_new.prepare_dungeon_launch);

    let select_current = reduce_map_select(2, 4, MapSelectIntent::SelectMap(2));
    assert_eq!(select_current.next_map_index, 2);
    assert!(select_current.dirty);
    assert_eq!(
        select_current.navigation,
        HostedAppNavigation::Switch(Screen::DungeonCore)
    );
    assert!(select_current.prepare_dungeon_launch);
}

#[test]
fn map_select_launch_and_exit_use_hosted_navigation_contract() {
    let launch = reduce_map_select(1, 3, MapSelectIntent::LaunchCurrent);
    assert_eq!(launch.next_map_index, 1);
    assert!(launch.dirty);
    assert_eq!(launch.navigation, HostedAppNavigation::Switch(Screen::DungeonCore));
    assert!(launch.prepare_dungeon_launch);

    let exit = reduce_map_select(1, 3, MapSelectIntent::ExitToGameCenter);
    assert_eq!(exit.next_map_index, 1);
    assert!(exit.dirty);
    assert_eq!(
        exit.navigation,
        HostedAppNavigation::Exit {
            app_id: AppId::DungeonCore,
            persist_state: false,
        }
    );
    assert!(!exit.prepare_dungeon_launch);
}

#[test]
fn game_center_wraps_selection_and_launches_current_app() {
    let apps = [
        AppId::DungeonCore,
        AppId::AutoBattle,
        AppId::PseudoRacer,
        AppId::GraphicsLab,
    ];

    let prev = reduce_game_center(0, &apps, GameCenterIntent::Previous);
    assert_eq!(prev.next_selected, 3);
    assert!(prev.dirty);
    assert_eq!(prev.navigation, HostedAppNavigation::Stay);

    let next = reduce_game_center(3, &apps, GameCenterIntent::Next);
    assert_eq!(next.next_selected, 0);
    assert!(next.dirty);
    assert_eq!(next.navigation, HostedAppNavigation::Stay);

    let launch = reduce_game_center(2, &apps, GameCenterIntent::LaunchCurrent);
    assert_eq!(launch.next_selected, 2);
    assert!(launch.dirty);
    assert_eq!(launch.navigation, HostedAppNavigation::Launch(AppId::PseudoRacer));
}

#[test]
fn game_center_select_slot_reuses_hosted_navigation_contract() {
    let apps = [
        AppId::DungeonCore,
        AppId::AutoBattle,
        AppId::PseudoRacer,
        AppId::GraphicsLab,
    ];

    let select_new = reduce_game_center(0, &apps, GameCenterIntent::SelectSlot(3));
    assert_eq!(select_new.next_selected, 3);
    assert!(select_new.dirty);
    assert_eq!(select_new.navigation, HostedAppNavigation::Stay);

    let select_current = reduce_game_center(3, &apps, GameCenterIntent::SelectSlot(3));
    assert_eq!(select_current.next_selected, 3);
    assert!(select_current.dirty);
    assert_eq!(
        select_current.navigation,
        HostedAppNavigation::Launch(AppId::GraphicsLab)
    );

    let exit = reduce_game_center(1, &apps, GameCenterIntent::ExitHome);
    assert_eq!(exit.next_selected, 1);
    assert!(exit.dirty);
    assert_eq!(
        exit.navigation,
        HostedAppNavigation::Exit {
            app_id: AppId::GameCenter,
            persist_state: false,
        }
    );
}

#[test]
fn dungeon_host_stay_marks_dirty_when_runtime_signals_change() {
    let outcome = reduce_dungeon_host(
        DungeonHostIntent::Stay,
        DungeonHostSignals {
            animation_active: false,
            redraw_requested: true,
            k0_just_pressed: false,
            k1_just_pressed: false,
            wkup_just_pressed: false,
            home_chord: false,
            touch_just_pressed: false,
            touch_just_released: false,
        },
    );
    assert!(outcome.dirty);
    assert_eq!(outcome.navigation, HostedAppNavigation::Stay);

    let idle = reduce_dungeon_host(
        DungeonHostIntent::Stay,
        DungeonHostSignals {
            animation_active: false,
            redraw_requested: false,
            k0_just_pressed: false,
            k1_just_pressed: false,
            wkup_just_pressed: false,
            home_chord: false,
            touch_just_pressed: false,
            touch_just_released: false,
        },
    );
    assert!(!idle.dirty);
    assert_eq!(idle.navigation, HostedAppNavigation::Stay);
}

#[test]
fn dungeon_host_navigation_uses_shared_shell_contract() {
    let exit = reduce_dungeon_host(
        DungeonHostIntent::ExitToGameCenter,
        DungeonHostSignals {
            animation_active: false,
            redraw_requested: false,
            k0_just_pressed: false,
            k1_just_pressed: false,
            wkup_just_pressed: false,
            home_chord: false,
            touch_just_pressed: false,
            touch_just_released: false,
        },
    );
    assert!(exit.dirty);
    assert_eq!(
        exit.navigation,
        HostedAppNavigation::Exit {
            app_id: AppId::DungeonCore,
            persist_state: false,
        }
    );

    let open_map = reduce_dungeon_host(
        DungeonHostIntent::OpenMapSelect,
        DungeonHostSignals {
            animation_active: false,
            redraw_requested: false,
            k0_just_pressed: false,
            k1_just_pressed: false,
            wkup_just_pressed: false,
            home_chord: false,
            touch_just_pressed: false,
            touch_just_released: false,
        },
    );
    assert!(open_map.dirty);
    assert_eq!(
        open_map.navigation,
        HostedAppNavigation::Switch(Screen::MapSelect)
    );
}
