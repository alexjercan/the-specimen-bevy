use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{prelude::*, time::TimeUpdateStrategy};
use gameplay::levels::{
    build_exit_cinematic, Door, DoorLock, DoorOf, DoorPlugin, DoorRef, DoorState, DoorSwing, Doors,
    ExitCinematic, ExitDoor, LevelRoot, LightConfig, LightEffect, Prop, PropLightsPlugin, Room,
    ToggleDoor,
};

#[derive(Resource)]
struct Captured(ExitCinematic);

fn spawn(mut commands: Commands) {
    let cinematic = build_exit_cinematic(&mut commands);
    commands.insert_resource(Captured(cinematic));
}

#[test]
fn exit_cinematic_mirrors_the_real_exit_with_an_open_door_in_a_corridor() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PropLightsPlugin)
        .add_systems(Startup, spawn);
    app.update();

    let world = app.world_mut();
    assert_eq!(world.query::<&LevelRoot>().iter(world).count(), 1);
    assert_eq!(world.query::<&Room>().iter(world).count(), 2);
    assert_eq!(
        world
            .query_filtered::<Entity, With<ExitDoor>>()
            .iter(world)
            .count(),
        0
    );
    assert_eq!(
        world
            .query_filtered::<Entity, With<DoorLock>>()
            .iter(world)
            .count(),
        0
    );

    let mut doors = world.query::<(Entity, &Door, &DoorSwing)>();
    let (door, door_data, swing) = doors.single(world).expect("one exit cinematic door");
    assert_eq!(door_data.state, DoorState::Open);
    assert_eq!(swing.0, FRAC_PI_2);

    let mut rooms = world.query::<(Entity, &Room, &Doors)>();
    let authored: Vec<_> = rooms
        .iter(world)
        .map(|(entity, room, links)| {
            let link = links.iter().next().expect("room must link to the door");
            assert_eq!(world.get::<DoorRef>(link).unwrap().0, door);
            assert_eq!(world.get::<DoorOf>(link).unwrap().0, entity);
            room.0
        })
        .collect();
    let exterior = Rect::new(-1.25, -38.75, 1.25, -31.25);
    let corridor = Rect::new(-1.25, -31.25, 1.25, -26.25);
    assert!(authored.contains(&exterior));
    assert!(authored.contains(&corridor));

    let mut lamps = world.query::<(&Prop, &LightConfig, &LightEffect)>();
    let (_, config, effect) = lamps
        .iter(world)
        .find(|(prop, _, _)| prop.0 == "wall_lamp_red")
        .expect("red lamp prop");
    assert!(matches!(effect, LightEffect::Pulse));
    assert!(config.intensity.unwrap() > 0.0);

    let cinematic = &world.resource::<Captured>().0;
    assert_eq!(cinematic.door, door);
    assert!(exterior.contains(cinematic.view.translation.xz()));
    assert!(cinematic.view.forward().z > 0.0);
}

#[test]
fn toggle_door_swings_the_open_exit_cinematic_door_shut() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(DoorPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            500,
        )))
        .add_systems(Startup, spawn);
    app.update();

    let door = app.world().resource::<Captured>().0.door;
    assert_eq!(app.world().get::<DoorSwing>(door).unwrap().0, FRAC_PI_2);

    app.world_mut().write_message(ToggleDoor(door));
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Closed
    );
    assert_eq!(app.world().get::<DoorSwing>(door).unwrap().0, 0.0);
}
