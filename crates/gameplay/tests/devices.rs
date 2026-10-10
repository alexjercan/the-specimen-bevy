use std::{f32::consts::PI, time::Duration};

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, Sound};
use gameplay::{
    achievements::{Achievement, AchievementPlugin, AchievementProgress, RunAchievements},
    controller::{Flashlight, PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        detector_reading, pulse_interval, Caught, Detector, DevicePlugin, DoorPlugin,
        FlashExposure, Flashbangs, Flashed, FuseInventory, FusePlugin, Monster, PickupKind, Room,
        DETECTOR_RANGE, FLASHBANG_BURST_DELAY, FLASHBANG_DURATION, PULSE_FAST, PULSE_NEAR,
        PULSE_SLOW,
    },
};

#[derive(Resource, Default)]
struct Heard(Vec<Sound>);

fn collect(mut sounds: MessageReader<PlaySound>, mut heard: ResMut<Heard>) {
    heard.0.extend(sounds.read().map(|cue| cue.sound));
}

fn cues(app: &mut App, sound: Sound) -> usize {
    let heard = std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0);
    heard.into_iter().filter(|&cue| cue == sound).count()
}

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins((DoorPlugin, FusePlugin, DevicePlugin, AchievementPlugin))
        .init_resource::<Heard>()
        .add_systems(PostUpdate, collect);
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.5, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

fn device(app: &mut App, kind: PickupKind, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((kind, Transform::from_translation(position)))
        .id()
}

fn aim(app: &mut App, player: Entity, target: Vec3) {
    let eye = app.world().get::<Transform>(player).unwrap().translation;
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_translation(eye).looking_at(target, Vec3::Y));
}

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
    app.update();
}

fn click(app: &mut App, button: MouseButton) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(button);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(button);
    app.update();
}

fn exists(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

fn flashbangs(app: &App, player: Entity) -> Option<usize> {
    app.world().get::<Flashbangs>(player).map(|count| count.0)
}

#[test]
fn f_picks_up_the_aimed_flashbang_and_detector_once_each() {
    let (mut app, player) = app();
    let flashbang = device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -1.5));
    let detector = device(&mut app, PickupKind::Detector, Vec3::new(1.5, 0.9, -1.5));
    assert_eq!(flashbangs(&app, player), None);
    assert!(app.world().get::<Detector>(player).is_none());

    aim(&mut app, player, Vec3::new(0.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(flashbangs(&app, player), Some(1));
    assert!(
        app.world()
            .get::<RunAchievements>(player)
            .unwrap()
            .picked_flashbang
    );
    assert!(!exists(&app, flashbang));
    assert!(app.world().get::<Detector>(player).is_none());

    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(flashbangs(&app, player), Some(1));

    aim(&mut app, player, Vec3::new(1.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert!(!exists(&app, detector));
    assert!(
        app.world()
            .get::<RunAchievements>(player)
            .unwrap()
            .picked_detector
    );
    assert_eq!(
        app.world().get::<Detector>(player),
        Some(&Detector { reading: None })
    );
    assert_eq!(flashbangs(&app, player), Some(1));
}

#[test]
fn walls_block_device_pickup_through_structural_sight() {
    let (mut app, player) = app();
    let room = app
        .world_mut()
        .spawn(Room(Rect::new(-2.5, -2.5, 2.5, 2.5)))
        .id();
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.5, 1.6, -1.5));
    let flashbang = device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -3.0));
    aim(&mut app, player, Vec3::new(0.5, 0.96, -3.0));
    press_key(&mut app, KeyCode::KeyF);
    assert!(exists(&app, flashbang));
    assert_eq!(flashbangs(&app, player), None);

    app.world_mut().despawn(room);
    press_key(&mut app, KeyCode::KeyF);
    assert!(!exists(&app, flashbang));
    assert_eq!(flashbangs(&app, player), Some(1));
}

#[test]
fn nearer_fuse_wins_arbitration_over_a_device_behind_it() {
    let (mut app, player) = app();
    let fuse = app
        .world_mut()
        .spawn((
            PickupKind::Fuse { slot: 0 },
            Transform::from_xyz(0.5, 0.9, -1.0),
        ))
        .id();
    let flashbang = device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -2.0));
    aim(&mut app, player, Vec3::new(0.5, 0.93, -1.0));
    press_key(&mut app, KeyCode::KeyF);
    assert!(!exists(&app, fuse));
    assert!(exists(&app, flashbang));
    assert_eq!(app.world().get::<FuseInventory>(player).unwrap().0, 1);

    aim(&mut app, player, Vec3::new(0.5, 0.96, -2.0));
    press_key(&mut app, KeyCode::KeyF);
    assert!(!exists(&app, flashbang));
    assert_eq!(flashbangs(&app, player), Some(1));
}

#[test]
fn right_click_consumes_one_flashbang_and_protects_only_after_a_hit() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -4.0)));
    click(&mut app, MouseButton::Right);
    assert!(app.world().get::<Flashed>(player).is_none());

    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(0));
    assert!(app.world().get::<Flashed>(player).is_none());
    for _ in 0..8 {
        app.update();
    }
    let flashed = *app.world().get::<Flashed>(player).unwrap();
    assert!(flashed.remaining > FLASHBANG_DURATION - 0.5);
    assert!(!app.world().get::<Flashlight>(player).unwrap().on);

    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(0));
    assert!(app.world().get::<Flashed>(player).unwrap().remaining < flashed.remaining);
}

#[test]
fn white_flash_requires_facing_the_visible_nearby_burst_not_a_monster_hit() {
    let (mut app, player) = app();
    app.world_mut().entity_mut(player).insert(Flashbangs(4));

    click(&mut app, MouseButton::Right);
    assert!(app.world().get::<FlashExposure>(player).is_none());
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<FlashExposure>(player).is_some());
    assert!(app.world().get::<Flashed>(player).is_none());
    for _ in 0..12 {
        app.update();
    }
    assert!(app.world().get::<FlashExposure>(player).is_none());

    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -4.0)));
    click(&mut app, MouseButton::Right);
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .rotation = Quat::from_rotation_y(PI);
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<FlashExposure>(player).is_none());
    assert!(app.world().get::<Flashed>(player).is_some());

    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .rotation = Quat::IDENTITY;
    click(&mut app, MouseButton::Right);
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .translation
        .x += 15.0;
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<FlashExposure>(player).is_none());
}

#[test]
fn wall_blocks_white_flash_even_when_player_faces_burst() {
    let (mut app, player) = app();
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    app.world_mut().spawn(Room(Rect::new(-2.0, -2.0, 2.0, 2.0)));
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<FlashExposure>(player).is_none());
}

#[test]
fn left_click_still_toggles_the_flashlight_while_holding_a_flashbang() {
    let (mut app, player) = app();
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Left);
    assert!(app.world().get::<Flashlight>(player).unwrap().on);
    assert_eq!(flashbangs(&app, player), Some(1));
    assert!(app.world().get::<Flashed>(player).is_none());
}

#[test]
fn flash_effect_expires_after_its_duration() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -4.0)));
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    for _ in 0..8 {
        app.update();
    }
    let remaining = app.world().get::<Flashed>(player).unwrap().remaining;
    let mut frames = 0;
    while app.world().get::<Flashed>(player).is_some() {
        app.update();
        frames += 1;
        assert!(frames < 80, "flash must expire");
    }
    let expected = (remaining / 0.1).round() as usize;
    assert!(
        (expected..=expected + 1).contains(&frames),
        "flash lasted {frames} frames"
    );
    for _ in 0..100 {
        app.update();
    }
    assert_eq!(flashbangs(&app, player), Some(0));
    click(&mut app, MouseButton::Right);
    assert!(app.world().get::<Flashed>(player).is_none());
    assert_eq!(flashbangs(&app, player), Some(0));
}

#[test]
fn paused_controls_block_pickup_and_use_and_freeze_the_flash() {
    let (mut app, player) = app();
    let flashbang = device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -1.5));
    aim(&mut app, player, Vec3::new(0.5, 0.96, -1.5));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    press_key(&mut app, KeyCode::KeyF);
    assert!(exists(&app, flashbang));
    assert_eq!(flashbangs(&app, player), None);

    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(1));
    assert!(app.world().get::<Flashed>(player).is_none());

    app.world_mut()
        .entity_mut(player)
        .insert(Flashed { remaining: 2.0 });
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Flashed>(player),
        Some(&Flashed { remaining: 2.0 })
    );

    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    for _ in 0..25 {
        app.update();
    }
    assert!(app.world().get::<Flashed>(player).is_none());
}

#[test]
fn caught_player_cannot_pick_up_or_use_a_flashbang() {
    let (mut app, player) = app();
    let monster = app.world_mut().spawn_empty().id();
    let flashbang = device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -1.5));
    aim(&mut app, player, Vec3::new(0.5, 0.96, -1.5));
    app.world_mut()
        .entity_mut(player)
        .insert((Caught::new(monster), Flashbangs(1)));
    press_key(&mut app, KeyCode::KeyF);
    assert!(exists(&app, flashbang));
    assert_eq!(flashbangs(&app, player), Some(1));
    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(1));
    assert!(app.world().get::<Flashed>(player).is_none());
    assert!(app.world().get::<Caught>(player).is_some());
}

#[test]
fn detector_reading_gives_horizontal_range_and_signed_bearing_within_the_cap() {
    let facing_forward = Transform::from_xyz(0.0, 1.6, 0.0);
    let ahead = detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -10.0)).unwrap();
    assert!((ahead.distance - 10.0).abs() < 1e-4);
    assert!(ahead.bearing.abs() < 1e-4);
    let right = detector_reading(&facing_forward, Vec3::new(5.0, 0.0, 0.0)).unwrap();
    assert!((right.bearing - PI / 2.0).abs() < 1e-4);
    let left = detector_reading(&facing_forward, Vec3::new(-5.0, 0.0, 0.0)).unwrap();
    assert!((left.bearing + PI / 2.0).abs() < 1e-4);
    let behind = detector_reading(&facing_forward, Vec3::new(0.0, 0.0, 5.0)).unwrap();
    assert!((behind.bearing.abs() - PI).abs() < 1e-4);

    let turned = Transform::from_xyz(0.0, 1.6, 0.0).with_rotation(Quat::from_euler(
        EulerRot::YXZ,
        PI / 2.0,
        0.4,
        0.0,
    ));
    let ahead_after_turn = detector_reading(&turned, Vec3::new(-10.0, 0.0, 0.0)).unwrap();
    assert!(ahead_after_turn.bearing.abs() < 1e-4);

    let inside = detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -DETECTOR_RANGE + 0.01));
    assert!(inside.is_some());
    let outside = detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -DETECTOR_RANGE - 0.01));
    assert!(outside.is_none());
}

#[test]
fn detector_tracks_the_monster_only_after_pickup_and_inside_the_cap() {
    let (mut app, player) = app();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -12.0)))
        .id();
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.5, 1.6, 0.0));
    app.update();
    assert!(app.world().get::<Detector>(player).is_none());

    app.world_mut()
        .entity_mut(player)
        .insert(Detector::default());
    app.update();
    let reading = app
        .world()
        .get::<Detector>(player)
        .unwrap()
        .reading
        .unwrap();
    assert!((reading.distance - 12.0).abs() < 1e-3);
    assert!(reading.bearing.abs() < 1e-3);

    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(0.5, 0.0, -30.0));
    app.update();
    assert_eq!(
        app.world().get::<Detector>(player),
        Some(&Detector { reading: None })
    );
}

#[test]
fn detector_range_reaches_twenty_five_metres() {
    assert_eq!(DETECTOR_RANGE, 25.0);
    let facing_forward = Transform::from_xyz(0.0, 1.6, 0.0);
    let twenty = detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -20.0)).unwrap();
    assert!((twenty.distance - 20.0).abs() < 1e-4);
    assert!(detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -24.99)).is_some());
    assert!(detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -25.0)).is_some());
    assert!(detector_reading(&facing_forward, Vec3::new(0.0, 0.0, -25.01)).is_none());
}

#[test]
fn pulse_cadence_boundaries_follow_the_twenty_five_metre_cap() {
    assert_eq!(pulse_interval(PULSE_NEAR), PULSE_FAST);
    assert!(pulse_interval(20.0) > PULSE_FAST);
    assert!(pulse_interval(20.0) < PULSE_SLOW - 0.2);
    assert!(pulse_interval(24.99) < PULSE_SLOW);
    assert!((pulse_interval(25.0) - PULSE_SLOW).abs() < 1e-6);
    assert_eq!(pulse_interval(25.01), PULSE_SLOW);
    let midpoint = (PULSE_NEAR + 25.0) / 2.0;
    assert!((pulse_interval(midpoint) - (PULSE_FAST + PULSE_SLOW) / 2.0).abs() < 1e-5);
}

#[test]
fn nearby_cue_keeps_pulsing_between_twenty_and_twenty_five_metres() {
    let (mut app, player, _) = detector_app(22.0);
    app.world_mut()
        .entity_mut(player)
        .insert(Detector::default());
    let pulses = nearby_cues(&mut app, 40);
    assert!((2..=3).contains(&pulses), "pulses at 22 m: {pulses}");
}

#[test]
fn only_the_detector_pickup_plays_the_detector_pickup_cue() {
    let (mut app, player) = app();
    device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -1.5));
    device(&mut app, PickupKind::Detector, Vec3::new(1.5, 0.9, -1.5));
    cues(&mut app, Sound::DetectorPickup);
    aim(&mut app, player, Vec3::new(0.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::DetectorPickup), 0);
    aim(&mut app, player, Vec3::new(1.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::DetectorPickup), 1);
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::DetectorPickup), 0);
}

#[test]
fn pulse_interval_shrinks_from_the_cap_to_the_near_floor() {
    assert_eq!(pulse_interval(0.0), PULSE_FAST);
    assert_eq!(pulse_interval(PULSE_NEAR), PULSE_FAST);
    assert!((pulse_interval(DETECTOR_RANGE) - PULSE_SLOW).abs() < 1e-6);
    assert_eq!(pulse_interval(DETECTOR_RANGE + 5.0), PULSE_SLOW);
    let mut previous = 0.0;
    for step in 0..=40 {
        let interval = pulse_interval(step as f32 * 0.5);
        assert!(interval >= previous);
        previous = interval;
    }
}

fn nearby_cues(app: &mut App, frames: usize) -> usize {
    cues(app, Sound::DetectorNearby);
    for _ in 0..frames {
        app.update();
    }
    cues(app, Sound::DetectorNearby)
}

fn detector_app(monster_distance: f32) -> (App, Entity, Entity) {
    let (mut app, player) = app();
    let monster = app
        .world_mut()
        .spawn((
            Monster::default(),
            Transform::from_xyz(0.5, 0.0, -monster_distance),
        ))
        .id();
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.5, 1.6, 0.0));
    (app, player, monster)
}

#[test]
fn nearby_cue_needs_the_detector_and_pulses_faster_when_the_monster_is_closer() {
    let (mut app, player, monster) = detector_app(2.0);
    assert_eq!(nearby_cues(&mut app, 40), 0);

    app.world_mut()
        .entity_mut(player)
        .insert(Detector::default());
    let near = nearby_cues(&mut app, 40);
    assert!((19..=20).contains(&near), "near pulses: {near}");

    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(0.5, 0.0, -11.0));
    let middle = nearby_cues(&mut app, 40);
    assert!((4..=5).contains(&middle), "middle pulses: {middle}");

    app.world_mut()
        .entity_mut(monster)
        .insert(Transform::from_xyz(0.5, 0.0, -(DETECTOR_RANGE - 0.05)));
    let far = nearby_cues(&mut app, 40);
    assert!((2..=3).contains(&far), "far pulses: {far}");
    assert!(near > middle && middle > far);
}

#[test]
fn nearby_cue_is_silent_beyond_the_detector_cap() {
    let (mut app, player, _) = detector_app(DETECTOR_RANGE + 0.05);
    app.world_mut()
        .entity_mut(player)
        .insert(Detector::default());
    assert_eq!(nearby_cues(&mut app, 80), 0);
    assert_eq!(
        app.world().get::<Detector>(player),
        Some(&Detector { reading: None })
    );
}

#[test]
fn nearby_cue_stops_while_controls_are_disabled_or_the_player_is_caught() {
    let (mut app, player, monster) = detector_app(2.0);
    app.world_mut()
        .entity_mut(player)
        .insert(Detector::default());
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    assert_eq!(nearby_cues(&mut app, 40), 0);

    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    assert!(nearby_cues(&mut app, 40) > 0);

    app.world_mut()
        .entity_mut(player)
        .insert(Caught::new(monster));
    assert_eq!(nearby_cues(&mut app, 40), 0);
}

#[test]
fn flashbang_pickup_and_throw_each_play_their_cue_exactly_once() {
    let (mut app, player) = app();
    device(&mut app, PickupKind::Detector, Vec3::new(1.5, 0.9, -1.5));
    device(&mut app, PickupKind::Flashbang, Vec3::new(0.5, 0.9, -1.5));
    aim(&mut app, player, Vec3::new(1.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::FlashbangPickup), 0);

    click(&mut app, MouseButton::Right);
    assert_eq!(cues(&mut app, Sound::FlashbangThrow), 0);

    aim(&mut app, player, Vec3::new(0.5, 0.96, -1.5));
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::FlashbangPickup), 1);
    press_key(&mut app, KeyCode::KeyF);
    assert_eq!(cues(&mut app, Sound::FlashbangPickup), 0);

    click(&mut app, MouseButton::Right);
    assert_eq!(cues(&mut app, Sound::FlashbangThrow), 1);
    click(&mut app, MouseButton::Right);
    for _ in 0..60 {
        app.update();
    }
    click(&mut app, MouseButton::Right);
    assert_eq!(cues(&mut app, Sound::FlashbangThrow), 0);
    assert_eq!(cues(&mut app, Sound::FlashbangPickup), 0);
}

#[test]
fn blocked_flashbang_throw_plays_no_cue() {
    let (mut app, player) = app();
    let monster = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    click(&mut app, MouseButton::Right);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.world_mut()
        .entity_mut(player)
        .insert(Caught::new(monster));
    click(&mut app, MouseButton::Right);
    assert_eq!(cues(&mut app, Sound::FlashbangThrow), 0);
    assert_eq!(flashbangs(&app, player), Some(1));
}

fn heard(app: &mut App) -> Vec<Sound> {
    std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0)
}

#[test]
fn flashbang_burst_plays_once_at_detonation_after_the_throw() {
    let (mut app, player) = app();
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    heard(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    assert!(app.world().get::<Flashed>(player).is_none());
    let detonation: Vec<_> = heard(&mut app)
        .into_iter()
        .filter(|sound| matches!(sound, Sound::FlashbangThrow | Sound::FlashbangBurst))
        .collect();
    assert_eq!(detonation, vec![Sound::FlashbangThrow]);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    for _ in 1..(FLASHBANG_BURST_DELAY * 10.0) as usize - 1 {
        app.update();
    }
    assert_eq!(cues(&mut app, Sound::FlashbangBurst), 0);
    app.update();
    assert_eq!(cues(&mut app, Sound::FlashbangBurst), 1);
    click(&mut app, MouseButton::Right);
    for _ in 0..60 {
        app.update();
    }
    assert!(app.world().get::<Flashed>(player).is_none());
    assert_eq!(cues(&mut app, Sound::FlashbangBurst), 0);
}

#[test]
fn burst_records_hit_on_nearby_visible_monster_without_changing_it() {
    let (mut app, player) = app();
    let near = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -4.0)))
        .id();
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    for _ in 0..8 {
        app.update();
    }
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::FlashbangHitMonster));
    assert!(app.world().get::<Monster>(near).is_some());
}

#[test]
fn rapid_throws_with_cheated_stack_each_burst_independently() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -4.0)));
    app.world_mut().entity_mut(player).insert(Flashbangs(10));
    heard(&mut app);
    click(&mut app, MouseButton::Right);
    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(8));
    assert_eq!(cues(&mut app, Sound::FlashbangThrow), 2);
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(cues(&mut app, Sound::FlashbangBurst), 2);
    assert!(app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::FlashbangHitMonster));
    assert!(app.world().get::<Flashed>(player).is_some());
    click(&mut app, MouseButton::Right);
    assert_eq!(flashbangs(&app, player), Some(7));
    assert!(app.world().get::<Flashed>(player).is_some());
}

#[test]
fn twenty_rapid_flashbang_throws_each_burst_once() {
    let (mut app, player) = app();
    app.world_mut().entity_mut(player).insert(Flashbangs(20));
    heard(&mut app);
    for _ in 0..20 {
        click(&mut app, MouseButton::Right);
    }
    assert_eq!(flashbangs(&app, player), Some(0));
    for _ in 0..8 {
        app.update();
    }
    let sounds = heard(&mut app);
    assert_eq!(
        sounds
            .iter()
            .filter(|&&cue| cue == Sound::FlashbangThrow)
            .count(),
        20
    );
    assert_eq!(
        sounds
            .iter()
            .filter(|&&cue| cue == Sound::FlashbangBurst)
            .count(),
        20
    );
}

#[test]
fn distant_monster_is_not_a_flashbang_hit() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -14.0)));
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    for _ in 0..8 {
        app.update();
    }
    assert!(!app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::FlashbangHitMonster));
    assert!(app.world().get::<Flashed>(player).is_none());
}

#[test]
fn wall_blocks_flashbang_hit() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn(Room(Rect::new(-2.0, -4.0, 2.0, -3.0)));
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.5, 0.0, -2.5)));
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    click(&mut app, MouseButton::Right);
    for _ in 0..8 {
        app.update();
    }
    assert!(!app
        .world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&Achievement::FlashbangHitMonster));
}

#[test]
fn blocked_flashbang_use_never_bursts() {
    let (mut app, player) = app();
    let monster = app.world_mut().spawn_empty().id();
    click(&mut app, MouseButton::Right);
    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    click(&mut app, MouseButton::Right);
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = true;
    app.world_mut()
        .entity_mut(player)
        .insert(Caught::new(monster));
    click(&mut app, MouseButton::Right);
    assert_eq!(cues(&mut app, Sound::FlashbangBurst), 0);
    assert_eq!(flashbangs(&app, player), Some(1));
    assert!(app.world().get::<Flashed>(player).is_none());
}
