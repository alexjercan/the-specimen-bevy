use std::{f32::consts::PI, io::Cursor};

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        Door, DoorLock, DoorOf, DoorPlugin, DoorRef, DoorState, ExitDoor, FuseInventory,
        FusePanel, FusePlugin, ObjectivePlugin, Room, FUSE_COUNT,
    },
};
use transport::{run, TransportPlugin, TransportTimeline};

fn setup(mut commands: Commands, mut timeline: ResMut<TransportTimeline>) {
    let exit = commands
        .spawn((
            Door {
                position: Vec2::new(0.0, -31.25),
                rotation: Quat::from_rotation_y(PI),
                frame: String::new(),
                panel: String::new(),
                state: DoorState::Closed,
            },
            ExitDoor,
            DoorLock,
        ))
        .id();
    commands
        .spawn(Room(Rect::new(-1.25, -31.25, 8.75, -21.25)))
        .with_related::<DoorOf>(DoorRef(exit));
    commands.spawn((
        FusePanel::default(),
        Transform::from_xyz(0.0, 1.6, -30.6),
    ));
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
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        PlayerControllerPlugin::default().without_camera(),
        DoorPlugin,
        FusePlugin,
        ObjectivePlugin,
        TransportPlugin,
    ))
    .add_systems(Startup, setup);
    let mut output = Vec::new();
    assert_eq!(
        run(
            app,
            Cursor::new("{\"tick\":1,\"input\":{\"f\":true}}\n{\"tick\":2,\"input\":{\"f\":false}}\n{\"tick\":3,\"input\":{\"f\":true}}\n{\"tick\":4,\"input\":{\"f\":false}}\n{\"tick\":40}\n{\"tick\":100,\"input\":{\"w\":true}}\n{\"tick\":101,\"input\":{\"w\":false}}\n"),
            &mut output,
        ),
        AppExit::Success,
    );
    let lines: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 7);
    let z = |index: usize| lines[index]["player"]["position"][2].as_f64().unwrap();
    for line in &lines[..5] {
        assert_eq!(line["won"], false);
    }
    assert!(z(4) > -31.25);
    assert!(z(5) < -31.25);
    assert_eq!(lines[5]["won"], true);
    assert_eq!(lines[6]["tick"], 101);
    assert_eq!(lines[6]["won"], true);
}
