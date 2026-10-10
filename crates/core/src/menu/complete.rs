use bevy::prelude::*;
use game_assets::UiAssets;
use gameplay::{
    controller::PlayerController,
    levels::{build_exit_cinematic, Escaped, ToggleDoor},
};

use super::{
    cinematic::{self, Cinematic, CinematicCamera, REVEAL_AT},
    credits::CreditsRoute,
    release_cursor, GameState,
};

pub(super) const DOOR_CLOSE_AT: f32 = 0.8;
const DRIFT: f32 = 0.6;
const DRIFT_SECS: f32 = 6.0;
const SHADE: f32 = 0.4;
const SHADE_SECS: f32 = 4.0;

#[derive(Component)]
pub(super) struct CompleteScreen;

#[derive(Component)]
struct ClosingDoor(Entity);

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ToggleDoor>()
        .add_systems(Update, finish_run.run_if(in_state(GameState::Playing)))
        .add_systems(
            OnEnter(GameState::Complete),
            (spawn_complete_screen, release_cursor),
        )
        .add_systems(
            Update,
            (close_door, roll_credits).run_if(in_state(GameState::Complete)),
        );
}

fn finish_run(
    players: Query<(), (With<PlayerController>, With<Escaped>)>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !players.is_empty() {
        next.set(GameState::Complete);
    }
}

fn spawn_complete_screen(mut commands: Commands, assets: Res<UiAssets>) {
    let scene = build_exit_cinematic(&mut commands);
    commands
        .entity(scene.root)
        .insert(DespawnOnExit(GameState::Complete));
    let camera = cinematic::spawn_cameras(&mut commands, GameState::Complete, scene.view);
    let back = scene.view.back() * DRIFT;
    commands.entity(camera).insert(CinematicCamera::new(
        scene.view,
        scene.view.with_translation(scene.view.translation + back),
        DRIFT_SECS,
    ));
    commands.spawn((
        CompleteScreen,
        ClosingDoor(scene.door),
        cinematic::overlay(
            GameState::Complete,
            "Complete screen",
            "ESCAPED",
            SHADE,
            SHADE_SECS,
            assets.font.clone(),
        ),
    ));
}

fn close_door(
    screens: Query<(Entity, &Cinematic, &ClosingDoor)>,
    mut toggles: MessageWriter<ToggleDoor>,
    mut commands: Commands,
) {
    for (entity, cinematic, door) in &screens {
        if cinematic.elapsed >= DOOR_CLOSE_AT {
            toggles.write(ToggleDoor(door.0));
            commands.entity(entity).remove::<ClosingDoor>();
        }
    }
}

fn roll_credits(
    screens: Query<&Cinematic, With<CompleteScreen>>,
    mut route: ResMut<CreditsRoute>,
    mut next: ResMut<NextState<GameState>>,
) {
    if screens
        .iter()
        .any(|cinematic| cinematic.elapsed >= REVEAL_AT)
    {
        *route = CreditsRoute::Ending;
        next.set(GameState::Credits);
    }
}
