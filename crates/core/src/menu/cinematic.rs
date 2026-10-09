use bevy::prelude::*;
use game_ui::{menu_button, text, theme};

use super::{GameState, MenuAction};

pub(super) const TITLE_AT: f32 = 2.5;
pub(super) const TITLE_FADE: f32 = 1.0;
pub(super) const REVEAL_AT: f32 = 4.0;
const SHADE: Color = Color::srgb(0.004, 0.003, 0.004);
const DEATH_SHADE: Color = Color::srgb(0.25, 0.005, 0.008);

#[derive(Component)]
pub(super) struct DeathTint;

#[derive(Component)]
pub(super) struct Cinematic {
    pub(super) elapsed: f32,
    shade: f32,
    shade_secs: f32,
    revealed: bool,
    font: Handle<Font>,
}

#[derive(Component)]
struct CinematicTitle;

#[derive(Component)]
pub(super) struct CinematicOptions;

#[derive(Component)]
pub(super) struct CinematicCamera {
    from: Transform,
    to: Transform,
    secs: f32,
}

impl CinematicCamera {
    pub(super) fn new(from: Transform, to: Transform, secs: f32) -> Self {
        Self { from, to, secs }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (advance, move_cameras, reveal_options)
            .chain()
            .run_if(in_state(GameState::Complete).or_else(in_state(GameState::GameOver))),
    );
}

pub(super) fn spawn_cameras(commands: &mut Commands, state: GameState, view: Transform) -> Entity {
    commands.spawn((
        Name::new("Cinematic UI camera"),
        Camera2d,
        IsDefaultUiCamera,
        Camera {
            clear_color: ClearColorConfig::None,
            order: 1,
            ..default()
        },
        DespawnOnExit(state),
    ));
    commands
        .spawn((
            Name::new("Cinematic camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 55.0_f32.to_radians(),
                ..default()
            }),
            Camera {
                clear_color: ClearColorConfig::Custom(SHADE),
                order: 0,
                ..default()
            },
            view,
            game_audio::spatial_listener(0.18),
            DespawnOnExit(state),
        ))
        .id()
}

pub(super) fn overlay(
    state: GameState,
    name: &'static str,
    title: &'static str,
    shade: f32,
    shade_secs: f32,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Name::new(name),
        Cinematic {
            elapsed: 0.0,
            shade,
            shade_secs,
            revealed: false,
            font: font.clone(),
        },
        DespawnOnExit(state),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::left(px(96)),
            row_gap: px(12),
            ..default()
        },
        BackgroundColor(SHADE.with_alpha(0.0)),
        children![
            (
                CinematicTitle,
                text(title, 64.0, theme::ACCENT.with_alpha(0.0), font),
                Node {
                    margin: UiRect::bottom(px(28)),
                    ..default()
                },
            ),
            (
                CinematicOptions,
                Node {
                    width: px(240),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
            ),
        ],
    )
}

fn advance(
    time: Res<Time>,
    mut screens: Query<(&mut Cinematic, &mut BackgroundColor, Has<DeathTint>)>,
    mut titles: Query<&mut TextColor, With<CinematicTitle>>,
) {
    for (mut cinematic, mut background, death) in &mut screens {
        cinematic.elapsed += time.delta_secs();
        let shade = (cinematic.elapsed / cinematic.shade_secs).clamp(0.0, 1.0);
        let color = if death { DEATH_SHADE } else { SHADE };
        background.0 = color.with_alpha(cinematic.shade * shade);
        let title = ((cinematic.elapsed - TITLE_AT) / TITLE_FADE).clamp(0.0, 1.0);
        for mut color in &mut titles {
            color.0 = theme::ACCENT.with_alpha(title);
        }
    }
}

fn move_cameras(
    screens: Query<&Cinematic>,
    mut cameras: Query<(&CinematicCamera, &mut Transform)>,
) {
    let Some(elapsed) = screens.iter().map(|cinematic| cinematic.elapsed).next() else {
        return;
    };
    for (camera, mut transform) in &mut cameras {
        let t = (elapsed / camera.secs).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        transform.translation = camera.from.translation.lerp(camera.to.translation, t);
        transform.rotation = camera.from.rotation.slerp(camera.to.rotation, t);
    }
}

fn reveal_options(
    mut screens: Query<&mut Cinematic>,
    slots: Query<Entity, With<CinematicOptions>>,
    mut commands: Commands,
) {
    for mut cinematic in &mut screens {
        if cinematic.revealed || cinematic.elapsed < REVEAL_AT {
            continue;
        }
        cinematic.revealed = true;
        let font = cinematic.font.clone();
        for slot in &slots {
            commands.entity(slot).with_children(|options| {
                options.spawn((
                    Name::new("Retry button"),
                    MenuAction::Retry,
                    menu_button("Retry", font.clone()),
                ));
                options.spawn((
                    Name::new("Main menu button"),
                    MenuAction::MainMenu,
                    menu_button("Main Menu", font.clone()),
                ));
                options.spawn((
                    Name::new("Quit button"),
                    MenuAction::Quit,
                    menu_button("Quit", font.clone()),
                ));
            });
        }
    }
}
