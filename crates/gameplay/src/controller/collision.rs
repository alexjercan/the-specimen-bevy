use bevy::prelude::*;

use super::player::PlayerController;

use crate::levels::{
    Door, DoorOf, DoorRef, DoorSwing, Doors, Passage, Prop, PropCollider, Room,
    PANEL_HALF_THICKNESS, PANEL_OFFSET, PANEL_WIDTH,
};

const TILE: f32 = 2.5;
const WALL_DEPTH: f32 = 0.25;
const FRAME_HALF_WIDTH: f32 = 0.7;
const FRAME_DEPTH: f32 = 0.135;
const PLAYER_RADIUS: f32 = 0.25;
const CONTACT_MARGIN: f32 = 0.001;

#[derive(Clone, Copy)]
struct Obstacle {
    center: Vec2,
    half: Vec2,
    angle: f32,
}

impl Obstacle {
    fn new(center: Vec2, half: Vec2, angle: f32) -> Self {
        Self {
            center,
            half,
            angle,
        }
    }

    fn to_local(self, point: Vec2) -> Vec2 {
        let delta = point - self.center;
        let (s, c) = self.angle.sin_cos();
        Vec2::new(c * delta.x - s * delta.y, s * delta.x + c * delta.y)
    }

    fn direction_local(self, delta: Vec2) -> Vec2 {
        let (s, c) = self.angle.sin_cos();
        Vec2::new(c * delta.x - s * delta.y, s * delta.x + c * delta.y)
    }

    fn entry(self, origin: Vec2, delta: Vec2) -> Option<(f32, Vec2)> {
        let origin = self.to_local(origin);
        let delta = self.direction_local(delta);
        let closest = origin.clamp(-self.half, self.half);
        if (origin - closest).length_squared() < PLAYER_RADIUS * PLAYER_RADIUS - 1e-6 {
            let offset = origin - closest;
            let normal = if offset.length_squared() > 1e-12 {
                offset.normalize()
            } else {
                let free = self.half - origin.abs();
                if free.x < free.y {
                    Vec2::new(if origin.x < 0.0 { -1.0 } else { 1.0 }, 0.0)
                } else {
                    Vec2::new(0.0, if origin.y < 0.0 { -1.0 } else { 1.0 })
                }
            };
            if delta.dot(normal) < -1e-6 {
                let (s, c) = self.angle.sin_cos();
                return Some((
                    0.0,
                    Vec2::new(c * normal.x + s * normal.y, -s * normal.x + c * normal.y),
                ));
            }
            return None;
        }

        let mut hit: Option<(f32, Vec2)> = None;
        for axis in 0..2 {
            if delta[axis].abs() < 1e-8 {
                continue;
            }
            for side in [-1.0_f32, 1.0] {
                let normal = if axis == 0 {
                    Vec2::new(side, 0.0)
                } else {
                    Vec2::new(0.0, side)
                };
                if delta.dot(normal) >= 0.0 {
                    continue;
                }
                let t = (side * (self.half[axis] + PLAYER_RADIUS) - origin[axis]) / delta[axis];
                let other = 1 - axis;
                if (0.0..=1.0).contains(&t)
                    && (origin[other] + delta[other] * t).abs() <= self.half[other]
                    && hit.is_none_or(|(first, _)| t < first)
                {
                    hit = Some((t, normal));
                }
            }
        }

        for x in [-1.0_f32, 1.0] {
            for y in [-1.0_f32, 1.0] {
                let corner = Vec2::new(x * self.half.x, y * self.half.y);
                let offset = origin - corner;
                let a = delta.length_squared();
                if a < 1e-12 {
                    continue;
                }
                let b = offset.dot(delta);
                let discriminant = b * b - a * (offset.length_squared() - PLAYER_RADIUS.powi(2));
                if discriminant < 0.0 {
                    continue;
                }
                let t = (-b - discriminant.sqrt()) / a;
                let contact = offset + delta * t;
                if (0.0..=1.0).contains(&t)
                    && contact.x * x >= 0.0
                    && contact.y * y >= 0.0
                    && contact.dot(delta) < 0.0
                    && hit.is_none_or(|(first, _)| t < first)
                {
                    hit = Some((t, contact.normalize()));
                }
            }
        }
        hit.map(|(t, normal)| {
            let (s, c) = self.angle.sin_cos();
            (
                t,
                Vec2::new(c * normal.x + s * normal.y, -s * normal.x + c * normal.y),
            )
        })
    }
}

pub(crate) fn move_player(start: Vec2, delta: Vec2, obstacles: &[(Vec2, Vec2, f32)]) -> Vec2 {
    let obstacles = obstacles
        .iter()
        .map(|&(center, half, angle)| Obstacle::new(center, half, angle));
    let obstacles: Vec<_> = obstacles.collect();
    let mut position = start;
    let mut remaining = delta;
    for _ in 0..4 {
        if remaining.length_squared() < 1e-12 {
            break;
        }
        let hit = obstacles
            .iter()
            .filter_map(|obstacle| obstacle.entry(position, remaining))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((fraction, normal)) = hit else {
            position += remaining;
            break;
        };
        position += remaining * fraction;
        position += normal * CONTACT_MARGIN;
        remaining *= 1.0 - fraction;
        remaining -= normal * remaining.dot(normal).min(0.0);
        if delta.x.abs() < 1e-8 {
            remaining.x = 0.0;
        }
        if delta.y.abs() < 1e-8 {
            remaining.y = 0.0;
        }
    }
    position
}

fn prop_footprint(module: &str) -> Option<PropCollider> {
    let (min, max) = match module {
        "storage_crate" => (Vec2::new(-0.51, -0.415), Vec2::new(0.51, 0.41)),
        "steel_drum" => (Vec2::splat(-0.3), Vec2::splat(0.3)),
        "drum_spilled" => (Vec2::new(-0.41, -0.35), Vec2::new(0.75, 0.35)),
        "shelf_unit" => (Vec2::new(-0.9, -0.25), Vec2::new(0.9, 0.258)),
        "shelf_unit_bins" => (Vec2::new(-0.9, -0.25), Vec2::new(0.9, 0.25)),
        "shelf_unit_low" => (Vec2::new(-0.6, -0.225), Vec2::new(0.6, 0.225)),
        "workbench" => (Vec2::new(-0.8, -0.35), Vec2::new(0.8, 0.35)),
        "concept_table" => (Vec2::new(-0.9, -0.45), Vec2::new(0.9, 0.46)),
        "concept_locker" => (Vec2::new(-0.3, -0.4873), Vec2::new(0.3144, 0.3)),
        "work_island" => (Vec2::new(-0.95, -0.45), Vec2::new(0.95, 0.4529)),
        "lab_console" => (Vec2::new(-0.75, -0.35), Vec2::new(0.75, 0.35)),
        "boiler_unit" => (Vec2::new(-0.635, -0.7), Vec2::new(0.635, 0.635)),
        "concept_containment_tank" => (Vec2::new(-0.7327, -0.9189), Vec2::new(0.702, 0.6687)),
        "chair_tipped" => (Vec2::new(-0.4141, -0.3134), Vec2::new(0.4141, 0.311)),
        _ => return None,
    };
    Some(PropCollider {
        center: (min + max) / 2.0,
        half: (max - min) / 2.0,
    })
}

pub(crate) fn attach_prop_collider(
    added: On<Add, Prop>,
    props: Query<&Prop>,
    mut commands: Commands,
) {
    if let Ok(prop) = props.get(added.entity) {
        if let Some(collider) = prop_footprint(&prop.0) {
            commands.entity(added.entity).insert(collider);
        }
    }
}

pub(crate) fn colliders(
    rooms: &Query<(&Room, Option<&Doors>)>,
    links: &Query<(&DoorRef, &DoorOf)>,
    doors: &Query<(&Door, &DoorSwing)>,
    passages: &Query<&Passage>,
    props: &Query<(&PropCollider, &Transform), Without<PlayerController>>,
) -> Vec<(Vec2, Vec2, f32)> {
    let mut walls = Vec::new();
    for (room, room_doors) in rooms {
        let bounds = room.0;
        let openings: Vec<_> = room_doors
            .into_iter()
            .flat_map(|doors| doors.iter())
            .filter_map(|link| {
                let (door, _) = links.get(link).ok()?;
                doors
                    .get(door.0)
                    .map(|(door, _)| door.position)
                    .or_else(|_| passages.get(door.0).map(|passage| passage.0))
                    .ok()
            })
            .collect();
        for x in 0..(bounds.width() / TILE).round() as usize {
            let center = bounds.min.x + (x as f32 + 0.5) * TILE;
            for z in [bounds.min.y, bounds.max.y] {
                if !openings.contains(&Vec2::new(center, z)) {
                    walls.push((false, z, center - TILE / 2.0, center + TILE / 2.0));
                }
            }
        }
        for z in 0..(bounds.height() / TILE).round() as usize {
            let center = bounds.min.y + (z as f32 + 0.5) * TILE;
            for x in [bounds.min.x, bounds.max.x] {
                if !openings.contains(&Vec2::new(x, center)) {
                    walls.push((true, x, center - TILE / 2.0, center + TILE / 2.0));
                }
            }
        }
    }
    let mut obstacles = merge_walls(walls);
    for (door, swing) in doors {
        let (yaw, _, _) = door.rotation.to_euler(EulerRot::YXZ);
        let position = door.position;
        for side in [-1.0, 1.0] {
            let distance = (TILE / 2.0 + FRAME_HALF_WIDTH) / 2.0;
            let local = Vec2::new(side * distance, 0.0);
            obstacles.push((
                position + rotate(local, yaw),
                Vec2::new((TILE / 2.0 - FRAME_HALF_WIDTH) / 2.0, FRAME_DEPTH),
                yaw,
            ));
        }
        let angle = yaw + swing.0;
        let hinge = position + rotate(PANEL_OFFSET.xz(), yaw);
        obstacles.push((
            hinge + rotate(Vec2::new(PANEL_WIDTH / 2.0, 0.0), angle),
            Vec2::new(PANEL_WIDTH / 2.0, PANEL_HALF_THICKNESS),
            angle,
        ));
    }
    for (collider, transform) in props {
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let scale = transform.scale.xz();
        let center = transform.translation.xz() + rotate(collider.center * scale, yaw);
        obstacles.push((center, collider.half * scale.abs(), yaw));
    }
    obstacles
}

fn merge_walls(mut walls: Vec<(bool, f32, f32, f32)>) -> Vec<(Vec2, Vec2, f32)> {
    walls.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
    });
    let mut merged: Vec<(bool, f32, f32, f32)> = Vec::new();
    for (vertical, line, start, end) in walls {
        if let Some(last) = merged.last_mut() {
            if last.0 == vertical && last.1 == line && start <= last.3 {
                last.3 = last.3.max(end);
                continue;
            }
        }
        merged.push((vertical, line, start, end));
    }
    merged
        .into_iter()
        .map(|(vertical, line, start, end)| {
            let center = (start + end) / 2.0;
            let half = (end - start) / 2.0;
            if vertical {
                (
                    Vec2::new(line, center),
                    Vec2::new(half, WALL_DEPTH / 2.0),
                    std::f32::consts::FRAC_PI_2,
                )
            } else {
                (
                    Vec2::new(center, line),
                    Vec2::new(half, WALL_DEPTH / 2.0),
                    0.0,
                )
            }
        })
        .collect()
}

fn rotate(point: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(c * point.x + s * point.y, -s * point.x + c * point.y)
}
