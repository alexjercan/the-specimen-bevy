mod ambience;

pub use ambience::{AmbientEmitter, AmbientEmitters, AmbientSound};

use bevy::{
    audio::{
        AudioSink, AudioSinkPlayback, PlaybackSettings, SpatialAudioSink, SpatialListener,
        SpatialScale, Volume,
    },
    prelude::*,
};
use game_assets::SoundAssets;
use game_settings::GameSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    DoorUnlatch,
    DoorSwing,
    DoorShut,
    DoorLocked,
    FuseSlot(usize),
    FuseComplete,
    FlashlightClick,
    SprintExhausted,
    PowerDown,
    BreakerTrip,
    BoilerReset,
    BoilerRestart,
    BoilerTick,
    FaucetBurst,
    LockerOpen,
    LockerClose,
    TableEnter,
    TableLeave,
    Step(usize),
    MonsterPresence,
    MonsterStep(usize),
    MonsterDetected,
    MonsterAttack,
    MonsterHeartbeat,
    UiBack,
    UiConfirm,
    UiDenied,
    UiFocus,
    UiHover,
    UiPause,
    UiPress,
    UiResume,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySound {
    pub sound: Sound,
    pub position: Option<Vec3>,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySoundFrom {
    pub sound: Sound,
    pub source: Entity,
    pub offset: Vec3,
}

#[derive(Component, Clone, Debug)]
pub struct SourceSounds(pub Vec<(Sound, Vec3)>);

#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySourceSound {
    pub source: Entity,
    pub sound: Sound,
}

#[cfg(test)]
#[path = "../tests/unit/support.rs"]
mod test_support;

#[cfg(test)]
#[path = "../tests/unit/playback.rs"]
mod tests;

impl Sound {
    pub fn requires_power(self) -> bool {
        matches!(self, Self::BoilerTick)
    }

    pub fn audible_range(self) -> f32 {
        match self {
            Self::MonsterStep(_) => 16.0,
            Self::MonsterPresence => 32.0,
            _ => 25.0,
        }
    }

    fn spatial_gain(self, distance: f32) -> f32 {
        match self {
            Self::MonsterStep(_) | Self::MonsterPresence => {
                let fade = (1.0 - distance / self.audible_range()).max(0.0);
                0.8 * fade * fade
            }
            Self::MonsterDetected => 1.0 / (1.0 + 0.015 * distance * distance),
            _ => 0.6 / (1.0 + 0.06 * distance * distance),
        }
    }

    fn spatial_scale(self) -> SpatialScale {
        match self {
            Self::MonsterStep(_) | Self::MonsterPresence => SpatialScale::new(0.12),
            _ => SpatialScale::new(0.3),
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

#[derive(Resource, Default)]
pub struct AmbienceActive(pub bool);

#[derive(Resource, Default)]
pub struct ConduitAmbience(pub bool);

#[derive(Component)]
struct WorldAudio;

#[derive(Component, Clone, Copy)]
struct AudioGain(f32);

fn volume(settings: Option<&GameSettings>, base: f32) -> Volume {
    Volume::Linear(base * settings.map_or(1.0, |settings| settings.master * settings.sfx))
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AmbientUpdate;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .add_message::<PlaySoundFrom>()
            .add_message::<PlaySourceSound>()
            .init_resource::<AudioPaused>()
            .init_resource::<AmbienceActive>()
            .init_resource::<ConduitAmbience>()
            .add_systems(
                Update,
                (
                    update_pause,
                    update_volume,
                    play_sounds,
                    play_authored_sounds,
                    play_source_sounds,
                    ambience::update_ambience.in_set(AmbientUpdate),
                )
                    .chain(),
            );
    }
}

fn update_volume(
    settings: Option<Res<GameSettings>>,
    mut regular: Query<(&AudioGain, &mut AudioSink)>,
    mut spatial: Query<(&AudioGain, &mut SpatialAudioSink)>,
) {
    if !settings
        .as_ref()
        .is_some_and(|settings| settings.is_changed())
    {
        return;
    }
    for (gain, mut sink) in &mut regular {
        sink.set_volume(volume(settings.as_deref(), gain.0));
    }
    for (gain, mut sink) in &mut spatial {
        sink.set_volume(volume(settings.as_deref(), gain.0));
    }
}

fn update_pause(
    paused: Res<AudioPaused>,
    mut sinks: Query<&mut AudioSink, With<WorldAudio>>,
    mut spatial: Query<&mut SpatialAudioSink, With<WorldAudio>>,
) {
    if !paused.is_changed() {
        return;
    }
    for sink in &mut sinks {
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
    for sink in &mut spatial {
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

fn sound_handle(sound: Sound, assets: &SoundAssets) -> Option<&Handle<AudioSource>> {
    Some(match sound {
        Sound::DoorUnlatch => &assets.door_unlatch,
        Sound::DoorSwing => &assets.door_swing,
        Sound::DoorShut => &assets.door_shut,
        Sound::DoorLocked => &assets.door_locked,
        Sound::FuseSlot(1) => &assets.fuse_slot_1,
        Sound::FuseSlot(2) => &assets.fuse_slot_2,
        Sound::FuseSlot(3) => &assets.fuse_slot_3,
        Sound::FuseSlot(_) => return None,
        Sound::FuseComplete => &assets.fuse_complete,
        Sound::FlashlightClick => &assets.flashlight_click,
        Sound::SprintExhausted => &assets.sprint_exhausted,
        Sound::PowerDown => &assets.power_down,
        Sound::BreakerTrip => &assets.breaker_trip,
        Sound::BoilerReset => &assets.boiler_reset,
        Sound::BoilerRestart => &assets.boiler_restart,
        Sound::BoilerTick => &assets.boiler_tick,
        Sound::FaucetBurst => &assets.faucet,
        Sound::LockerOpen => &assets.locker_open,
        Sound::LockerClose => &assets.locker_close,
        Sound::TableEnter => &assets.table_enter,
        Sound::TableLeave => &assets.table_leave,
        Sound::Step(index) => match index % 3 {
            0 => &assets.step_01,
            1 => &assets.step_02,
            _ => &assets.step_04,
        },
        Sound::MonsterPresence => &assets.monster_presence,
        Sound::MonsterDetected => &assets.monster_detected,
        Sound::MonsterAttack => &assets.monster_attack,
        Sound::MonsterHeartbeat => &assets.monster_heartbeat,
        Sound::MonsterStep(index) => match index % 6 {
            0 => &assets.monster_step_a,
            1 => &assets.monster_step_b,
            2 => &assets.monster_step_c,
            3 => &assets.monster_step_d,
            4 => &assets.monster_step_e,
            _ => &assets.monster_step_f,
        },
        Sound::UiBack => &assets.ui_back,
        Sound::UiConfirm => &assets.ui_confirm,
        Sound::UiDenied => &assets.ui_denied,
        Sound::UiFocus => &assets.ui_focus,
        Sound::UiHover => &assets.ui_hover,
        Sound::UiPause => &assets.ui_pause,
        Sound::UiPress => &assets.ui_press,
        Sound::UiResume => &assets.ui_resume,
    })
}

fn play_source_sounds(
    mut sounds: MessageReader<PlaySoundFrom>,
    assets: Option<Res<SoundAssets>>,
    settings: Option<Res<GameSettings>>,
    paused: Res<AudioPaused>,
    sources: Query<&GlobalTransform>,
    listeners: Query<&GlobalTransform, With<SpatialListener>>,
    mut commands: Commands,
) {
    for cue in sounds.read() {
        if paused.0 {
            continue;
        }
        let (Some(assets), Ok(listener)) = (assets.as_deref(), listeners.single()) else {
            continue;
        };
        let (Ok(source), Some(handle)) =
            (sources.get(cue.source), sound_handle(cue.sound, &assets))
        else {
            continue;
        };
        let position = source.transform_point(cue.offset);
        let distance = listener.translation().distance(position);
        if distance >= cue.sound.audible_range() {
            continue;
        }
        let gain = cue.sound.spatial_gain(distance);
        if let Ok(mut parent) = commands.get_entity(cue.source) {
            parent.with_children(|children| {
                children.spawn((
                    WorldAudio,
                    AudioGain(gain),
                    AudioPlayer::new(handle.clone()),
                    PlaybackSettings::DESPAWN
                        .with_volume(volume(settings.as_deref(), gain))
                        .with_spatial(true)
                        .with_spatial_scale(cue.sound.spatial_scale()),
                    Transform::from_translation(cue.offset),
                ));
            });
        }
    }
}

fn play_authored_sounds(
    mut sounds: MessageReader<PlaySourceSound>,
    sources: Query<&SourceSounds>,
    mut outgoing: MessageWriter<PlaySoundFrom>,
) {
    for cue in sounds.read() {
        let Ok(config) = sources.get(cue.source) else {
            continue;
        };
        let Some((_, offset)) = config.0.iter().find(|(sound, _)| *sound == cue.sound) else {
            continue;
        };
        outgoing.write(PlaySoundFrom {
            sound: cue.sound,
            source: cue.source,
            offset: *offset,
        });
    }
}

fn play_sounds(
    mut sounds: MessageReader<PlaySound>,
    assets: Option<Res<SoundAssets>>,
    settings: Option<Res<GameSettings>>,
    paused: Res<AudioPaused>,
    listeners: Query<&GlobalTransform, With<SpatialListener>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else { return };
    for cue in sounds.read() {
        let world_sound = matches!(
            cue.sound,
            Sound::DoorUnlatch
                | Sound::DoorSwing
                | Sound::DoorShut
                | Sound::DoorLocked
                | Sound::BoilerTick
                | Sound::PowerDown
                | Sound::BreakerTrip
                | Sound::BoilerReset
                | Sound::BoilerRestart
                | Sound::FaucetBurst
                | Sound::LockerOpen
                | Sound::LockerClose
                | Sound::TableEnter
                | Sound::TableLeave
                | Sound::Step(_)
                | Sound::MonsterStep(_)
                | Sound::MonsterPresence
                | Sound::MonsterDetected
                | Sound::MonsterAttack
                | Sound::MonsterHeartbeat
                | Sound::SprintExhausted
        );
        if paused.0 && world_sound {
            continue;
        }
        let Some(handle) = sound_handle(cue.sound, &assets) else {
            continue;
        };
        if let Some(position) = cue.position {
            let Ok(listener) = listeners.single() else {
                continue;
            };
            let distance = listener.translation().distance(position);
            if distance >= cue.sound.audible_range() {
                continue;
            }
            let gain = cue.sound.spatial_gain(distance);
            commands.spawn((
                WorldAudio,
                AudioGain(gain),
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN
                    .with_volume(volume(settings.as_deref(), gain))
                    .with_spatial(true)
                    .with_spatial_scale(cue.sound.spatial_scale()),
                Transform::from_translation(position),
            ));
        } else if world_sound {
            commands.spawn((
                WorldAudio,
                AudioGain(0.6),
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(volume(settings.as_deref(), 0.6)),
            ));
        } else {
            commands.spawn((
                AudioGain(0.6),
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(volume(settings.as_deref(), 0.6)),
            ));
        }
    }
}
