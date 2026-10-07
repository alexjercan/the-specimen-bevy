use bevy::{ecs::system::SystemParam, prelude::*};

use super::{
    animation::DoorSwing,
    builder::{Door, DoorRef, Doors, Passage, Room},
    doors::{aimed_door, panel_center, panel_hinge, panel_rotation, PANEL_WIDTH},
    fuses::{fuse_center, fuse_hit, FusePickup},
};

const TILE: f32 = 2.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractTarget {
    Door(Entity),
    Fuse(Entity),
}

#[derive(SystemParam)]
pub struct InteractTargets<'w, 's> {
    doors: Query<'w, 's, (Entity, &'static Door, &'static DoorSwing)>,
    fuses: Query<'w, 's, (Entity, &'static Transform), With<FusePickup>>,
    rooms: Query<'w, 's, (&'static Room, Option<&'static Doors>)>,
    links: Query<'w, 's, &'static DoorRef>,
    passages: Query<'w, 's, &'static Passage>,
}

impl InteractTargets<'_, '_> {
    pub fn aimed(&self, player: &Transform) -> Option<InteractTarget> {
        let origin = player.translation;
        let forward = player.rotation * -Vec3::Z;
        let door = aimed_door(player, &self.doors);
        let mut fuses: Vec<_> = self
            .fuses
            .iter()
            .filter_map(|(entity, transform)| {
                let center = fuse_center(transform);
                fuse_hit(origin, forward, center).map(|distance| (entity, center, distance))
            })
            .filter(|&(_, _, distance)| door.is_none_or(|(_, nearest)| distance <= nearest))
            .collect();
        if !fuses.is_empty() {
            fuses.sort_by(|a, b| a.2.total_cmp(&b.2));
            let blockers = self.sight_blockers();
            if let Some(&(entity, ..)) = fuses.iter().find(|(_, center, _)| {
                let sight = (origin.xz(), center.xz());
                !blockers.iter().any(|&blocker| crosses(sight, blocker))
            }) {
                return Some(InteractTarget::Fuse(entity));
            }
        }
        door.map(|(entity, _)| InteractTarget::Door(entity))
    }

    pub fn anchor(&self, target: InteractTarget) -> Option<Vec3> {
        match target {
            InteractTarget::Door(entity) => self
                .doors
                .get(entity)
                .ok()
                .map(|(_, door, swing)| panel_center(door, swing)),
            InteractTarget::Fuse(entity) => self
                .fuses
                .get(entity)
                .ok()
                .map(|(_, transform)| fuse_center(transform)),
        }
    }

    pub fn door(&self, entity: Entity) -> Option<&Door> {
        self.doors.get(entity).ok().map(|(_, door, _)| door)
    }

    fn opening(&self, link: Entity) -> Option<Vec2> {
        let target = self.links.get(link).ok()?.0;
        self.doors
            .get(target)
            .map(|(_, door, _)| door.position)
            .or_else(|_| self.passages.get(target).map(|passage| passage.0))
            .ok()
    }

    fn sight_blockers(&self) -> Vec<(Vec2, Vec2)> {
        let mut blockers = Vec::new();
        for (room, room_doors) in &self.rooms {
            let bounds = room.0;
            let openings: Vec<Vec2> = room_doors
                .into_iter()
                .flat_map(|links| links.iter())
                .filter_map(|link| self.opening(link))
                .collect();
            for x in 0..(bounds.width() / TILE).round() as usize {
                let center = bounds.min.x + (x as f32 + 0.5) * TILE;
                for z in [bounds.min.y, bounds.max.y] {
                    if !openings.contains(&Vec2::new(center, z)) {
                        blockers.push((
                            Vec2::new(center - TILE / 2.0, z),
                            Vec2::new(center + TILE / 2.0, z),
                        ));
                    }
                }
            }
            for z in 0..(bounds.height() / TILE).round() as usize {
                let center = bounds.min.y + (z as f32 + 0.5) * TILE;
                for x in [bounds.min.x, bounds.max.x] {
                    if !openings.contains(&Vec2::new(x, center)) {
                        blockers.push((
                            Vec2::new(x, center - TILE / 2.0),
                            Vec2::new(x, center + TILE / 2.0),
                        ));
                    }
                }
            }
        }
        for (_, door, swing) in &self.doors {
            let position = Vec3::new(door.position.x, 0.0, door.position.y);
            for side in [-1.0, 1.0] {
                let inner = position + door.rotation * Vec3::X * (side * PANEL_WIDTH / 2.0);
                let outer = position + door.rotation * Vec3::X * (side * TILE / 2.0);
                blockers.push((inner.xz(), outer.xz()));
            }
            let hinge = panel_hinge(door);
            let tip = hinge + panel_rotation(door, swing) * Vec3::X * PANEL_WIDTH;
            blockers.push((hinge.xz(), tip.xz()));
        }
        blockers
    }
}

fn crosses((start, end): (Vec2, Vec2), (from, to): (Vec2, Vec2)) -> bool {
    let sight = end - start;
    let blocker = to - from;
    let denominator = sight.perp_dot(blocker);
    if denominator.abs() < 1e-8 {
        return false;
    }
    let offset = from - start;
    let along_sight = offset.perp_dot(blocker) / denominator;
    let along_blocker = offset.perp_dot(sight) / denominator;
    (0.0..=1.0).contains(&along_sight) && (0.0..=1.0).contains(&along_blocker)
}
