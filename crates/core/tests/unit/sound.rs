use std::time::Duration;

use bevy::time::TimeUpdateStrategy;
use bevy_rand::prelude::EntropyPlugin;

use super::*;

#[test]
fn ambience_layout_has_one_bed_and_local_facility_sources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(SoundGluePlugin);
    let sources = &app.world().resource::<AmbientEmitters>().0;
    assert_eq!(sources.len(), 14);
    assert_eq!(
        sources
            .iter()
            .filter(|emitter| emitter.position.is_none())
            .count(),
        3
    );
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
        3
    );
    assert!(sources
        .iter()
        .all(|emitter| emitter.volume > 0.0 && emitter.volume <= 0.2));
}

#[test]
fn hiding_cues_keep_their_kind_and_world_position_in_audio_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<GameplaySound>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_gameplay_sounds);

    let cues = [
        (GameplaySoundKind::LockerOpen, Sound::LockerOpen),
        (GameplaySoundKind::LockerClose, Sound::LockerClose),
        (GameplaySoundKind::TableEnter, Sound::TableEnter),
        (GameplaySoundKind::TableLeave, Sound::TableLeave),
    ];
    for (index, &(kind, _)) in cues.iter().enumerate() {
        app.world_mut().write_message(GameplaySound {
            kind,
            position: Vec3::new(index as f32, 0.0, -2.0),
        });
    }
    app.update();

    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), cues.len());
    for (index, (play, &(_, sound))) in played.iter().zip(&cues).enumerate() {
        assert_eq!(play.sound, sound);
        assert_eq!(play.position, Some(Vec3::new(index as f32, 0.0, -2.0)));
    }
}

#[test]
fn sprint_exhaustion_bridges_once_as_non_spatial_player_audio() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<SprintExhausted>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_sprint_exhaustion);
    app.world_mut().write_message(SprintExhausted {
        position: Vec3::new(1.0, 1.6, -3.0),
    });
    app.update();
    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].sound, Sound::SprintExhausted);
    assert_eq!(played[0].position, None);
    app.update();
    assert_eq!(app.world().resource::<Messages<PlaySound>>().len(), 0);
}

#[test]
fn locked_door_rattle_keeps_its_world_position_in_audio_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<GameplaySound>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_gameplay_sounds);
    let position = Vec3::new(0.0, 1.0, -31.25);
    app.world_mut().write_message(GameplaySound {
        kind: GameplaySoundKind::DoorLocked,
        position,
    });
    app.update();
    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].sound, Sound::DoorLocked);
    assert_eq!(played[0].position, Some(position));
}

#[test]
fn flashlight_click_bridges_as_non_spatial_player_audio() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<GameplaySound>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_gameplay_sounds);
    app.world_mut().write_message(GameplaySound {
        kind: GameplaySoundKind::FlashlightClick,
        position: Vec3::new(1.0, 1.6, -3.0),
    });
    app.update();
    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].sound, Sound::FlashlightClick);
    assert_eq!(played[0].position, None);
}

#[test]
fn fuse_cues_bridge_as_non_spatial_ui_sounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<GameplaySound>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_gameplay_sounds);
    for kind in [
        GameplaySoundKind::FuseSlot(1),
        GameplaySoundKind::FuseSlot(2),
        GameplaySoundKind::FuseSlot(3),
        GameplaySoundKind::FuseComplete,
    ] {
        app.world_mut().write_message(GameplaySound {
            kind,
            position: Vec3::X,
        });
    }
    app.update();
    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), 4);
    for (play, sound) in played.iter().zip([
        Sound::FuseSlot(1),
        Sound::FuseSlot(2),
        Sound::FuseSlot(3),
        Sound::FuseComplete,
    ]) {
        assert_eq!(play.sound, sound);
        assert_eq!(play.position, None);
    }
}

const FAUCET: Vec3 = Vec3::new(-13.4, 1.5, 0.0);

fn faucet_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed([7; 32]))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)))
        .init_resource::<AudioPaused>()
        .add_message::<PlaySound>()
        .add_systems(Update, faucet_bursts);
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
        for play in app
            .world_mut()
            .resource_mut::<Messages<PlaySound>>()
            .drain()
        {
            assert_eq!(play.sound, Sound::FaucetBurst);
            assert_eq!(play.position, Some(FAUCET));
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
fn faucet_bursts_stay_silent_far_from_the_sink() {
    let mut app = faucet_app();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(5.0, 0.0, -20.0)));
    assert!(faucet_seconds(&mut app, 80).is_empty());
}
