use std::collections::{HashMap, HashSet, VecDeque};

use bevy::{animation::graph::AnimationNodeIndex, prelude::*, world_serialization::WorldAssetRoot};
use bevy_rand::prelude::ChaCha8Rng;
use game_assets::MonsterAssets;
use game_audio::{PlaySourceSound, Sound, SourceSounds};
use rand_core::Rng;

use crate::controller::{collision, PlayerController, PlayerControlsEnabled, Stamina, RUN_SPEED};

use super::{
    interaction::InteractTargets, Door, DoorLock, DoorOf, DoorRef, DoorState, DoorSwing, Doors,
    Escaped, Hidden, Passage, PropCollider, Room, ToggleDoor,
};

const SPEED: f32 = 0.51;
const CHASE_SPEED: f32 = RUN_SPEED + 1.0;
const SIGHT_RANGE: f32 = 12.0;
const SIGHT_COS: f32 = 0.45;
const WALK_HEARING: f32 = 3.0;
const RUN_HEARING: f32 = 9.0;
const SEARCH_TIME: f32 = 8.0;
const SEARCH_RADIUS: f32 = 2.5;
const SEARCH_REACH: f32 = 10.0;
const STALL_LIMIT: u8 = 3;
const SEARCH_RETRY: f32 = 0.5;
const ATTACK_REACH: f32 = 0.9;
const ATTACK_DURATION: f32 = 19.0 / 24.0;
const JUMPSCARE_TURN_SPEED: f32 = 9.0;
const JUMPSCARE_FOCUS: f32 = 1.8;
const GRID: f32 = 0.5;
const ARRIVAL: f32 = 0.16;
const DOOR_APPROACH: f32 = 1.35;
const STEP_PERIOD: f32 = 0.6;
const CHASE_STEP_PERIOD: f32 = 0.32;
const PRESENCE_INTERVAL: f32 = 24.0;
const MODEL_SCALE: f32 = 0.42;
const MODEL_LIFT: f32 = 1.93;
const MODEL_PIVOT: Vec2 = Vec2::new(0.091, 3.793);
const TURN_SPEED: f32 = 2.1;
const CHASE_TURN_SPEED: f32 = 8.0;
const MOVE_FACING_TOLERANCE: f32 = 0.18;

#[cfg(test)]
#[path = "../../tests/unit/monster.rs"]
mod tests;

#[derive(Component)]
pub struct Monster {
    route: VecDeque<Vec2>,
    step_clock: f32,
    presence_clock: f32,
    step_index: usize,
    route_retry: f32,
    stalls: u8,
    pursuit: Option<Pursuit>,
}

#[derive(Clone, Copy, Debug)]
struct Pursuit {
    last_sensed: Vec2,
    remaining: f32,
    searching: bool,
    planned: Option<Vec2>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Caught {
    pub monster: Entity,
    pub remaining: f32,
}

impl Caught {
    pub fn new(monster: Entity) -> Self {
        Self {
            monster,
            remaining: ATTACK_DURATION,
        }
    }

    pub fn finished(&self) -> bool {
        self.remaining <= 0.0
    }
}

impl Default for Monster {
    fn default() -> Self {
        Self {
            route: VecDeque::new(),
            step_clock: 0.0,
            presence_clock: 0.0,
            step_index: 0,
            route_retry: 0.0,
            stalls: 0,
            pursuit: None,
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
    chase: AnimationNodeIndex,
    attack: AnimationNodeIndex,
}

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleDoor>()
            .add_message::<PlaySourceSound>()
            .add_observer(attach_monster_sounds)
            .add_systems(
                Update,
                (sense_player, manage_doors, patrol, catch_player).chain(),
            );
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
        (Sound::MonsterDetected, Vec3::Y),
        (Sound::MonsterAttack, Vec3::Y),
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
        let (graph, clips) = AnimationGraph::from_clips([
            assets.idle.clone(),
            assets.walk.clone(),
            assets.chase.clone(),
            assets.attack.clone(),
        ]);
        commands.insert_resource(MonsterAnimations {
            graph: graphs.add(graph),
            idle: clips[0],
            walk: clips[1],
            chase: clips[2],
            attack: clips[3],
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
    monsters: Query<(Entity, &Monster, &Children)>,
    caught: Query<&Caught>,
    scene_roots: Query<(), With<MonsterScene>>,
    children: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    let Some(animations) = animations else { return };
    for (entity, monster, roots) in &monsters {
        let attacking = caught.iter().any(|caught| caught.monster == entity);
        for root in roots.iter().filter(|root| scene_roots.contains(*root)) {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok((entity, mut player, graph)) = players.get_mut(entity) {
                    if graph.is_none() {
                        commands
                            .entity(entity)
                            .insert(AnimationGraphHandle(animations.graph.clone()));
                    }
                    let clip = if attacking {
                        animations.attack
                    } else if monster.pursuit.is_some() {
                        animations.chase
                    } else if monster.route.is_empty() {
                        animations.idle
                    } else {
                        animations.walk
                    };
                    if player.is_added() || !player.is_playing_animation(clip) {
                        player.stop_all();
                        let playing = player.play(clip);
                        if !attacking {
                            playing.repeat();
                        }
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
    let origin = start;
    let goal = node(end);
    let valid = |cell: IVec2| {
        let position = point(cell);
        room_at(position, rooms).is_some() && collision::clear_for_player(position, blockers)
    };
    let start = (-1..=1)
        .flat_map(|x| (-1..=1).map(move |y| node(origin) + IVec2::new(x, y)))
        .filter(|&cell| valid(cell) && clear_path(origin, point(cell), blockers))
        .min_by(|a, b| {
            origin
                .distance_squared(point(*a))
                .total_cmp(&origin.distance_squared(point(*b)))
        })?;
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
            let mut path: VecDeque<_> = cells.into_iter().map(point).collect();
            if path
                .front()
                .is_some_and(|first| first.distance(origin) < ARRIVAL)
            {
                path.pop_front();
            }
            return Some(path);
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

fn clear_path(start: Vec2, end: Vec2, blockers: &[(Vec2, Vec2, f32)]) -> bool {
    collision::move_player(start, end - start, blockers).distance(end) < 0.01
}

fn search_route(
    position: Vec2,
    center: Vec2,
    rng: &mut ChaCha8Rng,
    rooms: &[(Entity, Rect)],
    blockers: &[(Vec2, Vec2, f32)],
) -> VecDeque<Vec2> {
    let first = rng.next_u32() as usize;
    (0..16)
        .map(|index| (first + index) % 16)
        .map(|index| {
            let radius = if index < 8 {
                SEARCH_RADIUS
            } else {
                SEARCH_RADIUS * 0.5
            };
            center + Vec2::from_angle(index as f32 * std::f32::consts::FRAC_PI_4) * radius
        })
        .filter(|&target| {
            position.distance(target) > GRID
                && room_at(target, rooms).is_some()
                && collision::clear_for_player(target, blockers)
        })
        .find_map(|target| {
            route(position, target, rooms, blockers)
                .filter(|path| !path.is_empty() && path.len() as f32 * GRID <= SEARCH_REACH)
        })
        .unwrap_or_default()
}

fn turn_toward(transform: &mut Transform, direction: Vec2, delta: f32, speed: f32) -> bool {
    let target = (-direction.x).atan2(-direction.y);
    let (current, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let difference = (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let step = difference.clamp(-speed * delta, speed * delta);
    transform.rotation = Quat::from_rotation_y(current + step);
    (difference - step).abs() <= MOVE_FACING_TOLERANCE
}

fn sense_player(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    sight: InteractTargets,
    players: Query<
        (
            &Transform,
            &crate::controller::PlayerInput,
            &Stamina,
            Has<Hidden>,
        ),
        (With<PlayerController>, Without<Escaped>, Without<Caught>),
    >,
    mut monsters: Query<(Entity, &Transform, &mut Monster), Without<PlayerController>>,
    mut sounds: MessageWriter<PlaySourceSound>,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (entity, transform, mut monster) in &mut monsters {
        let position = transform.translation.xz();
        let sensed = players.iter().find_map(|(player, input, stamina, hidden)| {
            if hidden {
                return None;
            }
            let target = player.translation.xz();
            let distance = position.distance(target);
            let direction = (target - position).normalize_or_zero();
            let facing = (transform.rotation * -Vec3::Z).xz();
            let visible = distance <= SIGHT_RANGE
                && (distance <= ATTACK_REACH || direction.dot(facing) >= SIGHT_COS);
            let heard = input.movement.length_squared() > 0.01
                && distance
                    <= if stamina.sprinting {
                        RUN_HEARING
                    } else {
                        WALK_HEARING
                    };
            (visible || heard)
                .then(|| sight.clear_sight(position, target))
                .filter(|clear| *clear)
                .map(|_| target)
        });
        if let Some(target) = sensed {
            if let Some(mut pursuit) = monster.pursuit {
                if pursuit.searching {
                    pursuit.searching = false;
                    pursuit.planned = None;
                    monster.route.clear();
                }
                pursuit.last_sensed = target;
                pursuit.remaining = SEARCH_TIME;
                monster.pursuit = Some(pursuit);
            } else {
                sounds.write(PlaySourceSound {
                    source: entity,
                    sound: Sound::MonsterDetected,
                });
                monster.route.clear();
                monster.stalls = 0;
                monster.pursuit = Some(Pursuit {
                    last_sensed: target,
                    remaining: SEARCH_TIME,
                    searching: false,
                    planned: None,
                });
            }
        } else if let Some(mut pursuit) = monster.pursuit {
            pursuit.remaining -= time.delta_secs();
            if pursuit.remaining <= 0.0 {
                monster.pursuit = None;
                monster.route.clear();
            } else {
                monster.pursuit = Some(pursuit);
            }
        }
    }
}

fn catch_player(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut players: Query<
        (Entity, &mut Transform, Has<Hidden>, Option<&mut Caught>),
        (With<PlayerController>, Without<Escaped>),
    >,
    mut monsters: Query<(Entity, &mut Transform, &mut Monster), Without<PlayerController>>,
    mut sounds: MessageWriter<PlaySourceSound>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    let delta = time.delta_secs();
    let mut staged = false;
    for (_, mut view, _, caught) in &mut players {
        let Some(mut caught) = caught else {
            continue;
        };
        staged = true;
        if caught.finished() {
            continue;
        }
        caught.remaining = (caught.remaining - delta).max(0.0);
        let Ok((_, mut pose, _)) = monsters.get_mut(caught.monster) else {
            continue;
        };
        let toward = (view.translation.xz() - pose.translation.xz()).normalize_or_zero();
        if toward != Vec2::ZERO {
            turn_toward(&mut pose, toward, delta, CHASE_TURN_SPEED);
            let focus = pose.translation + Vec3::Y * JUMPSCARE_FOCUS;
            let facing = view.looking_at(focus, Vec3::Y).rotation;
            view.rotation = view
                .rotation
                .rotate_towards(facing, JUMPSCARE_TURN_SPEED * delta);
        }
    }
    if staged {
        return;
    }
    for (monster_entity, pose, mut monster) in &mut monsters {
        if monster.pursuit.is_none() {
            continue;
        }
        let Some((player, ..)) = players.iter().find(|(_, view, hidden, _)| {
            !hidden && pose.translation.xz().distance(view.translation.xz()) <= ATTACK_REACH
        }) else {
            continue;
        };
        monster.route.clear();
        sounds.write(PlaySourceSound {
            source: monster_entity,
            sound: Sound::MonsterAttack,
        });
        commands.entity(player).insert(Caught::new(monster_entity));
        return;
    }
}

fn manage_doors(
    enabled: Option<Res<PlayerControlsEnabled>>,
    doors: Query<(Entity, &Door, Has<DoorLock>)>,
    monsters: Query<&Transform, With<Monster>>,
    caught: Query<(), (With<PlayerController>, With<Caught>)>,
    mut toggles: MessageWriter<ToggleDoor>,
) {
    if !caught.is_empty() || enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (entity, door, locked) in &doors {
        if !locked
            && door.state == DoorState::Closed
            && monsters
                .iter()
                .any(|monster| monster.translation.xz().distance(door.position) < DOOR_APPROACH)
        {
            toggles.write(ToggleDoor(entity));
        }
    }
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
    mut monsters: Query<(Entity, &mut Monster, &mut Transform), Without<PlayerController>>,
    caught: Query<(), (With<PlayerController>, With<Caught>)>,
    mut sounds: MessageWriter<PlaySourceSound>,
) {
    if !caught.is_empty() || enabled.is_some_and(|enabled| !enabled.0) {
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
    let mut dynamic = blockers.clone();
    for (_, door, swing, locked) in &doors {
        if !locked {
            dynamic.push(collision::door_panel(door, swing));
        }
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
        if let Some(mut pursuit) = monster.pursuit {
            if !pursuit.searching
                && pursuit
                    .planned
                    .is_none_or(|planned| planned.distance(pursuit.last_sensed) > GRID)
            {
                pursuit.planned = Some(pursuit.last_sensed);
                monster.route = route(position, pursuit.last_sensed, &rooms_data, &blockers)
                    .unwrap_or_default();
                monster.stalls = 0;
            }
            if monster.route.is_empty() && !pursuit.searching && pursuit.remaining >= SEARCH_TIME {
                let toward = (pursuit.last_sensed - position).normalize_or_zero();
                if toward != Vec2::ZERO {
                    turn_toward(&mut transform, toward, delta, CHASE_TURN_SPEED);
                }
                monster.pursuit = Some(pursuit);
                continue;
            }
            if monster.route.is_empty() {
                pursuit.searching = true;
                if monster.route_retry > 0.0 {
                    monster.route_retry -= delta;
                } else {
                    monster.route = search_route(
                        position,
                        pursuit.last_sensed,
                        &mut rng.0,
                        &rooms_data,
                        &blockers,
                    );
                    if monster.route.is_empty() {
                        monster.route_retry = SEARCH_RETRY;
                    }
                }
            }
            monster.pursuit = Some(pursuit);
            if monster.route.is_empty() {
                continue;
            }
        }
        if monster.route.is_empty() && monster.route_retry > 0.0 {
            monster.route_retry -= delta;
            continue;
        }
        if monster.route.is_empty() {
            if let Some(room) = room_at(position, &rooms_data) {
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
                    let (_, bounds) = neighbors[pick];
                    let center = (bounds.min + bounds.max) * 0.5;
                    if let Some(path) = route(position, center, &rooms_data, &blockers) {
                        monster.route = path;
                    } else {
                        monster.route_retry = 1.0;
                    }
                }
            }
        }
        while monster
            .route
            .front()
            .is_some_and(|next| position.distance(*next) < ARRIVAL)
        {
            monster.route.pop_front();
        }
        while monster.route.len() > 1 && clear_path(position, monster.route[1], &dynamic) {
            monster.route.pop_front();
        }
        let Some(next) = monster.route.front().copied() else {
            continue;
        };
        let forward = (next - position).normalize_or_zero();
        let chasing = monster.pursuit.is_some();
        let facing_waypoint = turn_toward(
            &mut transform,
            forward,
            delta,
            if chasing {
                CHASE_TURN_SPEED
            } else {
                TURN_SPEED
            },
        );
        if doors.iter().any(|(_, door, swing, locked)| {
            !locked
                && position.distance(door.position) < DOOR_APPROACH
                && (door.state == DoorState::Closed || swing.0 < 1.4)
        }) {
            continue;
        }
        if !facing_waypoint {
            continue;
        }
        let speed = if monster.pursuit.is_some() {
            CHASE_SPEED
        } else {
            SPEED
        };
        let step = forward * (speed * delta).min(position.distance(next));
        let moved = collision::move_player(position, step, &dynamic);
        if moved.distance(position) < step.length() * 0.15 {
            monster.stalls += 1;
            let goal = monster.route.back().copied();
            monster.route = goal
                .filter(|_| monster.stalls < STALL_LIMIT)
                .and_then(|goal| route(position, goal, &rooms_data, &dynamic))
                .unwrap_or_default();
            if monster.route.is_empty() {
                monster.stalls = 0;
                match monster.pursuit.as_mut() {
                    Some(pursuit) => pursuit.searching = true,
                    None => monster.route_retry = 1.0,
                }
            }
            continue;
        }
        monster.stalls = 0;
        transform.translation.x = moved.x;
        transform.translation.z = moved.y;
        monster.step_clock += delta;
        let step_period = if monster.pursuit.is_some() {
            CHASE_STEP_PERIOD
        } else {
            STEP_PERIOD
        };
        if monster.step_clock >= step_period {
            monster.step_clock -= step_period;
            sounds.write(PlaySourceSound {
                source: entity,
                sound: Sound::MonsterStep(monster.step_index),
            });
            monster.step_index = (monster.step_index + 1) % 6;
        }
    }
}
