use std::io::Cursor;

use bevy::{input::InputPlugin, math::Rect, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, Sound};
use game_settings::GameSettings;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        Detector, DetectorReading, DevicePlugin, Flashbangs, Monster, PickupKind, PickupPlugin,
        Room,
    },
};
use serde_json::Value;
use transport::{run, TransportPlugin, TransportTimeline};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        TransportPlugin,
    ));
    app
}

fn responses(app: App, input: &str) -> Vec<Value> {
    let mut output = Vec::new();
    assert_eq!(run(app, Cursor::new(input), &mut output), AppExit::Success);
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn device_scene(kind: PickupKind, position: Vec3, room: Option<Rect>) -> App {
    let mut app = app();
    app.add_systems(
        Startup,
        move |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn((kind, Transform::from_translation(position)));
            if let Some(room) = room {
                commands.spawn(Room(room));
            }
            timeline.ready();
        },
    );
    app
}

#[test]
fn snapshot_reports_no_devices_before_pickup() {
    let mut app = app();
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );

    let responses = responses(app, "");
    let player = &responses[0]["player"];
    assert_eq!(player["flashbangs"], 0);
    assert_eq!(player["flash_remaining"], 0.0);
    assert_eq!(player["has_detector"], false);
    assert_eq!(player["detector"], Value::Null);
}

#[test]
fn snapshot_reports_detector_reading_only_when_available() {
    for reading in [
        Some(DetectorReading {
            distance: 4.25,
            bearing: std::f32::consts::FRAC_PI_4,
        }),
        None,
    ] {
        let mut app = app();
        app.add_systems(
            Startup,
            move |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
                commands.spawn((
                    PlayerController,
                    Transform::from_xyz(0.0, 1.6, 0.0),
                    Detector { reading },
                ));
                timeline.ready();
            },
        );

        let responses = responses(app, "");
        let player = &responses[0]["player"];
        assert_eq!(player["has_detector"], true);
        if let Some(reading) = reading {
            assert_eq!(player["detector"]["distance_m"], reading.distance);
            let bearing = player["detector"]["bearing_deg"].as_f64().unwrap();
            assert!((bearing - reading.bearing.to_degrees() as f64).abs() < 0.0001);
        } else {
            assert_eq!(player["detector"], Value::Null);
        }
    }
}

#[test]
fn snapshot_detector_reaches_twenty_five_metres() {
    for (distance, expected) in [(22.0, true), (24.9, true), (25.2, false)] {
        let mut app = app();
        app.add_plugins(DevicePlugin);
        app.add_systems(
            Startup,
            move |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
                commands.spawn((
                    PlayerController,
                    Transform::from_xyz(0.0, 1.6, 0.0),
                    Detector::default(),
                ));
                commands.spawn((Monster::default(), Transform::from_xyz(0.0, 0.0, -distance)));
                timeline.ready();
            },
        );

        let responses = responses(app, "{\"tick\":1,\"input\":{}}\n");
        let detector = &responses.last().unwrap()["player"]["detector"];
        if expected {
            let reported = detector["distance_m"].as_f64().unwrap();
            assert!(
                (reported - distance as f64).abs() < 0.001,
                "{distance}: {detector}"
            );
        } else {
            assert_eq!(detector, &Value::Null, "{distance}");
        }
    }
}

#[test]
fn flashbang_and_flashlight_controls_follow_rebound_settings() {
    let mut settings = GameSettings::default();
    settings.keys.flashlight = "KeyH".into();
    settings.keys.flashbang = "KeyG".into();
    let mut app = app();
    app.insert_resource(settings).add_plugins(DevicePlugin);
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((
                PlayerController,
                Transform::from_xyz(0.0, 1.6, 0.0),
                Flashbangs(1),
            ));
            timeline.ready();
        },
    );

    let responses = responses(
        app,
        "{\"tick\":1,\"input\":{\"flashbang\":true,\"flashlight\":true}}\n",
    );
    assert_eq!(responses[1]["player"]["flashbangs"], 0);
    assert_eq!(responses[1]["player"]["flashlight_on"], true);
}

#[test]
fn flashbang_control_uses_right_mouse_and_reports_consumption() {
    let mut app = app();
    app.add_plugins(DevicePlugin);
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((
                PlayerController,
                Transform::from_xyz(0.0, 1.6, 0.0),
                Flashbangs(1),
            ));
            timeline.ready();
        },
    );

    let mut input = String::from("{\"tick\":1,\"input\":{\"flashbang\":true}}\n");
    for tick in 2..=42 {
        input.push_str(&format!("{{\"tick\":{tick},\"input\":{{}}}}\n"));
    }
    let responses = responses(app, &input);
    assert_eq!(responses[1]["player"]["flashbangs"], 0);
    assert_eq!(responses[1]["player"]["flash_remaining"], 0.0);
    assert_eq!(responses[42]["player"]["flash_remaining"], 0.0);
    let heard: Vec<_> = responses
        .iter()
        .skip(1)
        .flat_map(|response| response["heard"].as_array().unwrap())
        .map(|sound| sound["sound"].as_str().unwrap())
        .filter(|sound| sound.starts_with("flashbang"))
        .collect();
    assert_eq!(heard, ["flashbang_throw", "flashbang_burst"]);
}

#[test]
fn visible_device_is_not_added_to_the_initial_map() {
    let responses = responses(
        device_scene(PickupKind::Flashbang, Vec3::new(0.0, 0.0, -5.0), None),
        "",
    );
    let snapshot = &responses[0];
    let seen: Vec<&Value> = snapshot["visible"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|thing| thing["kind"] == "device")
        .collect();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0]["device"], "flashbang");
    let expected_distance = Vec3::new(0.0, 1.6, 0.0).distance(Vec3::new(0.0, 0.06, -5.0)) as f64;
    let actual_distance = seen[0]["distance_m"].as_f64().unwrap();
    assert!((actual_distance - expected_distance).abs() < 0.0001);
    assert!(!snapshot["map"].to_string().contains("device"));
}

#[test]
fn detector_nearby_sound_is_reported_by_name() {
    let mut app = app();
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    )
    .add_systems(
        Update,
        |mut fired: Local<bool>, mut sounds: MessageWriter<PlaySound>| {
            if !*fired {
                *fired = true;
                sounds.write(PlaySound {
                    sound: Sound::DetectorNearby,
                    position: None,
                });
            }
        },
    );

    let responses = responses(app, "");
    assert!(responses[0]["heard"]
        .as_array()
        .unwrap()
        .iter()
        .any(|sound| sound["sound"] == "detector_nearby"));
}

#[test]
fn detector_pickup_reports_detector_and_pickup_sound() {
    let mut app = app();
    app.add_plugins(PickupPlugin);
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            let mut player = Transform::from_xyz(0.0, 1.6, 0.0);
            player.rotation = Quat::from_rotation_x((-1.54_f32).atan2(2.0));
            commands.spawn((PlayerController, player));
            commands.spawn((PickupKind::Detector, Transform::from_xyz(0.0, 0.0, -2.0)));
            timeline.ready();
        },
    );

    let responses = responses(app, "{\"tick\":1,\"input\":{\"f\":true}}\n");
    assert_eq!(responses[1]["player"]["has_detector"], true);
    assert!(responses[1]["heard"]
        .as_array()
        .unwrap()
        .iter()
        .any(|sound| sound["sound"] == "detector_pickup"));
}

#[test]
fn visible_detector_is_reported_as_a_device() {
    let responses = responses(
        device_scene(PickupKind::Detector, Vec3::new(0.0, 0.0, -5.0), None),
        "",
    );
    assert!(responses[0]["visible"]
        .as_array()
        .unwrap()
        .iter()
        .any(|thing| thing["kind"] == "device" && thing["device"] == "detector"));
}

#[test]
fn device_visibility_respects_facing_and_occlusion() {
    let behind = responses(
        device_scene(PickupKind::Flashbang, Vec3::new(0.0, 0.0, 5.0), None),
        "",
    );
    assert!(behind[0]["visible"]
        .as_array()
        .unwrap()
        .iter()
        .all(|thing| thing["kind"] != "device"));

    let occluded = responses(
        device_scene(
            PickupKind::Flashbang,
            Vec3::new(0.0, 0.0, -5.0),
            Some(Rect::new(-1.25, -5.0, 1.25, -2.5)),
        ),
        "",
    );
    assert!(occluded[0]["visible"]
        .as_array()
        .unwrap()
        .iter()
        .all(|thing| thing["kind"] != "device"));
}
