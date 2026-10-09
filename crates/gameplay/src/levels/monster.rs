use std::collections::{HashMap, HashSet, VecDeque};

use bevy::{animation::graph::AnimationNodeIndex, prelude::*, world_serialization::WorldAssetRoot};
use bevy_rand::prelude::ChaCha8Rng;
use game_assets::MonsterAssets;
use game_audio::{PlaySourceSound, Sound, SourceSounds};
use rand_core::Rng;

use crate::controller::{collision, PlayerController, PlayerControlsEnabled};

use super::{
    Door, DoorLock, DoorOf, DoorRef, DoorState, DoorSwing, Doors, Passage, PropCollider, Room,
    ToggleDoor,
};

const SPEED: f32 = 0.51;
const GRID: f32 = 0.5;
const ARRIVAL: f32 = 0.16;
const DOOR_APPROACH: f32 = 1.35;
const DOOR_CLEAR: f32 = 1.8;
const STEP_PERIOD: f32 = 0.6;
const PRESENCE_INTERVAL: f32 = 6.0;
const MODEL_SCALE: f32 = 0.42;
const MODEL_LIFT: f32 = 1.93;
const MODEL_PIVOT: Vec2 = Vec2::new(0.091, 3.793);
const TURN_SPEED: f32 = 2.1;
const MOVE_FACING_TOLERANCE: f32 = 0.18;

#[cfg(test)]
#[path = "../../tests/unit/monster.rs"]
mod tests;

#[derive(Component)]
pub struct Monster {
    room: Option<Entity>,
    route: VecDeque<Vec2>,
    opened_doors: Vec<(Entity, Vec2)>,
    step_clock: f32,
    presence_clock: f32,
    step_index: usize,
    route_retry: f32,
}

impl Default for Monster {
    fn default() -> Self {
        Self {
            room: None,
            route: VecDeque::new(),
            opened_doors: Vec::new(),
            step_clock: 0.0,
            presence_clock: 0.0,
            step_index: 0,
            route_retry: 0.0,
        }
    }
}

#[derive(Resource)]
pub(super) struct PatrolRng(pub(super) ChaCha8Rng);

#[derive(Component)]
struct MonsterScene;

#[derive(Resource)]
struct MonsterAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
}

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleDoor>()
            .add_message::<PlaySourceSound>()
            .add_observer(attach_monster_sounds)
            .add_systems(Update, patrol);
    }
}

pub struct MonsterRenderPlugin;

impl Plugin for MonsterRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_scene, animate_monster).chain());
    }
}

fn attach_monster_sounds(added: On<Add, Monster>, mut commands: Commands) {
    commands.entity(added.entity).insert(SourceSounds(vec![
        (Sound::MonsterPresence, Vec3::Y),
        (Sound::MonsterStep(0), Vec3::Y),
        (Sound::MonsterStep(1), Vec3::Y),
        (Sound::MonsterStep(2), Vec3::Y),
        (Sound::MonsterStep(3), Vec3::Y),
        (Sound::MonsterStep(4), Vec3::Y),
        (Sound::MonsterStep(5), Vec3::Y),
    ]));
}

fn attach_scene(
    mut commands: Commands,
    monsters: Query<Entity, (With<Monster>, Without<MonsterScene>)>,
    assets: Option<Res<MonsterAssets>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animations: Option<Res<MonsterAnimations>>,
) {
    let Some(assets) = assets else { return };
    if animations.is_none() {
        let (graph, clips) = AnimationGraph::from_clips([assets.idle.clone(), assets.walk.clone()]);
        commands.insert_resource(MonsterAnimations {
            graph: graphs.add(graph),
            idle: clips[0],
            walk: clips[1],
        });
    }
    for entity in &monsters {
        commands.entity(entity).with_children(|children| {
            children.spawn((
                MonsterScene,
                WorldAssetRoot(assets.scene.clone()),
                Transform::from_xyz(
                    -MODEL_PIVOT.x * MODEL_SCALE,
                    MODEL_LIFT,
                    -MODEL_PIVOT.y * MODEL_SCALE,
                )
                .with_scale(Vec3::splat(MODEL_SCALE)),
            ));
        });
        commands.entity(entity).insert(MonsterScene);
    }
}

fn animate_monster(
    mut commands: Commands,
    animations: Option<Res<MonsterAnimations>>,
    monsters: Query<(&Monster, &Children)>,
    scene_roots: Query<(), With<MonsterScene>>,
    children: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    let Some(animations) = animations else { return };
    for (monster, roots) in &monsters {
        for root in roots.iter().filter(|root| scene_roots.contains(*root)) {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok((entity, mut player, graph)) = players.get_mut(entity) {
                    if graph.is_none() {
                        commands
                            .entity(entity)
                            .insert(AnimationGraphHandle(animations.graph.clone()));
                    }
                    let clip = if monster.route.is_empty() {
                        animations.idle
                    } else {
                        animations.walk
                    };
                    if player.is_added() || !player.is_playing_animation(clip) {
                        player.stop_all();
                        player.play(clip).repeat();
                    }
                }
                if let Ok(descendants) = children.get(entity) {
                    stack.extend(descendants.iter());
                }
            }
        }
    }
}

fn room_at(position: Vec2, rooms: &[(Entity, Rect)]) -> Option<Entity> {
    rooms
        .iter()
        .find(|(_, bounds)| {
            let inset = 0.05;
            position.x >= bounds.min.x - inset
                && position.x <= bounds.max.x + inset
                && position.y >= bounds.min.y - inset
                && position.y <= bounds.max.y + inset
        })
        .map(|(entity, _)| *entity)
}

fn node(point: Vec2) -> IVec2 {
    (point / GRID).round().as_ivec2()
}

fn point(cell: IVec2) -> Vec2 {
    cell.as_vec2() * GRID
}

fn route(
    start: Vec2,
    end: Vec2,
    rooms: &[(Entity, Rect)],
    blockers: &[(Vec2, Vec2, f32)],
) -> Option<VecDeque<Vec2>> {
    let start = node(start);
    let goal = node(end);
    let valid = |cell: IVec2| {
        let position = point(cell);
        room_at(position, rooms).is_some() && collision::clear_for_player(position, blockers)
    };
    if !valid(start) {
        return None;
    }
    let goal = if valid(goal) {
        goal
    } else {
        (-6..=6)
            .flat_map(|x| (-6..=6).map(move |y| goal + IVec2::new(x, y)))
            .filter(|&candidate| {
                valid(candidate) && room_at(point(candidate), rooms) == room_at(end, rooms)
            })
            .min_by_key(|candidate| (candidate.x - goal.x).pow(2) + (candidate.y - goal.y).pow(2))?
    };
    let mut open = VecDeque::from([start]);
    let mut seen = HashSet::from([start]);
    let mut previous = HashMap::new();
    while let Some(current) = open.pop_front() {
        if current == goal {
            let mut cells = vec![current];
            let mut cursor = current;
            while cursor != start {
                cursor = previous[&cursor];
                cells.push(cursor);
            }
            cells.reverse();
            return Some(cells.into_iter().skip(1).map(point).collect());
        }
        for direction in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            let next = current + direction;
            if seen.contains(&next) || !valid(next) {
                continue;
            }
            let from = point(current);
            let delta = point(next) - from;
            if collision::move_player(from, delta, blockers).distance(from + delta) > 0.01 {
                continue;
            }
            seen.insert(next);
            previous.insert(next, current);
            open.push_back(next);
        }
    }
    None
}

fn turn_toward(transform: &mut Transform, direction: Vec2, delta: f32) -> bool {
    let target = (-direction.x).atan2(-direction.y);
    let (current, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let difference = (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let step = difference.clamp(-TURN_SPEED * delta, TURN_SPEED * delta);
    transform.rotation = Quat::from_rotation_y(current + step);
    (difference - step).abs() <= MOVE_FACING_TOLERANCE
}

fn patrol(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut rng: Option<ResMut<PatrolRng>>,
    rooms: Query<(Entity, &Room, Option<&Doors>)>,
    room_walls: Query<(&Room, Option<&Doors>)>,
    links: Query<(&DoorRef, &DoorOf)>,
    doors: Query<(Entity, &Door, &DoorSwing, Has<DoorLock>)>,
    door_walls: Query<(&Door, &DoorSwing)>,
    passages: Query<&Passage>,
    props: Query<(&PropCollider, &Transform), (Without<PlayerController>, Without<Monster>)>,
    players: Query<&Transform, With<PlayerController>>,
    mut monsters: Query<(Entity, &mut Monster, &mut Transform), Without<PlayerController>>,
    mut toggles: MessageWriter<ToggleDoor>,
    mut sounds: MessageWriter<PlaySourceSound>,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    let Some(rng) = rng.as_deref_mut() else {
        return;
    };
    let rooms_data: Vec<_> = rooms
        .iter()
        .map(|(entity, room, _)| (entity, room.0))
        .collect();
    let mut blockers = collision::wall_obstacles(&room_walls, &links, &door_walls, &passages);
    for (_, door, _, locked) in &doors {
        blockers.extend(collision::door_frames(door));
        if locked {
            blockers.push(collision::door_panel(door, &DoorSwing::default()));
        }
    }
    for (collider, transform) in &props {
        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let center = transform.translation.xz()
            + collision::rotate(collider.center * transform.scale.xz(), yaw);
        blockers.push((center, collider.half * transform.scale.xz().abs(), yaw));
    }
    let delta = time.delta_secs().min(0.1);
    for (entity, mut monster, mut transform) in &mut monsters {
        monster.presence_clock += delta;
        if monster.presence_clock >= PRESENCE_INTERVAL {
            monster.presence_clock -= PRESENCE_INTERVAL;
            sounds.write(PlaySourceSound {
                source: entity,
                sound: Sound::MonsterPresence,
            });
        }
        let position = transform.translation.xz();
        monster.opened_doors.retain(|&(door_entity, origin)| {
            let Ok((_, door, _, locked)) = doors.get(door_entity) else {
                return false;
            };
            let normal = (door.rotation * -Vec3::Z).xz();
            let crossed =
                (origin - door.position).dot(normal) * (position - door.position).dot(normal) < 0.0;
            let player_near = players
                .iter()
                .any(|player| player.translation.xz().distance(door.position) < DOOR_CLEAR);
            if !locked && crossed && !player_near && position.distance(door.position) > DOOR_CLEAR {
                if door.state == DoorState::Open {
                    toggles.write(ToggleDoor(door_entity));
                }
                return false;
            }
            true
        });
        if monster.route.is_empty() && monster.route_retry > 0.0 {
            monster.route_retry -= delta;
            continue;
        }
        if monster.route.is_empty() {
            monster.room = room_at(position, &rooms_data).or(monster.room);
            if let Some(room) = monster.room {
                let neighbors: Vec<_> = rooms
                    .iter()
                    .filter_map(|(candidate, room_data, links_in_room)| {
                        (candidate != room
                            && links_in_room.is_some_and(|room_links| {
                                room_links.iter().any(|link| {
                                    let Ok((door_ref, _)) = links.get(link) else {
                                        return false;
                                    };
                                    let opening = door_ref.0;
                                    let connected = rooms
                                        .get(room)
                                        .ok()
                                        .and_then(|(_, _, old_links)| old_links)
                                        .is_some_and(|old_links| {
                                            old_links.iter().any(|other| {
                                                links.get(other).is_ok_and(|(reference, _)| {
                                                    reference.0 == opening
                                                })
                                            })
                                        });
                                    connected
                                        && !doors.get(opening).is_ok_and(|(_, _, _, locked)| locked)
                                })
                            }))
                        .then_some((candidate, room_data.0))
                    })
                    .collect();
                if !neighbors.is_empty() {
                    let count = neighbors.len() as u64;
                    let limit = u64::MAX - u64::MAX % count;
                    let sample = loop {
                        let sample = rng.0.next_u64();
                        if sample < limit {
                            break sample;
                        }
                    };
                    let pick = (sample % count) as usize;
                    let (destination, bounds) = neighbors[pick];
                    let center = (bounds.min + bounds.max) * 0.5;
                    if let Some(path) = route(position, center, &rooms_data, &blockers) {
                        monster.route = path;
                        monster.room = Some(destination);
                    } else {
                        monster.route_retry = 1.0;
                    }
                }
            }
        }
        let Some(next) = monster.route.front().copied() else {
            continue;
        };
        if position.distance(next) < ARRIVAL {
            monster.route.pop_front();
            continue;
        }
        let forward = (next - position).normalize_or_zero();
        let facing_waypoint = turn_toward(&mut transform, forward, delta);
        let target_door = doors.iter().find(|(_, door, _, locked)| {
            !locked
                && position.distance(door.position) < DOOR_APPROACH
                && monster
                    .route
                    .iter()
                    .take(5)
                    .any(|waypoint| waypoint.distance(door.position) < DOOR_APPROACH)
        });
        if let Some((door_entity, door, swing, _)) = target_door {
            if !monster
                .opened_doors
                .iter()
                .any(|(opened, _)| *opened == door_entity)
            {
                if door.state == DoorState::Closed {
                    toggles.write(ToggleDoor(door_entity));
                }
                monster.opened_doors.push((door_entity, position));
            }
            if swing.0 < 1.4 {
                continue;
            }
        }
        if !facing_waypoint {
            continue;
        }
        let step = forward * SPEED * delta;
        let mut dynamic = blockers.clone();
        for (_, door, swing, locked) in &doors {
            if !locked {
                dynamic.push(collision::door_panel(door, swing));
            }
        }
        let moved = collision::move_player(position, step, &dynamic);
        if moved.distance(position) < step.length() * 0.15 {
            monster.route.clear();
            monster.room = room_at(position, &rooms_data);
            monster.route_retry = 1.0;
            continue;
        }
        transform.translation.x = moved.x;
        transform.translation.z = moved.y;
        monster.step_clock += delta;
        if monster.step_clock >= STEP_PERIOD {
            monster.step_clock -= STEP_PERIOD;
            sounds.write(PlaySourceSound {
                source: entity,
                sound: Sound::MonsterStep(monster.step_index),
            });
            monster.step_index = (monster.step_index + 1) % 6;
        }
    }
}
