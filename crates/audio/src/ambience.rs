use bevy::{
    audio::{PlaybackSettings, SpatialScale},
    prelude::*,
};
use game_assets::SoundAssets;

use super::{volume, AmbienceActive, AudioGain, AudioPaused, ConduitAmbience, WorldAudio};
use game_settings::GameSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmbientSound {
    Roomtone,
    LowPressure,
    Conduit,
    Boiler,
    Tank,
    Vent,
    VentWind,
    CoolBuzz,
}

#[derive(Clone, Copy)]
pub struct AmbientEmitter {
    pub source: Entity,
    pub sound: AmbientSound,
    pub spatial: bool,
    pub volume: f32,
}

#[derive(Resource)]
pub struct AmbientEmitters(pub Vec<AmbientEmitter>);

#[derive(Component)]
pub(super) struct AmbientVoice(Entity);

impl AmbientSound {
    pub fn spatial(self) -> bool {
        !matches!(self, Self::Roomtone | Self::LowPressure | Self::Conduit)
    }

    fn handle(self, assets: &SoundAssets) -> Handle<AudioSource> {
        match self {
            Self::Roomtone => assets.roomtone.clone(),
            Self::LowPressure => assets.low_pressure.clone(),
            Self::Conduit => assets.conduit_roomtone.clone(),
            Self::Boiler => assets.furnace.clone(),
            Self::Tank => assets.tank_hum.clone(),
            Self::Vent => assets.vent_hvac.clone(),
            Self::VentWind => assets.vent_wind.clone(),
            Self::CoolBuzz => assets.cool_buzz.clone(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/ambience.rs"]
mod tests;

pub(super) fn update_ambience(
    active: Res<AmbienceActive>,
    conduit: Res<ConduitAmbience>,
    paused: Res<AudioPaused>,
    assets: Option<Res<SoundAssets>>,
    settings: Option<Res<GameSettings>>,
    emitters: Option<Res<AmbientEmitters>>,
    playing: Query<(Entity, &AmbientVoice)>,
    mut commands: Commands,
) {
    let Some(emitters) = emitters else { return };
    if !active.0 {
        for (entity, _) in &playing {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Some(assets) = assets else { return };
    for (entity, voice) in &playing {
        if !emitters.0.iter().any(|source| {
            source.source == voice.0 && (source.sound != AmbientSound::Conduit || conduit.0)
        }) {
            commands.entity(entity).despawn();
        }
    }
    for source in &emitters.0 {
        if source.sound == AmbientSound::Conduit && !conduit.0 {
            continue;
        }
        if playing.iter().any(|(_, voice)| voice.0 == source.source) {
            continue;
        }
        let mut playback =
            PlaybackSettings::LOOP.with_volume(volume(settings.as_deref(), source.volume));
        if source.spatial {
            playback = playback
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new(0.3));
        }
        if paused.0 {
            playback = playback.paused();
        }
        if let Ok(mut parent) = commands.get_entity(source.source) {
            parent.with_children(|children| {
                children.spawn((
                    AmbientVoice(source.source),
                    WorldAudio,
                    AudioGain(source.volume),
                    AudioPlayer::new(source.sound.handle(&assets)),
                    playback,
                    Transform::IDENTITY,
                ));
            });
        }
    }
}
