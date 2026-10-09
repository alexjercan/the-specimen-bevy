use bevy::prelude::*;
use game_assets::UiAssets;
use gameplay::{
    controller::PlayerController,
    levels::{Caught, Monster, MonsterFigure},
};

use super::{
    cinematic::{self, CinematicCamera, DeathTint},
    release_cursor, GameState,
};

const FALL_SECS: f32 = 1.1;
const FALL_HEIGHT: f32 = 0.35;
const FALL_ROLL: f32 = 0.45;
const SHADE: f32 = 0.97;
const SHADE_SECS: f32 = 2.4;
const GLOW: Color = Color::srgb(0.75, 0.08, 0.05);

#[derive(Component)]
pub(super) struct GameOverScreen;

#[derive(Resource, Clone, Copy)]
pub(super) struct CaughtShot {
    pub(super) view: Transform,
    pub(super) monster: Option<Transform>,
}

#[cfg(test)]
#[path = "../../tests/unit/death_view.rs"]
mod tests;

fn fallen_view(view: Transform) -> Transform {
    view.with_translation(view.translation.with_y(FALL_HEIGHT))
        .with_rotation(view.rotation * Quat::from_rotation_z(FALL_ROLL))
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, finish_run.run_if(in_state(GameState::Playing)))
        .add_systems(
            OnEnter(GameState::GameOver),
            (spawn_game_over, release_cursor),
        );
}

fn finish_run(
    players: Query<(&Caught, &Transform), With<PlayerController>>,
    monsters: Query<&Transform, With<Monster>>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let Some((caught, view)) = players.iter().find(|(caught, _)| caught.finished()) else {
        return;
    };
    commands.insert_resource(CaughtShot {
        view: *view,
        monster: monsters.get(caught.monster).ok().copied(),
    });
    next.set(GameState::GameOver);
}

fn spawn_game_over(mut commands: Commands, assets: Res<UiAssets>, shot: Option<Res<CaughtShot>>) {
    let view = shot.as_ref().map_or(Transform::IDENTITY, |shot| shot.view);
    let camera = cinematic::spawn_cameras(&mut commands, GameState::GameOver, view);
    let fallen = fallen_view(view);
    commands
        .entity(camera)
        .insert(CinematicCamera::new(view, fallen, FALL_SECS));
    if let Some(monster) = shot.as_ref().and_then(|shot| shot.monster) {
        commands.spawn((
            Name::new("Caught figure"),
            MonsterFigure,
            monster,
            DespawnOnExit(GameState::GameOver),
        ));
        commands.spawn((
            Name::new("Caught glow"),
            PointLight {
                color: GLOW,
                intensity: 6_000.0,
                range: 6.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(view.translation + view.down() * 0.6),
            DespawnOnExit(GameState::GameOver),
        ));
    }
    commands.remove_resource::<CaughtShot>();
    commands.spawn((
        GameOverScreen,
        DeathTint,
        cinematic::overlay(
            GameState::GameOver,
            "Game over screen",
            "CAUGHT",
            SHADE,
            SHADE_SECS,
            assets.font.clone(),
        ),
    ));
}
