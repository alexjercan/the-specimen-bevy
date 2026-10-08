use bevy::prelude::*;
use game_audio::{Sound, SourceSounds};

use super::{
    builder::Prop,
    module_names::{
        BOILER_UNIT, CEILING_LIGHT_COOL, CONCEPT_CONTAINMENT_TANK, PIPE_MANIFOLD, WALL_VENT,
    },
};

pub use game_audio::{AmbientSound as AmbientSourceKind, Sound as IntermittentSoundKind};

#[derive(Component, Clone, Copy, Debug)]
pub struct AmbientSource {
    pub kind: AmbientSourceKind,
    pub volume: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct IntermittentSound {
    pub kind: IntermittentSoundKind,
    pub offset: Vec3,
    pub range: f32,
    pub interval: f32,
    pub variation: f32,
    pub remaining: f32,
}

pub struct PropSoundsPlugin;

impl Plugin for PropSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_prop_sounds);
    }
}

fn attach_prop_sounds(added: On<Add, Prop>, props: Query<&Prop>, mut commands: Commands) {
    let Ok(prop) = props.get(added.entity) else {
        return;
    };
    let mut entity = commands.entity(added.entity);
    match prop.0.as_str() {
        WALL_VENT => {
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::Vent,
                        volume: 0.09,
                    },
                    Transform::IDENTITY,
                ));
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::VentWind,
                        volume: 0.035,
                    },
                    Transform::IDENTITY,
                ));
            });
        }
        CEILING_LIGHT_COOL => {
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::CoolBuzz,
                        volume: 0.045,
                    },
                    Transform::from_xyz(0.0, 2.7, 0.0),
                ));
            });
        }
        BOILER_UNIT => {
            entity.insert((
                SourceSounds(vec![(Sound::BoilerRestart, Vec3::Y * 1.2)]),
                IntermittentSound {
                    kind: IntermittentSoundKind::BoilerTick,
                    offset: Vec3::Y * 1.2,
                    range: 18.0,
                    interval: 6.0,
                    variation: 14.0,
                    remaining: 8.0,
                },
            ));
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::Boiler,
                        volume: 0.17,
                    },
                    Transform::from_xyz(0.0, 1.0, 0.0),
                ));
            });
        }
        PIPE_MANIFOLD => {
            entity.insert(IntermittentSound {
                kind: IntermittentSoundKind::FaucetBurst,
                offset: Vec3::new(0.0, -0.1, -0.25),
                range: 12.0,
                interval: 10.0,
                variation: 8.0,
                remaining: 10.0,
            });
        }
        CONCEPT_CONTAINMENT_TANK => {
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::Tank,
                        volume: 0.15,
                    },
                    Transform::from_xyz(0.0, 1.3, 0.0),
                ));
            });
        }
        _ => {}
    }
}
