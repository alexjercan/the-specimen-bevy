mod common;

use std::{f32::consts::PI, io::Cursor};

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_app::{FrameStep, Specimen};
use gameplay::facility::Module;
use gameplay::{
    facility::{Door, DoorLock, DoorOf, DoorRef, DoorState, ExitDoor, ObjectivePlugin, Room},
    fuses::{FuseInventory, FusePanel, FUSE_COUNT},
    player::PlayerController,
    presets::GamePreset,
};
use stdio::{run, HeadlessStdio, StdioTimeline};

fn setup(mut commands: Commands, mut timeline: ResMut<StdioTimeline>) {
    let exit = commands
        .spawn((
            Door {
                position: Vec2::new(0.0, -31.25),
                rotation: Quat::from_rotation_y(PI),
                frame: Module::WallDoorway,
                panel: Module::DoorPanel,
                state: DoorState::Closed,
            },
            ExitDoor,
            DoorLock,
        ))
        .id();
    commands
        .spawn(Room(Rect::new(-1.25, -31.25, 8.75, -21.25)))
        .with_related::<DoorOf>(DoorRef(exit));
    commands.spawn((FusePanel::default(), Transform::from_xyz(0.0, 1.6, -30.6)));
    commands.spawn((
        PlayerController,
        FuseInventory(FUSE_COUNT),
        Transform::from_xyz(0.0, 1.6, -30.0),
    ));
    timeline.ready();
}

#[test]
fn headless_snapshot_reports_the_win_after_install_and_exit_without_exiting_early() {
    let mut app = App::new();
    app.insert_resource(common::controller_settings())
        .insert_resource(GamePreset::default());
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        common::CueOwners,
        ObjectivePlugin,
        HeadlessStdio::<Specimen>::new(FrameStep::DEFAULT.duration()),
    ))
    .add_systems(Startup, setup);
    let mut output = Vec::new();
    assert_eq!(
        run::<Specimen>(
            app,
            Cursor::new("{\"version\":1,\"tick\":1,\"input\":{\"f\":true}}\n{\"version\":1,\"tick\":80,\"input\":{\"f\":false}}\n{\"version\":1,\"tick\":81,\"input\":{\"f\":true}}\n{\"version\":1,\"tick\":82,\"input\":{\"f\":false}}\n{\"version\":1,\"tick\":120}\n{\"version\":1,\"tick\":200,\"input\":{\"w\":true}}\n{\"version\":1,\"tick\":201,\"input\":{\"w\":false}}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let mut lines: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.remove(0)["tick"], 0);
    assert_eq!(lines.len(), 7);
    let z = |index: usize| lines[index]["player"]["position"][2].as_f64().unwrap();
    for line in &lines[..5] {
        assert_eq!(line["won"], false);
    }
    assert!(z(4) > -31.25);
    assert!(z(5) < -31.25);
    assert_eq!(lines[5]["won"], true);
    assert_eq!(lines[6]["tick"], 201);
    assert_eq!(lines[6]["won"], true);
}
