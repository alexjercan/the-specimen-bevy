use bevy::{
    ecs::resource::IsResource,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState, InputPlugin,
    },
    state::app::StatesPlugin,
    ui_widgets::{SliderDragState, SliderValue, ValueChange},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use game_assets::UiAssets;
use game_settings::{GameSettings, SettingsDirty, MAX_SENSITIVITY};
use game_ui::SliderFill;
use gameplay::{
    controller::{Flashlight, PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{Caught, Door, DoorPlugin, DoorRef, Escaped, FusePanel, LevelRoot, Room},
};

use super::{
    complete::CompleteScreen,
    loading::LoadingScreen,
    main_menu::MainMenu,
    pause::PauseMenu,
    settings::{SettingSlider, SettingsAction, SettingsOverlay, SliderReadout},
    GameState, MenuAction, MenuPlugin, PauseState,
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
    .insert_resource(UiAssets::default())
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
fn settings_wait_for_ui_assets_during_loading() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, StatesPlugin))
        .init_state::<CoreState>()
        .add_plugins(MenuPlugin);
    app.finish();
    app.cleanup();
    app.update();
    assert_eq!(count::<With<LoadingScreen>>(&mut app), 1);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 0);

    app.insert_resource(UiAssets::default());
    ready(&mut app);
    let open = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| (*action == SettingsAction::Open).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(open)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 1);
}

fn open_settings(app: &mut App) {
    let open = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| (*action == SettingsAction::Open).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(open)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
}

fn slider(app: &mut App, field: SettingSlider) -> Entity {
    app.world_mut()
        .query::<(Entity, &SettingSlider)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (*candidate == field).then_some(entity))
        .unwrap()
}

fn slide(app: &mut App, field: SettingSlider, value: f32) {
    let source = slider(app, field);
    app.world_mut().trigger(ValueChange {
        source,
        value,
        is_final: false,
    });
    app.update();
}

fn slider_value(app: &mut App, field: SettingSlider) -> f32 {
    let entity = slider(app, field);
    app.world().get::<SliderValue>(entity).unwrap().0
}

fn fill(app: &mut App, field: SettingSlider) -> f32 {
    let entity = slider(app, field);
    let child = app.world().get::<Children>(entity).unwrap()[0];
    assert!(app.world().get::<SliderFill>(child).is_some());
    match app.world().get::<Node>(child).unwrap().width {
        Val::Percent(value) => value,
        other => panic!("unexpected fill width {other:?}"),
    }
}

fn readout(app: &mut App, field: SettingSlider) -> String {
    app.world_mut()
        .query::<(&SliderReadout, &Text)>()
        .iter(app.world())
        .find_map(|(readout, text)| (readout.0 == field).then(|| text.0.clone()))
        .unwrap()
}

#[test]
fn settings_sliders_change_persist_and_sync_in_menu_and_pause() {
    let mut app = app();
    ready(&mut app);
    app.world_mut().resource_mut::<GameSettings>().master = 0.0;
    open_settings(&mut app);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 1);
    assert_eq!(count::<With<SettingSlider>>(&mut app), 4);
    assert_eq!(slider_value(&mut app, SettingSlider::Master), 0.0);
    assert_eq!(readout(&mut app, SettingSlider::Master), "0%");

    slide(&mut app, SettingSlider::Master, 0.35);
    assert!((app.world().resource::<GameSettings>().master - 0.35).abs() < 0.001);
    assert!(app.world().resource::<SettingsDirty>().0);
    assert!((slider_value(&mut app, SettingSlider::Master) - 0.35).abs() < 0.001);
    assert!((fill(&mut app, SettingSlider::Master) - 35.0).abs() < 0.01);
    assert_eq!(readout(&mut app, SettingSlider::Master), "35%");

    slide(&mut app, SettingSlider::Sfx, 1.7);
    assert_eq!(app.world().resource::<GameSettings>().sfx, 1.0);
    slide(&mut app, SettingSlider::Music, 0.5);
    assert_eq!(app.world().resource::<GameSettings>().music, 0.5);
    assert_eq!(readout(&mut app, SettingSlider::Music), "50%");
    slide(&mut app, SettingSlider::Sensitivity, 1.0);
    assert_eq!(
        app.world().resource::<GameSettings>().mouse_sensitivity,
        MAX_SENSITIVITY
    );
    assert_eq!(readout(&mut app, SettingSlider::Sensitivity), "5.0x");

    app.world_mut().resource_mut::<SettingsDirty>().0 = false;
    slide(&mut app, SettingSlider::Master, 0.35);
    assert!(!app.world().resource::<SettingsDirty>().0);

    let master = slider(&mut app, SettingSlider::Master);
    app.world_mut()
        .get_mut::<SliderDragState>(master)
        .unwrap()
        .dragging = true;
    slide(&mut app, SettingSlider::Master, 0.4);
    slide(&mut app, SettingSlider::Master, 0.45);
    assert!((app.world().resource::<GameSettings>().master - 0.45).abs() < 0.001);
    assert_eq!(readout(&mut app, SettingSlider::Master), "45%");
    assert!(!app.world().resource::<SettingsDirty>().0);
    app.world_mut()
        .get_mut::<SliderDragState>(master)
        .unwrap()
        .dragging = false;
    app.update();
    assert!(app.world().resource::<SettingsDirty>().0);
    slide(&mut app, SettingSlider::Master, 0.35);

    app.world_mut().resource_mut::<GameSettings>().sfx = 0.2;
    app.update();
    app.update();
    assert!((slider_value(&mut app, SettingSlider::Sfx) - 0.2).abs() < 0.001);
    assert!((fill(&mut app, SettingSlider::Sfx) - 20.0).abs() < 0.01);
    assert_eq!(readout(&mut app, SettingSlider::Sfx), "20%");
    assert_eq!(app.world().resource::<GameSettings>().sfx, 0.2);

    let back = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| (*action == SettingsAction::Back).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(back)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 0);
    assert_eq!(count::<With<SettingSlider>>(&mut app), 0);
    assert_eq!(game_state(&app), GameState::MainMenu);

    press(&mut app, MenuAction::Play);
    escape(&mut app);
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Paused
    );
    open_settings(&mut app);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 1);
    assert!((slider_value(&mut app, SettingSlider::Master) - 0.35).abs() < 0.001);
    assert_eq!(readout(&mut app, SettingSlider::Master), "35%");
    assert!((slider_value(&mut app, SettingSlider::Sfx) - 0.2).abs() < 0.001);

    slide(&mut app, SettingSlider::Master, 0.6);
    assert!((app.world().resource::<GameSettings>().master - 0.6).abs() < 0.001);
    assert!(app.world().resource::<SettingsDirty>().0);
    assert_eq!(readout(&mut app, SettingSlider::Master), "60%");
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Paused
    );

    escape(&mut app);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 0);
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Paused
    );
}

#[test]
fn settings_rebind_and_overlay_block_play() {
    let mut app = app();
    ready(&mut app);
    let open = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| (*action == SettingsAction::Open).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(open)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::MainMenu);
    let forward = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| (*action == SettingsAction::Forward).then_some(entity))
        .unwrap();
    app.world_mut()
        .entity_mut(forward)
        .insert(Interaction::Pressed);
    app.update();
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::ArrowUp,
        logical_key: Key::ArrowUp,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
    assert_eq!(
        app.world().resource::<GameSettings>().keys.forward,
        "ArrowUp"
    );
    assert!(app.world().resource::<SettingsDirty>().0);
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
    assert_eq!(count::<With<FusePanel>>(&mut app), 0);
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
    let playing = count::<(Without<IsResource>, Without<NonLevelRoot>)>(&mut app);
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
    assert_eq!(count::<With<FusePanel>>(&mut app), 0);
    assert_eq!(count::<With<DoorRef>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(
        count::<(With<IsResource>, With<DespawnOnExit<GameState>>)>(&mut app),
        0
    );
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
    assert_eq!(
        count::<(Without<IsResource>, Without<NonLevelRoot>)>(&mut app),
        playing
    );
}

#[test]
fn monster_catch_waits_for_the_attack_then_transitions_to_game_over() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(player).insert(Caught {
        monster: Entity::PLACEHOLDER,
        remaining: 60.0,
    });
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::Playing);
    app.world_mut().get_mut::<Caught>(player).unwrap().remaining = 0.0;
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::GameOver);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    press(&mut app, MenuAction::MainMenu);
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::Playing);
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
    assert!(
        !app.world_mut()
            .query_filtered::<&Flashlight, With<PlayerController>>()
            .single(app.world())
            .unwrap()
            .on
    );
}

#[test]
fn quit_requests_app_exit() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Quit);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
