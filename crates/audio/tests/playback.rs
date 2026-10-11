mod common;

use bevy::{audio::PlaybackMode, prelude::*};
use game_audio::{
    spatial_listener, AudioPaused, PlaySound, PlaySoundFrom, PlaySourceSound, Sound, SourceSounds,
};
use game_settings::GameSettings;
use rodio::source::{SineWave, Spatial};

use common::{audio_app, sound_assets};

fn playback_app(listener: Option<Vec3>) -> App {
    let mut app = audio_app();
    if let Some(position) = listener {
        app.world_mut().spawn((
            spatial_listener(0.18),
            Transform::from_translation(position),
        ));
    }
    app.update();
    app
}

fn play(
    app: &mut App,
    cues: &[PlaySound],
) -> Vec<(Handle<AudioSource>, PlaybackSettings, Option<Vec3>)> {
    for cue in cues {
        app.world_mut().write_message(*cue);
    }
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &AudioPlayer, &PlaybackSettings, Option<&Transform>)>();
    let played: Vec<_> = query
        .iter(world)
        .map(|(entity, player, settings, transform)| {
            (
                entity,
                (
                    player.0.clone(),
                    *settings,
                    transform.map(|transform| transform.translation),
                ),
            )
        })
        .collect();
    for (entity, _) in &played {
        world.despawn(*entity);
    }
    played.into_iter().map(|(_, voice)| voice).collect()
}

fn source_voice(app: &mut App, sound: Sound, position: Vec3) -> Option<(Handle<AudioSource>, f32)> {
    let source = app
        .world_mut()
        .spawn(Transform::from_translation(position))
        .id();
    app.update();
    app.world_mut().write_message(PlaySoundFrom {
        sound,
        source,
        offset: Vec3::ZERO,
    });
    app.update();
    let voice = app.world().get::<Children>(source).map(|children| {
        assert_eq!(children.len(), 1);
        let voice = children[0];
        (
            app.world().get::<AudioPlayer>(voice).unwrap().0.clone(),
            app.world()
                .get::<PlaybackSettings>(voice)
                .unwrap()
                .volume
                .to_linear(),
        )
    });
    app.world_mut().entity_mut(source).despawn();
    voice
}

fn source_volume(app: &mut App, sound: Sound, distance: f32) -> Option<f32> {
    source_voice(app, sound, Vec3::new(distance, 0.0, 0.0)).map(|(_, volume)| volume)
}

#[test]
fn settings_scale_new_ui_and_spatial_cues_without_changing_base_gain() {
    let cues = [
        PlaySound {
            sound: Sound::UiPress,
            position: None,
        },
        PlaySound {
            sound: Sound::DoorShut,
            position: Some(Vec3::ZERO),
        },
    ];
    let base = play(&mut playback_app(Some(Vec3::ZERO)), &cues);
    let mut scaled_app = playback_app(Some(Vec3::ZERO));
    scaled_app.insert_resource(GameSettings {
        master: 0.5,
        sfx: 0.4,
        ..default()
    });
    let scaled = play(&mut scaled_app, &cues);
    assert_eq!(base.len(), 2);
    assert_eq!(scaled.len(), 2);
    for (base, scaled) in base.iter().zip(&scaled) {
        assert_eq!(base.0, scaled.0);
        let base = base.1.volume.to_linear();
        let scaled = scaled.1.volume.to_linear();
        assert!(base > 0.0);
        assert!((scaled - base * 0.2).abs() < 0.001);
    }
}

#[test]
fn device_cues_are_non_spatial_and_stop_when_world_audio_is_paused() {
    let mut app = playback_app(None);
    let voices = play(
        &mut app,
        &[
            PlaySound {
                sound: Sound::DetectorPickup,
                position: None,
            },
            PlaySound {
                sound: Sound::DetectorNearby,
                position: None,
            },
            PlaySound {
                sound: Sound::FlashbangPickup,
                position: None,
            },
            PlaySound {
                sound: Sound::FlashbangThrow,
                position: None,
            },
            PlaySound {
                sound: Sound::FlashbangBurst,
                position: None,
            },
        ],
    );
    assert_eq!(voices.len(), 5);
    let assets = sound_assets();
    assert_eq!(voices[0].0, assets.detector_pickup);
    assert_eq!(voices[1].0, assets.detector_nearby);
    assert_eq!(voices[2].0, assets.flashbang_pickup);
    assert_eq!(voices[3].0, assets.flashbang_throw);
    assert_eq!(voices[4].0, assets.flashbang_burst);
    assert!(voices
        .iter()
        .all(|(_, settings, position)| !settings.spatial && position.is_none()));
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(
        &mut app,
        &[PlaySound {
            sound: Sound::FlashbangBurst,
            position: None
        }]
    )
    .is_empty());
}

#[test]
fn distant_spatial_cues_pan_toward_the_source_for_both_listener_orientations() {
    for yaw in [0.0, std::f32::consts::FRAC_PI_2] {
        let transform = Transform::from_rotation(Quat::from_rotation_y(yaw));
        let listener = spatial_listener(0.18);
        let left_ear = transform
            .transform_point(listener.left_ear_offset)
            .to_array();
        let right_ear = transform
            .transform_point(listener.right_ear_offset)
            .to_array();
        for (side, left_should_be_louder) in [(-1.0, true), (1.0, false)] {
            let source = transform.transform_point(Vec3::new(side * 12.0, 0.0, 0.0));
            let samples: Vec<f32> =
                Spatial::new(SineWave::new(440.0), source.to_array(), left_ear, right_ear)
                    .take(2048)
                    .collect();
            let left_energy: f32 = samples
                .iter()
                .step_by(2)
                .map(|sample| sample * sample)
                .sum();
            let right_energy: f32 = samples
                .iter()
                .skip(1)
                .step_by(2)
                .map(|sample| sample * sample)
                .sum();
            assert_eq!(left_energy > right_energy, left_should_be_louder);
        }
    }
}

#[test]
fn source_cue_is_a_child_of_the_boiler_and_follows_its_transform() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let boiler = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 0.0, 0.0))
        .id();
    app.update();
    app.world_mut().write_message(PlaySoundFrom {
        sound: Sound::BoilerTick,
        source: boiler,
        offset: Vec3::Y * 1.2,
    });
    app.update();
    let children = app.world().get::<Children>(boiler).unwrap();
    assert_eq!(children.len(), 1);
    let voice = children[0];
    assert_eq!(
        app.world().get::<AudioPlayer>(voice).unwrap().0,
        sound_assets().boiler_tick
    );
    assert_eq!(
        app.world().get::<Transform>(voice).unwrap().translation,
        Vec3::Y * 1.2
    );
    app.world_mut()
        .entity_mut(boiler)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .x = 3.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<GlobalTransform>(voice)
            .unwrap()
            .translation(),
        Vec3::new(3.0, 1.2, 0.0)
    );
    app.world_mut().entity_mut(boiler).despawn();
    assert!(app.world().get_entity(voice).is_err());
}

#[test]
fn authored_source_cue_plays_only_its_own_sound() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let door = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            SourceSounds(vec![(Sound::DoorLocked, Vec3::Y)]),
        ))
        .id();
    let other = app
        .world_mut()
        .spawn(Transform::from_xyz(3.0, 0.0, 0.0))
        .id();
    app.world_mut().write_message(PlaySourceSound {
        source: other,
        sound: Sound::DoorLocked,
    });
    app.world_mut().write_message(PlaySourceSound {
        source: door,
        sound: Sound::DoorSwing,
    });
    app.update();
    assert!(app.world().get::<Children>(door).is_none());
    app.world_mut().write_message(PlaySourceSound {
        source: door,
        sound: Sound::DoorLocked,
    });
    app.update();
    let voice = app.world().get::<Children>(door).unwrap()[0];
    assert_eq!(
        app.world().get::<Transform>(voice).unwrap().translation,
        Vec3::Y
    );
    assert_eq!(
        app.world().get::<AudioPlayer>(voice).unwrap().0,
        sound_assets().door_locked
    );
    assert_eq!(app.world().get::<ChildOf>(voice).unwrap().parent(), door);
}

#[test]
fn monster_cues_fade_across_distinct_room_distances() {
    let step = Sound::MonsterStep(0);
    let presence = Sound::MonsterPresence;
    assert_eq!(step.audible_range(), 16.0);
    assert_eq!(presence.audible_range(), 32.0);
    assert_eq!(Sound::DoorShut.audible_range(), 25.0);

    let mut app = playback_app(Some(Vec3::ZERO));
    let step_near = source_volume(&mut app, step, 10.0).unwrap();
    let step_far = source_volume(&mut app, step, 15.0).unwrap();
    assert!(step_near > step_far);
    assert_eq!(source_volume(&mut app, step, 16.0), None);
    let presence_near = source_volume(&mut app, presence, 20.0).unwrap();
    let presence_far = source_volume(&mut app, presence, 30.0).unwrap();
    assert!(presence_near > presence_far);
    assert!(presence_far > 0.0);
    assert_eq!(source_volume(&mut app, presence, 32.0), None);

    let fade = 1.0 - 20.0 / presence.audible_range();
    assert!((presence_near - 0.8 * fade * fade).abs() < 0.001);
    assert_eq!(source_volume(&mut app, step, 20.0), None);
    let (handle, _) = source_voice(&mut app, presence, Vec3::new(20.0, 0.0, 0.0)).unwrap();
    assert_eq!(handle, sound_assets().monster_presence);
}

#[test]
fn detection_cue_remains_audible_across_the_sight_range() {
    let detection = Sound::MonsterDetected;
    let mut app = playback_app(Some(Vec3::ZERO));
    let close = source_volume(&mut app, detection, 0.0).unwrap();
    let far = source_volume(&mut app, detection, 12.0).unwrap();
    let step = source_volume(&mut app, Sound::MonsterStep(0), 12.0).unwrap();
    assert!(close > far);
    assert!(far > 0.25);
    assert!(far > step);

    let (handle, volume) = source_voice(&mut app, detection, Vec3::new(12.0, 0.0, 0.0)).unwrap();
    assert_eq!(handle, sound_assets().monster_detected);
    assert!((volume - far).abs() < 0.001);
}

#[test]
fn monster_patrol_steps_and_presence_use_source_attached_clips() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let monster = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            SourceSounds(vec![
                (Sound::MonsterStep(4), Vec3::Y),
                (Sound::MonsterPresence, Vec3::Y),
            ]),
        ))
        .id();
    for sound in [Sound::MonsterStep(4), Sound::MonsterPresence] {
        app.world_mut().write_message(PlaySourceSound {
            source: monster,
            sound,
        });
    }
    app.update();
    let children = app.world().get::<Children>(monster).unwrap();
    assert_eq!(children.len(), 2);
    let actual: Vec<_> = children
        .iter()
        .map(|child| app.world().get::<AudioPlayer>(child).unwrap().0.clone())
        .collect();
    let assets = sound_assets();
    assert!(actual.contains(&assets.monster_step_e));
    assert!(actual.contains(&assets.monster_presence));
    for child in children.iter() {
        assert_eq!(
            app.world().get::<Transform>(child).unwrap().translation,
            Vec3::Y
        );
    }
}

#[test]
fn source_cues_require_a_near_listener_and_unpaused_world() {
    let mut app = playback_app(None);
    let boiler = app.world_mut().spawn(Transform::IDENTITY).id();
    let cue = PlaySoundFrom {
        sound: Sound::BoilerTick,
        source: boiler,
        offset: Vec3::Y,
    };
    app.world_mut().write_message(cue);
    app.update();
    assert!(app.world().get::<Children>(boiler).is_none());
    app.world_mut()
        .spawn((spatial_listener(0.18), Transform::IDENTITY));
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.world_mut().write_message(cue);
    app.update();
    assert!(app.world().get::<Children>(boiler).is_none());
    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.world_mut().write_message(cue);
    app.update();
    assert_eq!(app.world().get::<Children>(boiler).unwrap().len(), 1);
}

#[test]
fn faucet_burst_plays_once_as_positional_world_audio() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let faucet = Vec3::new(-13.4, 1.5, 0.0);
    let cue = PlaySound {
        sound: Sound::FaucetBurst,
        position: Some(faucet),
    };
    let played = play(&mut app, &[cue]);

    assert_eq!(played.len(), 1);
    let (handle, settings, translation) = &played[0];
    assert_eq!(*handle, sound_assets().faucet);
    assert!(matches!(settings.mode, PlaybackMode::Despawn));
    assert!(settings.spatial);
    assert_eq!(*translation, Some(faucet));
    let distance = faucet.length();
    let gain = 0.6 / (1.0 + 0.06 * distance * distance);
    assert_eq!(settings.volume.to_linear(), gain);

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[cue]).is_empty());
}

#[test]
fn positional_cues_need_a_near_listener() {
    let cue = PlaySound {
        sound: Sound::FaucetBurst,
        position: Some(Vec3::new(-13.4, 1.5, 0.0)),
    };
    assert!(play(&mut playback_app(None), &[cue]).is_empty());
    assert!(play(&mut playback_app(Some(Vec3::new(20.0, 0.0, 0.0))), &[cue]).is_empty());
}

#[test]
fn hiding_and_boiler_cues_use_their_clips() {
    let assets = sound_assets();
    let cues = [
        (Sound::LockerOpen, &assets.locker_open),
        (Sound::LockerClose, &assets.locker_close),
        (Sound::TableEnter, &assets.table_enter),
        (Sound::TableLeave, &assets.table_leave),
        (Sound::BoilerTick, &assets.boiler_tick),
    ];
    let mut app = playback_app(Some(Vec3::ZERO));
    let requests = cues.map(|(sound, _)| PlaySound {
        sound,
        position: Some(Vec3::X),
    });
    let played = play(&mut app, &requests);

    assert_eq!(played.len(), cues.len());
    for (_, clip) in cues {
        assert_eq!(played.iter().filter(|voice| voice.0 == *clip).count(), 1);
    }
    assert!(played.iter().all(|(_, settings, _)| settings.spatial));

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &requests).is_empty());
}

#[test]
fn fuse_slots_and_completion_use_only_the_new_ui_cues() {
    let assets = sound_assets();
    let mut app = playback_app(None);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    let played = play(
        &mut app,
        &[
            PlaySound {
                sound: Sound::FuseSlot(1),
                position: None,
            },
            PlaySound {
                sound: Sound::FuseSlot(2),
                position: None,
            },
            PlaySound {
                sound: Sound::FuseSlot(3),
                position: None,
            },
            PlaySound {
                sound: Sound::FuseComplete,
                position: None,
            },
        ],
    );
    assert_eq!(played.len(), 4);
    for (voice, handle) in played.iter().zip([
        &assets.fuse_slot_1,
        &assets.fuse_slot_2,
        &assets.fuse_slot_3,
        &assets.fuse_complete,
    ]) {
        assert_eq!(&voice.0, handle);
        assert!(!voice.1.spatial);
    }
}

#[test]
fn flashlight_click_uses_approved_clip_as_non_spatial_audio() {
    let mut app = playback_app(None);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    let played = play(
        &mut app,
        &[PlaySound {
            sound: Sound::FlashlightClick,
            position: None,
        }],
    );
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].0, sound_assets().flashlight_click);
    assert!(!played[0].1.spatial);
}

#[test]
fn sprint_exhaustion_is_non_spatial_player_audio_and_dropped_while_paused() {
    let mut app = playback_app(None);
    let cue = PlaySound {
        sound: Sound::SprintExhausted,
        position: None,
    };
    let played = play(&mut app, &[cue]);
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].0, sound_assets().sprint_exhausted);
    assert!(!played[0].1.spatial);

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[cue]).is_empty());
}

#[test]
fn locked_door_rattle_is_spatial_and_dropped_while_paused() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let cue = PlaySound {
        sound: Sound::DoorLocked,
        position: Some(Vec3::new(0.0, 1.0, -1.5)),
    };
    let played = play(&mut app, &[cue]);
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].0, sound_assets().door_locked);
    assert_eq!(played[0].2, cue.position);
    assert!(played[0].1.spatial);

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[cue]).is_empty());
}

#[test]
fn outage_has_global_power_down_and_spatial_breaker_trip_while_repair_is_spatial() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let boiler = Vec3::new(-10.0, 1.2, 0.0);
    let outage = PlaySound {
        sound: Sound::PowerDown,
        position: None,
    };
    let trip = PlaySound {
        sound: Sound::BreakerTrip,
        position: Some(boiler),
    };
    let reset = PlaySound {
        sound: Sound::BoilerReset,
        position: Some(boiler),
    };
    let repair = PlaySound {
        sound: Sound::BoilerRestart,
        position: Some(boiler),
    };
    let played = play(&mut app, &[outage, trip, reset, repair]);
    assert_eq!(played.len(), 4);
    let assets = sound_assets();
    let voice = |clip: &Handle<AudioSource>| played.iter().find(|voice| voice.0 == *clip).unwrap();
    let power_down = voice(&assets.power_down);
    assert!(!power_down.1.spatial);
    assert_eq!(power_down.2, None);
    for clip in [
        &assets.breaker_trip,
        &assets.boiler_reset,
        &assets.boiler_restart,
    ] {
        let cue = voice(clip);
        assert!(cue.1.spatial);
        assert_eq!(cue.2, Some(boiler));
    }

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[outage, trip, reset, repair]).is_empty());
}

#[test]
fn pause_drops_world_cues_but_keeps_menu_cues() {
    let mut app = playback_app(Some(Vec3::ZERO));
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    let played = play(
        &mut app,
        &[
            PlaySound {
                sound: Sound::FaucetBurst,
                position: Some(Vec3::X),
            },
            PlaySound {
                sound: Sound::LockerOpen,
                position: Some(Vec3::X),
            },
            PlaySound {
                sound: Sound::UiResume,
                position: None,
            },
        ],
    );

    assert_eq!(played.len(), 1);
    assert_eq!(played[0].0, sound_assets().ui_resume);
}
