use bevy::prelude::*;
use game_assets::{FacilityAssets, UiAssets};
use game_ui::{menu_button, text, theme};
use gameplay::achievements::AchievementProgress;
use gameplay::controller::PlayerController;
use gameplay::levels::{
    build_first_floor, spawn_first_floor_actors, Door, DoorRef, FusePanel, LevelRoot, Monster,
    Passage, PickupKind, Prop, Room,
};
use gameplay::presets::GameMode;

use super::{background, release_cursor, screen_camera, GameState, MenuAction, TITLE};

const MODES: [GameMode; 2] = [GameMode::Normal, GameMode::Insane];

#[derive(Component)]
pub(super) struct MainMenu;

#[derive(Component)]
struct ModeLabel;

#[derive(Resource, Default)]
pub(super) struct ModeSelection(pub(super) GameMode);

impl ModeSelection {
    pub(super) fn cycle(&mut self, step: isize) {
        let index = MODES.iter().position(|mode| *mode == self.0).unwrap_or(0) as isize;
        self.0 = MODES[(index + step).rem_euclid(MODES.len() as isize) as usize];
    }
}

fn mode_name(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Normal => "Normal",
        GameMode::Insane => "Insane",
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ModeSelection>()
        .add_systems(
            OnEnter(GameState::MainMenu),
            (reset_mode, spawn_main_menu, release_cursor).chain(),
        )
        .add_systems(
            Update,
            sync_mode_label.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnEnter(GameState::Playing), enter_world);
}

fn reset_mode(mut selection: ResMut<ModeSelection>) {
    *selection = ModeSelection::default();
}

fn sync_mode_label(selection: Res<ModeSelection>, mut labels: Query<&mut Text, With<ModeLabel>>) {
    if !selection.is_changed() {
        return;
    }
    for mut label in &mut labels {
        label.0 = mode_name(selection.0).to_string();
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    assets: Res<UiAssets>,
    facility: Option<Res<FacilityAssets>>,
    progress: Res<AchievementProgress>,
) {
    let font = assets.font.clone();
    if facility.is_some() {
        background::spawn(&mut commands, GameState::MainMenu);
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
        .spawn(Node {
            width: px(240),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        })
        .id();
    commands.entity(buttons).with_children(|parent| {
        if progress.insane_unlocked() {
            parent
                .spawn((
                    Name::new("Play row"),
                    Node {
                        width: percent(100),
                        column_gap: px(8),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        Name::new("Play button"),
                        MenuAction::Play,
                        menu_button("Play", font.clone()),
                    ))
                    .insert(Node {
                        flex_grow: 1.0,
                        padding: UiRect::axes(px(18), px(10)),
                        border: UiRect::all(px(theme::BORDER)),
                        border_radius: BorderRadius::all(px(theme::RADIUS)),
                        align_items: AlignItems::Center,
                        ..default()
                    });
                    spawn_mode_selector(row, font.clone());
                });
        } else {
            parent.spawn((
                Name::new("Play button"),
                MenuAction::Play,
                menu_button("Play", font.clone()),
            ));
        }
        parent.spawn((
            Name::new("Settings button"),
            super::settings::SettingsAction::Open,
            menu_button("Settings", font.clone()),
        ));
        parent.spawn((
            Name::new("Credits button"),
            MenuAction::Credits,
            menu_button("Credits", font.clone()),
        ));
        parent.spawn((
            Name::new("Achievements button"),
            MenuAction::Achievements,
            menu_button("Achievements", font.clone()),
        ));
    });
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

fn spawn_mode_selector(row: &mut ChildSpawnerCommands, font: Handle<Font>) {
    row.spawn((
        Name::new("Mode selector"),
        Node {
            align_items: AlignItems::Center,
            column_gap: px(4),
            ..default()
        },
    ))
    .with_children(|selector| {
        spawn_arrow(
            selector,
            "Mode previous button",
            MenuAction::PreviousMode,
            "<",
            font.clone(),
        );
        selector.spawn((
            Name::new("Mode label"),
            Node {
                width: px(64),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                ModeLabel,
                text(mode_name(GameMode::Normal), 18.0, theme::TEXT, font.clone()),
            )],
        ));
        spawn_arrow(
            selector,
            "Mode next button",
            MenuAction::NextMode,
            ">",
            font,
        );
    });
}

fn spawn_arrow(
    selector: &mut ChildSpawnerCommands,
    name: &'static str,
    action: MenuAction,
    glyph: &'static str,
    font: Handle<Font>,
) {
    selector
        .spawn((Name::new(name), action, menu_button(glyph, font)))
        .insert(Node {
            width: px(32),
            padding: UiRect::vertical(px(10)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
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
