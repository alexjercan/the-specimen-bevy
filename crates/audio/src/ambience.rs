use bevy::{
    audio::{PlaybackSettings, SpatialScale},
    prelude::*,
};

use super::{volume, AudioGain, AudioPaused, Bus, Volumes};

const AMBIENT_SPATIAL_SCALE: f32 = 0.3;

#[derive(Clone, Debug, PartialEq)]
pub struct AmbientEmitter {
    pub source: Entity,
    pub handle: Handle<AudioSource>,
    pub volume: f32,
    pub spatial: bool,
}

#[derive(Resource, Default)]
pub struct AmbientEmitters(pub Vec<AmbientEmitter>);

#[derive(Resource, Default)]
pub struct AmbienceActive(pub bool);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AmbientUpdate;

#[derive(Component)]
pub(super) struct AmbientVoice(Entity);

pub(super) fn update_ambience(
    active: Res<AmbienceActive>,
    paused: Res<AudioPaused>,
    volumes: Res<Volumes>,
    emitters: Res<AmbientEmitters>,
    playing: Query<(Entity, &AmbientVoice)>,
    mut commands: Commands,
) {
    if !active.0 {
        for (entity, _) in &playing {
            commands.entity(entity).despawn();
        }
        return;
    }
    for (entity, voice) in &playing {
        if !emitters.0.iter().any(|source| source.source == voice.0) {
            commands.entity(entity).despawn();
        }
    }
    for source in &emitters.0 {
        if playing.iter().any(|(_, voice)| voice.0 == source.source) {
            continue;
        }
        let mut playback =
            PlaybackSettings::LOOP.with_volume(volume(&volumes, Bus::World, source.volume));
        if source.spatial {
            playback = playback
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new(AMBIENT_SPATIAL_SCALE));
        }
        if paused.0 {
            playback = playback.paused();
        }
        if let Ok(mut parent) = commands.get_entity(source.source) {
            parent.with_children(|children| {
                children.spawn((
                    AmbientVoice(source.source),
                    Bus::World,
                    AudioGain(source.volume),
                    AudioPlayer::new(source.handle.clone()),
                    playback,
                    Transform::IDENTITY,
                ));
            });
        }
    }
}
