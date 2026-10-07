use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::controller::player::{Interact, PlayerController, PlayerControlsEnabled};

use super::{
    animation::{animate_doors, DoorSwing},
    builder::{Door, DoorState},
    fuses::FuseInventory,
    interaction::{InteractTarget, InteractTargets},
};

pub(crate) const INTERACT_RANGE: f32 = 2.5;
pub(crate) const PANEL_WIDTH: f32 = 1.18;
const PANEL_HEIGHT: f32 = 2.2;
pub(crate) const PANEL_HALF_THICKNESS: f32 = 0.03;
pub(crate) const PANEL_OFFSET: Vec3 = Vec3::new(-0.59, 0.0, 0.0);

#[derive(Message)]
pub struct ToggleDoor(pub Entity);

#[derive(Component)]
pub struct DoorPanel;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExitDoor;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorLock;

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleDoor>()
            .add_observer(interact)
            .add_systems(Update, (toggle_doors, animate_doors).chain())
            .add_systems(PostUpdate, update_panels);
    }
}

fn interact(
    _: On<Start<Interact>>,
    players: Query<(&Transform, Option<&FuseInventory>), With<PlayerController>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    mut toggles: MessageWriter<ToggleDoor>,
) {
    if !enabled.0 {
        return;
    }
    for (player, inventory) in &players {
        if let Some(InteractTarget::Door(door)) = targets.aimed(player, inventory) {
            toggles.write(ToggleDoor(door));
        }
    }
}

pub(crate) fn aimed_door(
    player: &Transform,
    doors: &Query<(Entity, &Door, &DoorSwing)>,
) -> Option<(Entity, f32)> {
    let origin = player.translation;
    let forward = player.rotation * -Vec3::Z;
    doors
        .iter()
        .filter_map(|(entity, door, swing)| {
            let hinge = panel_hinge(door);
            let rotation = panel_rotation(door, swing);
            let local_origin = rotation.inverse() * (origin - hinge);
            let local_direction = rotation.inverse() * forward;
            panel_hit(local_origin, local_direction).map(|distance| (entity, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

pub(crate) fn panel_hinge(door: &Door) -> Vec3 {
    Vec3::new(door.position.x, 0.0, door.position.y) + door.rotation * PANEL_OFFSET
}

pub(crate) fn panel_rotation(door: &Door, swing: &DoorSwing) -> Quat {
    door.rotation * Quat::from_rotation_y(swing.0)
}

pub fn panel_center(door: &Door, swing: &DoorSwing) -> Vec3 {
    panel_hinge(door)
        + panel_rotation(door, swing) * Vec3::new(PANEL_WIDTH * 0.5, PANEL_HEIGHT * 0.5, 0.0)
}

pub fn panel_top(door: &Door, swing: &DoorSwing) -> Vec3 {
    panel_hinge(door)
        + panel_rotation(door, swing) * Vec3::new(PANEL_WIDTH * 0.5, PANEL_HEIGHT + 0.1, 0.0)
}

fn panel_hit(origin: Vec3, direction: Vec3) -> Option<f32> {
    let min = Vec3::new(0.0, 0.01, -PANEL_HALF_THICKNESS).to_array();
    let max = Vec3::new(PANEL_WIDTH, PANEL_HEIGHT - 0.01, PANEL_HALF_THICKNESS).to_array();
    let origin = origin.to_array();
    let direction = direction.to_array();
    let mut entry: f32 = 0.0;
    let mut exit = INTERACT_RANGE;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-6 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let near = (min[axis] - origin[axis]) / direction[axis];
        let far = (max[axis] - origin[axis]) / direction[axis];
        entry = entry.max(near.min(far));
        exit = exit.min(near.max(far));
        if entry > exit {
            return None;
        }
    }
    (exit > 0.0).then_some(entry)
}

fn toggle_doors(
    mut toggles: MessageReader<ToggleDoor>,
    mut doors: Query<&mut Door, Without<DoorLock>>,
) {
    for ToggleDoor(entity) in toggles.read() {
        if let Ok(mut door) = doors.get_mut(*entity) {
            door.state = match door.state {
                DoorState::Closed => DoorState::Open,
                DoorState::Open => DoorState::Closed,
            };
        }
    }
}

pub(crate) fn panel_transform(swing: f32) -> Transform {
    Transform::from_translation(PANEL_OFFSET).with_rotation(Quat::from_rotation_y(swing))
}

fn update_panels(
    doors: Query<(&DoorSwing, &Children), Changed<DoorSwing>>,
    mut panels: Query<&mut Transform, With<DoorPanel>>,
) {
    for (swing, children) in &doors {
        for child in children.iter() {
            if let Ok(mut panel) = panels.get_mut(child) {
                *panel = panel_transform(swing.0);
            }
        }
    }
}
