use bevy::{
    audio::{PlaybackSettings, SpatialScale, Volume},
    prelude::*,
};
use game_assets::SoundAssets;

use super::{AmbienceActive, AudioPaused, ConduitAmbience, WorldAudio};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AmbientSound {
    Roomtone,
    Conduit,
    Boiler,
    Tank,
    Vent,
    CoolBuzz,
}

#[derive(Clone, Copy)]
pub struct AmbientEmitter {
    pub sound: AmbientSound,
    pub position: Option<Vec3>,
    pub volume: f32,
}

#[derive(Resource)]
pub struct AmbientEmitters(pub Vec<AmbientEmitter>);

#[derive(Component)]
struct AmbientVoice(usize);

impl AmbientSound {
    fn handle(self, assets: &SoundAssets) -> Handle<AudioSource> {
        match self {
            Self::Roomtone => assets.roomtone.clone(),
            Self::Conduit => assets.conduit_roomtone.clone(),
            Self::Boiler => assets.boiler.clone(),
            Self::Tank => assets.tank_hum.clone(),
            Self::Vent => assets.vent_hvac.clone(),
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
    for (index, source) in emitters.0.iter().enumerate() {
        let enabled = source.sound != AmbientSound::Conduit || conduit.0;
        let existing = playing.iter().find(|(_, voice)| voice.0 == index);
        if !enabled {
            if let Some((entity, _)) = existing {
                commands.entity(entity).despawn();
            }
            continue;
        }
        if existing.is_some() {
            continue;
        }
        let mut settings = PlaybackSettings::LOOP.with_volume(Volume::Linear(source.volume));
        if source.position.is_some() {
            settings = settings
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new(0.3));
        }
        if paused.0 {
            settings = settings.paused();
        }
        let mut entity = commands.spawn((
            AmbientVoice(index),
            WorldAudio,
            AudioPlayer::new(source.sound.handle(&assets)),
            settings,
        ));
        if let Some(position) = source.position {
            entity.insert(Transform::from_translation(position));
        }
    }
}
