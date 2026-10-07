use bevy::{
    input::InputPlugin,
    prelude::*,
    state::app::StatesPlugin,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use game_assets::UiAssets;
use game_core::{CoreState, GameState, MenuPlugin};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        DoorLock, DoorPlugin, Escaped, ExitDoor, FuseInventory, FusePanel, FusePlugin,
        InstallFuses, ObjectivePlugin, Room, FUSE_COUNT,
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
    .insert_resource(UiAssets {
        interact_key: Handle::default(),
        font: Handle::default(),
    })
    .add_plugins(PlayerControllerPlugin::default().without_camera())
    .add_plugins((DoorPlugin, FusePlugin, ObjectivePlugin, MenuPlugin));
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

fn assert_fresh_run(app: &mut App) {
    assert_eq!(game_state(app), GameState::Playing);
    assert_eq!(count::<With<Room>>(app), 18);
    let player = single::<With<PlayerController>>(app);
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 0);
    assert!(app.world().get::<Escaped>(player).is_none());
    let exit = single::<With<ExitDoor>>(app);
    assert!(app.world().get::<DoorLock>(exit).is_some());
    let panel = single::<With<FusePanel>>(app);
    assert_eq!(app.world().get::<FusePanel>(panel).unwrap().installed, 0);
}

#[test]
fn escaping_shows_the_completion_screen_and_replay_starts_a_fresh_run() {
    let mut app = app();
    press(&mut app, "Play button");
    assert_fresh_run(&mut app);

    escape(&mut app);
    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    assert_eq!(count::<With<PlayerController>>(&mut app), 0);
    assert_eq!(count::<With<FusePanel>>(&mut app), 0);
    assert_eq!(count::<With<ExitDoor>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
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

    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);

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
    press(&mut app, "Quit button");
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
