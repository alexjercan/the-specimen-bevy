use bevy::prelude::*;

use crate::{monster::Caught, player::PlayerController};

use super::{Door, DoorLock, ExitDoor, Room};

const EXIT_REACH: f32 = 2.5;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Escaped;

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ObjectiveSystems;

pub struct ObjectivePlugin;

impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, escape.in_set(ObjectiveSystems));
    }
}

fn escape(
    exits: Query<&Door, (With<ExitDoor>, Without<DoorLock>)>,
    rooms: Query<&Room>,
    players: Query<
        (Entity, &Transform),
        (With<PlayerController>, Without<Escaped>, Without<Caught>),
    >,
    mut commands: Commands,
) {
    for (entity, player) in &players {
        let position = player.translation.xz();
        if rooms.iter().any(|room| room.0.contains(position)) {
            continue;
        }
        if exits
            .iter()
            .any(|door| door.position.distance(position) <= EXIT_REACH)
        {
            commands.entity(entity).insert(Escaped);
        }
    }
}
