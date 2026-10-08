use std::io::Cursor;

use bevy::{app::PluginsState, input::InputPlugin, math::Rect, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{AudioPaused, PlaySound, Sound};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        build_first_floor, select_fuse_slots, Door, DoorOf, DoorPlugin, DoorRef, DoorState,
        FacilityPower, FacilityPowerPlugin, FuseSeed, HidingPlugin, Prop, Room, ToggleDoor,
        FUSE_TABLES,
    },
};
use serde_json::Value;
use transport::{run, snapshot, TransportPlugin, TransportTimeline};

fn lines(output: Vec<u8>) -> Vec<Value> {
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn spawn_player(position: Vec3) -> impl Fn(Commands, ResMut<TransportTimeline>) {
    move |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
        commands.spawn((PlayerController, Transform::from_translation(position)));
        timeline.ready();
    }
}

fn player_only_app() -> App {
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

fn first_floor_app(seed: u64, position: Vec3) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        TransportPlugin,
    ))
    .insert_resource(FuseSeed(seed))
    .add_systems(Startup, (build_first_floor, spawn_player(position)).chain());
    app
}

fn sees_module(response: &Value, module: &str) -> bool {
    response["visible"]
        .as_array()
        .unwrap()
        .iter()
        .any(|seen| seen["module"] == module)
}

#[test]
fn map_appears_once_and_describes_the_first_floor() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        HidingPlugin,
        TransportPlugin,
    ))
    .insert_resource(FuseSeed(11))
    .add_systems(
        Startup,
        (build_first_floor, spawn_player(Vec3::new(0.0, 1.6, 0.0))).chain(),
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":1}\n{\"tick\":2}\n"),
            &mut output
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0]["tick"], 0);
    assert!(responses[0]["map"].is_object());
    assert!(responses[1..]
        .iter()
        .all(|response| response.get("map").is_none()));

    let map = &responses[0]["map"];
    assert!(!map.to_string().contains("has_fuse"));
    assert_eq!(map["rooms"].as_array().unwrap().len(), 18);
    assert_eq!(map["doors"].as_array().unwrap().len(), 18);
    assert_eq!(map["passages"].as_array().unwrap().len(), 10);
    assert_eq!(map["fuse_candidates"].as_array().unwrap().len(), 5);
    assert!(map["doors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|door| door["exit"] == true));
    let props = map["props"].as_array().unwrap();
    assert!(props.iter().any(|prop| prop["hiding"] == "table"));
    assert!(props.iter().any(|prop| prop["hiding"] == "locker"));
}

#[test]
fn candidates_are_not_leaked_when_out_of_view_regardless_of_seed() {
    let (seed_a, seed_b) = (0u64..)
        .zip(1u64..)
        .find(|&(a, b)| {
            select_fuse_slots(a, FUSE_TABLES.len()) != select_fuse_slots(b, FUSE_TABLES.len())
        })
        .expect("two adjacent seeds with different fuse slots");

    let position = Vec3::new(-10.0, 1.6, 2.0);
    let commands = "{\"tick\":1}\n{\"tick\":5}\n";

    let mut output_a = Vec::new();
    assert_eq!(
        run(
            first_floor_app(seed_a, position),
            Cursor::new(commands),
            &mut output_a,
        ),
        AppExit::Success,
    );
    let mut output_b = Vec::new();
    assert_eq!(
        run(
            first_floor_app(seed_b, position),
            Cursor::new(commands),
            &mut output_b,
        ),
        AppExit::Success,
    );
    assert_eq!(output_a, output_b);
    let responses = lines(output_a);
    assert_eq!(responses.len(), 3);
    assert!(responses
        .iter()
        .flat_map(|response| response["visible"].as_array().unwrap())
        .all(|seen| seen["kind"] != "fuse_candidate"));
}

#[test]
fn candidate_has_fuse_reflects_whether_the_seed_selected_that_slot() {
    for selected in [true, false] {
        let seed = (0u64..)
            .find(|&seed| select_fuse_slots(seed, FUSE_TABLES.len()).contains(&0) == selected)
            .unwrap();
        let mut output = Vec::new();
        assert_eq!(
            run(
                first_floor_app(seed, Vec3::new(8.8, 1.6, 3.0)),
                Cursor::new("{\"tick\":1}\n"),
                &mut output,
            ),
            AppExit::Success,
        );
        let responses = lines(output);
        let candidates: Vec<&Value> = responses[1]["visible"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|seen| seen["kind"] == "fuse_candidate")
            .collect();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0]["candidate"], 0);
        assert_eq!(candidates[0]["has_fuse"], selected);
    }
}

#[test]
fn cone_and_range_gate_visibility_and_pitch_can_hide_a_front_object() {
    let mut app = player_only_app();
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn((Prop("front".into()), Transform::from_xyz(0.0, 0.0, -5.0)));
            commands.spawn((Prop("behind".into()), Transform::from_xyz(0.0, 0.0, 5.0)));
            commands.spawn((
                Prop("wide_angle".into()),
                Transform::from_xyz(10.0, 0.0, -2.0),
            ));
            commands.spawn((Prop("too_far".into()), Transform::from_xyz(0.0, 0.0, -20.0)));
            timeline.ready();
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":1}\n{\"tick\":2,\"input\":{\"look\":[0.0,-1000.0]}}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(sees_module(&responses[1], "front"));
    assert!(!sees_module(&responses[1], "behind"));
    assert!(!sees_module(&responses[1], "wide_angle"));
    assert!(!sees_module(&responses[1], "too_far"));
    assert!(!sees_module(&responses[2], "front"));
}

#[test]
fn closed_door_hides_a_prop_until_it_swings_open_and_reports_open() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        TransportPlugin,
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            let door = commands
                .spawn(Door {
                    position: Vec2::new(0.0, -1.25),
                    rotation: Quat::IDENTITY,
                    frame: String::new(),
                    panel: String::new(),
                    state: DoorState::Closed,
                })
                .id();
            commands
                .spawn(Room(Rect::new(-1.25, -1.25, 1.25, 1.25)))
                .with_related::<DoorOf>(DoorRef(door));
            commands.spawn((Prop("beyond".into()), Transform::from_xyz(0.0, 0.0, -5.0)));
            timeline.ready();
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":1}\n{\"tick\":2,\"input\":{\"f\":true}}\n{\"tick\":60}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(!sees_module(&responses[0], "beyond"));
    assert!(!sees_module(&responses[1], "beyond"));
    assert!(!sees_module(&responses[2], "beyond"));
    assert!(sees_module(&responses[3], "beyond"));

    let door_open = |response: &Value| {
        response["visible"]
            .as_array()
            .unwrap()
            .iter()
            .find(|seen| seen["kind"] == "door")
            .map(|seen| seen["open"].as_bool().unwrap())
    };
    assert_eq!(door_open(&responses[1]), Some(false));
    assert_eq!(door_open(&responses[2]), Some(true));
    assert_eq!(door_open(&responses[3]), Some(true));
}

#[test]
fn power_loss_hides_a_prop_until_the_flashlight_reveals_it() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        FacilityPowerPlugin,
        TransportPlugin,
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn((Prop("ahead".into()), Transform::from_xyz(0.0, 0.0, -5.0)));
            let mut power = FacilityPower::new(0);
            power.remaining_secs = 0.04;
            commands.insert_resource(power);
            timeline.ready();
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new(
                "{\"tick\":1}\n{\"tick\":2}\n{\"tick\":3}\n{\"tick\":4}\n{\"tick\":5,\"input\":{\"flashlight\":true}}\n"
            ),
            &mut output,
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    let trip = responses
        .iter()
        .position(|response| response["power_on"] == false)
        .expect("facility power trips within a few ticks");
    assert!(trip > 1 && trip < 5);
    assert_eq!(responses[trip - 1]["power_on"], true);
    assert!(sees_module(&responses[trip - 1], "ahead"));
    let heard = responses[trip]["heard"].as_array().unwrap();
    let power_down = heard
        .iter()
        .find(|noise| noise["sound"] == "power_down")
        .expect("power_down is heard when the facility trips");
    assert!(power_down["distance_m"].is_null());
    assert!(power_down["source"].is_null());
    assert_eq!(power_down["tick"], responses[trip]["tick"]);

    assert!(!sees_module(&responses[trip], "ahead"));
    assert!(!sees_module(&responses[4], "ahead"));
    assert_eq!(responses[5]["player"]["flashlight_on"], true);
    assert!(sees_module(&responses[5], "ahead"));
}

#[test]
fn nearby_door_sounds_are_heard_with_source_and_drained_after_reporting() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        TransportPlugin,
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn(Door {
                position: Vec2::new(0.0, -1.25),
                rotation: Quat::IDENTITY,
                frame: String::new(),
                panel: String::new(),
                state: DoorState::Closed,
            });
            timeline.ready();
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":1,\"input\":{\"f\":true}}\n{\"tick\":2}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(responses[0]["heard"].as_array().unwrap().is_empty());
    let heard = responses[1]["heard"].as_array().unwrap();
    let sounds: Vec<&str> = heard
        .iter()
        .map(|noise| noise["sound"].as_str().unwrap())
        .collect();
    assert!(sounds.contains(&"door_unlatch"));
    assert!(sounds.contains(&"door_swing"));
    let door = responses[0]["map"]["doors"][0]["id"].clone();
    assert!(heard.iter().all(|noise| noise["source"] == door
        && noise["tick"] == 1
        && noise["distance_m"].as_f64().unwrap() < 2.5));
    assert!(responses[2]["heard"].as_array().unwrap().is_empty());
}

#[derive(Resource)]
struct FarDoor(Entity);

#[test]
fn door_sounds_beyond_hearing_range_are_not_heard() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        TransportPlugin,
    ));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            let far = commands
                .spawn(Door {
                    position: Vec2::new(0.0, -30.0),
                    rotation: Quat::IDENTITY,
                    frame: String::new(),
                    panel: String::new(),
                    state: DoorState::Closed,
                })
                .id();
            commands.insert_resource(FarDoor(far));
            timeline.ready();
        },
    );
    app.add_systems(
        Update,
        |mut fired: Local<bool>, far: Res<FarDoor>, mut toggles: MessageWriter<ToggleDoor>| {
            if !*fired {
                *fired = true;
                toggles.write(ToggleDoor(far.0));
            }
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(app, Cursor::new("{\"tick\":1}\n"), &mut output),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(responses
        .iter()
        .all(|response| response["heard"].as_array().unwrap().is_empty()));
}

#[test]
fn ui_sounds_are_dropped_from_heard() {
    let mut app = player_only_app();
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            timeline.ready();
        },
    );
    app.add_systems(
        Update,
        |mut fired: Local<bool>, mut sounds: MessageWriter<PlaySound>| {
            if !*fired {
                *fired = true;
                sounds.write(PlaySound {
                    sound: Sound::UiConfirm,
                    position: None,
                });
            }
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(app, Cursor::new("{\"tick\":1}\n"), &mut output),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(responses
        .iter()
        .all(|response| response["heard"].as_array().unwrap().is_empty()));
}

#[test]
fn audio_paused_suppresses_heard_sounds() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        TransportPlugin,
    ));
    app.insert_resource(AudioPaused(true));
    app.add_systems(
        Startup,
        |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
            commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
            commands.spawn(Door {
                position: Vec2::new(0.0, -1.25),
                rotation: Quat::IDENTITY,
                frame: String::new(),
                panel: String::new(),
                state: DoorState::Closed,
            });
            timeline.ready();
        },
    );
    app.add_systems(
        Update,
        |mut count: Local<u32>,
         timeline: Res<TransportTimeline>,
         mut paused: ResMut<AudioPaused>| {
            if timeline.first_tick.is_some() {
                *count += 1;
                if *count == 3 {
                    paused.0 = false;
                }
            }
        },
    );

    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new(
                "{\"tick\":1,\"input\":{\"f\":true}}\n\
                 {\"tick\":2,\"input\":{\"f\":false}}\n\
                 {\"tick\":3,\"input\":{\"f\":true}}\n"
            ),
            &mut output,
        ),
        AppExit::Success,
    );
    let responses = lines(output);
    assert!(responses[1]["heard"].as_array().unwrap().is_empty());
    assert!(!responses[3]["heard"].as_array().unwrap().is_empty());
}

#[test]
fn snapshot_matches_between_run_and_manual_stepping() {
    fn build() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            InputPlugin,
            EnhancedInputPlugin,
            PlayerControllerPlugin::default().without_camera(),
            TransportPlugin,
        ))
        .add_systems(
            Startup,
            |mut commands: Commands, mut timeline: ResMut<TransportTimeline>| {
                commands.spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
                commands.spawn((Prop("prop".into()), Transform::from_xyz(0.0, 0.0, -5.0)));
                timeline.ready();
            },
        );
        app
    }

    let mut output = Vec::new();
    assert_eq!(
        run(
            build(),
            Cursor::new("{\"tick\":1}\n{\"tick\":5}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let run_lines: Vec<String> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();

    let mut manual = build();
    if manual.plugins_state() != PluginsState::Cleaned {
        while manual.plugins_state() == PluginsState::Adding {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        manual.finish();
        manual.cleanup();
    }
    while manual
        .world()
        .resource::<TransportTimeline>()
        .first_tick
        .is_none()
    {
        manual.update();
        manual
            .world_mut()
            .resource_mut::<TransportTimeline>()
            .advance();
    }
    assert_eq!(run_lines.len(), 3);
    for (target, expected) in [0u64, 1, 5].into_iter().zip(&run_lines) {
        while manual.world().resource::<TransportTimeline>().tick() != Some(target) {
            manual.update();
            manual
                .world_mut()
                .resource_mut::<TransportTimeline>()
                .advance();
        }
        assert_eq!(&snapshot(manual.world_mut(), target), expected);
    }
}
