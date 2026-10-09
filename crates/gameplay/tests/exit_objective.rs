use std::{f32::consts::PI, time::Duration};

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, PlaySourceSound, Sound, SourceSounds};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin},
    levels::{
        build_first_floor, Door, DoorLock, DoorPlugin, DoorState, DoorSwing, Escaped, ExitDoor,
        FuseInventory, FusePanel, FusePlugin, InstallFuses, ObjectivePlugin, Prop, Room,
        ToggleDoor, FUSE_COUNT,
    },
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins((DoorPlugin, FusePlugin, ObjectivePlugin));
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

fn door(app: &mut App, x: f32, z: f32, yaw: f32) -> Entity {
    app.world_mut()
        .spawn(Door {
            position: Vec2::new(x, z),
            rotation: Quat::from_rotation_y(yaw),
            frame: "wall_doorway".to_owned(),
            panel: "door_panel".to_owned(),
            state: DoorState::Closed,
        })
        .id()
}

fn place(app: &mut App, player: Entity, eye: Vec3, target: Vec3) {
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_translation(eye).looking_at(target, Vec3::Y));
}

fn press_f(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
}

fn walk(app: &mut App, frames: usize) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    for _ in 0..frames {
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyW);
    app.update();
}

fn settle(app: &mut App) {
    for _ in 0..6 {
        app.update();
    }
}

fn inventory(app: &App, player: Entity) -> usize {
    app.world().get::<FuseInventory>(player).unwrap().0
}

fn installed(app: &App, panel: Entity) -> usize {
    app.world().get::<FusePanel>(panel).unwrap().installed
}

fn locked(app: &App, door: Entity) -> bool {
    app.world().get::<DoorLock>(door).is_some()
}

fn state(app: &App, door: Entity) -> DoorState {
    app.world().get::<Door>(door).unwrap().state
}

fn escaped(app: &App, player: Entity) -> bool {
    app.world().get::<Escaped>(player).is_some()
}

fn z(app: &App, player: Entity) -> f32 {
    app.world().get::<Transform>(player).unwrap().translation.z
}

fn teleport(app: &mut App, player: Entity, x: f32, z: f32) {
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(x, 1.6, z);
    app.update();
}

#[test]
fn first_floor_locks_only_the_outside_exit_and_adds_a_logical_panel() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let exits: Vec<_> = world
        .query_filtered::<(Entity, &Name, &Door), With<ExitDoor>>()
        .iter(world)
        .map(|(entity, name, door)| (entity, name.as_str().to_owned(), door.position))
        .collect();
    assert_eq!(exits.len(), 1);
    let (exit, name, position) = &exits[0];
    assert_eq!(name, "exit / outside");
    assert_eq!(*position, Vec2::new(0.0, -31.25));
    let locks: Vec<_> = world
        .query_filtered::<Entity, With<DoorLock>>()
        .iter(world)
        .collect();
    assert_eq!(locks, vec![*exit]);

    let panels: Vec<_> = world
        .query::<(Entity, &FusePanel, &Transform)>()
        .iter(world)
        .map(|(entity, panel, transform)| (entity, *panel, *transform))
        .collect();
    assert_eq!(panels.len(), 1);
    let (panel, logical, transform) = panels[0];
    assert_eq!(logical.installed, 0);
    assert!(world.get::<Prop>(panel).is_none());
    assert!(world.get::<ChildOf>(panel).is_none());
    assert!(world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .any(|(prop, prop_transform)| prop.0 == "fuse_panel" && *prop_transform == transform));
}

#[test]
fn locked_door_ignores_toggle_messages_and_f_until_unlocked() {
    let (mut app, _) = app();
    let entity = door(&mut app, 0.0, -1.5, 0.0);
    app.world_mut().entity_mut(entity).insert(DoorLock);

    app.world_mut().write_message(ToggleDoor(entity));
    app.update();
    let cues: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].sound, Sound::DoorLocked);
    assert_eq!(cues[0].source, entity);
    assert!(app
        .world()
        .get::<SourceSounds>(entity)
        .unwrap()
        .0
        .contains(&(Sound::DoorLocked, Vec3::Y)));

    press_f(&mut app);
    let cues: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].sound, Sound::DoorLocked);
    settle(&mut app);
    assert_eq!(state(&app, entity), DoorState::Closed);
    assert_eq!(app.world().get::<DoorSwing>(entity).unwrap().0, 0.0);

    app.world_mut().entity_mut(entity).remove::<DoorLock>();
    app.world_mut().write_message(ToggleDoor(entity));
    app.update();
    assert_eq!(state(&app, entity), DoorState::Open);
    assert!(app.world().get::<DoorSwing>(entity).unwrap().0 > 0.0);
    let cues: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySourceSound>>()
        .drain()
        .collect();
    assert!(cues.iter().all(|cue| cue.sound != Sound::DoorLocked));
}

#[test]
fn panel_installs_all_fuses_once_and_unlocks_only_the_exit() {
    let (mut app, player) = app();
    let exit = door(&mut app, 5.0, -5.0, 0.0);
    app.world_mut()
        .entity_mut(exit)
        .insert((ExitDoor, DoorLock));
    let other = door(&mut app, -5.0, -5.0, 0.0);
    app.world_mut().entity_mut(other).insert(DoorLock);
    let panel = app
        .world_mut()
        .spawn((FusePanel::default(), Transform::from_xyz(0.0, 1.6, -1.2)))
        .id();
    place(
        &mut app,
        player,
        Vec3::new(0.0, 1.6, 0.0),
        Vec3::new(0.0, 1.6, -1.2),
    );

    for held in 0..FUSE_COUNT {
        app.world_mut()
            .entity_mut(player)
            .insert(FuseInventory(held));
        press_f(&mut app);
        assert_eq!(inventory(&app, player), held);
        assert_eq!(installed(&app, panel), 0);
        assert!(locked(&app, exit));
    }

    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    press_f(&mut app);
    assert_eq!(inventory(&app, player), 0);
    assert_eq!(installed(&app, panel), 0);
    assert!(locked(&app, exit));
    let mut cues = Vec::new();
    for _ in 0..12 {
        app.update();
        cues.extend(
            app.world_mut()
                .resource_mut::<Messages<PlaySound>>()
                .drain(),
        );
    }
    assert_eq!(installed(&app, panel), FUSE_COUNT);
    assert!(!locked(&app, exit));
    assert!(locked(&app, other));
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].sound, Sound::FuseComplete);
    assert_eq!(cues[0].position, None);

    press_f(&mut app);
    assert_eq!(inventory(&app, player), 0);
    assert_eq!(installed(&app, panel), FUSE_COUNT);

    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    press_f(&mut app);
    for _ in 0..2 {
        app.world_mut()
            .write_message(InstallFuses { player, panel });
    }
    app.update();
    assert_eq!(inventory(&app, player), FUSE_COUNT);
    assert_eq!(installed(&app, panel), FUSE_COUNT);
    assert!(locked(&app, other));
}

#[test]
fn duplicate_install_messages_consume_fuses_once() {
    let (mut app, player) = app();
    let exit = door(&mut app, 5.0, -5.0, 0.0);
    app.world_mut()
        .entity_mut(exit)
        .insert((ExitDoor, DoorLock));
    let panel = app
        .world_mut()
        .spawn((FusePanel::default(), Transform::from_xyz(0.0, 1.6, -1.2)))
        .id();
    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    for _ in 0..2 {
        app.world_mut()
            .write_message(InstallFuses { player, panel });
    }
    app.update();
    assert_eq!(inventory(&app, player), 0);
    assert_eq!(installed(&app, panel), 0);
    assert!(locked(&app, exit));
    for _ in 0..12 {
        app.update();
    }
    assert_eq!(installed(&app, panel), FUSE_COUNT);
    assert!(!locked(&app, exit));
}

#[test]
fn crossing_the_exit_threshold_wins_only_after_unlock() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn(Room(Rect::new(-1.25, -31.25, 8.75, -21.25)));
    let exit = door(&mut app, 0.0, -31.25, PI);
    app.world_mut()
        .entity_mut(exit)
        .insert((ExitDoor, DoorLock));

    teleport(&mut app, player, 0.0, -30.0);
    assert!(!escaped(&app, player));
    teleport(&mut app, player, 0.0, -31.6);
    assert!(!escaped(&app, player));

    app.world_mut().entity_mut(exit).remove::<DoorLock>();
    teleport(&mut app, player, 0.0, -30.0);
    assert!(!escaped(&app, player));
    teleport(&mut app, player, 0.0, -31.6);
    assert!(escaped(&app, player));
    teleport(&mut app, player, 0.0, -30.0);
    assert!(escaped(&app, player));
}

#[test]
fn leaving_the_rooms_away_from_the_exit_does_not_win() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn(Room(Rect::new(-1.25, -31.25, 8.75, -21.25)));
    let exit = door(&mut app, 0.0, -31.25, PI);
    app.world_mut().entity_mut(exit).insert(ExitDoor);
    teleport(&mut app, player, 20.0, -40.0);
    assert!(!escaped(&app, player));
}

#[test]
fn first_floor_run_installs_fuses_opens_the_exit_and_walks_out() {
    let (mut app, player) = app();
    app.world_mut()
        .run_system_cached(build_first_floor)
        .unwrap();
    app.update();
    let world = app.world_mut();
    let exit = world
        .query_filtered::<Entity, With<ExitDoor>>()
        .single(world)
        .unwrap();
    let panel = world
        .query_filtered::<Entity, With<FusePanel>>()
        .single(world)
        .unwrap();

    place(
        &mut app,
        player,
        Vec3::new(0.0, 1.6, -30.0),
        Vec3::new(0.0, 1.1, -31.25),
    );
    press_f(&mut app);
    settle(&mut app);
    assert_eq!(state(&app, exit), DoorState::Closed);
    walk(&mut app, 12);
    assert!(z(&app, player) > -31.25);
    assert!(!escaped(&app, player));

    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT - 1));
    place(
        &mut app,
        player,
        Vec3::new(5.0, 1.6, -29.5),
        Vec3::new(5.0, 1.85, -31.15),
    );
    press_f(&mut app);
    assert!(locked(&app, exit));
    assert_eq!(inventory(&app, player), FUSE_COUNT - 1);

    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    press_f(&mut app);
    assert!(locked(&app, exit));
    assert_eq!(inventory(&app, player), 0);
    assert_eq!(installed(&app, panel), 0);
    for _ in 0..12 {
        app.update();
    }
    assert!(!locked(&app, exit));
    assert_eq!(installed(&app, panel), FUSE_COUNT);

    place(
        &mut app,
        player,
        Vec3::new(0.0, 1.6, -30.0),
        Vec3::new(0.0, 1.1, -31.25),
    );
    press_f(&mut app);
    settle(&mut app);
    assert_eq!(state(&app, exit), DoorState::Open);
    assert!(!escaped(&app, player));
    walk(&mut app, 12);
    assert!(z(&app, player) < -31.25);
    assert!(escaped(&app, player));
}
