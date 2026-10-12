use bevy::prelude::*;
use game_audio::{AmbienceActive, AmbientEmitter, AmbientEmitters, AmbientUpdate};

use super::{
    logic::{AmbientSource, AmbientSourceKind},
    sounds::{AmbienceSounds, AmbienceSoundsPlugin},
};
use crate::{
    facility::{FacilityPower, MainsPowered, Module, Room, Walls},
    player::PlayerController,
};

pub struct AmbienceViewPlugin;

impl Plugin for AmbienceViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AmbienceSoundsPlugin).add_systems(
            Update,
            sync_emitters
                .run_if(resource_exists::<AmbienceSounds>)
                .before(AmbientUpdate),
        );
    }
}

fn sync_emitters(
    player: Option<Single<&Transform, With<PlayerController>>>,
    rooms: Query<(&Room, &Walls)>,
    sources: Query<(Entity, &AmbientSource, Has<MainsPowered>)>,
    power: Res<FacilityPower>,
    sounds: Res<AmbienceSounds>,
    mut active: ResMut<AmbienceActive>,
    mut emitters: ResMut<AmbientEmitters>,
) {
    active.0 = player.is_some();
    let conduit = player.is_some_and(|player| {
        let position = player.translation.xz();
        rooms
            .iter()
            .any(|(room, walls)| walls.0 == Module::WallConduit && room.0.contains(position))
    });
    let mut current: Vec<_> = sources
        .iter()
        .filter(|(_, _, mains)| power.is_on() || !mains)
        .filter(|(_, source, _)| conduit || source.kind != AmbientSourceKind::Conduit)
        .map(|(entity, source, _)| AmbientEmitter {
            source: entity,
            handle: sounds.source(source.kind),
            volume: source.volume,
            spatial: source.kind.spatial(),
        })
        .collect();
    current.sort_by_key(|emitter| emitter.source.to_bits());
    if emitters.0 != current {
        emitters.0 = current;
    }
}
