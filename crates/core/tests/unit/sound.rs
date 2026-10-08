use std::time::Duration;

use bevy::{input::InputPlugin, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::prelude::EntropyPlugin;
use game_audio::AmbientSound;

use super::*;
use gameplay::levels::IntermittentSoundKind;

#[test]
fn ambience_layout_has_one_bed_and_local_facility_sources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins((TransformPlugin, InputPlugin, EnhancedInputPlugin))
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed([7; 32]))
        .add_plugins(gameplay::controller::PlayerControllerPlugin::default())
        .add_message::<PlaySound>()
        .add_message::<PlaySoundFrom>()
        .init_resource::<PlayerControlsEnabled>()
        .init_resource::<AudioPaused>()
        .init_resource::<AmbienceActive>()
        .init_resource::<ConduitAmbience>()
        .add_plugins((gameplay::levels::PropSoundsPlugin, SoundGluePlugin));
    app.finish();
    app.cleanup();
    app.world_mut()
        .run_system_cached(gameplay::levels::build_first_floor)
        .unwrap();
    app.world_mut().spawn(PlayerController);
    app.update();
    app.update();
    let listener_count = {
        let world = app.world_mut();
        let mut query = world.query::<&bevy::audio::SpatialListener>();
        query.iter(world).count()
    };
    assert_eq!(listener_count, 1);
    let sources = &app.world().resource::<AmbientEmitters>().0;
    assert_eq!(sources.len(), 21);
    assert!(sources.iter().any(|emitter| {
        emitter.sound == AmbientSound::Boiler
            && app
                .world()
                .get::<GlobalTransform>(emitter.source)
                .map(GlobalTransform::translation)
                == Some(Vec3::new(-10.0, 1.0, 0.0))
    }));
    assert!(sources.iter().any(|emitter| {
        emitter.sound == AmbientSound::Tank
            && app
                .world()
                .get::<GlobalTransform>(emitter.source)
                .map(GlobalTransform::translation)
                == Some(Vec3::new(0.0, 1.3, 0.0))
    }));
    assert_eq!(sources.iter().filter(|emitter| !emitter.spatial).count(), 3);
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.sound == AmbientSound::LowPressure)
            .count(),
        1
    );
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.sound == AmbientSound::Vent)
            .count(),
        3
    );
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.sound == AmbientSound::VentWind)
            .count(),
        3
    );
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.sound == AmbientSound::Boiler)
            .count(),
        1
    );
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.sound == AmbientSound::CoolBuzz)
            .count(),
        10
    );
    assert!(sources
        .iter()
        .all(|emitter| emitter.volume > 0.0 && emitter.volume <= 0.2));

    app.world_mut()
        .resource_mut::<gameplay::levels::FacilityPower>()
        .outage();
    app.update();
    let sources = &app.world().resource::<AmbientEmitters>().0;
    assert_eq!(sources.len(), 4);
    assert!(sources.iter().all(|emitter| matches!(
        emitter.sound,
        AmbientSound::Roomtone
            | AmbientSound::LowPressure
            | AmbientSound::Conduit
            | AmbientSound::Tank
    )));
    assert!(sources
        .iter()
        .any(|emitter| emitter.sound == AmbientSound::Tank));
    app.world_mut()
        .resource_mut::<gameplay::levels::FacilityPower>()
        .restore();
    app.update();
    assert_eq!(app.world().resource::<AmbientEmitters>().0.len(), 21);
}

const FAUCET: Vec3 = Vec3::new(-13.4, 1.5, 0.0);

fn faucet_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed([7; 32]))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .init_resource::<AudioPaused>()
        .add_message::<PlaySoundFrom>()
        .add_systems(Update, play_intermittent_sounds);
    app.world_mut().spawn((
        IntermittentSound {
            kind: IntermittentSoundKind::FaucetBurst,
            offset: Vec3::new(0.0, -0.1, -0.25),
            range: 12.0,
            interval: 10.0,
            variation: 8.0,
            remaining: 10.0,
        },
        Transform::from_xyz(-13.65, 1.6, 0.0)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    ));
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(2));
    app.update();
    app
}

fn faucet_seconds(app: &mut App, seconds: usize) -> Vec<usize> {
    let mut bursts = Vec::new();
    for second in 0..seconds {
        app.update();
        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<PlaySoundFrom>>()
            .drain()
            .collect();
        for play in events {
            assert_eq!(play.sound, Sound::FaucetBurst);
            let transform = app.world().get::<GlobalTransform>(play.source).unwrap();
            assert!(transform.transform_point(play.offset).distance(FAUCET) < 0.0001);
            bursts.push(second);
        }
    }
    bursts
}

#[test]
fn faucet_bursts_are_intermittent_near_the_sink_and_hold_while_paused() {
    let mut app = faucet_app();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(-12.0, 0.0, 3.0)));

    let bursts = faucet_seconds(&mut app, 80);
    assert!(bursts.len() >= 4, "{bursts:?}");
    assert!((8..=11).contains(&bursts[0]), "{bursts:?}");
    assert!(
        bursts
            .windows(2)
            .all(|pair| (10..=18).contains(&(pair[1] - pair[0]))),
        "{bursts:?}"
    );

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(faucet_seconds(&mut app, 40).is_empty());
}

#[test]
fn intermittent_cue_follows_its_source_and_stops_when_removed() {
    let mut app = faucet_app();
    let source = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<IntermittentSound>>();
        query.single(world).unwrap()
    };
    app.world_mut()
        .entity_mut(source)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .x += 2.0;
    app.world_mut()
        .entity_mut(source)
        .get_mut::<IntermittentSound>()
        .unwrap()
        .remaining = 0.1;
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(-11.0, 0.0, 0.0)));
    app.update();
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySoundFrom>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, source);
    assert!(
        app.world()
            .get::<GlobalTransform>(source)
            .unwrap()
            .transform_point(events[0].offset)
            .distance(FAUCET + Vec3::X * 2.0)
            < 0.0001
    );
    app.world_mut().entity_mut(source).despawn();
    for _ in 0..30 {
        app.update();
    }
    assert!(app
        .world_mut()
        .resource_mut::<Messages<PlaySoundFrom>>()
        .drain()
        .next()
        .is_none());
}

#[test]
fn powered_intermittent_cues_stop_during_outage_but_faucet_keeps_running() {
    let mut app = faucet_app();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(-11.0, 0.0, 0.0)));
    let boiler = app
        .world_mut()
        .spawn((
            IntermittentSound {
                kind: IntermittentSoundKind::BoilerTick,
                offset: Vec3::ZERO,
                range: 18.0,
                interval: 6.0,
                variation: 0.0,
                remaining: 0.1,
            },
            Transform::from_xyz(-10.0, 0.0, 0.0),
        ))
        .id();
    app.world_mut()
        .insert_resource(gameplay::levels::FacilityPower::new(42));
    app.world_mut()
        .resource_mut::<gameplay::levels::FacilityPower>()
        .outage();
    let source = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<IntermittentSound>>();
        query.iter(world).find(|entity| *entity != boiler).unwrap()
    };
    app.world_mut()
        .entity_mut(source)
        .get_mut::<IntermittentSound>()
        .unwrap()
        .remaining = 0.1;
    app.update();
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySoundFrom>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sound, Sound::FaucetBurst);
    assert_eq!(
        app.world()
            .get::<IntermittentSound>(boiler)
            .unwrap()
            .remaining,
        0.1
    );
    app.world_mut()
        .resource_mut::<gameplay::levels::FacilityPower>()
        .restore();
    app.update();
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySoundFrom>>()
        .drain()
        .collect();
    assert!(events.iter().any(|event| event.sound == Sound::BoilerTick));
}

#[test]
fn faucet_bursts_stay_silent_far_from_the_sink() {
    let mut app = faucet_app();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(5.0, 0.0, -20.0)));
    assert!(faucet_seconds(&mut app, 80).is_empty());
}
