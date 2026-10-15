mod ambience;

pub use ambience::{AmbienceActive, AmbientEmitter, AmbientEmitters, AmbientUpdate};

use bevy::{
    audio::{
        AudioSink, AudioSinkPlayback, PlaybackSettings, SpatialAudioSink, SpatialListener,
        SpatialScale, Volume,
    },
    prelude::*,
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bus {
    World,
    Ui,
    Music,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct Volumes {
    pub master: f32,
    pub world: f32,
    pub ui: f32,
    pub music: f32,
}

impl Default for Volumes {
    fn default() -> Self {
        Self {
            master: 1.0,
            world: 1.0,
            ui: 1.0,
            music: 1.0,
        }
    }
}

impl Volumes {
    pub fn gain(&self, bus: Bus) -> f32 {
        self.master
            * match bus {
                Bus::World => self.world,
                Bus::Ui => self.ui,
                Bus::Music => self.music,
            }
    }
}

pub const LISTENER_GAIN: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    InverseSquare { gain: f32, coefficient: f32 },
    SquaredFade { gain: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Falloff {
    pub range: f32,
    pub scale: f32,
    pub curve: Curve,
}

impl Falloff {
    pub const DEFAULT: Self = Self {
        range: 25.0,
        scale: 0.3,
        curve: Curve::InverseSquare {
            gain: 0.6,
            coefficient: 0.06,
        },
    };

    pub fn gain(&self, distance: f32) -> Option<f32> {
        if distance >= self.range {
            return None;
        }
        Some(match self.curve {
            Curve::InverseSquare { gain, coefficient } => {
                gain / (1.0 + coefficient * distance * distance)
            }
            Curve::SquaredFade { gain } => {
                let fade = (1.0 - distance / self.range).max(0.0);
                gain * fade * fade
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    Global {
        gain: f32,
    },
    Loop {
        gain: f32,
    },
    At {
        position: Vec3,
        falloff: Falloff,
    },
    From {
        source: Entity,
        offset: Vec3,
        falloff: Falloff,
    },
}

#[derive(Message, Clone, Debug)]
pub struct PlaySound {
    pub handle: Handle<AudioSource>,
    pub bus: Bus,
    pub placement: Placement,
}

impl PlaySound {
    pub fn global(handle: Handle<AudioSource>, bus: Bus, gain: f32) -> Self {
        Self {
            handle,
            bus,
            placement: Placement::Global { gain },
        }
    }

    pub fn loop_global(handle: Handle<AudioSource>, bus: Bus, gain: f32) -> Self {
        Self {
            handle,
            bus,
            placement: Placement::Loop { gain },
        }
    }

    pub fn at(handle: Handle<AudioSource>, position: Vec3, falloff: Falloff) -> Self {
        Self {
            handle,
            bus: Bus::World,
            placement: Placement::At { position, falloff },
        }
    }

    pub fn from(
        handle: Handle<AudioSource>,
        source: Entity,
        offset: Vec3,
        falloff: Falloff,
    ) -> Self {
        Self {
            handle,
            bus: Bus::World,
            placement: Placement::From {
                source,
                offset,
                falloff,
            },
        }
    }
}

pub fn spatial_listener(ear_gap: f32) -> SpatialListener {
    SpatialListener {
        left_ear_offset: Vec3::X * (ear_gap / 2.0),
        right_ear_offset: Vec3::NEG_X * (ear_gap / 2.0),
    }
}

#[derive(Resource, Default)]
pub struct AudioPaused(pub bool);

#[derive(Component, Clone, Copy)]
struct AudioGain(f32);

fn volume(volumes: &Volumes, bus: Bus, base: f32) -> Volume {
    Volume::Linear(base * volumes.gain(bus))
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlaySoundsSet;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .init_resource::<Volumes>()
            .init_resource::<AudioPaused>()
            .init_resource::<AmbienceActive>()
            .init_resource::<AmbientEmitters>()
            .add_systems(
                Update,
                (
                    update_pause,
                    update_volume,
                    play_sounds.in_set(PlaySoundsSet),
                    ambience::update_ambience.in_set(AmbientUpdate),
                )
                    .chain(),
            );
    }
}

fn update_volume(
    volumes: Res<Volumes>,
    mut regular: Query<(&AudioGain, &Bus, &mut AudioSink)>,
    mut spatial: Query<(&AudioGain, &Bus, &mut SpatialAudioSink)>,
) {
    if !volumes.is_changed() {
        return;
    }
    for (gain, bus, mut sink) in &mut regular {
        sink.set_volume(volume(&volumes, *bus, gain.0));
    }
    for (gain, bus, mut sink) in &mut spatial {
        sink.set_volume(volume(&volumes, *bus, gain.0));
    }
}

fn update_pause(
    paused: Res<AudioPaused>,
    sinks: Query<(&Bus, Ref<AudioSink>)>,
    spatial: Query<(&Bus, Ref<SpatialAudioSink>)>,
) {
    let changed = paused.is_changed();
    for (bus, sink) in &sinks {
        if *bus != Bus::World || !(changed || (paused.0 && sink.is_added())) {
            continue;
        }
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
    for (bus, sink) in &spatial {
        if *bus != Bus::World || !(changed || (paused.0 && sink.is_added())) {
            continue;
        }
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

fn play_sounds(
    mut sounds: MessageReader<PlaySound>,
    volumes: Res<Volumes>,
    paused: Res<AudioPaused>,
    sources: Query<&GlobalTransform>,
    listeners: Query<&GlobalTransform, With<SpatialListener>>,
    mut commands: Commands,
) {
    for cue in sounds.read() {
        if paused.0 && cue.bus == Bus::World {
            continue;
        }
        match cue.placement {
            Placement::Global { gain } => {
                commands.spawn((
                    cue.bus,
                    AudioGain(gain),
                    AudioPlayer::new(cue.handle.clone()),
                    PlaybackSettings::DESPAWN.with_volume(volume(&volumes, cue.bus, gain)),
                ));
            }
            Placement::Loop { gain } => {
                commands.spawn((
                    cue.bus,
                    AudioGain(gain),
                    AudioPlayer::new(cue.handle.clone()),
                    PlaybackSettings::LOOP.with_volume(volume(&volumes, cue.bus, gain)),
                ));
            }
            Placement::At { position, falloff } => {
                let Ok(listener) = listeners.single() else {
                    continue;
                };
                let Some(gain) = falloff.gain(listener.translation().distance(position)) else {
                    continue;
                };
                commands.spawn((
                    cue.bus,
                    AudioGain(gain),
                    AudioPlayer::new(cue.handle.clone()),
                    PlaybackSettings::DESPAWN
                        .with_volume(volume(&volumes, cue.bus, gain))
                        .with_spatial(true)
                        .with_spatial_scale(SpatialScale::new(falloff.scale)),
                    Transform::from_translation(position),
                ));
            }
            Placement::From {
                source,
                offset,
                falloff,
            } => {
                let (Ok(listener), Ok(transform)) = (listeners.single(), sources.get(source))
                else {
                    continue;
                };
                let position = transform.transform_point(offset);
                let Some(gain) = falloff.gain(listener.translation().distance(position)) else {
                    continue;
                };
                let Ok(mut parent) = commands.get_entity(source) else {
                    continue;
                };
                let bus = cue.bus;
                let handle = cue.handle.clone();
                let settings = PlaybackSettings::DESPAWN
                    .with_volume(volume(&volumes, bus, gain))
                    .with_spatial(true)
                    .with_spatial_scale(SpatialScale::new(falloff.scale));
                parent.with_children(|children| {
                    children.spawn((
                        bus,
                        AudioGain(gain),
                        AudioPlayer::new(handle),
                        settings,
                        Transform::from_translation(offset),
                    ));
                });
            }
        }
    }
}
