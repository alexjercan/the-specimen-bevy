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
    FusePickup,
    PanelInstall,
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

#[derive(Resource, Default)]
pub struct AudioPaused(pub bool);

#[derive(Resource, Default)]
pub struct AmbienceActive(pub bool);

#[derive(Component)]
struct WorldAudio;

#[derive(Component)]
struct Roomtone;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .init_resource::<AudioPaused>()
            .init_resource::<AmbienceActive>()
            .add_systems(Update, (update_pause, play_sounds, update_ambience).chain());
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
    for mut sink in &mut sinks {
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
    for mut sink in &mut spatial {
        if paused.0 {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

fn update_ambience(
    active: Res<AmbienceActive>,
    paused: Res<AudioPaused>,
    assets: Option<Res<SoundAssets>>,
    players: Query<Entity, With<Roomtone>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else { return };
    if active.0 && players.is_empty() {
        let mut settings = PlaybackSettings::LOOP.with_volume(Volume::Linear(0.12));
        if paused.0 {
            settings = settings.paused();
        }
        commands.spawn((
            Roomtone,
            WorldAudio,
            AudioPlayer::new(assets.roomtone.clone()),
            settings,
        ));
    } else if !active.0 {
        for entity in &players {
            commands.entity(entity).despawn();
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
                | Sound::FusePickup
                | Sound::PanelInstall
                | Sound::Step(_)
        );
        if paused.0 && world_sound {
            continue;
        }
        let handle = match cue.sound {
            Sound::DoorUnlatch => &assets.door_unlatch,
            Sound::DoorSwing => &assets.door_swing,
            Sound::DoorShut => &assets.door_shut,
            Sound::FusePickup => &assets.fuse_pickup,
            Sound::PanelInstall => &assets.panel_install,
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
            let Ok(listener) = listeners.single() else { continue };
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
