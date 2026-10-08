use bevy::audio::PlaybackMode;

use super::*;

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
fn sound_layout_is_handle_independent() {
    let sources = [
        AmbientEmitter {
            sound: AmbientSound::Roomtone,
            position: None,
            volume: 0.12,
        },
        AmbientEmitter {
            sound: AmbientSound::Vent,
            position: Some(Vec3::new(1.0, 2.0, 3.0)),
            volume: 0.09,
        },
    ];
    assert!(sources[0].position.is_none());
    assert!(sources[1].position.is_some());
    assert!(sources
        .iter()
        .all(|source| (0.0..=0.2).contains(&source.volume)));
}

fn ambience_app(emitters: Vec<AmbientEmitter>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(AmbienceActive(true))
        .init_resource::<AudioPaused>()
        .init_resource::<ConduitAmbience>()
        .insert_resource(crate::test_support::sound_assets())
        .insert_resource(AmbientEmitters(emitters))
        .add_systems(Update, update_ambience);
    app
}

fn voices(app: &mut App) -> Vec<(usize, Handle<AudioSource>, PlaybackSettings, Option<Vec3>)> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<(
        &AmbientVoice,
        &AudioPlayer,
        &PlaybackSettings,
        Option<&Transform>,
    ), With<WorldAudio>>();
    let mut voices: Vec<_> = query
        .iter(world)
        .map(|(voice, player, settings, transform)| {
            (
                voice.0,
                player.0.clone(),
                settings.clone(),
                transform.map(|transform| transform.translation),
            )
        })
        .collect();
    voices.sort_by_key(|voice| voice.0);
    voices
}

#[test]
fn ambient_sounds_use_approved_clips() {
    let assets = crate::test_support::sound_assets();
    let expected = [
        (AmbientSound::Roomtone, &assets.roomtone),
        (AmbientSound::LowPressure, &assets.low_pressure),
        (AmbientSound::Conduit, &assets.conduit_roomtone),
        (AmbientSound::Boiler, &assets.furnace),
        (AmbientSound::Tank, &assets.tank_hum),
        (AmbientSound::Vent, &assets.vent_hvac),
        (AmbientSound::VentWind, &assets.vent_wind),
        (AmbientSound::CoolBuzz, &assets.cool_buzz),
    ];
    for (sound, handle) in expected {
        assert_eq!(sound.handle(&assets), *handle);
    }
}

#[test]
fn ambient_loops_spawn_once_with_bed_and_spatial_settings() {
    let furnace = Vec3::new(-10.0, 1.0, 0.0);
    let vent = Vec3::new(-13.65, 0.45, -1.1);
    let mut app = ambience_app(vec![
        AmbientEmitter {
            sound: AmbientSound::LowPressure,
            position: None,
            volume: 0.04,
        },
        AmbientEmitter {
            sound: AmbientSound::Boiler,
            position: Some(furnace),
            volume: 0.17,
        },
        AmbientEmitter {
            sound: AmbientSound::VentWind,
            position: Some(vent),
            volume: 0.035,
        },
    ]);
    app.update();
    app.update();

    let assets = crate::test_support::sound_assets();
    let voices = voices(&mut app);
    assert_eq!(voices.len(), 3);
    let expected = [
        (&assets.low_pressure, None, 0.04),
        (&assets.furnace, Some(furnace), 0.17),
        (&assets.vent_wind, Some(vent), 0.035),
    ];
    for (index, ((voice, handle, settings, translation), (clip, position, volume))) in
        voices.iter().zip(expected).enumerate()
    {
        assert_eq!(*voice, index);
        assert_eq!(handle, clip);
        assert!(matches!(settings.mode, PlaybackMode::Loop));
        assert!(!settings.paused);
        assert_eq!(settings.volume.to_linear(), volume);
        assert_eq!(*translation, position);
        assert_eq!(settings.spatial, position.is_some());
        assert_eq!(
            settings.spatial_scale.map(|scale| scale.0),
            position.map(|_| Vec3::splat(0.3))
        );
    }
}

#[test]
fn ambient_loops_follow_pause_activity_and_conduit_state() {
    let mut app = ambience_app(vec![
        AmbientEmitter {
            sound: AmbientSound::Roomtone,
            position: None,
            volume: 0.12,
        },
        AmbientEmitter {
            sound: AmbientSound::Conduit,
            position: None,
            volume: 0.055,
        },
    ]);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.update();
    let spawned = voices(&mut app);
    assert_eq!(spawned.len(), 1);
    assert!(spawned[0].2.paused);

    app.world_mut().resource_mut::<ConduitAmbience>().0 = true;
    app.update();
    assert_eq!(
        voices(&mut app)
            .iter()
            .map(|voice| voice.0)
            .collect::<Vec<_>>(),
        [0, 1]
    );

    app.world_mut().resource_mut::<ConduitAmbience>().0 = false;
    app.update();
    assert_eq!(
        voices(&mut app)
            .iter()
            .map(|voice| voice.0)
            .collect::<Vec<_>>(),
        [0]
    );

    app.world_mut().resource_mut::<AmbienceActive>().0 = false;
    app.update();
    assert!(voices(&mut app).is_empty());

    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.world_mut().resource_mut::<AmbienceActive>().0 = true;
    app.update();
    let restarted = voices(&mut app);
    assert_eq!(restarted.len(), 1);
    assert!(!restarted[0].2.paused);
}
