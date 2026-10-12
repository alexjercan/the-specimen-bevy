use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use game_audio::Falloff;
use rand_core::Rng;

use crate::{
    audible::{Audible, Origin},
    facility::{FacilityPower, MainsPowered, Module, Prop},
    player::PlayerController,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmbientSourceKind {
    Roomtone,
    LowPressure,
    Conduit,
    Boiler,
    Tank,
    Vent,
    VentWind,
    CoolBuzz,
}

impl AmbientSourceKind {
    pub fn spatial(self) -> bool {
        !matches!(self, Self::Roomtone | Self::LowPressure | Self::Conduit)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct AmbientSource {
    pub kind: AmbientSourceKind,
    pub volume: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmbientCueKind {
    BoilerTick,
    FaucetBurst,
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct AmbientCue {
    pub kind: AmbientCueKind,
    pub prop: Entity,
    pub offset: Vec3,
}

impl Audible for AmbientCue {
    fn name(&self) -> Option<&'static str> {
        Some(match self.kind {
            AmbientCueKind::BoilerTick => "boiler_tick",
            AmbientCueKind::FaucetBurst => "faucet_burst",
        })
    }

    fn origin(&self) -> Origin {
        Origin::From(self.prop)
    }

    fn falloff(&self) -> Falloff {
        Falloff::DEFAULT
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct IntermittentSound {
    pub kind: AmbientCueKind,
    pub offset: Vec3,
    pub range: f32,
    pub interval: f32,
    pub variation: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct IntermittentTimer {
    pub remaining: f32,
}

pub struct AmbiencePlugin;

impl Plugin for AmbiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AmbientCue>()
            .add_observer(attach_prop_sounds)
            .add_systems(Update, play_intermittent_sounds);
    }
}

fn attach_prop_sounds(added: On<Add, Prop>, props: Query<&Prop>, mut commands: Commands) {
    let Ok(prop) = props.get(added.entity) else {
        return;
    };
    let mut entity = commands.entity(added.entity);
    match prop.0 {
        Module::WallVent => {
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::Vent,
                        volume: 0.09,
                    },
                    MainsPowered,
                    Transform::IDENTITY,
                ));
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::VentWind,
                        volume: 0.035,
                    },
                    MainsPowered,
                    Transform::IDENTITY,
                ));
            });
        }
        Module::CeilingLightCool => {
            entity.with_children(|children| {
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::CoolBuzz,
                        volume: 0.045,
                    },
                    MainsPowered,
                    Transform::from_xyz(0.0, 2.7, 0.0),
                ));
            });
        }
        Module::BoilerUnit => {
            entity.with_children(|children| {
                children.spawn((
                    IntermittentSound {
                        kind: AmbientCueKind::BoilerTick,
                        offset: Vec3::Y * 1.2,
                        range: 18.0,
                        interval: 6.0,
                        variation: 14.0,
                    },
                    IntermittentTimer { remaining: 8.0 },
                    MainsPowered,
                ));
                children.spawn((
                    AmbientSource {
                        kind: AmbientSourceKind::Boiler,
                        volume: 0.17,
                    },
                    MainsPowered,
                    Transform::from_xyz(0.0, 1.0, 0.0),
                ));
            });
        }
        Module::PipeManifold => {
            entity.with_children(|children| {
                children.spawn((
                    IntermittentSound {
                        kind: AmbientCueKind::FaucetBurst,
                        offset: Vec3::new(0.0, -0.1, -0.25),
                        range: 12.0,
                        interval: 10.0,
                        variation: 8.0,
                    },
                    IntermittentTimer { remaining: 10.0 },
                ));
            });
        }
        Module::ConceptContainmentTank => {
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

// NOTE: the pause menu stops virtual time, which freezes these timers; no audio pause check here.
fn play_intermittent_sounds(
    time: Res<Time>,
    power: Res<FacilityPower>,
    player: Single<&GlobalTransform, With<PlayerController>>,
    props: Query<&GlobalTransform>,
    mut sources: Query<(
        &ChildOf,
        &IntermittentSound,
        &mut IntermittentTimer,
        Has<MainsPowered>,
    )>,
    mut rng: Single<&mut ChaCha8Rng, With<GlobalRng>>,
    mut cues: MessageWriter<AmbientCue>,
) {
    for (prop, source, mut timer, mains) in &mut sources {
        if mains && !power.is_on() {
            continue;
        }
        timer.remaining -= time.delta_secs();
        if timer.remaining > 0.0 {
            continue;
        }
        timer.remaining = source.interval
            + source.variation * (rng.next_u32() as f64 / (u32::MAX as f64 + 1.0)) as f32;
        let Ok(transform) = props.get(prop.parent()) else {
            continue;
        };
        let at = transform.transform_point(source.offset);
        if player.translation().distance(at) > source.range {
            continue;
        }
        cues.write(AmbientCue {
            kind: source.kind,
            prop: prop.parent(),
            offset: source.offset,
        });
    }
}
