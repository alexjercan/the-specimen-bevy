use bevy::{
    ecs::resource::IsResource,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState, InputPlugin,
    },
    state::app::StatesPlugin,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use game_assets::UiAssets;
use gameplay::{
    controller::{Flashlight, PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{Door, DoorPlugin, DoorRef, Escaped, LevelRoot, Room},
};

use super::{
    complete::CompleteScreen, loading::LoadingScreen, main_menu::MainMenu, pause::PauseMenu,
    GameState, MenuAction,
    MenuPlugin, PauseState,
};
use crate::{AppBuilder, CoreState};
use bevy::prelude::*;

struct Custom;

#[derive(Component)]
struct NonLevelRoot;

impl Plugin for Custom {
    fn build(&self, _: &mut App) {}
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .init_state::<CoreState>()
    .insert_resource(UiAssets {
        interact_key: Handle::default(),
        font: Handle::default(),
    })
    .add_plugins(PlayerControllerPlugin::default().without_camera())
    .add_plugins((DoorPlugin, MenuPlugin));
    app.world_mut()
        .spawn((Window::default(), CursorOptions::default(), PrimaryWindow));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn ready(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<CoreState>>()
        .set(CoreState::Ready);
    app.update();
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}

fn press(app: &mut App, action: MenuAction) {
    let button = app
        .world_mut()
        .query::<(Entity, &MenuAction)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (*candidate == action).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
}

fn key(app: &mut App, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::Escape,
        logical_key: Key::Escape,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

fn escape(app: &mut App) {
    key(app, ButtonState::Pressed);
    key(app, ButtonState::Released);
}

fn game_state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

fn cursor(app: &mut App) -> (CursorGrabMode, bool) {
    let cursor = app
        .world_mut()
        .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    (cursor.grab_mode, cursor.visible)
}

#[test]
fn builder_installs_menu_only_for_windowed_default_game() {
    assert!(AppBuilder::new().with_menu().menu_enabled());
    assert!(!AppBuilder::new().menu_enabled());
    assert!(!AppBuilder::new()
        .with_menu()
        .with_transport()
        .menu_enabled());
    assert!(!AppBuilder::headless().with_menu().menu_enabled());
    assert!(!AppBuilder::new()
        .with_menu()
        .with_main_plugin(Custom)
        .menu_enabled());
}

#[test]
fn loading_screen_shows_until_core_is_ready_then_main_menu() {
    let mut app = app();
    assert_eq!(count::<With<LoadingScreen>>(&mut app), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<MainMenu>>(&mut app), 0);
    ready(&mut app);
    assert_eq!(count::<With<LoadingScreen>>(&mut app), 0);
    assert_eq!(count::<With<MainMenu>>(&mut app), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
}

#[test]
fn play_pause_and_main_menu_cycle_without_duplicate_worlds() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<MainMenu>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 18);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 2);
    assert_eq!(
        count::<(With<IsResource>, With<DespawnOnExit<GameState>>)>(&mut app),
        0
    );
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    let playing = count::<Without<IsResource>>(&mut app);

    escape(&mut app);
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Paused
    );
    assert_eq!(count::<With<PauseMenu>>(&mut app), 1);
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    assert!(!app.world().resource::<PlayerControlsEnabled>().0);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    press(&mut app, MenuAction::Resume);
    assert_eq!(count::<With<PauseMenu>>(&mut app), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert!(app.world().resource::<PlayerControlsEnabled>().0);
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Running
    );

    escape(&mut app);
    escape(&mut app);
    assert_eq!(count::<With<PauseMenu>>(&mut app), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());

    escape(&mut app);
    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<PauseMenu>>(&mut app), 0);
    assert_eq!(count::<With<MainMenu>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);

    assert_eq!(count::<With<Door>>(&mut app), 0);
    assert_eq!(count::<With<DoorRef>>(&mut app), 0);
    assert_eq!(count::<With<PointLight>>(&mut app), 0);

    press(&mut app, MenuAction::Play);
    assert_eq!(count::<With<Room>>(&mut app), 18);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<Without<IsResource>>(&mut app), playing);
}

#[test]
fn escape_transitions_to_completion_and_returns_to_menu() {
    let mut app = app();
    app.add_observer(|_: On<Add, Room>, mut commands: Commands| {
        commands.spawn(NonLevelRoot);
    });
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 2);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();

    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(count::<With<CompleteScreen>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<CompleteScreen>>(&mut app), 0);
    assert_eq!(count::<With<MainMenu>>(&mut app), 1);
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<Room>>(&mut app), 18);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 2);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
}

#[test]
fn play_click_does_not_switch_on_flashlight() {
    let mut app = app();
    ready(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    press(&mut app, MenuAction::Play);
    let flashlight = app
        .world_mut()
        .query_filtered::<&Flashlight, With<PlayerController>>()
        .single(app.world())
        .unwrap();
    assert!(!flashlight.on);
    assert_eq!(flashlight.charge, 1.0);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    assert!(!app
        .world_mut()
        .query_filtered::<&Flashlight, With<PlayerController>>()
        .single(app.world())
        .unwrap()
        .on);
}

#[test]
fn quit_requests_app_exit() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Quit);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
