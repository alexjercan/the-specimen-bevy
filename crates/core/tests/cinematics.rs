use bevy::{prelude::*, window::CursorGrabMode};
use game_core::{AppBuilder, GameState};
use gameplay::{
    controller::PlayerController,
    levels::{
        Caught, Door, DoorState, Escaped, ExitDoor, FusePanel, LevelRoot, Monster, MonsterFigure,
        Room,
    },
};

mod common;

use common::*;

const DOOR_CLOSE_AT: f32 = 0.8;
const TITLE_AT: f32 = 2.5;
const TITLE_FADE: f32 = 1.0;

#[derive(Component)]
struct CinematicRoom;

#[derive(Component)]
struct NonLevelRoot;

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

fn title_alpha(app: &mut App) -> f32 {
    app.world_mut()
        .query::<(&Text, &TextColor)>()
        .iter(app.world())
        .find_map(|(text, color)| {
            matches!(text.0.as_str(), "ESCAPED" | "CAUGHT").then(|| color.0.alpha())
        })
        .unwrap()
}

fn door_state(app: &mut App) -> DoorState {
    app.world_mut()
        .query::<&Door>()
        .single(app.world())
        .unwrap()
        .state
}

fn cinematic_camera(app: &mut App) -> Transform {
    let camera = named(app, "Cinematic camera").unwrap();
    *app.world().get::<Transform>(camera).unwrap()
}

fn stable_entities(app: &mut App) -> usize {
    entities(app) - named_count(app, "Achievement toast container")
}

fn assert_run_gone(app: &mut App) {
    assert_eq!(count::<With<PlayerController>>(app), 0);
    assert_eq!(count::<With<Monster>>(app), 0);
    assert_eq!(count::<With<FusePanel>>(app), 0);
    assert_eq!(count::<With<ExitDoor>>(app), 0);
    assert_eq!(count::<With<Room>>(app), count::<With<CinematicRoom>>(app));
}

fn assert_cinematic_gone(app: &mut App) {
    assert!(named(app, "Complete screen").is_none());
    assert!(named(app, "Game over screen").is_none());
    assert!(named(app, "Credits screen").is_none());
    assert!(named(app, "Cinematic camera").is_none());
    assert!(named(app, "Cinematic UI camera").is_none());
    assert_eq!(count::<With<MonsterFigure>>(app), 0);
}

fn assert_ending_options(app: &mut App) {
    assert_eq!(
        button_names(app),
        vec!["Main menu button", "Quit button", "Retry button"]
    );
    assert_eq!(count::<With<Button>>(app), 3);
}

fn assert_credits_layout(app: &mut App) {
    let expected_buttons = count::<With<Button>>(app);
    let root = named(app, "Credits screen").unwrap();
    let panes = app.world().get::<Children>(root).unwrap();
    assert_eq!(panes.len(), 2);
    let button_count = app
        .world()
        .get::<Children>(panes[0])
        .unwrap()
        .iter()
        .filter(|entity| app.world().get::<Button>(*entity).is_some())
        .count();
    assert_eq!(button_count, expected_buttons);
    assert!(app
        .world()
        .get::<Children>(panes[1])
        .unwrap()
        .iter()
        .any(|entity| app
            .world()
            .get::<Text>(entity)
            .is_some_and(|text| text.0 == "CREDITS")));
}

fn assert_menu_credits(app: &mut App) {
    assert_eq!(game_state(app), GameState::Credits);
    assert_eq!(named_count(app, "Credits screen"), 1);
    assert_credits_layout(app);
    assert!(named(app, "Main menu").is_none());
    assert_eq!(count::<With<Camera2d>>(app), 1);
    assert_eq!(count::<With<Camera3d>>(app), 1);
    assert_eq!(door_state(app), DoorState::Closed);
    assert_eq!(button_names(app), vec!["Back button"]);
    assert!(has_text(app, "Back"));
    assert!(!has_text(app, "Retry"));
    assert!(!has_text(app, "ESCAPED"));
    assert!(has_text_containing(app, "\"The Human Deer\""));
    assert!(has_text_containing(app, "CC-BY-4.0"));
    assert!(has_text(app, "Alex Jercan"));
}

#[test]
fn escape_transitions_to_completion_and_returns_to_menu() {
    let mut app = menu_app();
    tag_cinematic_rooms(&mut app);
    app.add_observer(|_: On<Add, Room>, mut commands: Commands| {
        commands.spawn(NonLevelRoot);
    });
    ready(&mut app);
    press(&mut app, "Play button");
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    let playing = stable_entities(&mut app) - count::<With<NonLevelRoot>>(&mut app);
    let player = player(&mut app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();

    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(named_count(&mut app, "Complete screen"), 1);
    assert_run_gone(&mut app);
    assert_eq!(count::<With<CinematicRoom>>(&mut app), 2);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 1);
    assert!(count::<With<NonLevelRoot>>(&mut app) > 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<Camera3d>>(&mut app), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));

    run_for(&mut app, REVEAL_AT + 0.2);
    assert_eq!(game_state(&app), GameState::Credits);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    assert_eq!(count::<With<CinematicRoom>>(&mut app), 0);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 1);
    assert_eq!(named_count(&mut app, "Main menu"), 1);
    press(&mut app, "Play button");
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<Room>>(&mut app), 26);
    assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(
        stable_entities(&mut app) - count::<With<NonLevelRoot>>(&mut app),
        playing
    );
}

#[test]
fn completion_cinematic_closes_the_door_then_rolls_credits() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    let player = player(&mut app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();
    assert_eq!(game_state(&app), GameState::Complete);
    assert_eq!(count::<With<Button>>(&mut app), 0);
    assert_eq!(title_alpha(&mut app), 0.0);
    assert_eq!(door_state(&mut app), DoorState::Open);

    run_for(&mut app, DOOR_CLOSE_AT + 0.6);
    assert_eq!(door_state(&mut app), DoorState::Closed);
    assert_eq!(count::<With<Button>>(&mut app), 0);
    assert!(named(&mut app, "Credits screen").is_none());

    run_for(&mut app, TITLE_AT + TITLE_FADE - DOOR_CLOSE_AT - 0.4);
    assert!(title_alpha(&mut app) > 0.99);
    assert_eq!(count::<With<Button>>(&mut app), 0);
    assert_eq!(game_state(&app), GameState::Complete);
    assert!(named(&mut app, "Credits screen").is_none());

    run_for(&mut app, REVEAL_AT - TITLE_AT - TITLE_FADE + 0.2);
    assert_eq!(game_state(&app), GameState::Credits);
    assert_eq!(named_count(&mut app, "Credits screen"), 1);
    assert_credits_layout(&mut app);
    assert!(named(&mut app, "Complete screen").is_none());
    assert!(named(&mut app, "Cinematic camera").is_some());
    assert_eq!(count::<With<Door>>(&mut app), 1);
    assert_eq!(count::<With<Room>>(&mut app), 2);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(count::<With<Camera3d>>(&mut app), 1);
    assert_eq!(door_state(&mut app), DoorState::Closed);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert_ending_options(&mut app);
    assert!(has_text(&mut app, "Main Menu"));
    assert!(has_text(&mut app, "ESCAPED"));
    run_for(&mut app, 2.0);
    assert_ending_options(&mut app);
    assert_eq!(named_count(&mut app, "Credits screen"), 1);
}

#[test]
fn main_menu_credits_show_back_without_retry_and_clean_up() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    win(&mut app);
    assert_ending_options(&mut app);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    let menu = stable_entities(&mut app);
    press(&mut app, "Credits button");
    assert_menu_credits(&mut app);

    escape(&mut app);
    assert_eq!(game_state(&app), GameState::Credits);
    assert!(named(&mut app, "Pause menu").is_none());
    assert!(named(&mut app, "Settings overlay").is_none());

    press(&mut app, "Back button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert!(named(&mut app, "Credits screen").is_none());
    assert_eq!(named_count(&mut app, "Main menu"), 1);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(stable_entities(&mut app), menu);

    press(&mut app, "Play button");
    win(&mut app);
    assert_ending_options(&mut app);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    assert_eq!(stable_entities(&mut app), menu);
    press(&mut app, "Credits button");
    assert_menu_credits(&mut app);
}

#[test]
fn settings_overlay_blocks_the_credits_button() {
    let mut app = ready_menu_app();
    press(&mut app, "Settings button");
    assert_eq!(named_count(&mut app, "Settings overlay"), 1);
    press(&mut app, "Credits button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert!(named(&mut app, "Credits screen").is_none());
}

#[test]
fn monster_catch_waits_for_the_attack_then_stages_the_caught_cinematic() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    let player = player(&mut app);
    let monster = single::<With<Monster>>(&mut app);
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
    let view = *app.world().get::<Transform>(player).unwrap();
    app.world_mut().get_mut::<Caught>(player).unwrap().remaining = 0.0;
    app.update();
    app.update();

    assert_eq!(game_state(&app), GameState::GameOver);
    assert_eq!(named_count(&mut app, "Game over screen"), 1);
    assert_run_gone(&mut app);
    assert_eq!(count::<With<Room>>(&mut app), 0);
    let figure = app
        .world_mut()
        .query_filtered::<&Transform, (With<MonsterFigure>, Without<Monster>)>()
        .single(app.world())
        .copied()
        .unwrap();
    assert_eq!(figure.translation, pose.translation);
    assert!(figure.rotation.abs_diff_eq(pose.rotation, 1e-5));
    let camera = cinematic_camera(&mut app);
    assert!(camera.translation.distance(view.translation) < 0.05);
    assert!(camera.rotation.abs_diff_eq(view.rotation, 1e-4));
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(cursor(&mut app), (CursorGrabMode::None, true));
    assert_eq!(count::<With<Button>>(&mut app), 0);

    run_for(&mut app, REVEAL_AT - 0.3);
    assert_eq!(count::<With<Button>>(&mut app), 0);
    let camera = cinematic_camera(&mut app);
    assert!(camera.translation.distance(view.translation) > 0.1);
    assert!((camera.translation.x - view.translation.x).abs() < 1e-4);
    assert!((camera.translation.z - view.translation.z).abs() < 1e-4);
    assert!(camera.translation.y < view.translation.y);
    assert!(camera.forward().dot(*view.forward()) > 0.999);
    run_for(&mut app, 0.5);
    assert_ending_options(&mut app);

    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_cinematic_gone(&mut app);
    press(&mut app, "Play button");
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(count::<With<PlayerController>>(&mut app), 1);
    assert_eq!(count::<With<Monster>>(&mut app), 1);
}

#[test]
fn retry_starts_a_fresh_run_directly_without_leaking_cinematics() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    let playing = stable_entities(&mut app);
    let rooms = count::<With<Room>>(&mut app);
    let main_menus = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let entered = main_menus.clone();
    app.add_systems(OnEnter(GameState::MainMenu), move || {
        entered.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });

    for round in 0..4 {
        let player = common::player(&mut app);
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
        assert_eq!(named_count(&mut app, "Credits screen"), credits);
        press(&mut app, "Retry button");
        assert_eq!(game_state(&app), GameState::Playing);
        assert_cinematic_gone(&mut app);
        assert!(named(&mut app, "Main menu").is_none());
        assert_eq!(count::<With<Room>>(&mut app), rooms);
        assert_eq!(count::<With<LevelRoot>>(&mut app), 4);
        assert_eq!(count::<With<Camera2d>>(&mut app), 0);
        assert_eq!(count::<With<PlayerController>>(&mut app), 1);
        assert_eq!(count::<With<Monster>>(&mut app), 1);
        let player = common::player(&mut app);
        assert!(app.world().get::<Escaped>(player).is_none());
        assert!(app.world().get::<Caught>(player).is_none());
        assert_eq!(stable_entities(&mut app), playing);
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
        assert_eq!(game_state(&app), GameState::Playing);
        let player = player(&mut app);
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
        assert_eq!(common::player(&mut app), player);
        assert_eq!(app.world().get::<Escaped>(player).is_some(), outcome == 0);
        assert_eq!(app.world().get::<Caught>(player).is_some(), outcome == 1);
        assert_eq!(game_state(&app), GameState::Playing);
        assert_eq!(count::<With<Room>>(&mut app), 26);
        assert!(named(&mut app, "Complete screen").is_none());
        assert!(named(&mut app, "Game over screen").is_none());
        assert_eq!(count::<With<MonsterFigure>>(&mut app), 0);
        assert_eq!(count::<With<Camera>>(&mut app), 0);
    }
}
