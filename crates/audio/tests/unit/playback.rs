use bevy::audio::PlaybackMode;

use super::*;

fn playback_app(listener: Option<Vec3>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<PlaySound>()
        .init_resource::<AudioPaused>()
        .insert_resource(test_support::sound_assets())
        .add_systems(Update, play_sounds);
    if let Some(position) = listener {
        app.world_mut().spawn((
            SpatialListener::new(0.18),
            GlobalTransform::from_translation(position),
        ));
    }
    app
}

fn play(
    app: &mut App,
    cues: &[PlaySound],
) -> Vec<(Handle<AudioSource>, PlaybackSettings, Option<Vec3>, bool)> {
    for cue in cues {
        app.world_mut().write_message(*cue);
    }
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(
        Entity,
        &AudioPlayer,
        &PlaybackSettings,
        Option<&Transform>,
        Has<WorldAudio>,
    )>();
    let played: Vec<_> = query
        .iter(world)
        .map(|(entity, player, settings, transform, world_audio)| {
            (
                entity,
                (
                    player.0.clone(),
                    settings.clone(),
                    transform.map(|transform| transform.translation),
                    world_audio,
                ),
            )
        })
        .collect();
    for (entity, _) in &played {
        world.despawn(*entity);
    }
    played.into_iter().map(|(_, voice)| voice).collect()
}

#[test]
fn faucet_burst_plays_once_as_positional_world_audio() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let faucet = Vec3::new(-13.4, 1.5, 0.0);
    let played = play(
        &mut app,
        &[PlaySound {
            sound: Sound::FaucetBurst,
            position: Some(faucet),
        }],
    );

    assert_eq!(played.len(), 1);
    let (handle, settings, translation, world_audio) = &played[0];
    assert_eq!(*handle, test_support::sound_assets().faucet);
    assert!(matches!(settings.mode, PlaybackMode::Despawn));
    assert!(settings.spatial);
    assert_eq!(*translation, Some(faucet));
    assert!(*world_audio);
    let distance = faucet.length();
    let gain = 0.6 / (1.0 + 0.06 * distance * distance);
    assert_eq!(settings.volume.to_linear(), gain);
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
    let assets = test_support::sound_assets();
    let cues = [
        (Sound::LockerOpen, &assets.locker_open),
        (Sound::LockerClose, &assets.locker_close),
        (Sound::TableEnter, &assets.table_enter),
        (Sound::TableLeave, &assets.table_leave),
        (Sound::BoilerTick, &assets.boiler_tick),
    ];
    let mut app = playback_app(Some(Vec3::ZERO));
    let played = play(
        &mut app,
        &cues.map(|(sound, _)| PlaySound {
            sound,
            position: Some(Vec3::X),
        }),
    );

    assert_eq!(played.len(), cues.len());
    for (_, clip) in cues {
        assert_eq!(played.iter().filter(|voice| voice.0 == *clip).count(), 1);
    }
    assert!(played
        .iter()
        .all(|(_, settings, _, world_audio)| settings.spatial && *world_audio));
}

#[test]
fn fuse_slots_and_completion_use_only_the_new_ui_cues() {
    let assets = test_support::sound_assets();
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
        assert!(!voice.3);
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
    assert_eq!(played[0].0, test_support::sound_assets().flashlight_click);
    assert!(!played[0].1.spatial);
    assert!(!played[0].3);
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
    assert_eq!(played[0].0, test_support::sound_assets().sprint_exhausted);
    assert!(!played[0].1.spatial);
    assert!(played[0].3);

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
    assert_eq!(played[0].0, test_support::sound_assets().door_locked);
    assert_eq!(played[0].2, cue.position);
    assert!(played[0].1.spatial);
    assert!(played[0].3);

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[cue]).is_empty());
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
    assert_eq!(played[0].0, test_support::sound_assets().ui_resume);
    assert!(!played[0].3);
}
