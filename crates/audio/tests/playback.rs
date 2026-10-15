mod common;

use bevy::{
    audio::{AudioSink, AudioSinkPlayback, PlaybackMode, SpatialAudioSink},
    prelude::*,
};
use game_audio::{
    spatial_listener, AudioPaused, Bus, Curve, Falloff, Placement, PlaySound, Volumes,
};
use rodio::source::{SineWave, Spatial};

use common::{audio_app, clip};

const EAR_GAP: f32 = 0.18;

fn playback_app(listener: Option<Vec3>) -> App {
    let mut app = audio_app();
    if let Some(position) = listener {
        app.world_mut().spawn((
            spatial_listener(EAR_GAP),
            Transform::from_translation(position),
        ));
    }
    app.update();
    app
}

type Voice = (Handle<AudioSource>, Bus, PlaybackSettings, Option<Vec3>);

fn play(app: &mut App, cues: &[PlaySound]) -> Vec<Voice> {
    for cue in cues {
        app.world_mut().write_message(cue.clone());
    }
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<(
        Entity,
        &AudioPlayer,
        &Bus,
        &PlaybackSettings,
        Option<&Transform>,
    )>();
    let played: Vec<_> = query
        .iter(world)
        .map(|(entity, player, bus, settings, transform)| {
            (
                entity,
                (
                    player.0.clone(),
                    *bus,
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

fn source_voice(
    app: &mut App,
    handle: Handle<AudioSource>,
    position: Vec3,
    falloff: Falloff,
) -> Option<(Handle<AudioSource>, f32)> {
    let source = app
        .world_mut()
        .spawn(Transform::from_translation(position))
        .id();
    app.update();
    app.world_mut()
        .write_message(PlaySound::from(handle, source, Vec3::ZERO, falloff));
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

fn source_volume(app: &mut App, distance: f32, falloff: Falloff) -> Option<f32> {
    source_voice(app, clip(1), Vec3::new(distance, 0.0, 0.0), falloff).map(|(_, volume)| volume)
}

#[test]
fn volumes_scale_ui_and_world_cues_by_master_and_bus() {
    let cues = [
        PlaySound::global(clip(1), Bus::Ui, 0.7),
        PlaySound::global(clip(2), Bus::World, 0.9),
        PlaySound::at(clip(3), Vec3::X, Falloff::DEFAULT),
    ];
    let base = play(&mut playback_app(Some(Vec3::ZERO)), &cues);
    let mut scaled_app = playback_app(Some(Vec3::ZERO));
    scaled_app.insert_resource(Volumes {
        master: 0.5,
        world: 0.4,
        ui: 0.2,
        music: 0.3,
    });
    let scaled = play(&mut scaled_app, &cues);
    assert_eq!(base.len(), 3);
    assert_eq!(scaled.len(), 3);
    let voice = |voices: &[Voice], id: u128| {
        voices
            .iter()
            .find(|voice| voice.0 == clip(id))
            .cloned()
            .unwrap()
    };
    for id in 1..=3 {
        let base = voice(&base, id);
        let scaled = voice(&scaled, id);
        assert_eq!(base.1, scaled.1);
        let factor = match base.1 {
            Bus::Ui => 0.1,
            Bus::World => 0.2,
            Bus::Music => 0.15,
        };
        let base = base.2.volume.to_linear();
        let scaled = scaled.2.volume.to_linear();
        assert!(base > 0.0);
        assert!((scaled - base * factor).abs() < 0.001);
    }
    assert_eq!(voice(&base, 1).1, Bus::Ui);
    assert_eq!(voice(&base, 1).2.volume.to_linear(), 0.7);
    assert_eq!(voice(&base, 2).1, Bus::World);
    assert_eq!(voice(&base, 2).2.volume.to_linear(), 0.9);
    assert_eq!(
        voice(&base, 3).2.volume.to_linear(),
        Falloff::DEFAULT.gain(1.0).unwrap()
    );
}

#[test]
fn volumes_gain_multiplies_master_with_the_bus() {
    let volumes = Volumes {
        master: 0.5,
        world: 0.4,
        ui: 0.2,
        music: 0.6,
    };
    assert!((volumes.gain(Bus::World) - 0.2).abs() < 0.001);
    assert!((volumes.gain(Bus::Ui) - 0.1).abs() < 0.001);
    assert!((volumes.gain(Bus::Music) - 0.3).abs() < 0.001);
    assert_eq!(Volumes::default().gain(Bus::World), 1.0);
    assert_eq!(Volumes::default().gain(Bus::Ui), 1.0);
    assert_eq!(Volumes::default().gain(Bus::Music), 1.0);
}

#[test]
fn global_cues_are_non_spatial_despawn_voices_on_their_bus() {
    let mut app = playback_app(None);
    let voices = play(
        &mut app,
        &[
            PlaySound::global(clip(1), Bus::World, 1.0),
            PlaySound::global(clip(2), Bus::World, 0.5),
            PlaySound::global(clip(3), Bus::Ui, 0.8),
        ],
    );
    assert_eq!(voices.len(), 3);
    assert_eq!(voices[0].0, clip(1));
    assert_eq!(voices[1].0, clip(2));
    assert_eq!(voices[2].0, clip(3));
    assert_eq!(voices[0].1, Bus::World);
    assert_eq!(voices[1].1, Bus::World);
    assert_eq!(voices[2].1, Bus::Ui);
    assert_eq!(voices[1].2.volume.to_linear(), 0.5);
    assert!(voices.iter().all(|(_, _, settings, position)| {
        !settings.spatial && position.is_none() && matches!(settings.mode, PlaybackMode::Despawn)
    }));
}

#[test]
fn loop_cues_are_non_spatial_looping_voices_on_their_bus() {
    let mut app = playback_app(None);
    let voices = play(
        &mut app,
        &[PlaySound::loop_global(clip(1), Bus::Music, 0.8)],
    );
    assert_eq!(voices.len(), 1);
    let (handle, bus, settings, position) = &voices[0];
    assert_eq!(*handle, clip(1));
    assert_eq!(*bus, Bus::Music);
    assert!(!settings.spatial && position.is_none());
    assert!(matches!(settings.mode, PlaybackMode::Loop));
    assert_eq!(settings.volume.to_linear(), 0.8);
}

#[test]
fn loop_cues_scale_with_the_music_bus_and_keep_playing_while_paused() {
    let mut app = playback_app(None);
    app.insert_resource(Volumes {
        master: 0.5,
        world: 1.0,
        ui: 1.0,
        music: 0.4,
    });
    let voices = play(
        &mut app,
        &[PlaySound::loop_global(clip(1), Bus::Music, 1.0)],
    );
    assert_eq!(voices.len(), 1);
    assert!((voices[0].2.volume.to_linear() - 0.2).abs() < 0.001);

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    let voices = play(
        &mut app,
        &[PlaySound::loop_global(clip(2), Bus::Music, 1.0)],
    );
    assert_eq!(voices.len(), 1);
    assert_eq!(voices[0].0, clip(2));
}

#[test]
fn pause_drops_world_cues_but_keeps_ui_cues() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let source = app.world_mut().spawn(Transform::IDENTITY).id();
    app.update();
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    let played = play(
        &mut app,
        &[
            PlaySound::global(clip(1), Bus::World, 1.0),
            PlaySound::at(clip(2), Vec3::X, Falloff::DEFAULT),
            PlaySound::from(clip(3), source, Vec3::ZERO, Falloff::DEFAULT),
            PlaySound::global(clip(4), Bus::Ui, 1.0),
        ],
    );
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].0, clip(4));
    assert_eq!(played[0].1, Bus::Ui);
    assert!(app.world().get::<Children>(source).is_none());

    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    let played = play(
        &mut app,
        &[
            PlaySound::global(clip(1), Bus::World, 1.0),
            PlaySound::at(clip(2), Vec3::X, Falloff::DEFAULT),
        ],
    );
    assert_eq!(played.len(), 2);
}

#[test]
fn distant_spatial_cues_pan_toward_the_source_for_both_listener_orientations() {
    for yaw in [0.0, std::f32::consts::FRAC_PI_2] {
        let transform = Transform::from_rotation(Quat::from_rotation_y(yaw));
        let listener = spatial_listener(EAR_GAP);
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
fn spatial_listener_splits_the_ear_gap_across_the_x_axis() {
    let listener = spatial_listener(EAR_GAP);
    assert_eq!(listener.left_ear_offset, Vec3::X * 0.09);
    assert_eq!(listener.right_ear_offset, Vec3::NEG_X * 0.09);
}

#[test]
fn positional_cue_plays_once_as_spatial_world_audio_with_falloff_gain() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let position = Vec3::new(-13.4, 1.5, 0.0);
    let cue = PlaySound::at(clip(7), position, Falloff::DEFAULT);
    assert_eq!(cue.bus, Bus::World);
    assert_eq!(
        cue.placement,
        Placement::At {
            position,
            falloff: Falloff::DEFAULT
        }
    );
    let played = play(&mut app, std::slice::from_ref(&cue));

    assert_eq!(played.len(), 1);
    let (handle, bus, settings, translation) = &played[0];
    assert_eq!(*handle, clip(7));
    assert_eq!(*bus, Bus::World);
    assert!(matches!(settings.mode, PlaybackMode::Despawn));
    assert!(settings.spatial);
    assert_eq!(
        settings.spatial_scale.map(|scale| scale.0),
        Some(Vec3::splat(Falloff::DEFAULT.scale))
    );
    assert_eq!(*translation, Some(position));
    let distance = position.length();
    let gain = 0.6 / (1.0 + 0.06 * distance * distance);
    assert_eq!(settings.volume.to_linear(), gain);
    assert_eq!(Falloff::DEFAULT.gain(distance), Some(gain));

    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    assert!(play(&mut app, &[cue]).is_empty());
}

#[test]
fn positional_cues_need_a_listener_within_range() {
    let position = Vec3::new(-13.4, 1.5, 0.0);
    let cue = PlaySound::at(clip(1), position, Falloff::DEFAULT);
    assert!(play(&mut playback_app(None), std::slice::from_ref(&cue)).is_empty());
    assert!(play(
        &mut playback_app(Some(Vec3::new(20.0, 0.0, 0.0))),
        std::slice::from_ref(&cue)
    )
    .is_empty());
    assert_eq!(
        play(&mut playback_app(Some(Vec3::new(-13.4, 1.5, 24.0))), &[cue]).len(),
        1
    );
}

#[test]
fn source_cue_is_a_child_of_its_source_and_follows_its_transform() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let boiler = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 0.0, 0.0))
        .id();
    app.update();
    let cue = PlaySound::from(clip(5), boiler, Vec3::Y * 1.2, Falloff::DEFAULT);
    assert_eq!(cue.bus, Bus::World);
    assert_eq!(
        cue.placement,
        Placement::From {
            source: boiler,
            offset: Vec3::Y * 1.2,
            falloff: Falloff::DEFAULT
        }
    );
    app.world_mut().write_message(cue);
    app.update();
    let children = app.world().get::<Children>(boiler).unwrap();
    assert_eq!(children.len(), 1);
    let voice = children[0];
    assert_eq!(app.world().get::<AudioPlayer>(voice).unwrap().0, clip(5));
    assert_eq!(*app.world().get::<Bus>(voice).unwrap(), Bus::World);
    assert_eq!(app.world().get::<ChildOf>(voice).unwrap().parent(), boiler);
    assert_eq!(
        app.world().get::<Transform>(voice).unwrap().translation,
        Vec3::Y * 1.2
    );
    let settings = app.world().get::<PlaybackSettings>(voice).unwrap();
    assert!(settings.spatial);
    assert!(matches!(settings.mode, PlaybackMode::Despawn));
    let distance = Vec3::new(1.0, 1.2, 0.0).length();
    assert_eq!(
        settings.volume.to_linear(),
        Falloff::DEFAULT.gain(distance).unwrap()
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
fn source_cue_measures_distance_from_the_offset_point() {
    let mut app = playback_app(Some(Vec3::ZERO));
    let source = app
        .world_mut()
        .spawn(Transform::from_xyz(20.0, 0.0, 0.0))
        .id();
    app.update();
    app.world_mut().write_message(PlaySound::from(
        clip(1),
        source,
        Vec3::X * 6.0,
        Falloff::DEFAULT,
    ));
    app.update();
    assert!(app.world().get::<Children>(source).is_none());
    app.world_mut().write_message(PlaySound::from(
        clip(1),
        source,
        Vec3::NEG_X * 6.0,
        Falloff::DEFAULT,
    ));
    app.update();
    let voice = app.world().get::<Children>(source).unwrap()[0];
    assert_eq!(
        app.world()
            .get::<PlaybackSettings>(voice)
            .unwrap()
            .volume
            .to_linear(),
        Falloff::DEFAULT.gain(14.0).unwrap()
    );
}

#[test]
fn source_cues_require_a_listener_a_source_and_unpaused_world() {
    let mut app = playback_app(None);
    let boiler = app.world_mut().spawn(Transform::IDENTITY).id();
    let cue = PlaySound::from(clip(1), boiler, Vec3::Y, Falloff::DEFAULT);
    app.world_mut().write_message(cue.clone());
    app.update();
    assert!(app.world().get::<Children>(boiler).is_none());
    app.world_mut()
        .spawn((spatial_listener(EAR_GAP), Transform::IDENTITY));
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.world_mut().write_message(cue.clone());
    app.update();
    assert!(app.world().get::<Children>(boiler).is_none());
    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.world_mut().write_message(cue);
    app.update();
    assert_eq!(app.world().get::<Children>(boiler).unwrap().len(), 1);

    let missing = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(PlaySound::from(
        clip(2),
        missing,
        Vec3::ZERO,
        Falloff::DEFAULT,
    ));
    app.update();
    assert!(app.world().get::<Children>(missing).is_none());
    let world = app.world_mut();
    let mut query = world.query::<&AudioPlayer>();
    assert_eq!(query.iter(world).count(), 1);
}

#[test]
fn falloff_curves_fade_across_distinct_ranges() {
    let step = Falloff {
        range: 16.0,
        scale: 0.3,
        curve: Curve::SquaredFade { gain: 1.0 },
    };
    let presence = Falloff {
        range: 32.0,
        scale: 0.3,
        curve: Curve::SquaredFade { gain: 0.8 },
    };
    assert_eq!(Falloff::DEFAULT.range, 25.0);

    let mut app = playback_app(Some(Vec3::ZERO));
    let step_near = source_volume(&mut app, 10.0, step).unwrap();
    let step_far = source_volume(&mut app, 15.0, step).unwrap();
    assert!(step_near > step_far);
    assert_eq!(source_volume(&mut app, 16.0, step), None);
    assert_eq!(source_volume(&mut app, 20.0, step), None);
    let presence_near = source_volume(&mut app, 20.0, presence).unwrap();
    let presence_far = source_volume(&mut app, 30.0, presence).unwrap();
    assert!(presence_near > presence_far);
    assert!(presence_far > 0.0);
    assert_eq!(source_volume(&mut app, 32.0, presence), None);

    let fade = 1.0 - 20.0 / presence.range;
    assert!((presence_near - 0.8 * fade * fade).abs() < 0.001);
    assert_eq!(presence.gain(20.0), Some(presence_near));
    let (handle, _) = source_voice(&mut app, clip(9), Vec3::new(20.0, 0.0, 0.0), presence).unwrap();
    assert_eq!(handle, clip(9));
}

#[test]
fn inverse_square_falloff_remains_audible_across_its_range() {
    let detection = Falloff {
        range: 25.0,
        scale: 0.3,
        curve: Curve::InverseSquare {
            gain: 1.0,
            coefficient: 0.01,
        },
    };
    let step = Falloff {
        range: 16.0,
        scale: 0.3,
        curve: Curve::SquaredFade { gain: 1.0 },
    };
    let mut app = playback_app(Some(Vec3::ZERO));
    let close = source_volume(&mut app, 0.0, detection).unwrap();
    let far = source_volume(&mut app, 12.0, detection).unwrap();
    let step_far = source_volume(&mut app, 12.0, step).unwrap();
    assert_eq!(close, 1.0);
    assert!(close > far);
    assert!(far > 0.25);
    assert!(far > step_far);
    assert_eq!(detection.gain(12.0), Some(far));
    assert_eq!(detection.gain(25.0), None);
    assert_eq!(detection.gain(40.0), None);
}

#[test]
fn falloff_gain_is_none_at_and_beyond_range() {
    let falloff = Falloff::DEFAULT;
    assert_eq!(falloff.gain(0.0), Some(0.6));
    assert!(falloff.gain(24.9).is_some());
    assert_eq!(falloff.gain(25.0), None);
    assert_eq!(falloff.gain(100.0), None);
    let fade = Falloff {
        range: 10.0,
        scale: 1.0,
        curve: Curve::SquaredFade { gain: 0.5 },
    };
    assert_eq!(fade.gain(0.0), Some(0.5));
    assert_eq!(fade.gain(5.0), Some(0.125));
    assert_eq!(fade.gain(10.0), None);
}

fn sinks(app: &mut App) -> (Entity, Entity, Entity, rodio::mixer::MixerSource) {
    let (world_player, _) = rodio::Player::new();
    let (ui_player, _) = rodio::Player::new();
    let (mixer, source) = rodio::mixer::mixer(
        std::num::NonZero::new(2).unwrap(),
        std::num::NonZero::new(48_000).unwrap(),
    );
    let spatial = rodio::SpatialPlayer::connect_new(&mixer, [0.0; 3], [0.0; 3], [0.0; 3]);
    let world = app
        .world_mut()
        .spawn((Bus::World, AudioSink::new(world_player)))
        .id();
    let ui = app
        .world_mut()
        .spawn((Bus::Ui, AudioSink::new(ui_player)))
        .id();
    let spatial = app
        .world_mut()
        .spawn((Bus::World, SpatialAudioSink::new(spatial)))
        .id();
    (world, ui, spatial, source)
}

fn paused_sinks(app: &App, world: Entity, ui: Entity, spatial: Entity) -> [bool; 3] {
    [
        app.world().get::<AudioSink>(world).unwrap().is_paused(),
        app.world().get::<AudioSink>(ui).unwrap().is_paused(),
        app.world()
            .get::<SpatialAudioSink>(spatial)
            .unwrap()
            .is_paused(),
    ]
}

#[test]
fn world_sinks_created_during_a_pause_start_paused() {
    let mut app = playback_app(None);
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.update();
    let (world, ui, spatial, _source) = sinks(&mut app);
    app.update();
    assert_eq!(paused_sinks(&app, world, ui, spatial), [true, false, true]);
    app.update();
    assert_eq!(paused_sinks(&app, world, ui, spatial), [true, false, true]);
    app.world_mut().resource_mut::<AudioPaused>().0 = false;
    app.update();
    assert_eq!(
        paused_sinks(&app, world, ui, spatial),
        [false, false, false]
    );
}

#[test]
fn sinks_created_while_playing_keep_playing() {
    let mut app = playback_app(None);
    let (world, ui, spatial, _source) = sinks(&mut app);
    app.update();
    assert_eq!(
        paused_sinks(&app, world, ui, spatial),
        [false, false, false]
    );
    app.world_mut().resource_mut::<AudioPaused>().0 = true;
    app.update();
    assert_eq!(paused_sinks(&app, world, ui, spatial), [true, false, true]);
}
