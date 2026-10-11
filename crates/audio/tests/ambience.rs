mod common;

use bevy::{audio::PlaybackMode, prelude::*, state::app::StatesPlugin};
use game_assets::SoundAssets;
use game_audio::{
    AmbienceActive, AmbientEmitter, AmbientEmitters, AmbientSound, AudioPaused, ConduitAmbience,
    GameAudioPlugin,
};
use game_settings::GameSettings;

use common::{audio_app, sound_assets};

fn ambient_clip(sound: AmbientSound, assets: &SoundAssets) -> Handle<AudioSource> {
    match sound {
        AmbientSound::Roomtone => assets.roomtone.clone(),
        AmbientSound::LowPressure => assets.low_pressure.clone(),
        AmbientSound::Conduit => assets.conduit_roomtone.clone(),
        AmbientSound::Boiler => assets.furnace.clone(),
        AmbientSound::Tank => assets.tank_hum.clone(),
        AmbientSound::Vent => assets.vent_hvac.clone(),
        AmbientSound::VentWind => assets.vent_wind.clone(),
        AmbientSound::CoolBuzz => assets.cool_buzz.clone(),
    }
}

fn ambience_app(sources: &[(AmbientSound, Option<Vec3>, f32)]) -> App {
    let mut app = audio_app();
    app.insert_resource(AmbienceActive(true));
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
                volume,
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
fn ambience_is_inert_before_assets_are_ready() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, GameAudioPlugin))
        .insert_resource(GameSettings::default())
        .insert_resource(AmbienceActive(true))
        .insert_resource(AmbientEmitters(vec![AmbientEmitter {
            source: Entity::PLACEHOLDER,
            sound: AmbientSound::Roomtone,
            volume: 0.1,
        }]));
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<&AudioPlayer>();
    assert_eq!(query.iter(world).count(), 0);
}

#[test]
fn ambient_sounds_use_approved_clips() {
    let sounds = [
        AmbientSound::Roomtone,
        AmbientSound::LowPressure,
        AmbientSound::Conduit,
        AmbientSound::Boiler,
        AmbientSound::Tank,
        AmbientSound::Vent,
        AmbientSound::VentWind,
        AmbientSound::CoolBuzz,
    ];
    let sources: Vec<_> = sounds.iter().map(|&sound| (sound, None, 0.1)).collect();
    let mut app = ambience_app(&sources);
    app.insert_resource(ConduitAmbience(true));
    app.update();
    let assets = sound_assets();
    let emitters = app.world().resource::<AmbientEmitters>().0.clone();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), sounds.len());
    for emitter in &emitters {
        let (_, handle, _) = playing
            .iter()
            .find(|voice| voice.0 == emitter.source)
            .unwrap();
        assert_eq!(*handle, ambient_clip(emitter.sound, &assets));
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
    let assets = sound_assets();
    let emitters = app.world().resource::<AmbientEmitters>().0.clone();
    let playing = voices(&mut app);
    assert_eq!(playing.len(), 3);
    for emitter in &emitters {
        let (parent, handle, settings) = playing
            .iter()
            .find(|voice| voice.0 == emitter.source)
            .unwrap();
        assert_eq!(*parent, emitter.source);
        assert_eq!(*handle, ambient_clip(emitter.sound, &assets));
        assert!(matches!(settings.mode, PlaybackMode::Loop));
        assert_eq!(settings.volume.to_linear(), emitter.volume);
        assert_eq!(settings.spatial, emitter.sound.spatial());
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
fn removing_powered_sources_stops_their_voices_without_stopping_tank_audio() {
    let mut app = ambience_app(&[
        (AmbientSound::Vent, Some(Vec3::ZERO), 0.09),
        (AmbientSound::CoolBuzz, Some(Vec3::ZERO), 0.045),
        (AmbientSound::Boiler, Some(Vec3::ZERO), 0.17),
        (AmbientSound::Tank, Some(Vec3::ZERO), 0.15),
    ]);
    app.update();
    assert_eq!(voices(&mut app).len(), 4);
    app.world_mut()
        .resource_mut::<AmbientEmitters>()
        .0
        .retain(|source| !source.sound.requires_power());
    app.update();
    let remaining = voices(&mut app);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].1, sound_assets().tank_hum);
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
