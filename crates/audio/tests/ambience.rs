mod common;

use bevy::{audio::PlaybackMode, prelude::*};
use game_audio::{AmbienceActive, AmbientEmitter, AmbientEmitters, AudioPaused, Bus, Volumes};

use common::{audio_app, clip};

fn ambience_app(sources: &[(u128, Option<Vec3>, f32)]) -> App {
    let mut app = audio_app();
    app.insert_resource(AmbienceActive(true));
    let emitters = sources
        .iter()
        .map(|&(id, position, volume)| {
            let source = app
                .world_mut()
                .spawn(Transform::from_translation(position.unwrap_or(Vec3::ZERO)))
                .id();
            AmbientEmitter {
                source,
                handle: clip(id),
                volume,
                spatial: position.is_some(),
            }
        })
        .collect();
    app.insert_resource(AmbientEmitters(emitters));
    app
}

fn voices(app: &mut App) -> Vec<(Entity, Handle<AudioSource>, PlaybackSettings)> {
    let world = app.world_mut();
    let mut query = world.query::<(&AudioPlayer, &PlaybackSettings, &ChildOf)>();
    let mut voices: Vec<_> = query
        .iter(world)
        .map(|(player, settings, parent)| (parent.parent(), player.0.clone(), *settings))
        .collect();
    voices.sort_by_key(|voice| voice.0.to_bits());
    voices
}

#[test]
fn ambience_plays_nothing_while_inactive() {
    let mut app = ambience_app(&[(1, None, 0.1), (2, Some(Vec3::X), 0.2)]);
    app.insert_resource(AmbienceActive(false));
    app.update();
    app.update();
    assert!(voices(&mut app).is_empty());
    assert!(!app.world().resource::<AmbienceActive>().0);
}

#[test]
fn ambience_is_inactive_by_default() {
    let mut app = audio_app();
    let source = app.world_mut().spawn(Transform::IDENTITY).id();
    app.insert_resource(AmbientEmitters(vec![AmbientEmitter {
        source,
        handle: clip(1),
        volume: 0.1,
        spatial: false,
    }]));
    app.update();
    assert!(voices(&mut app).is_empty());
}

#[test]
fn ambient_loops_attach_to_their_sources_and_follow_lifecycle() {
    let mut app = ambience_app(&[
        (1, None, 0.04),
        (2, Some(Vec3::new(-10.0, 1.0, 0.0)), 0.17),
        (3, Some(Vec3::new(-13.65, 0.45, -1.1)), 0.035),
    ]);
    app.update();
    app.update();
    let emitters = app.world().resource::<AmbientEmitters>().0.clone();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 3);
    for emitter in &emitters {
        let (parent, handle, settings) = playing
            .iter()
            .find(|voice| voice.0 == emitter.source)
            .unwrap();
        assert_eq!(*parent, emitter.source);
        assert_eq!(*handle, emitter.handle);
        assert!(matches!(settings.mode, PlaybackMode::Loop));
        assert_eq!(settings.volume.to_linear(), emitter.volume);
        assert_eq!(settings.spatial, emitter.spatial);
        assert!(!settings.paused);
    }
    let world = app.world_mut();
    let mut query = world.query::<(&Bus, &Transform, &ChildOf)>();
    assert_eq!(query.iter(world).count(), 3);
    assert!(query
        .iter(world)
        .all(|(bus, transform, _)| *bus == Bus::World && *transform == Transform::IDENTITY));

    let boiler = emitters[1].source;
    app.world_mut()
        .entity_mut(boiler)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .x += 3.0;
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(&ChildOf, &GlobalTransform)>();
    assert!(query.iter(world).any(|(parent, transform)| {
        parent.parent() == boiler && transform.translation().x == -7.0
    }));
    app.world_mut().entity_mut(boiler).despawn();
    app.world_mut()
        .resource_mut::<AmbientEmitters>()
        .0
        .retain(|source| source.source != boiler);
    app.update();
    assert_eq!(voices(&mut app).len(), 2);
}

#[test]
fn removing_emitters_stops_their_voices_without_stopping_the_rest() {
    let mut app = ambience_app(&[
        (1, Some(Vec3::ZERO), 0.09),
        (2, Some(Vec3::ZERO), 0.045),
        (3, Some(Vec3::ZERO), 0.17),
        (4, Some(Vec3::ZERO), 0.15),
    ]);
    app.update();
    assert_eq!(voices(&mut app).len(), 4);
    let kept = app.world().resource::<AmbientEmitters>().0[3].source;
    app.world_mut()
        .resource_mut::<AmbientEmitters>()
        .0
        .retain(|source| source.source == kept);
    app.update();
    let remaining = voices(&mut app);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].0, kept);
    assert_eq!(remaining[0].1, clip(4));
    assert!(app.world().get_entity(kept).is_ok());
}

#[test]
fn ambient_emitters_skip_despawned_sources_and_play_once_per_source() {
    let mut app = ambience_app(&[(1, None, 0.1), (2, None, 0.1)]);
    let gone = app.world().resource::<AmbientEmitters>().0[1].source;
    app.world_mut().entity_mut(gone).despawn();
    app.update();
    app.update();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 1);
    assert_eq!(playing[0].1, clip(1));
}

#[test]
fn ambient_voices_scale_with_world_volume() {
    let mut app = ambience_app(&[(1, None, 0.5)]);
    app.insert_resource(Volumes {
        master: 0.5,
        world: 0.4,
        ui: 0.1,
    });
    app.update();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 1);
    assert!((playing[0].2.volume.to_linear() - 0.1).abs() < 0.001);
}

#[test]
fn spatial_and_non_spatial_emitters_differ_only_in_spatial_playback() {
    let mut app = ambience_app(&[(1, None, 0.12), (2, Some(Vec3::new(2.0, 0.0, 0.0)), 0.12)]);
    app.update();
    let emitters = app.world().resource::<AmbientEmitters>().0.clone();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 2);
    let room = playing
        .iter()
        .find(|voice| voice.0 == emitters[0].source)
        .unwrap();
    let machine = playing
        .iter()
        .find(|voice| voice.0 == emitters[1].source)
        .unwrap();
    assert!(!room.2.spatial);
    assert!(room.2.spatial_scale.is_none());
    assert!(machine.2.spatial);
    assert_eq!(
        machine.2.spatial_scale.map(|scale| scale.0),
        Some(Vec3::splat(0.3))
    );
    assert_eq!(room.2.volume, machine.2.volume);
    assert!(matches!(room.2.mode, PlaybackMode::Loop));
    assert!(matches!(machine.2.mode, PlaybackMode::Loop));
}

#[test]
fn ambient_loops_follow_pause_and_activity() {
    let mut app = ambience_app(&[(1, None, 0.12)]);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.update();
    assert_eq!(voices(&mut app).len(), 1);
    assert!(voices(&mut app)[0].2.paused);
    app.world_mut().resource_mut::<AmbienceActive>().0 = false;
    app.update();
    assert!(voices(&mut app).is_empty());
    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.world_mut().resource_mut::<AmbienceActive>().0 = true;
    app.update();
    assert_eq!(voices(&mut app).len(), 1);
    assert!(!voices(&mut app)[0].2.paused);
    let source = app.world().resource::<AmbientEmitters>().0[0].source;
    assert_eq!(voices(&mut app)[0].0, source);
}
