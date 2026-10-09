use bevy::{ecs::system::SystemParam, prelude::*};

use super::{
    animation::DoorSwing,
    builder::{Door, DoorRef, Doors, Passage, Prop, Room},
    doors::{aimed_door, panel_center, panel_hinge, panel_rotation, DoorLock, PANEL_WIDTH},
    fuses::{fuse_panel_hit, FuseInventory, FusePanel, FUSE_COUNT},
    hiding::{Hidden, HidingSpot},
    module_names::BOILER_UNIT,
    pickups::PickupKind,
    power::FacilityPower,
};

const TILE: f32 = 2.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractTarget {
    Door(Entity),
    Pickup(Entity),
    Panel(Entity),
    Boiler(Entity),
    Hide(Entity),
    Leave(Entity),
}

#[derive(SystemParam)]
pub struct InteractTargets<'w, 's> {
    doors: Query<'w, 's, (Entity, &'static Door, &'static DoorSwing)>,
    locks: Query<'w, 's, (), With<DoorLock>>,
    pickups: Query<'w, 's, (Entity, &'static Transform, &'static PickupKind)>,
    panels: Query<'w, 's, (Entity, &'static Transform, &'static FusePanel)>,
    boilers: Query<'w, 's, (Entity, &'static Prop, &'static Transform)>,
    power: Option<Res<'w, FacilityPower>>,
    spots: Query<'w, 's, (Entity, &'static Transform, &'static HidingSpot)>,
    sight: StructuralSight<'w, 's>,
}

#[derive(SystemParam)]
pub(crate) struct StructuralSight<'w, 's> {
    doors: Query<'w, 's, (&'static Door, &'static DoorSwing)>,
    rooms: Query<'w, 's, (&'static Room, Option<&'static Doors>)>,
    links: Query<'w, 's, &'static DoorRef>,
    passages: Query<'w, 's, &'static Passage>,
}

impl InteractTargets<'_, '_> {
    pub fn aimed(
        &self,
        player: &Transform,
        inventory: Option<&FuseInventory>,
        hidden: Option<&Hidden>,
    ) -> Option<InteractTarget> {
        if let Some(hidden) = hidden {
            return Some(InteractTarget::Leave(hidden.spot));
        }
        let origin = player.translation;
        let forward = player.rotation * -Vec3::Z;
        let door = aimed_door(player, &self.doors);
        let ready = inventory.is_some_and(|inventory| inventory.0 >= FUSE_COUNT);
        let pickups = self.pickups.iter().filter_map(|(entity, transform, kind)| {
            let center = kind.center(transform);
            kind.hit(origin, forward, center)
                .map(|distance| (InteractTarget::Pickup(entity), center, distance))
        });
        let panels = self
            .panels
            .iter()
            .filter(|(_, _, panel)| ready && panel.installed == 0)
            .filter_map(|(entity, transform, _)| {
                let center = transform.translation;
                fuse_panel_hit(origin, forward, center)
                    .map(|distance| (InteractTarget::Panel(entity), center, distance))
            });
        let boilers = self.boilers.iter().filter_map(|(entity, prop, transform)| {
            (prop.0 == BOILER_UNIT && self.power.as_ref().is_some_and(|power| !power.on))
                .then(|| {
                    let center = transform.translation + Vec3::Y * 1.2;
                    fuse_panel_hit(origin, forward, center)
                        .map(|distance| (InteractTarget::Boiler(entity), center, distance))
                })
                .flatten()
        });
        let spots = self.spots.iter().filter_map(|(entity, transform, spot)| {
            spot.hit(transform, origin, forward).map(|distance| {
                (
                    InteractTarget::Hide(entity),
                    transform.translation,
                    distance,
                )
            })
        });
        let mut candidates: Vec<_> = pickups
            .chain(panels)
            .chain(boilers)
            .chain(spots)
            .filter(|&(_, _, distance)| door.is_none_or(|(_, nearest)| distance <= nearest))
            .collect();
        if !candidates.is_empty() {
            candidates.sort_by(|a, b| a.2.total_cmp(&b.2));
            let blockers = self.sight.sight_blockers();
            if let Some(&(target, ..)) = candidates.iter().find(|(_, center, _)| {
                let sight = (origin.xz(), center.xz());
                !blockers.iter().any(|&blocker| crosses(sight, blocker))
            }) {
                return Some(target);
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
            InteractTarget::Pickup(entity) => self
                .pickups
                .get(entity)
                .ok()
                .map(|(_, transform, kind)| kind.center(transform)),
            InteractTarget::Panel(entity) => self
                .panels
                .get(entity)
                .ok()
                .map(|(_, transform, _)| transform.translation),
            InteractTarget::Boiler(entity) => self
                .boilers
                .get(entity)
                .ok()
                .map(|(_, _, transform)| transform.translation + Vec3::Y * 1.2),
            InteractTarget::Hide(entity) | InteractTarget::Leave(entity) => self
                .spots
                .get(entity)
                .ok()
                .map(|(_, transform, spot)| spot.anchor(transform)),
        }
    }

    pub fn door(&self, entity: Entity) -> Option<&Door> {
        self.doors.get(entity).ok().map(|(_, door, _)| door)
    }

    pub fn pickup(&self, entity: Entity) -> Option<PickupKind> {
        self.pickups.get(entity).ok().map(|(_, _, kind)| *kind)
    }

    pub fn locked(&self, entity: Entity) -> bool {
        self.locks.contains(entity)
    }
}

impl StructuralSight<'_, '_> {
    pub(crate) fn clear_sight(&self, start: Vec2, end: Vec2) -> bool {
        !self
            .sight_blockers()
            .into_iter()
            .any(|blocker| crosses((start, end), blocker))
    }

    fn opening(&self, link: Entity) -> Option<Vec2> {
        let target = self.links.get(link).ok()?.0;
        self.doors
            .get(target)
            .map(|(door, _)| door.position)
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
        for (door, swing) in &self.doors {
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
