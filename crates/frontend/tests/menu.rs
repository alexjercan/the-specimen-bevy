use bevy::{
    audio::{PlaybackMode, PlaybackSettings},
    ecs::resource::IsResource,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
    window::CursorGrabMode,
};
use game_audio::Bus;
use game_flow::{GameState, PauseState};
use gameplay::{
    facility::{Door, DoorRef, LevelRoot, MainMenuAccentLight, Room},
    fuses::FusePanel,
    player::{Flashlight, PlayerController, PlayerControlsEnabled},
};

mod common;

use common::*;

#[derive(Debug, PartialEq, Eq)]
struct MenuScene {
    rooms: usize,
    roots: usize,
    doors: usize,
    links: usize,
    panels: usize,
    lights: usize,
    entities: usize,
}

fn menu_scene(app: &mut App) -> MenuScene {
    MenuScene {
        rooms: count::<With<Room>>(app),
        roots: count::<With<LevelRoot>>(app),
        doors: count::<With<Door>>(app),
        links: count::<With<DoorRef>>(app),
        panels: count::<With<FusePanel>>(app),
        lights: count::<With<PointLight>>(app),
        entities: entities(app),
    }
}

#[test]
fn loading_screen_shows_until_core_is_ready_then_main_menu() {
    let mut app = menu_app();
    assert_eq!(named_count(&mut app, "Loading screen"), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert!(named(&mut app, "Main menu").is_none());
    ready(&mut app);
    assert_eq!(named_count(&mut app, "Loading screen"), 0);
    assert_eq!(named_count(&mut app, "Main menu"), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<FusePanel>>(&mut app), 0);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
}

#[test]
fn main_menu_lists_play_settings_credits_achievements_then_quit() {
    let mut app = ready_menu_app();
    let menu = named(&mut app, "Main menu").unwrap();
    let container = child(&app, menu, 1);
    let buttons = child(&app, container, 0);
    let names: Vec<String> = app
        .world()
        .get::<Children>(buttons)
        .unwrap()
        .iter()
        .map(|entity| app.world().get::<Name>(entity).unwrap().as_str().to_owned())
        .collect();
    assert_eq!(
        names,
        vec![
            "Play button",
            "Settings button",
            "Credits button",
            "Achievements button",
            "Quit button",
        ]
    );
}

fn music_voices(app: &mut App) -> Vec<PlaybackSettings> {
    app.world_mut()
        .query::<(&Bus, &PlaybackSettings, &AudioPlayer)>()
        .iter(app.world())
        .filter_map(|(bus, settings, _)| (*bus == Bus::Music).then_some(*settings))
        .collect()
}

#[test]
fn menu_music_loops_once_and_stops_during_a_run() {
    let mut app = ready_menu_app();
    app.update();
    let music = music_voices(&mut app);
    assert_eq!(music.len(), 1);
    assert!(matches!(music[0].mode, PlaybackMode::Loop));
    for _ in 0..3 {
        app.update();
        assert_eq!(music_voices(&mut app).len(), 1);
    }
    press(&mut app, "Play button");
    assert!(music_voices(&mut app).is_empty());
    escape(&mut app);
    press(&mut app, "Main menu button");
    app.update();
    assert_eq!(music_voices(&mut app).len(), 1);
}

#[test]
fn play_pause_and_main_menu_cycle_without_duplicate_worlds() {
    let mut app = ready_menu_app();
    let menu = menu_scene(&mut app);
    let idle = voices(&mut app);

    press(&mut app, "Play button");
    assert_eq!(game_state(&app), GameState::Playing);
    assert!(named(&mut app, "Main menu").is_none());
    assert_eq!(count::<With<Camera2d>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    assert_eq!(
        count::<(With<IsResource>, With<DespawnOnExit<GameState>>)>(&mut app),
        0
    );
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    let playing = entities(&mut app);

    escape(&mut app);
    assert_eq!(pause_state(&app), PauseState::Paused);
    assert_eq!(named_count(&mut app, "Pause menu"), 1);
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    assert!(!app.world().resource::<PlayerControlsEnabled>().0);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    press(&mut app, "Resume button");
    assert_eq!(named_count(&mut app, "Pause menu"), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert!(app.world().resource::<PlayerControlsEnabled>().0);
    assert_eq!(pause_state(&app), PauseState::Running);

    escape(&mut app);
    escape(&mut app);
    assert_eq!(named_count(&mut app, "Pause menu"), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());

    escape(&mut app);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(named_count(&mut app, "Pause menu"), 0);
    assert_eq!(named_count(&mut app, "Main menu"), 1);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(menu_scene(&mut app), menu);
    assert_eq!(voices(&mut app), idle);

    press(&mut app, "Play button");
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(entities(&mut app), playing);
}

#[test]
fn looping_voices_play_during_a_run_and_stop_after_it() {
    let mut app = ready_menu_app_with_ambience();
    let idle = voices(&mut app);

    press(&mut app, "Play button");
    app.update();
    let playing = voices(&mut app);
    assert!(playing > idle);

    escape(&mut app);
    press(&mut app, "Main menu button");
    app.update();
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(voices(&mut app), idle);

    press(&mut app, "Play button");
    app.update();
    assert_eq!(voices(&mut app), playing);
    win(&mut app);
    assert_eq!(voices(&mut app), idle);
    press(&mut app, "Retry button");
    app.update();
    assert_eq!(voices(&mut app), playing);
    catch(&mut app);
    assert_eq!(game_state(&app), GameState::GameOver);
    app.update();
    assert_eq!(voices(&mut app), idle);
}

#[test]
fn menu_room_and_cameras_are_state_scoped() {
    let mut app = ready_menu_app();
    let world = app.world_mut();
    let room = world
        .query_filtered::<Entity, With<LevelRoot>>()
        .single(world)
        .expect("menu room root");
    assert!(world.get::<DespawnOnExit<GameState>>(room).is_some());
    let ui = world
        .query_filtered::<&Camera, (With<Camera2d>, With<IsDefaultUiCamera>)>()
        .single(world)
        .expect("UI camera");
    assert_eq!(ui.order, 1);
    let room_camera = world
        .query_filtered::<&Camera, With<Camera3d>>()
        .single(world)
        .expect("room camera");
    assert_eq!(room_camera.order, 0);
    let accent = world
        .query_filtered::<(&SpotLight, &ChildOf), With<MainMenuAccentLight>>()
        .single(world)
        .expect("tagged menu accent");
    assert_eq!(accent.1.parent(), room);
    assert_eq!(accent.0.intensity, 18_000.0);
}

#[test]
fn menu_room_links_are_rebuilt_on_each_visit() {
    let mut app = ready_menu_app();
    let links = count::<With<DoorRef>>(&mut app);
    assert!(links > 0);
    for _ in 0..3 {
        press(&mut app, "Play button");
        assert_ne!(count::<With<DoorRef>>(&mut app), links);
        escape(&mut app);
        press(&mut app, "Main menu button");
        assert_eq!(count::<With<LevelRoot>>(&mut app), 1);
        assert_eq!(count::<With<DoorRef>>(&mut app), links);
    }
}

#[test]
fn all_main_menu_buttons_play_hover_once() {
    let mut app = ready_menu_app();
    played_sounds(&mut app);
    let buttons: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Button>>()
        .iter(app.world())
        .collect();
    assert_eq!(buttons.len(), 5);
    for button in buttons {
        app.world_mut().entity_mut(button).insert(Hovered(true));
        app.update();
        assert_eq!(played_sounds(&mut app), vec![menu_sounds().hover]);
        app.update();
        assert!(played_sounds(&mut app).is_empty());
    }
}

#[test]
fn play_click_does_not_switch_on_flashlight() {
    let mut app = ready_menu_app();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    press(&mut app, "Play button");
    let player = player(&mut app);
    let flashlight = *app.world().get::<Flashlight>(player).unwrap();
    assert!(!flashlight.on);
    assert_eq!(flashlight.charge, 1.0);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    assert!(!app.world().get::<Flashlight>(player).unwrap().on);
}

#[test]
fn quit_requests_app_exit() {
    let mut app = ready_menu_app();
    press(&mut app, "Quit button");
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
