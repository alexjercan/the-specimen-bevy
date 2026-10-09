use std::time::Duration;

use bevy::{
    input::InputPlugin,
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use game_assets::UiAssets;
use game_core::{CoreState, GameState, MenuPlugin};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        DoorLock, DoorPlugin, Escaped, ExitDoor, FacilityPower, FacilityPowerPlugin, FuseInventory,
        FusePanel, FusePlugin, HidingPlugin, InstallFuses, LevelRoot, ObjectivePlugin, Room,
        FUSE_COUNT,
    },
};

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
    .add_plugins((
        DoorPlugin,
        FusePlugin,
        HidingPlugin,
        ObjectivePlugin,
        FacilityPowerPlugin,
        MenuPlugin,
    ));
    app.world_mut()
        .spawn((Window::default(), CursorOptions::default(), PrimaryWindow));
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .resource_mut::<NextState<CoreState>>()
        .set(CoreState::Ready);
    app.update();
    app
}

fn press(app: &mut App, name: &str) {
    let button = app
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (candidate.as_str() == name).then_some(entity))
        .unwrap_or_else(|| panic!("missing {name}"));
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}

fn single<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, F>()
        .single(app.world())
        .unwrap()
}

fn game_state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

fn escape(app: &mut App) {
    let player = single::<With<PlayerController>>(app);
    let panel = single::<With<FusePanel>>(app);
    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    app.world_mut()
        .write_message(InstallFuses { player, panel });
    app.update();
    let exit = single::<With<ExitDoor>>(app);
    assert!(app.world().get::<DoorLock>(exit).is_none());
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(0.0, 1.6, -31.6);
    for _ in 0..3 {
        app.update();
    }
}

fn settle(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for _ in 0..45 {
        app.update();
    }
    app.insert_resource(TimeUpdateStrategy::Automatic);
}

fn buttons(app: &mut App) -> usize {
    count::<With<Button>>(app)
}

fn assert_completion_scene(app: &mut App) {
    assert_eq!(game_state(app), GameState::Complete);
    assert_eq!(count::<With<PlayerController>>(app), 0);
    assert_eq!(count::<With<FusePanel>>(app), 0);
    assert_eq!(count::<With<ExitDoor>>(app), 0);
    assert_eq!(count::<With<LevelRoot>>(app), 1);
    assert_eq!(count::<With<Room>>(app), 2);
    assert_eq!(count::<With<Camera2d>>(app), 1);
    assert_eq!(count::<With<Camera3d>>(app), 1);
    assert_eq!(buttons(app), 0);
}

fn assert_fresh_run(app: &mut App) {
    assert_eq!(game_state(app), GameState::Playing);
    let power = app.world().resource::<FacilityPower>();
    assert!(power.on && power.outage_pending);
    assert_eq!(count::<With<Room>>(app), 26);
    let player = single::<With<PlayerController>>(app);
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 0);
    assert!(app.world().get::<Escaped>(player).is_none());
    let exit = single::<With<ExitDoor>>(app);
    assert!(app.world().get::<DoorLock>(exit).is_some());
    let panel = single::<With<FusePanel>>(app);
    assert_eq!(app.world().get::<FusePanel>(panel).unwrap().installed, 0);
}

#[test]
fn escaping_shows_the_completion_cinematic_and_replay_starts_a_fresh_run() {
    let mut app = app();
    press(&mut app, "Play button");
    assert_fresh_run(&mut app);

    escape(&mut app);
    assert_completion_scene(&mut app);
    let screen = app
        .world_mut()
        .query::<&Name>()
        .iter(app.world())
        .filter(|name| name.as_str() == "Complete screen")
        .count();
    assert_eq!(screen, 1);
    let cursor = app
        .world_mut()
        .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        (cursor.grab_mode, cursor.visible),
        (CursorGrabMode::None, true)
    );

    settle(&mut app);
    assert_eq!(buttons(&mut app), 3);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 0);

    press(&mut app, "Play button");
    assert_fresh_run(&mut app);
    escape(&mut app);
    assert_completion_scene(&mut app);
    settle(&mut app);
    press(&mut app, "Retry button");
    assert_fresh_run(&mut app);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    assert_eq!(count::<With<Camera2d>>(&mut app), 0);
    assert_eq!(buttons(&mut app), 0);

    escape(&mut app);
    assert_completion_scene(&mut app);
    settle(&mut app);
    press(&mut app, "Main menu button");
    press(&mut app, "Play button");
    assert_fresh_run(&mut app);
    app.update();
    assert_eq!(game_state(&app), GameState::Playing);
}

#[test]
fn quit_from_the_completion_screen_requests_app_exit() {
    let mut app = app();
    press(&mut app, "Play button");
    escape(&mut app);
    assert_eq!(game_state(&app), GameState::Complete);
    settle(&mut app);
    press(&mut app, "Quit button");
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
