use std::time::Duration;

use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState, InputPlugin,
    },
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    window::{CursorOptions, PrimaryWindow},
};

use super::*;
use crate::{menu::main_menu::MainMenu, menu::MenuPlugin, CoreState};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, StatesPlugin))
        .init_state::<CoreState>()
        .insert_resource(UiAssets::default())
        .add_plugins(MenuPlugin);
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

fn has_text(app: &mut App, value: &str) -> bool {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0 == value)
}

fn button_names(app: &mut App) -> Vec<String> {
    let menu = app
        .world_mut()
        .query_filtered::<Entity, With<MainMenu>>()
        .single(app.world())
        .unwrap();
    let buttons = app.world().get::<Children>(menu).unwrap()[1];
    app.world()
        .get::<Children>(buttons)
        .unwrap()
        .iter()
        .map(|entity| app.world().get::<Name>(entity).unwrap().as_str().to_owned())
        .collect()
}

fn row_status(app: &mut App, achievement: Achievement) -> String {
    let name = format!("{} row", achievement.name());
    let row = app
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (candidate.as_str() == name).then_some(entity))
        .unwrap();
    let children = app.world().get::<Children>(row).unwrap();
    assert!(app.world().get::<ImageNode>(children[0]).is_some());
    let status = children[2];
    app.world().get::<Text>(status).unwrap().0.clone()
}

#[test]
fn every_achievement_has_a_pair_of_packaged_icons() {
    let assets =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/achievements");
    for achievement in Achievement::ALL {
        let stem = icon_stem(achievement);
        for variant in ["locked", "unlocked"] {
            assert!(assets.join(format!("{stem}-{variant}.png")).is_file());
        }
    }
}

#[test]
fn main_menu_lists_play_settings_credits_achievements_then_quit() {
    let mut app = app();
    ready(&mut app);
    assert_eq!(
        button_names(&mut app),
        vec![
            "Play button",
            "Settings button",
            "Credits button",
            "Achievements button",
            "Quit button",
        ]
    );
}

#[test]
fn achievements_screen_lists_all_entries_with_counts_and_status() {
    let mut app = app();
    ready(&mut app);
    app.insert_resource(AchievementProgress {
        unlocked: [Achievement::RestoreBoiler, Achievement::EscapeUndetected]
            .into_iter()
            .collect(),
    });

    press(&mut app, MenuAction::Achievements);
    assert_eq!(game_state(&app), GameState::Achievements);
    assert_eq!(count::<With<AchievementsScreen>>(&mut app), 1);
    let panel = app
        .world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, name)| (name.as_str() == "Achievements panel").then_some(entity))
        .unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(panel).unwrap().0,
        theme::PANEL
    );
    assert_eq!(app.world().get::<Node>(panel).unwrap().width, px(560));
    assert!(has_text(&mut app, "2 / 7 unlocked"));

    for achievement in Achievement::ALL {
        let expected = if matches!(
            achievement,
            Achievement::RestoreBoiler | Achievement::EscapeUndetected
        ) {
            "UNLOCKED"
        } else {
            "LOCKED"
        };
        assert_eq!(row_status(&mut app, achievement), expected);
        assert!(has_text(&mut app, achievement.name()));
    }
}

#[test]
fn back_button_and_escape_both_return_to_main_menu_and_clean_up() {
    let mut app = app();
    ready(&mut app);

    press(&mut app, MenuAction::Achievements);
    assert_eq!(game_state(&app), GameState::Achievements);
    press(&mut app, MenuAction::MainMenu);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<AchievementsScreen>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);

    press(&mut app, MenuAction::Achievements);
    assert_eq!(game_state(&app), GameState::Achievements);
    escape(&mut app);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<AchievementsScreen>>(&mut app), 0);
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
}

#[test]
fn achievement_unlock_spawns_and_then_clears_a_toast() {
    let mut app = app();
    ready(&mut app);

    app.world_mut()
        .trigger(AchievementUnlocked(Achievement::RestoreBoiler));
    app.update();
    assert_eq!(count::<With<Toast>>(&mut app), 1);
    assert_eq!(count::<With<ToastContainer>>(&mut app), 1);
    assert!(has_text(&mut app, "ACHIEVEMENT UNLOCKED"));
    assert!(has_text(&mut app, Achievement::RestoreBoiler.name()));
    let toast = app
        .world_mut()
        .query_filtered::<Entity, With<Toast>>()
        .single(app.world())
        .unwrap();
    let icon = app.world().get::<Children>(toast).unwrap()[0];
    assert!(app.world().get::<ImageNode>(icon).is_some());
    let sounds: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert!(sounds.iter().any(|sound| sound.sound == Sound::UiConfirm));
    let pickable = *app
        .world_mut()
        .query_filtered::<&Pickable, With<ToastContainer>>()
        .single(app.world())
        .unwrap();
    assert_eq!(pickable, Pickable::IGNORE);

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for _ in 0..(TOAST_SECS * 10.0).round() as usize + 2 {
        app.update();
    }
    app.insert_resource(TimeUpdateStrategy::Automatic);
    assert_eq!(count::<With<Toast>>(&mut app), 0);
}
