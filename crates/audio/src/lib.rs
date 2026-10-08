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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    DoorUnlatch,
    DoorSwing,
    DoorShut,
    DoorLocked,
    FuseSlot(usize),
    FuseComplete,
    BoilerTick,
    FaucetBurst,
    LockerOpen,
    LockerClose,
    TableEnter,
    TableLeave,
    Step(usize),
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

#[cfg(test)]
#[path = "../tests/unit/support.rs"]
mod test_support;

#[cfg(test)]
#[path = "../tests/unit/playback.rs"]
mod tests;

#[derive(Resource, Default)]
pub struct AudioPaused(pub bool);

#[derive(Resource, Default)]
pub struct AmbienceActive(pub bool);

#[derive(Resource, Default)]
pub struct ConduitAmbience(pub bool);

#[derive(Component)]
struct WorldAudio;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .init_resource::<AudioPaused>()
            .init_resource::<AmbienceActive>()
            .init_resource::<ConduitAmbience>()
            .add_systems(
                Update,
                (update_pause, play_sounds, ambience::update_ambience).chain(),
            );
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

fn play_sounds(
    mut sounds: MessageReader<PlaySound>,
    assets: Option<Res<SoundAssets>>,
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
                | Sound::FaucetBurst
                | Sound::LockerOpen
                | Sound::LockerClose
                | Sound::TableEnter
                | Sound::TableLeave
                | Sound::Step(_)
        );
        if paused.0 && world_sound {
            continue;
        }
        let handle = match cue.sound {
            Sound::DoorUnlatch => &assets.door_unlatch,
            Sound::DoorSwing => &assets.door_swing,
            Sound::DoorShut => &assets.door_shut,
            Sound::DoorLocked => &assets.door_locked,
            Sound::FuseSlot(1) => &assets.fuse_slot_1,
            Sound::FuseSlot(2) => &assets.fuse_slot_2,
            Sound::FuseSlot(3) => &assets.fuse_slot_3,
            Sound::FuseSlot(_) => continue,
            Sound::FuseComplete => &assets.fuse_complete,
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
            Sound::UiBack => &assets.ui_back,
            Sound::UiConfirm => &assets.ui_confirm,
            Sound::UiDenied => &assets.ui_denied,
            Sound::UiFocus => &assets.ui_focus,
            Sound::UiHover => &assets.ui_hover,
            Sound::UiPause => &assets.ui_pause,
            Sound::UiPress => &assets.ui_press,
            Sound::UiResume => &assets.ui_resume,
        };
        if let Some(position) = cue.position {
            let Ok(listener) = listeners.single() else {
                continue;
            };
            let distance = listener.translation().distance(position);
            if distance > 25.0 {
                continue;
            }
            let gain = 0.6 / (1.0 + 0.06 * distance * distance);
            commands.spawn((
                WorldAudio,
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN
                    .with_volume(Volume::Linear(gain))
                    .with_spatial(true)
                    .with_spatial_scale(SpatialScale::new(0.3)),
                Transform::from_translation(position),
            ));
        } else if world_sound {
            commands.spawn((
                WorldAudio,
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.6)),
            ));
        } else {
            commands.spawn((
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.6)),
            ));
        }
    }
}
