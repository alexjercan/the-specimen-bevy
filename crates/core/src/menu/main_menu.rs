use bevy::prelude::*;
use game_assets::{FacilityAssets, UiAssets};
use game_ui::{menu_button, text, theme};
use gameplay::controller::PlayerController;
use gameplay::levels::{
    build_first_floor, spawn_first_floor_actors, Door, DoorRef, FusePanel, LevelRoot, Monster,
    Passage, PickupKind, Prop, Room,
};

use super::{background, release_cursor, screen_camera, GameState, MenuAction, TITLE};

#[derive(Component)]
pub(super) struct MainMenu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::MainMenu),
        (spawn_main_menu, release_cursor),
    )
    .add_systems(OnEnter(GameState::Playing), enter_world);
}

fn spawn_main_menu(
    mut commands: Commands,
    assets: Res<UiAssets>,
    facility: Option<Res<FacilityAssets>>,
) {
    let font = assets.font.clone();
    if facility.is_some() {
        background::spawn(&mut commands);
    } else {
        commands.spawn(screen_camera(GameState::MainMenu));
    }
    let menu = commands
        .spawn((
            MainMenu,
            Name::new("Main menu"),
            DespawnOnExit(GameState::MainMenu),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::left(px(96)),
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(if facility.is_some() {
                Color::srgba(0.002, 0.004, 0.008, 0.10)
            } else {
                theme::BACKGROUND
            }),
            children![(
                text(TITLE, 56.0, theme::TEXT, font.clone()),
                Node {
                    margin: UiRect::bottom(px(28)),
                    ..default()
                },
            )],
        ))
        .id();
    let buttons = commands
        .spawn((
            Node {
                width: px(240),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            children![
                (
                    Name::new("Play button"),
                    MenuAction::Play,
                    menu_button("Play", font.clone()),
                ),
                (
                    Name::new("Settings button"),
                    super::settings::SettingsAction::Open,
                    menu_button("Settings", font.clone()),
                ),
            ],
        ))
        .id();
    commands.entity(menu).add_child(buttons);
    #[cfg(not(target_arch = "wasm32"))]
    commands.entity(buttons).with_children(|parent| {
        parent.spawn((
            Name::new("Quit button"),
            MenuAction::Quit,
            menu_button("Quit", font.clone()),
        ));
    });
}

fn enter_world(world: &mut World) {
    let build = world.register_system_cached(build_first_floor);
    if let Err(error) = world.run_system(build) {
        error!("first floor build failed: {error}");
        return;
    }
    let spawn_actors = world.register_system_cached(spawn_first_floor_actors);
    if let Err(error) = world.run_system(spawn_actors) {
        error!("first floor actor spawn failed: {error}");
        return;
    }
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, (
            Without<ChildOf>,
            Or<(
                With<Room>,
                With<LevelRoot>,
                With<Door>,
                With<DoorRef>,
                With<Passage>,
                With<Prop>,
                With<FusePanel>,
                With<PickupKind>,
                With<Monster>,
                With<PlayerController>,
            )>,
        )>()
        .iter(world)
        .collect();
    for entity in roots {
        world
            .entity_mut(entity)
            .insert(DespawnOnExit(GameState::Playing));
    }
}
