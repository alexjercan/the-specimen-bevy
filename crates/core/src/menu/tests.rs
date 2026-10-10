use std::time::Duration;

use bevy::{
    ecs::resource::IsResource,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState, InputPlugin,
    },
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    ui_widgets::{SliderDragState, SliderValue, ValueChange},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use game_assets::UiAssets;
use game_settings::{GameSettings, SettingsDirty, MAX_SENSITIVITY};
use game_ui::SliderFill;
use gameplay::{
    controller::{Flashlight, PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        Caught, Door, DoorPlugin, DoorRef, DoorState, Escaped, ExitDoor, FusePanel, LevelRoot,
        Monster, MonsterFigure, Room,
    },
};

use super::{
    cinematic::{Cinematic, CinematicCamera, REVEAL_AT, TITLE_AT, TITLE_FADE},
    complete::{CompleteScreen, DOOR_CLOSE_AT},
    credits::{wrap_offset, CreditsRoute, CreditsScreen, HUMAN_DEER_CREDIT, ROLL_SPEED},
    game_over::{CaughtShot, GameOverScreen},
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
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
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
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<Without<IsResource>>(&mut app), playing);
}

fn run_for(app: &mut App, secs: f32) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for _ in 0..(secs * 10.0).round() as usize {
        app.update();
    }
    app.insert_resource(TimeUpdateStrategy::Automatic);
}

fn run_player(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world())
        .unwrap()
}

fn options(app: &mut App) -> Vec<MenuAction> {
    let mut actions: Vec<_> = app
        .world_mut()
        .query::<&MenuAction>()
        .iter(app.world())
        .copied()
        .collect();
    actions.sort_by_key(|action| *action as u8);
    actions
}

fn title_alpha(app: &mut App) -> f32 {
    app.world_mut()
        .query::<(&Text, &TextColor)>()
        .iter(app.world())
        .find_map(|(text, color)| {
            matches!(text.0.as_str(), "ESCAPED" | "CAUGHT").then(|| color.0.alpha())
        })
        .unwrap()
}

fn catch(app: &mut App) {
    let player = run_player(app);
    let monster = app
        .world_mut()
        .query_filtered::<Entity, With<Monster>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(player).insert(Caught {
        monster,
        remaining: 0.0,
    });
    app.update();
    app.update();
}

fn assert_run_gone(app: &mut App) {
    assert_eq!(count::<With<PlayerController>>(app), 0);
    assert_eq!(count::<With<Monster>>(app), 0);
    assert_eq!(count::<With<FusePanel>>(app), 0);
    assert_eq!(count::<With<ExitDoor>>(app), 0);
    assert_eq!(count::<With<Room>>(app), count::<With<CinematicRoom>>(app));
    assert_eq!(
        count::<(With<IsResource>, With<DespawnOnExit<GameState>>)>(app),
        0
    );
}

fn assert_cinematic_gone(app: &mut App) {
    assert_eq!(count::<With<Cinematic>>(app), 0);
    assert_eq!(count::<With<CinematicCamera>>(app), 0);
    assert_eq!(count::<With<MonsterFigure>>(app), 0);
    assert_eq!(count::<With<CompleteScreen>>(app), 0);
    assert_eq!(count::<With<GameOverScreen>>(app), 0);
    assert_eq!(count::<With<CreditsScreen>>(app), 0);
    assert!(app.world().get_resource::<CaughtShot>().is_none());
}

#[derive(Component)]
struct CinematicRoom;

fn tag_cinematic_rooms(app: &mut App) {
    app.add_observer(
        |added: On<Add, Room>,
         parents: Query<&ChildOf>,
         roots: Query<&Name>,
         mut commands: Commands| {
            let Ok(parent) = parents.get(added.entity) else {
                return;
            };
            if roots
                .get(parent.parent())
                .is_ok_and(|name| name.as_str() == "Exit cinematic")
            {
                commands.entity(added.entity).insert(CinematicRoom);
            }
        },
    );
}

#[test]
fn escape_transitions_to_completion_and_returns_to_menu() {
    let mut app = app();
    tag_cinematic_rooms(&mut app);
    app.add_observer(|_: On<Add, Room>, mut commands: Commands| {
        commands.spawn(NonLevelRoot);
    });
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    let playing = count::<(Without<IsResource>, Without<NonLevelRoot>)>(&mut app);
    let player = run_player(&mut app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();

    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(count::<With<CompleteScreen>>(&mut app), 1);
    assert_run_gone(&mut app);
    assert_eq!(count::<With<CinematicRoom>>(&mut app), 2);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 1);
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<Camera3d>>(&mut app), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    run_for(&mut app, REVEAL_AT + 0.2);
    assert_eq!(game_state(&app), GameState::Credits);
    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 0);
    assert_eq!(count::<With<MainMenu>>(&mut app), 1);
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(
        count::<(Without<IsResource>, Without<NonLevelRoot>)>(&mut app),
        playing
    );
}

#[test]
fn completion_cinematic_closes_the_door_then_rolls_credits() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    let player = run_player(&mut app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::Complete);
    assert!(options(&mut app).is_empty());
    assert_eq!(count::<With<Button>>(&mut app), 0);
    assert_eq!(title_alpha(&mut app), 0.0);
    let door = app
        .world_mut()
        .query::<&Door>()
        .single(app.world())
        .unwrap()
        .state;
    assert_eq!(door, DoorState::Open);

    run_for(&mut app, DOOR_CLOSE_AT + 0.6);
    let door = app
        .world_mut()
        .query::<&Door>()
        .single(app.world())
        .unwrap()
        .state;
    assert_eq!(door, DoorState::Closed);
    assert!(options(&mut app).is_empty());
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 0);

    run_for(&mut app, TITLE_AT + TITLE_FADE - DOOR_CLOSE_AT - 0.4);
    assert!(title_alpha(&mut app) > 0.99);
    assert!(options(&mut app).is_empty());
    assert_eq!(count::<With<Button>>(&mut app), 0);
    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 0);

    run_for(&mut app, REVEAL_AT - TITLE_AT - TITLE_FADE + 0.2);
    assert_eq!(game_state(&app), GameState::Credits);
    assert_eq!(
        *app.world().resource::<CreditsRoute>(),
        CreditsRoute::Ending
    );
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 1);
    assert_eq!(count::<With<CompleteScreen>>(&mut app), 0);
    assert_eq!(count::<With<Cinematic>>(&mut app), 0);
    assert_eq!(count::<With<CinematicCamera>>(&mut app), 0);
    assert_eq!(count::<With<Door>>(&mut app), 0);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<Camera3d>>(&mut app), 0);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert_eq!(
        options(&mut app),
        vec![MenuAction::Retry, MenuAction::MainMenu, MenuAction::Quit]
    );
    assert!(has_text(&mut app, "Main Menu"));
    run_for(&mut app, 2.0);
    assert_eq!(options(&mut app).len(), 3);
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 1);
}

fn has_text(app: &mut App, value: &str) -> bool {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0 == value)
}

fn win(app: &mut App) {
    let player = run_player(app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();
    assert_eq!(game_state(app), GameState::Complete);
    run_for(app, REVEAL_AT + 0.2);
    assert_eq!(game_state(app), GameState::Credits);
}

fn assert_menu_credits(app: &mut App) {
    assert_eq!(game_state(app), GameState::Credits);
    assert_eq!(
        *app.world().resource::<CreditsRoute>(),
        CreditsRoute::MainMenu
    );
    assert_eq!(count::<With<CreditsScreen>>(app), 1);
    assert_eq!(count::<With<MainMenu>>(app), 0);
    assert_eq!(count::<With<Camera2d>>(app), 1);
    assert_eq!(options(app), vec![MenuAction::MainMenu]);
    assert!(has_text(app, "Back"));
    assert!(!has_text(app, "Retry"));
    assert!(has_text(app, HUMAN_DEER_CREDIT));
    assert!(has_text(app, "Alex Jercan"));
}

#[test]
fn main_menu_credits_show_back_without_retry_and_clean_up() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    escape(&mut app);
    press(&mut app, MenuAction::MainMenu);
    let menu = count::<Without<IsResource>>(&mut app);
    press(&mut app, MenuAction::Credits);
    assert_menu_credits(&mut app);

    escape(&mut app);
    assert_eq!(game_state(&app), GameState::Credits);
    assert_eq!(count::<With<PauseMenu>>(&mut app), 0);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 0);

    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 0);
    assert_eq!(count::<With<MainMenu>>(&mut app), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<Without<IsResource>>(&mut app), menu);

    press(&mut app, MenuAction::Play);
    win(&mut app);
    assert_eq!(
        *app.world().resource::<CreditsRoute>(),
        CreditsRoute::Ending
    );
    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    assert_eq!(count::<Without<IsResource>>(&mut app), menu);
    press(&mut app, MenuAction::Credits);
    assert_menu_credits(&mut app);
}

#[test]
fn settings_overlay_blocks_the_credits_button() {
    let mut app = app();
    ready(&mut app);
    open_settings(&mut app);
    assert_eq!(count::<With<SettingsOverlay>>(&mut app), 1);
    press(&mut app, MenuAction::Credits);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<CreditsScreen>>(&mut app), 0);
}

#[test]
fn credits_roll_wraps_and_never_goes_negative() {
    assert_eq!(wrap_offset(12.0, 0.0), 0.0);
    assert_eq!(wrap_offset(-5.0, 100.0), 0.0);
    assert_eq!(wrap_offset(40.0, 100.0), 40.0);
    assert_eq!(wrap_offset(100.0 + ROLL_SPEED, 100.0), ROLL_SPEED);
}

#[test]
fn credits_show_the_exact_human_deer_attribution() {
    let credits = include_str!("../../../../credits/CREDITS.md");
    let notice = include_str!("../../../../credits/licenses/The-Human-Deer-source-license.txt");
    assert!(credits.contains(HUMAN_DEER_CREDIT));
    assert!(notice.contains(HUMAN_DEER_CREDIT));
}

#[test]
fn monster_catch_waits_for_the_attack_then_stages_the_caught_cinematic() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    let player = run_player(&mut app);
    let monster = app
        .world_mut()
        .query_filtered::<Entity, With<Monster>>()
        .single(app.world())
        .unwrap();
    let pose = Transform::from_xyz(1.0, 0.0, -4.0).looking_to(Vec3::Z, Vec3::Y);
    *app.world_mut().get_mut::<Transform>(monster).unwrap() = pose;
    let view = Transform::from_xyz(1.0, 1.6, -2.6).looking_at(Vec3::new(1.0, 1.8, -4.0), Vec3::Y);
    *app.world_mut().get_mut::<Transform>(player).unwrap() = view;
    app.world_mut().entity_mut(player).insert(Caught {
        monster,
        remaining: 60.0,
    });
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<MonsterFigure>>(&mut app), 0);
    app.world_mut().get_mut::<Caught>(player).unwrap().remaining = 0.0;
    app.update();
    app.update();

    assert_eq!(game_state(&app), GameState::GameOver);
    assert_eq!(count::<With<GameOverScreen>>(&mut app), 1);
    assert_run_gone(&mut app);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    let figure = app
        .world_mut()
        .query_filtered::<&Transform, (With<MonsterFigure>, Without<Monster>)>()
        .single(app.world())
        .copied()
        .unwrap();
    assert_eq!(figure, pose);
    let camera = app
        .world_mut()
        .query_filtered::<&Transform, With<Camera3d>>()
        .single(app.world())
        .copied()
        .unwrap();
    assert!(camera.translation.distance(view.translation) < 0.05);
    assert!(camera.rotation.angle_between(view.rotation) < 0.01);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert!(app.world().get_resource::<CaughtShot>().is_none());
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert!(options(&mut app).is_empty());

    run_for(&mut app, REVEAL_AT - 0.3);
    assert!(options(&mut app).is_empty());
    let camera = app
        .world_mut()
        .query_filtered::<&Transform, With<Camera3d>>()
        .single(app.world())
        .copied()
        .unwrap();
    assert!(camera.translation.distance(view.translation) > 0.1);
    assert!(camera.forward().dot(*view.forward()) > 0.999);
    run_for(&mut app, 0.5);
    assert_eq!(
        options(&mut app),
        vec![MenuAction::Retry, MenuAction::MainMenu, MenuAction::Quit]
    );

    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    press(&mut app, MenuAction::Play);
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<With<Monster>>(&mut app), 1);
}

#[test]
fn retry_starts_a_fresh_run_directly_without_leaking_cinematics() {
    let mut app = app();
    ready(&mut app);
    press(&mut app, MenuAction::Play);
    let playing = count::<Without<IsResource>>(&mut app);
    let rooms = count::<With<Room>>(&mut app);
    let main_menus = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let entered = main_menus.clone();
    app.add_systems(OnEnter(GameState::MainMenu), move || {
        entered.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });

    for round in 0..4 {
        let player = run_player(&mut app);
        if round % 2 == 0 {
            app.world_mut().entity_mut(player).insert(Escaped);
            app.update();
            app.update();
            assert_eq!(game_state(&app), GameState::Complete);
        } else {
            catch(&mut app);
            assert_eq!(game_state(&app), GameState::GameOver);
            assert_eq!(count::<With<MonsterFigure>>(&mut app), 1);
        }
        run_for(&mut app, REVEAL_AT + 0.2);
        let credits = if round % 2 == 0 { 1 } else { 0 };
        assert_eq!(count::<With<CreditsScreen>>(&mut app), credits);
        press(&mut app, MenuAction::Retry);
        assert_eq!(game_state(&app), GameState::Playing);
        assert_cinematic_gone(&mut app);
        assert_eq!(count::<With<MainMenu>>(&mut app), 0);
        assert_eq!(count::<With<Room>>(&mut app), rooms);
        assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
        assert_eq!(count::<With<Camera2d>>(&mut app), 0);
        assert_eq!(count::<With<PlayerController>>(&mut app), 1);
        assert_eq!(count::<With<Monster>>(&mut app), 1);
        let player = run_player(&mut app);
        assert!(app.world().get::<Escaped>(player).is_none());
        assert!(app.world().get::<Caught>(player).is_none());
        assert_eq!(count::<Without<IsResource>>(&mut app), playing);
    }
    assert_eq!(main_menus.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[test]
fn headless_runs_keep_outcomes_queryable_without_menu_screens() {
    for outcome in 0..2 {
        let mut app = AppBuilder::headless().with_seed(7).build();
        app.finish();
        app.cleanup();
        app.update();
        app.update();
        assert!(app.world().get_resource::<State<GameState>>().is_none());
        let player = run_player(&mut app);
        if outcome == 0 {
            app.world_mut().entity_mut(player).insert(Escaped);
        } else {
            app.world_mut().entity_mut(player).insert(Caught {
                monster: Entity::PLACEHOLDER,
                remaining: 0.0,
            });
        }
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(run_player(&mut app), player);
        assert_eq!(app.world().get::<Escaped>(player).is_some(), outcome == 0);
        assert_eq!(app.world().get::<Caught>(player).is_some(), outcome == 1);
        assert_eq!(count::<With<Room>>(&mut app), 26);
        assert_eq!(count::<With<Cinematic>>(&mut app), 0);
        assert_eq!(count::<With<MonsterFigure>>(&mut app), 0);
        assert_eq!(count::<With<Camera>>(&mut app), 0);
    }
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
