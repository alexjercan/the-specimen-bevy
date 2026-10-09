use std::f32::consts::FRAC_PI_4;

use bevy::{
    app::{App, Last},
    ecs::system::SystemState,
    prelude::*,
};
use game_audio::{AudioPaused, PlaySound, PlaySourceSound, Sound};
use gameplay::{
    controller::{door_frames, door_panel, wall_obstacles, Flashlight, PlayerController},
    levels::{
        Door, DoorOf, DoorRef, DoorState, DoorSwing, Doors, ExitDoor, FacilityPower, FusePanel,
        FusePickup, HidingSpot, Monster, Passage, Prop, PropCollider, Room, FUSE_TABLES,
    },
};
use serde::Serialize;

use crate::TransportTimeline;

const VIEW_HALF_ANGLE: f32 = FRAC_PI_4;
const VIEW_RANGE: f32 = 15.0;
const FLASHLIGHT_HALF_ANGLE: f32 = 0.47;
const FLASHLIGHT_RANGE: f32 = 10.0;
const DARK_RANGE: f32 = 2.0;
const OCCLUSION_MARGIN: f32 = 0.2;
const DOOR_HEIGHT: f32 = 1.1;
const FUSE_HEIGHT: f32 = 0.05;
const PROP_HEIGHT: f32 = 0.5;

type Block = (Vec2, Vec2, f32);

#[derive(Resource, Default)]
pub(crate) struct Perception {
    map_sent: bool,
    heard: Vec<Noise>,
}

struct Noise {
    tick: u64,
    sound: &'static str,
    source: Option<Entity>,
    position: Option<Vec3>,
}

pub(crate) fn build(app: &mut App) {
    app.init_resource::<Perception>()
        .add_message::<PlaySound>()
        .add_message::<PlaySourceSound>()
        .add_systems(Last, listen);
}

#[derive(Serialize)]
pub(crate) struct Map {
    rooms: Vec<MapRoom>,
    walls: Vec<MapBlock>,
    doors: Vec<MapDoor>,
    passages: Vec<MapPassage>,
    props: Vec<MapProp>,
    fuse_panels: Vec<MapPanel>,
    fuse_candidates: Vec<MapCandidate>,
}

#[derive(Serialize)]
struct MapRoom {
    id: u64,
    name: String,
    bounds: [f32; 4],
}

#[derive(Serialize)]
struct MapBlock {
    center: [f32; 2],
    half: [f32; 2],
    yaw: f32,
}

#[derive(Serialize)]
struct MapDoor {
    id: u64,
    name: String,
    position: [f32; 2],
    yaw: f32,
    rooms: Vec<u64>,
    exit: bool,
}

#[derive(Serialize)]
struct MapPassage {
    id: u64,
    name: String,
    position: [f32; 2],
    rooms: Vec<u64>,
}

#[derive(Serialize)]
struct MapProp {
    id: u64,
    name: String,
    module: String,
    position: [f32; 3],
    yaw: f32,
    hiding: Option<&'static str>,
    footprint: Option<MapBlock>,
}

#[derive(Serialize)]
struct MapPanel {
    id: u64,
    position: [f32; 3],
}

#[derive(Serialize)]
struct MapCandidate {
    candidate: usize,
    room: &'static str,
    position: [f32; 3],
}

#[derive(Serialize)]
pub(crate) struct Seen {
    #[serde(flatten)]
    thing: Thing,
    distance_m: f32,
    bearing_deg: [f32; 2],
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Thing {
    Door { id: u64, open: bool },
    FuseCandidate { candidate: usize, has_fuse: bool },
    FusePanel { id: u64, installed: usize },
    Prop { id: u64, module: String },
    Monster { id: u64 },
}

#[derive(Serialize)]
pub(crate) struct Heard {
    tick: u64,
    sound: &'static str,
    source: Option<u64>,
    distance_m: Option<f32>,
    bearing_deg: Option<[f32; 2]>,
}

pub(crate) struct View {
    pub(crate) map: Option<Map>,
    pub(crate) power_on: Option<bool>,
    pub(crate) visible: Vec<Seen>,
    pub(crate) heard: Vec<Heard>,
}

type Structure<'w, 's> = (
    Query<'w, 's, (&'static Room, Option<&'static Doors>)>,
    Query<'w, 's, (&'static DoorRef, &'static DoorOf)>,
    Query<'w, 's, (&'static Door, &'static DoorSwing)>,
    Query<'w, 's, &'static Passage>,
);

pub(crate) fn view(world: &mut World) -> View {
    let mut perception = world.remove_resource::<Perception>().unwrap_or_default();
    let map = (!perception.map_sent).then(|| map(world));
    perception.map_sent = true;
    let eye = world
        .query_filtered::<(&Transform, Option<&Flashlight>), With<PlayerController>>()
        .iter(world)
        .next()
        .map(|(pose, flashlight)| (*pose, flashlight.is_some_and(|light| light.on)));
    let power_on = world.get_resource::<FacilityPower>().map(|power| power.on);
    let visible = eye
        .map(|(pose, flashlight)| visible(world, &pose, flashlight, power_on.unwrap_or(true)))
        .unwrap_or_default();
    let heard = std::mem::take(&mut perception.heard)
        .into_iter()
        .map(|noise| {
            let relative = noise
                .position
                .zip(eye)
                .map(|(position, (pose, _))| relative(&pose, position));
            Heard {
                tick: noise.tick,
                sound: noise.sound,
                source: noise.source.map(Entity::to_bits),
                distance_m: relative.map(|(distance, _)| distance),
                bearing_deg: relative.map(|(_, bearing)| bearing),
            }
        })
        .collect();
    world.insert_resource(perception);
    View {
        map,
        power_on,
        visible,
        heard,
    }
}

fn map(world: &mut World) -> Map {
    let mut structure = SystemState::<Structure>::new(world);
    let (rooms, links, doors, passages) =
        structure.get(world).expect("structure queries are valid");
    let mut walls: Vec<MapBlock> = wall_obstacles(&rooms, &links, &doors, &passages)
        .into_iter()
        .map(block)
        .collect();
    walls.extend(
        doors
            .iter()
            .flat_map(|(door, _)| door_frames(door))
            .map(block),
    );

    let room_links: Vec<(Entity, Entity)> = world
        .query::<(Entity, &Doors)>()
        .iter(world)
        .flat_map(|(room, doors)| doors.iter().map(move |link| (room, link)))
        .collect();
    let opening_rooms = |world: &World, opening: Entity| -> Vec<u64> {
        room_links
            .iter()
            .filter(|(_, link)| {
                world
                    .get::<DoorRef>(*link)
                    .is_some_and(|reference| reference.0 == opening)
            })
            .map(|(room, _)| room.to_bits())
            .collect()
    };

    let mut rooms: Vec<MapRoom> = world
        .query::<(Entity, &Room, Option<&Name>)>()
        .iter(world)
        .map(|(entity, room, name)| MapRoom {
            id: entity.to_bits(),
            name: label(name),
            bounds: [room.0.min.x, room.0.min.y, room.0.max.x, room.0.max.y],
        })
        .collect();
    rooms.sort_by_key(|room| room.id);

    let mut map_doors: Vec<MapDoor> = world
        .query::<(Entity, &Door, Option<&Name>, Has<ExitDoor>)>()
        .iter(world)
        .map(|(entity, door, name, exit)| MapDoor {
            id: entity.to_bits(),
            name: label(name),
            position: door.position.to_array(),
            yaw: yaw(door.rotation),
            rooms: opening_rooms(world, entity),
            exit,
        })
        .collect();
    map_doors.sort_by_key(|door| door.id);

    let mut map_passages: Vec<MapPassage> = world
        .query::<(Entity, &Passage, Option<&Name>)>()
        .iter(world)
        .map(|(entity, passage, name)| MapPassage {
            id: entity.to_bits(),
            name: label(name),
            position: passage.0.to_array(),
            rooms: opening_rooms(world, entity),
        })
        .collect();
    map_passages.sort_by_key(|passage| passage.id);

    let mut props: Vec<MapProp> = world
        .query::<(
            Entity,
            &Prop,
            Option<&Name>,
            Option<&HidingSpot>,
            Option<&PropCollider>,
        )>()
        .iter(world)
        .map(|(entity, prop, name, hiding, collider)| {
            let pose = placed(world, entity);
            MapProp {
                id: entity.to_bits(),
                name: label(name),
                module: prop.0.clone(),
                position: pose.translation.to_array(),
                yaw: yaw(pose.rotation),
                hiding: hiding.map(|spot| match spot {
                    HidingSpot::Locker => "locker",
                    HidingSpot::Table => "table",
                }),
                footprint: collider.map(|collider| footprint(&pose, collider)),
            }
        })
        .collect();
    props.sort_by_key(|prop| prop.id);

    let mut fuse_panels: Vec<MapPanel> = world
        .query_filtered::<(Entity, &Transform), With<FusePanel>>()
        .iter(world)
        .map(|(entity, transform)| MapPanel {
            id: entity.to_bits(),
            position: transform.translation.to_array(),
        })
        .collect();
    fuse_panels.sort_by_key(|panel| panel.id);

    let fuse_candidates = FUSE_TABLES
        .iter()
        .enumerate()
        .map(|(candidate, table)| MapCandidate {
            candidate,
            room: table.room,
            position: table.position.to_array(),
        })
        .collect();

    Map {
        rooms,
        walls,
        doors: map_doors,
        passages: map_passages,
        props,
        fuse_panels,
        fuse_candidates,
    }
}

fn visible(world: &mut World, pose: &Transform, flashlight: bool, power_on: bool) -> Vec<Seen> {
    let mut structure = SystemState::<Structure>::new(world);
    let (rooms, links, doors, passages) =
        structure.get(world).expect("structure queries are valid");
    let mut occluders = wall_obstacles(&rooms, &links, &doors, &passages);
    for (door, swing) in &doors {
        occluders.extend(door_frames(door));
        occluders.push(door_panel(door, swing));
    }

    let mut targets: Vec<(Thing, Vec3)> = Vec::new();
    for (entity, door) in world.query::<(Entity, &Door)>().iter(world) {
        targets.push((
            Thing::Door {
                id: entity.to_bits(),
                open: door.state == DoorState::Open,
            },
            Vec3::new(door.position.x, DOOR_HEIGHT, door.position.y),
        ));
    }
    let fuses: Vec<usize> = world
        .query::<&FusePickup>()
        .iter(world)
        .map(|fuse| fuse.slot)
        .collect();
    for (candidate, table) in FUSE_TABLES.iter().enumerate() {
        targets.push((
            Thing::FuseCandidate {
                candidate,
                has_fuse: fuses.contains(&candidate),
            },
            table.position + Vec3::Y * FUSE_HEIGHT,
        ));
    }
    for (entity, panel, transform) in world
        .query::<(Entity, &FusePanel, &Transform)>()
        .iter(world)
    {
        targets.push((
            Thing::FusePanel {
                id: entity.to_bits(),
                installed: panel.installed,
            },
            transform.translation,
        ));
    }
    for (entity, transform) in world
        .query_filtered::<(Entity, &Transform), With<Monster>>()
        .iter(world)
    {
        targets.push((
            Thing::Monster {
                id: entity.to_bits(),
            },
            transform.translation + Vec3::Y * 1.1,
        ));
    }
    for (entity, prop) in world.query::<(Entity, &Prop)>().iter(world) {
        targets.push((
            Thing::Prop {
                id: entity.to_bits(),
                module: prop.0.clone(),
            },
            placed(world, entity).translation + Vec3::Y * PROP_HEIGHT,
        ));
    }

    let mut seen: Vec<Seen> = targets
        .into_iter()
        .filter(|(_, target)| {
            perceives(pose, *target, flashlight, power_on)
                && clear(&occluders, pose.translation.xz(), target.xz())
        })
        .map(|(thing, target)| {
            let (distance_m, bearing_deg) = relative(pose, target);
            Seen {
                thing,
                distance_m,
                bearing_deg,
            }
        })
        .collect();
    seen.sort_by(|a, b| a.distance_m.total_cmp(&b.distance_m));
    seen
}

fn perceives(pose: &Transform, target: Vec3, flashlight: bool, power_on: bool) -> bool {
    let offset = target - pose.translation;
    let distance = offset.length();
    let angle = (pose.rotation * Vec3::NEG_Z).angle_between(offset);
    let within = |half_angle: f32, range: f32| distance <= range && angle <= half_angle;
    if power_on {
        within(VIEW_HALF_ANGLE, VIEW_RANGE)
    } else {
        within(VIEW_HALF_ANGLE, DARK_RANGE)
            || (flashlight && within(FLASHLIGHT_HALF_ANGLE, FLASHLIGHT_RANGE))
    }
}

fn clear(occluders: &[Block], eye: Vec2, target: Vec2) -> bool {
    let delta = target - eye;
    let length = delta.length();
    if length <= OCCLUSION_MARGIN {
        return true;
    }
    let limit = (length - OCCLUSION_MARGIN) / length;
    !occluders
        .iter()
        .any(|&(center, half, angle)| segment_hits(center, half, angle, eye, delta, limit))
}

fn segment_hits(
    center: Vec2,
    half: Vec2,
    angle: f32,
    origin: Vec2,
    delta: Vec2,
    limit: f32,
) -> bool {
    let (s, c) = angle.sin_cos();
    let local = |v: Vec2| Vec2::new(c * v.x - s * v.y, s * v.x + c * v.y);
    let origin = local(origin - center).to_array();
    let delta = local(delta).to_array();
    let half = half.to_array();
    let mut entry: f32 = 0.0;
    let mut exit = limit;
    for axis in 0..2 {
        if delta[axis].abs() < 1e-8 {
            if origin[axis].abs() > half[axis] {
                return false;
            }
            continue;
        }
        let near = (-half[axis] - origin[axis]) / delta[axis];
        let far = (half[axis] - origin[axis]) / delta[axis];
        entry = entry.max(near.min(far));
        exit = exit.min(near.max(far));
        if entry > exit {
            return false;
        }
    }
    true
}

fn relative(pose: &Transform, position: Vec3) -> (f32, [f32; 2]) {
    let offset = position - pose.translation;
    let local = pose.rotation.inverse() * offset;
    let azimuth = local.x.atan2(-local.z).to_degrees();
    let elevation = local.y.atan2(local.xz().length()).to_degrees();
    (offset.length(), [azimuth, elevation])
}

fn placed(world: &World, entity: Entity) -> Transform {
    let mut pose = world.get::<Transform>(entity).copied().unwrap_or_default();
    let mut current = entity;
    while let Some(parent) = world.get::<ChildOf>(current).map(ChildOf::parent) {
        if let Some(transform) = world.get::<Transform>(parent) {
            pose = transform.mul_transform(pose);
        }
        current = parent;
    }
    pose
}

fn footprint(pose: &Transform, collider: &PropCollider) -> MapBlock {
    let scale = pose.scale.xz();
    let offset = pose.rotation
        * Vec3::new(
            collider.center.x * scale.x,
            0.0,
            collider.center.y * scale.y,
        );
    MapBlock {
        center: (pose.translation.xz() + offset.xz()).to_array(),
        half: (collider.half * scale.abs()).to_array(),
        yaw: yaw(pose.rotation),
    }
}

fn block((center, half, yaw): Block) -> MapBlock {
    MapBlock {
        center: center.to_array(),
        half: half.to_array(),
        yaw,
    }
}

fn yaw(rotation: Quat) -> f32 {
    rotation.to_euler(EulerRot::YXZ).0
}

fn label(name: Option<&Name>) -> String {
    name.map(|name| name.as_str().to_owned())
        .unwrap_or_default()
}

fn listen(
    timeline: Res<TransportTimeline>,
    paused: Option<Res<AudioPaused>>,
    players: Query<&Transform, With<PlayerController>>,
    sources: Query<&Transform>,
    mut global: MessageReader<PlaySound>,
    mut sourced: MessageReader<PlaySourceSound>,
    mut perception: ResMut<Perception>,
) {
    let mut noises: Vec<(Sound, Option<Entity>, Option<Vec3>, bool)> = Vec::new();
    noises.extend(
        global
            .read()
            .map(|sound| (sound.sound, None, sound.position, sound.position.is_some())),
    );
    noises.extend(sourced.read().map(|sound| {
        let position = sources
            .get(sound.source)
            .ok()
            .map(|source| source.translation);
        (sound.sound, Some(sound.source), position, true)
    }));
    let (Some(tick), Ok(listener)) = (timeline.updating(), players.single()) else {
        return;
    };
    if paused.is_some_and(|paused| paused.0) {
        return;
    }
    for (sound, source, position, ranged) in noises {
        let Some(name) = sound_name(sound) else {
            continue;
        };
        if ranged
            && position.is_none_or(|position| {
                position.distance(listener.translation) >= sound.audible_range()
            })
        {
            continue;
        }
        perception.heard.push(Noise {
            tick,
            sound: name,
            source,
            position,
        });
    }
}

fn sound_name(sound: Sound) -> Option<&'static str> {
    Some(match sound {
        Sound::DoorUnlatch => "door_unlatch",
        Sound::DoorSwing => "door_swing",
        Sound::DoorShut => "door_shut",
        Sound::DoorLocked => "door_locked",
        Sound::FuseSlot(_) => "fuse_slot",
        Sound::FuseComplete => "fuse_complete",
        Sound::FlashlightClick => "flashlight_click",
        Sound::SprintExhausted => "sprint_exhausted",
        Sound::PowerDown => "power_down",
        Sound::BreakerTrip => "breaker_trip",
        Sound::BoilerReset => "boiler_reset",
        Sound::BoilerRestart => "boiler_restart",
        Sound::BoilerTick => "boiler_tick",
        Sound::FaucetBurst => "faucet_burst",
        Sound::LockerOpen => "locker_open",
        Sound::LockerClose => "locker_close",
        Sound::TableEnter => "table_enter",
        Sound::TableLeave => "table_leave",
        Sound::MonsterPresence => "monster_presence",
        Sound::MonsterStep(_) => "monster_step",
        Sound::MonsterDetected => "monster_detected",
        Sound::MonsterAttack => "monster_attack",
        Sound::MonsterHeartbeat => "monster_heartbeat",
        Sound::Step(_)
        | Sound::UiBack
        | Sound::UiConfirm
        | Sound::UiDenied
        | Sound::UiFocus
        | Sound::UiHover
        | Sound::UiPause
        | Sound::UiResume
        | Sound::UiPress => return None,
    })
}
