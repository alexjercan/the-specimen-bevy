use std::sync::Arc;

use autopilot::{any_entity, elapsed, frames, AutopilotPlugin, Predicate};
use bevy::prelude::*;
use gameplay::{
    facility::{select_fuse_slots, Door, DoorState, Escaped, ExitDoor, Room, FUSE_TABLES},
    fuses::{FuseInventory, FusePanel, FUSE_COUNT},
    interaction::PickupKind,
    player::{ControllerSettings, PlayerController},
};

const EYE_HEIGHT: f32 = 1.6;
const FUSE_STANDOFF: f32 = 1.3;
const PANEL_STAND: Vec3 = Vec3::new(5.0, EYE_HEIGHT, -29.5);
const PANEL_AIM: Vec3 = Vec3::new(5.0, 1.85, -31.15);
const EXIT_STAND: Vec3 = Vec3::new(0.0, EYE_HEIGHT, -29.6);
const EXIT_AIM: Vec3 = Vec3::new(0.0, 1.1, -31.25);

fn player_entity(world: &mut World) -> Entity {
    world
        .query_filtered::<Entity, With<PlayerController>>()
        .single(world)
        .expect("the run spawned one player")
}

fn look_at(eye: Vec3, target: Vec3) -> Quat {
    let forward = (target - eye).normalize();
    let yaw = (-forward.x).atan2(-forward.z);
    let pitch = forward.y.asin();
    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
}

fn teleport(world: &mut World, eye: Vec3, target: Vec3) {
    let player = player_entity(world);
    let mut transform = world.get_mut::<Transform>(player).unwrap();
    transform.translation = eye;
    transform.rotation = look_at(eye, target);
}

fn aim(world: &mut World, target: Vec3) {
    let player = player_entity(world);
    let mut transform = world.get_mut::<Transform>(player).unwrap();
    let eye = transform.translation;
    transform.rotation = look_at(eye, target);
}

fn fuse_center(slot: usize) -> Vec3 {
    let position = FUSE_TABLES[slot].position;
    Vec3::new(position.x, position.y + 0.03, position.z)
}

fn fuse_stand(world: &mut World, slot: usize) -> Vec3 {
    let center = fuse_center(slot);
    let room_center = world
        .query::<&Room>()
        .iter(world)
        .find(|room| room.0.contains(center.xz()))
        .map(|room| room.0.center())
        .expect("every fuse table sits inside a room");
    let away = (room_center - center.xz()).normalize_or(Vec2::NEG_Y) * FUSE_STANDOFF;
    Vec3::new(center.x + away.x, EYE_HEIGHT, center.z + away.y)
}

fn interact_key(world: &World) -> KeyCode {
    world.resource::<ControllerSettings>().keys.interact
}

fn forward_key(world: &World) -> KeyCode {
    world.resource::<ControllerSettings>().keys.forward
}

fn press(world: &mut World, key: fn(&World) -> KeyCode) {
    let key = key(world);
    world.resource_mut::<ButtonInput<KeyCode>>().press(key);
}

fn release(world: &mut World, key: fn(&World) -> KeyCode) {
    let key = key(world);
    world.resource_mut::<ButtonInput<KeyCode>>().release(key);
}

fn fuses_held(count: usize) -> Arc<Predicate> {
    Arc::new(move |world| {
        world
            .try_query_filtered::<&FuseInventory, With<PlayerController>>()
            .is_some_and(|mut query| query.iter(world).any(|inventory| inventory.0 >= count))
    })
}

fn panel_installed() -> Arc<Predicate> {
    Arc::new(|world| {
        world
            .try_query::<&FusePanel>()
            .is_some_and(|mut query| query.iter(world).any(|panel| panel.installed == FUSE_COUNT))
    })
}

fn exit_open() -> Arc<Predicate> {
    Arc::new(|world| {
        world
            .try_query_filtered::<&Door, With<ExitDoor>>()
            .is_some_and(|mut query| query.iter(world).any(|door| door.state == DoorState::Open))
    })
}

fn floor_ready() -> Arc<Predicate> {
    Arc::new(|world| {
        world
            .try_query_filtered::<Entity, With<PlayerController>>()
            .is_some_and(|mut query| query.iter(world).next().is_some())
            && world
                .try_query::<&PickupKind>()
                .is_some_and(|mut query| query.iter(world).count() > 0)
            && world
                .try_query::<&Room>()
                .is_some_and(|mut query| query.iter(world).count() > 0)
    })
}

fn interact(
    script: AutopilotPlugin,
    name: &str,
    done: Arc<Predicate>,
    secs: f32,
) -> AutopilotPlugin {
    script
        .step(format!("{name}: press"))
        .act(|world| press(world, interact_key))
        .until(done, secs)
        .add()
        .step(format!("{name}: release"))
        .act(|world| release(world, interact_key))
        .until(frames(2), 5.0)
        .add()
}

pub fn escape_run(seed: u64) -> AutopilotPlugin {
    let mut script = AutopilotPlugin::new()
        .with_deadline_secs(240.0)
        .step("floor ready")
        .until(floor_ready(), 120.0)
        .add();
    for (index, slot) in select_fuse_slots(seed).into_iter().enumerate() {
        let name = format!("fuse {}", FUSE_TABLES[slot].room);
        script = script
            .step(format!("{name}: approach"))
            .act(move |world| {
                let stand = fuse_stand(world, slot);
                teleport(world, stand, fuse_center(slot));
            })
            .until(frames(3), 5.0)
            .add()
            .step(format!("{name}: aim"))
            .act(move |world| aim(world, fuse_center(slot)))
            .until(frames(1), 5.0)
            .add();
        script = interact(script, &name, fuses_held(index + 1), 3.0);
    }
    script = script
        .step("panel: approach")
        .act(|world| teleport(world, PANEL_STAND, PANEL_AIM))
        .until(frames(3), 5.0)
        .add()
        .step("panel: aim")
        .act(|world| aim(world, PANEL_AIM))
        .until(frames(1), 5.0)
        .add();
    script = interact(script, "panel", panel_installed(), 5.0);
    script = script
        .step("exit: approach")
        .act(|world| teleport(world, EXIT_STAND, EXIT_AIM))
        .until(frames(3), 5.0)
        .add()
        .step("exit: aim")
        .act(|world| aim(world, EXIT_AIM))
        .until(frames(1), 5.0)
        .add();
    script = interact(script, "exit", exit_open(), 3.0);
    script
        .step("exit: swing")
        .until(elapsed(1.0), 5.0)
        .add()
        .step("exit: walk out")
        .act(|world| {
            aim(world, Vec3::new(0.0, EYE_HEIGHT, -40.0));
            press(world, forward_key);
        })
        .until(any_entity::<With<Escaped>>(), 10.0)
        .add()
        .step("exit: stop")
        .act(|world| release(world, forward_key))
        .until(frames(1), 5.0)
        .add()
}
