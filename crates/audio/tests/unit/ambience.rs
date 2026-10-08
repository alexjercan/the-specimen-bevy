use bevy::audio::PlaybackMode;

use super::*;

fn ambience_app(sources: &[(AmbientSound, Option<Vec3>, f32)]) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .insert_resource(AmbienceActive(true))
        .init_resource::<AudioPaused>()
        .init_resource::<ConduitAmbience>()
        .insert_resource(crate::test_support::sound_assets())
        .add_systems(Update, update_ambience);
    let emitters = sources
        .iter()
        .map(|&(sound, position, volume)| {
            let source = app
                .world_mut()
                .spawn(Transform::from_translation(position.unwrap_or(Vec3::ZERO)))
                .id();
            AmbientEmitter {
                source,
                sound,
                spatial: position.is_some(),
                volume,
            }
        })
        .collect();
    app.insert_resource(AmbientEmitters(emitters));
    app
}

fn voices(app: &mut App) -> Vec<(Entity, Handle<AudioSource>, PlaybackSettings, Entity)> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(&AmbientVoice, &AudioPlayer, &PlaybackSettings, &ChildOf), With<WorldAudio>>();
    let mut voices: Vec<_> = query
        .iter(world)
        .map(|(voice, player, settings, parent)| {
            (voice.0, player.0.clone(), settings.clone(), parent.parent())
        })
        .collect();
    voices.sort_by_key(|voice| voice.0.to_bits());
    voices
}

#[test]
fn ambience_without_layout_or_assets_is_inert() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<AmbienceActive>()
        .init_resource::<AudioPaused>()
        .init_resource::<ConduitAmbience>()
        .add_systems(Update, update_ambience);
    app.world_mut().resource_mut::<AmbienceActive>().0 = true;
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<&AmbientVoice>();
    assert_eq!(query.iter(world).count(), 0);
}

#[test]
fn ambient_sounds_use_approved_clips() {
    let assets = crate::test_support::sound_assets();
    for (sound, handle) in [
        (AmbientSound::Roomtone, &assets.roomtone),
        (AmbientSound::LowPressure, &assets.low_pressure),
        (AmbientSound::Conduit, &assets.conduit_roomtone),
        (AmbientSound::Boiler, &assets.furnace),
        (AmbientSound::Tank, &assets.tank_hum),
        (AmbientSound::Vent, &assets.vent_hvac),
        (AmbientSound::VentWind, &assets.vent_wind),
        (AmbientSound::CoolBuzz, &assets.cool_buzz),
    ] {
        assert_eq!(sound.handle(&assets), *handle);
    }
}

#[test]
fn ambient_loops_attach_to_their_sources_and_follow_lifecycle() {
    let mut app = ambience_app(&[
        (AmbientSound::LowPressure, None, 0.04),
        (AmbientSound::Boiler, Some(Vec3::new(-10.0, 1.0, 0.0)), 0.17),
        (
            AmbientSound::VentWind,
            Some(Vec3::new(-13.65, 0.45, -1.1)),
            0.035,
        ),
    ]);
    app.update();
    app.update();
    let assets = crate::test_support::sound_assets();
    let emitters = app.world().resource::<AmbientEmitters>().0.clone();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 3);
    for emitter in &emitters {
        let (_, handle, settings, parent) = playing
            .iter()
            .find(|voice| voice.0 == emitter.source)
            .unwrap();
        assert_eq!(*parent, emitter.source);
        assert_eq!(*handle, emitter.sound.handle(&assets));
        assert!(matches!(settings.mode, PlaybackMode::Loop));
        assert_eq!(settings.volume.to_linear(), emitter.volume);
        assert_eq!(settings.spatial, emitter.spatial);
    }
    let boiler = emitters[1].source;
    app.world_mut()
        .entity_mut(boiler)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .x += 3.0;
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(&AmbientVoice, &GlobalTransform)>();
    assert!(query
        .iter(world)
        .any(|(voice, transform)| voice.0 == boiler && transform.translation().x == -7.0));
    app.world_mut().entity_mut(boiler).despawn();
    app.world_mut()
        .resource_mut::<AmbientEmitters>()
        .0
        .retain(|source| source.source != boiler);
    app.update();
    assert_eq!(voices(&mut app).len(), 2);
}

#[test]
fn ambient_loops_follow_pause_activity_and_conduit_state() {
    let mut app = ambience_app(&[
        (AmbientSound::Roomtone, None, 0.12),
        (AmbientSound::Conduit, None, 0.055),
    ]);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.update();
    assert_eq!(voices(&mut app).len(), 1);
    assert!(voices(&mut app)[0].2.paused);
    app.world_mut().resource_mut::<ConduitAmbience>().0 = true;
    app.update();
    assert_eq!(voices(&mut app).len(), 2);
    app.world_mut().resource_mut::<ConduitAmbience>().0 = false;
    app.update();
    assert_eq!(voices(&mut app).len(), 1);
    app.world_mut().resource_mut::<AmbienceActive>().0 = false;
    app.update();
    assert!(voices(&mut app).is_empty());
    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.world_mut().resource_mut::<AmbienceActive>().0 = true;
    app.update();
    assert_eq!(voices(&mut app).len(), 1);
    assert!(!voices(&mut app)[0].2.paused);
}
