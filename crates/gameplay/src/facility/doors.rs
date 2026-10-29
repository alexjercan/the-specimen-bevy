use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    audible::{Audible, Origin},
    player::{Interact, PlayerController, PlayerControlsEnabled},
};
use game_audio::Falloff;

use crate::{
    fuses::FuseInventory,
    hiding::Hidden,
    interaction::{InteractTarget, InteractTargets},
    monster::Caught,
};

use super::{
    animation::{animate_doors, DoorSwing},
    builder::{Door, DoorState},
};

pub(crate) const INTERACT_RANGE: f32 = 2.5;
pub(crate) const PANEL_WIDTH: f32 = 1.18;
const PANEL_HEIGHT: f32 = 2.2;
pub(crate) const PANEL_HALF_THICKNESS: f32 = 0.03;
pub(crate) const PANEL_OFFSET: Vec3 = Vec3::new(-0.59, 0.0, 0.0);

#[derive(Message)]
pub struct ToggleDoor(pub Entity);

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExitOpened {
    pub door: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorCueKind {
    Unlatch,
    Swing,
    Shut,
    Locked,
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct DoorCue {
    pub door: Entity,
    pub kind: DoorCueKind,
}

impl Audible for DoorCue {
    fn name(&self) -> Option<&'static str> {
        Some(match self.kind {
            DoorCueKind::Unlatch => "door_unlatch",
            DoorCueKind::Swing => "door_swing",
            DoorCueKind::Shut => "door_shut",
            DoorCueKind::Locked => "door_locked",
        })
    }

    fn origin(&self) -> Origin {
        Origin::From(self.door)
    }

    fn falloff(&self) -> Falloff {
        Falloff::DEFAULT
    }
}

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
            .add_message::<ExitOpened>()
            .add_message::<DoorCue>()
            .add_observer(attach_door_transform)
            .add_observer(interact)
            .add_systems(Update, (toggle_doors, animate_doors).chain())
            .add_systems(PostUpdate, update_panels);
    }
}

fn interact(
    _: On<Start<Interact>>,
    players: Query<
        (&Transform, Option<&FuseInventory>, Option<&Hidden>),
        (With<PlayerController>, Without<Caught>),
    >,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    exits: Query<&Door, (With<ExitDoor>, Without<DoorLock>)>,
    mut toggles: MessageWriter<ToggleDoor>,
    mut exits_opened: MessageWriter<ExitOpened>,
) {
    if !enabled.0 {
        return;
    }
    for (player, inventory, hidden) in &players {
        if let Some(InteractTarget::Door(door)) = targets.aimed(player, inventory, hidden) {
            toggles.write(ToggleDoor(door));
            if exits
                .get(door)
                .is_ok_and(|exit| exit.state == DoorState::Closed)
            {
                exits_opened.write(ExitOpened { door });
            }
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
    box_hit(
        origin,
        direction,
        Vec3::new(0.0, 0.01, -PANEL_HALF_THICKNESS),
        Vec3::new(PANEL_WIDTH, PANEL_HEIGHT - 0.01, PANEL_HALF_THICKNESS),
    )
}

pub(crate) fn box_hit(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let min = min.to_array();
    let max = max.to_array();
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

fn attach_door_transform(added: On<Add<Door>>, doors: Query<&Door>, mut commands: Commands) {
    let Ok(door) = doors.get(added.entity) else {
        return;
    };
    commands.entity(added.entity).insert(
        Transform::from_xyz(door.position.x, 0.0, door.position.y).with_rotation(door.rotation),
    );
}

fn toggle_doors(
    mut toggles: MessageReader<ToggleDoor>,
    mut doors: Query<&mut Door, Without<DoorLock>>,
    locked: Query<&Door, With<DoorLock>>,
    mut cues: MessageWriter<DoorCue>,
) {
    for ToggleDoor(entity) in toggles.read() {
        if locked.get(*entity).is_ok() {
            cues.write(DoorCue {
                door: *entity,
                kind: DoorCueKind::Locked,
            });
            continue;
        }
        if let Ok(mut door) = doors.get_mut(*entity) {
            door.state = match door.state {
                DoorState::Closed => DoorState::Open,
                DoorState::Open => DoorState::Closed,
            };
            if door.state == DoorState::Open {
                cues.write(DoorCue {
                    door: *entity,
                    kind: DoorCueKind::Unlatch,
                });
            }
            cues.write(DoorCue {
                door: *entity,
                kind: DoorCueKind::Swing,
            });
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
