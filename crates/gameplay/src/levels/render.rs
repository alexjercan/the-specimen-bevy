use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    render::render_resource::Face,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use game_assets::FacilityAssets;

use crate::controller::{PlayerController, PlayerControlsEnabled};

use super::builder::{
    Ceiling, Door, DoorOf, DoorRef, Doors, Floor, LightEffect, LightIntensity, Passage, Prop, Room,
    Walls,
};
use super::{
    animation::DoorSwing,
    devices::{ThrownFlashbang, FLASHBANG_BURST_DELAY},
    doors::{panel_transform, DoorPanel},
    exterior::{PendingExteriorRender, EXIT_CORRIDOR},
    fuses::{
        FuseInstallation, FusePanel, FUSE_COUNT, FUSE_LENGTH, FUSE_MODULE, FUSE_RADIUS,
        INSERT_SECS_PER_FUSE,
    },
    menu_background::MenuBackground,
    module_names::{BOILER_UNIT, EXIT_SIGN, WALL_LAMP_RED},
    pickups::{PickupKind, PickupMotion},
    power::FacilityPower,
};

const TILE: f32 = 2.5;
const GLOWING_MATERIALS: [&str; 5] = [
    "lamp_cool",
    "lamp_amber",
    "lamp_red",
    "lamp_fire",
    "specimen_fluid",
];
const FUSE_OUTLINE_WIDTH: f32 = 0.008;
const FUSE_OUTLINE: Color = Color::srgb(1.0, 0.72, 0.22);
const FUSE_PLACEHOLDER: Color = Color::srgb(0.78, 0.74, 0.64);
const DEVICE_OUTLINE_WIDTH: f32 = 0.01;
const DEVICE_OUTLINE: Color = Color::srgb(0.55, 0.85, 1.0);
const FLASHBANG_RADIUS: f32 = 0.035;
const FLASHBANG_LENGTH: f32 = 0.12;
const FLASHBANG_BODY: Color = Color::srgb(0.16, 0.24, 0.14);
const DETECTOR_SIZE: Vec3 = Vec3::new(0.16, 0.07, 0.1);
const DETECTOR_BODY: Color = Color::srgb(0.32, 0.33, 0.34);
const DETECTOR_FACE: Color = Color::srgb(1.0, 0.62, 0.18);
const DETECTOR_FACE_SIZE: Vec3 = Vec3::new(0.1, 0.004, 0.06);
const PICKUP_MOTION_SECS: f32 = 0.55;
const PICKUP_ARC_HEIGHT: f32 = 0.15;
const THROW_SECS: f32 = FLASHBANG_BURST_DELAY;
const THROW_DISTANCE: f32 = 3.8;
const THROW_HEIGHT: f32 = 0.9;
const THROW_SPIN_RATE: f32 = 9.0;
const BURST_INTENSITY: f32 = 9_000_000.0;
const BURST_DECAY: f32 = 8.0;
const BURST_LIFETIME: f32 = 0.7;

pub struct LevelRenderPlugin;

#[derive(Resource)]
pub struct RenderCeilings(pub bool);

impl Default for RenderCeilings {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component)]
struct PendingRoomRender;

#[derive(Component)]
struct PendingDoorRender;

#[derive(Component)]
struct PendingPropRender;

#[derive(Component)]
struct PendingPickupRender;

#[derive(Component)]
struct FlashbangBurstLight;

#[derive(Component)]
struct PanelFuseVisual {
    slot: usize,
    from: Vec3,
}

const PANEL_FUSE_SPACING: f32 = 0.16;
const PANEL_FUSE_DEPTH: f32 = -0.13;
const PANEL_FUSE_START_HEIGHT: f32 = -0.15;

#[derive(Component)]
struct GlowSurface {
    effect: Option<LightEffect>,
    base: LinearRgba,
    needs_power: bool,
}

fn needs_mains_power(module: &str) -> bool {
    !matches!(module, WALL_LAMP_RED | EXIT_SIGN | BOILER_UNIT)
}

#[cfg(test)]
#[path = "../../tests/unit/level_render_power.rs"]
mod power_tests;
#[cfg(test)]
#[path = "../../tests/unit/presentation.rs"]
mod presentation_tests;

type PendingRooms<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Room,
        &'static Floor,
        &'static Walls,
        &'static Ceiling,
        Option<&'static Doors>,
    ),
    With<PendingRoomRender>,
>;

impl Plugin for LevelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderCeilings>()
            .add_observer(mark_room_for_render)
            .add_observer(mark_door_for_render)
            .add_observer(mark_prop_for_render)
            .add_observer(mark_pickup_for_render)
            .add_observer(attach_throw_visual)
            .add_observer(attach_panel_fuses)
            .add_observer(attach_prop_glow)
            .add_systems(
                PostUpdate,
                (
                    render_rooms,
                    render_exterior_corridor,
                    render_doors,
                    render_props,
                    render_fuses,
                    render_devices,
                ),
            )
            .add_systems(
                Update,
                (
                    animate_lights,
                    animate_surfaces,
                    animate_pickups,
                    animate_throws,
                    animate_panel_fuses,
                ),
            );
    }
}

fn mark_room_for_render(added: On<Add, Room>, mut commands: Commands) {
    commands.entity(added.entity).insert(PendingRoomRender);
}

fn mark_door_for_render(added: On<Add, Door>, mut commands: Commands) {
    commands.entity(added.entity).insert(PendingDoorRender);
}

fn mark_prop_for_render(added: On<Add, Prop>, mut commands: Commands) {
    commands.entity(added.entity).insert(PendingPropRender);
}

fn mark_pickup_for_render(added: On<Add, PickupKind>, mut commands: Commands) {
    commands.entity(added.entity).insert(PendingPickupRender);
}

fn attach_throw_visual(
    added: On<Add, ThrownFlashbang>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mesh = meshes.add(Cylinder::new(FLASHBANG_RADIUS, FLASHBANG_LENGTH));
    let material = materials.add(StandardMaterial {
        base_color: FLASHBANG_BODY,
        perceptual_roughness: 0.6,
        ..default()
    });
    commands.entity(added.entity).with_children(|children| {
        children.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, FLASHBANG_LENGTH / 2.0, 0.0),
        ));
        children.spawn((
            FlashbangBurstLight,
            PointLight {
                intensity: 0.0,
                range: 8.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::IDENTITY,
        ));
    });
}

fn panel_fuse_destination(slot: usize) -> Vec3 {
    Vec3::new(
        (slot as f32 - 1.0) * PANEL_FUSE_SPACING,
        0.0,
        PANEL_FUSE_DEPTH,
    )
}

fn panel_fuse_position(from: Vec3, slot: usize, elapsed: f32) -> Vec3 {
    let progress =
        ((elapsed - slot as f32 * INSERT_SECS_PER_FUSE) / INSERT_SECS_PER_FUSE).clamp(0.0, 1.0);
    from.lerp(
        panel_fuse_destination(slot),
        progress * progress * (3.0 - 2.0 * progress),
    )
}

fn attach_panel_fuses(
    added: On<Add, FuseInstallation>,
    panels: Query<(&FusePanel, &FuseInstallation, &Transform)>,
    players: Query<&Transform, With<PlayerController>>,
    assets: Option<Res<FacilityAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((panel, installation, transform)) = panels.get(added.entity) else {
        return;
    };
    if panel.installed != 0 {
        return;
    }
    let Ok(player) = players.get(installation.player) else {
        return;
    };
    let hand = player.translation + player.forward() * 0.3 - player.up() * 0.2;
    let from = transform.rotation.inverse() * (hand - transform.translation)
        + Vec3::Y * PANEL_FUSE_START_HEIGHT;
    let module = assets
        .as_ref()
        .and_then(|assets| assets.module(FUSE_MODULE));
    let fallback = module.is_none().then(|| {
        (
            meshes.add(Cylinder::new(FUSE_RADIUS, FUSE_LENGTH)),
            materials.add(StandardMaterial {
                base_color: FUSE_PLACEHOLDER,
                ..default()
            }),
        )
    });
    commands.entity(added.entity).with_children(|children| {
        for slot in 0..FUSE_COUNT {
            let fuse = children
                .spawn((
                    Name::new(format!("panel fuse {}", slot + 1)),
                    PanelFuseVisual { slot, from },
                    Transform::from_translation(from),
                    Visibility::Hidden,
                ))
                .id();
            if let Some(module) = module {
                children.spawn((
                    WorldAssetRoot(module.clone()),
                    Transform::IDENTITY,
                    ChildOf(fuse),
                ));
            } else if let Some((mesh, material)) = &fallback {
                children.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2)),
                    ChildOf(fuse),
                ));
            }
        }
    });
}

fn animate_panel_fuses(
    panels: Query<(Option<&FuseInstallation>, &FusePanel)>,
    mut visuals: Query<(&PanelFuseVisual, &ChildOf, &mut Transform, &mut Visibility)>,
) {
    for (visual, parent, mut transform, mut visibility) in &mut visuals {
        let Ok((installation, panel)) = panels.get(parent.parent()) else {
            continue;
        };
        let elapsed = installation.map_or_else(
            || {
                if panel.installed == FUSE_COUNT {
                    INSERT_SECS_PER_FUSE * FUSE_COUNT as f32
                } else {
                    0.0
                }
            },
            |installation| installation.elapsed,
        );
        if elapsed < visual.slot as f32 * INSERT_SECS_PER_FUSE {
            continue;
        }
        *visibility = Visibility::Visible;
        transform.translation = panel_fuse_position(visual.from, visual.slot, elapsed);
    }
}

fn pickup_position(from: Vec3, hand: Vec3, elapsed: f32) -> Vec3 {
    let u = (elapsed / PICKUP_MOTION_SECS).clamp(0.0, 1.0);
    from.lerp(hand, u) + Vec3::Y * PICKUP_ARC_HEIGHT * (u * PI).sin()
}

fn animate_pickups(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    player: Query<&Transform, (With<PlayerController>, Without<PickupMotion>)>,
    mut pickups: Query<(Entity, &mut Transform, &mut PickupMotion)>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    let Ok(player) = player.single() else { return };
    let hand = player.translation + player.forward() * 0.35 - player.up() * 0.22;
    for (entity, mut transform, mut motion) in &mut pickups {
        motion.elapsed += time.delta_secs();
        if motion.elapsed >= PICKUP_MOTION_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = pickup_position(motion.from, hand, motion.elapsed);
    }
}

fn throw_position(elapsed: f32, landing_y: f32) -> Vec3 {
    let u = (elapsed / THROW_SECS).clamp(0.0, 1.0);
    Vec3::new(0.18, -0.2, -0.35).lerp(Vec3::new(0.0, landing_y, -THROW_DISTANCE), u)
        + Vec3::Y * (THROW_HEIGHT * 4.0 * u * (1.0 - u))
}

fn animate_throws(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut throws: Query<(Entity, &mut ThrownFlashbang, &mut Transform, &Children)>,
    mut lights: Query<&mut PointLight, With<FlashbangBurstLight>>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (entity, mut thrown, mut transform, children) in &mut throws {
        thrown.elapsed += time.delta_secs();
        transform.translation = throw_position(thrown.elapsed, thrown.landing_y);
        transform.rotation = Quat::from_rotation_x(thrown.elapsed * THROW_SPIN_RATE);
        for child in children.iter() {
            if let Ok(mut light) = lights.get_mut(child) {
                let since = thrown.elapsed - THROW_SECS;
                light.intensity = if since >= 0.0 {
                    BURST_INTENSITY * (-BURST_DECAY * since).exp()
                } else {
                    0.0
                };
            }
        }
        if thrown.elapsed >= THROW_SECS + BURST_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}

fn render_devices(
    pickups: Query<(Entity, &PickupKind), With<PendingPickupRender>>,
    assets: Option<Res<FacilityAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let devices: Vec<_> = pickups
        .iter()
        .filter(|(_, kind)| !matches!(kind, PickupKind::Fuse { .. }))
        .collect();
    if assets.is_none() || devices.is_empty() {
        return;
    }
    let outline = materials.add(StandardMaterial {
        base_color: DEVICE_OUTLINE,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    for (entity, device) in devices {
        let parts = match device {
            PickupKind::Fuse { .. } => continue,
            PickupKind::Flashbang => {
                let shape = Transform::from_xyz(0.0, FLASHBANG_LENGTH / 2.0, 0.0);
                vec![
                    (
                        meshes.add(Cylinder::new(FLASHBANG_RADIUS, FLASHBANG_LENGTH)),
                        materials.add(StandardMaterial {
                            base_color: FLASHBANG_BODY,
                            perceptual_roughness: 0.6,
                            ..default()
                        }),
                        shape,
                    ),
                    (
                        meshes.add(Cylinder::new(
                            FLASHBANG_RADIUS + DEVICE_OUTLINE_WIDTH,
                            FLASHBANG_LENGTH + 2.0 * DEVICE_OUTLINE_WIDTH,
                        )),
                        outline.clone(),
                        shape,
                    ),
                ]
            }
            PickupKind::Detector => {
                let shape = Transform::from_xyz(0.0, DETECTOR_SIZE.y / 2.0, 0.0);
                vec![
                    (
                        meshes.add(Cuboid::from_size(DETECTOR_SIZE)),
                        materials.add(StandardMaterial {
                            base_color: DETECTOR_BODY,
                            perceptual_roughness: 0.5,
                            ..default()
                        }),
                        shape,
                    ),
                    (
                        meshes.add(Cuboid::from_size(DETECTOR_FACE_SIZE)),
                        materials.add(StandardMaterial {
                            base_color: DETECTOR_FACE,
                            emissive: LinearRgba::from(DETECTOR_FACE) * 2.0,
                            ..default()
                        }),
                        Transform::from_xyz(0.0, DETECTOR_SIZE.y, 0.0),
                    ),
                    (
                        meshes.add(Cuboid::from_size(
                            DETECTOR_SIZE + Vec3::splat(2.0 * DEVICE_OUTLINE_WIDTH),
                        )),
                        outline.clone(),
                        shape,
                    ),
                ]
            }
        };
        commands.entity(entity).with_children(|children| {
            for (mesh, material, transform) in parts {
                children.spawn((Mesh3d(mesh), MeshMaterial3d(material), transform));
            }
        });
        commands.entity(entity).remove::<PendingPickupRender>();
    }
}

fn render_fuses(
    pickups: Query<(Entity, &PickupKind), With<PendingPickupRender>>,
    assets: Option<Res<FacilityAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    let fuses: Vec<Entity> = pickups
        .iter()
        .filter(|(_, kind)| matches!(kind, PickupKind::Fuse { .. }))
        .map(|(entity, _)| entity)
        .collect();
    if fuses.is_empty() {
        return;
    }
    let module = assets.module(FUSE_MODULE);
    let shape =
        Transform::from_xyz(0.0, FUSE_RADIUS, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2));
    let placeholder = match module {
        Some(_) => None,
        None => {
            warn!("missing {FUSE_MODULE} module; rendering placeholder fuses");
            Some((
                meshes.add(Cylinder::new(FUSE_RADIUS, FUSE_LENGTH)),
                materials.add(StandardMaterial {
                    base_color: FUSE_PLACEHOLDER,
                    perceptual_roughness: 0.55,
                    ..default()
                }),
            ))
        }
    };
    let outline_mesh = meshes.add(Cylinder::new(
        FUSE_RADIUS + FUSE_OUTLINE_WIDTH,
        FUSE_LENGTH + 2.0 * FUSE_OUTLINE_WIDTH,
    ));
    let outline_material = materials.add(StandardMaterial {
        base_color: FUSE_OUTLINE,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    for entity in fuses {
        commands.entity(entity).with_children(|children| {
            if let Some(module) = module {
                children.spawn((WorldAssetRoot(module.clone()), Transform::IDENTITY));
            } else if let Some((mesh, material)) = &placeholder {
                children.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    shape,
                ));
            }
            children.spawn((
                Name::new("fuse outline"),
                Mesh3d(outline_mesh.clone()),
                MeshMaterial3d(outline_material.clone()),
                shape,
            ));
        });
        commands.entity(entity).remove::<PendingPickupRender>();
    }
}

fn render_props(
    props: Query<(Entity, &Prop, Option<&LightEffect>), With<PendingPropRender>>,
    assets: Option<Res<FacilityAssets>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, prop, effect) in &props {
        let module = assets
            .module(&prop.0)
            .unwrap_or_else(|| panic!("missing prop module: {}", prop.0));
        commands
            .entity(entity)
            .insert(Visibility::default())
            .with_children(|children| {
                let mut scene =
                    children.spawn((WorldAssetRoot(module.clone()), Transform::IDENTITY));
                if let Some(effect) = effect {
                    scene.insert(*effect);
                }
            });
        commands.entity(entity).remove::<PendingPropRender>();
    }
}

fn attach_prop_glow(
    ready: On<WorldInstanceReady>,
    roots: Query<(Option<&LightEffect>, &ChildOf)>,
    props: Query<(&Prop, Option<&ChildOf>)>,
    menu_roots: Query<(), With<MenuBackground>>,
    children: Query<&Children>,
    surfaces: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((effect, parent)) = roots.get(ready.entity) else {
        return;
    };
    let Ok((prop, root)) = props.get(parent.parent()) else {
        return;
    };
    let needs_power =
        needs_mains_power(&prop.0) && !root.is_some_and(|root| menu_roots.contains(root.parent()));
    for entity in children.iter_descendants(ready.entity) {
        let Ok((name, material)) = surfaces.get(entity) else {
            continue;
        };
        if !GLOWING_MATERIALS.contains(&name.0.as_str()) {
            continue;
        }
        let Some(source) = materials.get(&material.0).cloned() else {
            continue;
        };
        let base = source.emissive;
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(source)),
            GlowSurface {
                effect: effect.copied(),
                base,
                needs_power,
            },
        ));
    }
}

fn animate_lights(
    time: Res<Time>,
    power: Option<Res<FacilityPower>>,
    props: Query<(&Prop, Option<&ChildOf>)>,
    menu_roots: Query<(), With<MenuBackground>>,
    mut lights: Query<(
        Option<&LightEffect>,
        &LightIntensity,
        &ChildOf,
        &mut PointLight,
    )>,
) {
    for (effect, base, parent, mut light) in &mut lights {
        let powered = power.as_ref().is_none_or(|power| power.on)
            || props.get(parent.parent()).is_ok_and(|(prop, root)| {
                !needs_mains_power(&prop.0)
                    || root.is_some_and(|root| menu_roots.contains(root.parent()))
            });
        light.intensity = if powered {
            base.0 * effect.map_or(1.0, |effect| effect.factor(time.elapsed_secs()))
        } else {
            0.0
        };
    }
}

fn animate_surfaces(
    time: Res<Time>,
    power: Option<Res<FacilityPower>>,
    surfaces: Query<(&GlowSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (surface, material) in &surfaces {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = surface.base
                * if surface.needs_power && power.as_ref().is_some_and(|power| !power.on) {
                    0.0
                } else {
                    surface
                        .effect
                        .map_or(1.0, |effect| effect.factor(time.elapsed_secs()))
                };
        }
    }
}

fn render_rooms(
    rooms: PendingRooms,
    links: Query<(&DoorRef, &DoorOf)>,
    doors: Query<&Door>,
    passages: Query<&Passage>,
    assets: Option<Res<FacilityAssets>>,
    render_ceilings: Res<RenderCeilings>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, room, floor, walls, ceiling, room_doors) in &rooms {
        let openings: Vec<Vec2> = room_doors
            .into_iter()
            .flat_map(|room_doors| room_doors.iter())
            .map(|link| {
                let (door, belongs_to) = links
                    .get(link)
                    .expect("room references a missing door link");
                assert_eq!(belongs_to.0, entity, "door link points to another room");
                if let Ok(door) = doors.get(door.0) {
                    door.position
                } else {
                    passages
                        .get(door.0)
                        .expect("door link points to a missing opening")
                        .0
                }
            })
            .collect();
        let floor_asset = assets
            .module(&floor.0)
            .unwrap_or_else(|| panic!("missing floor module: {}", floor.0));
        let wall_asset = assets
            .module(&walls.0)
            .unwrap_or_else(|| panic!("missing wall module: {}", walls.0));
        let ceiling_asset = render_ceilings.0.then(|| {
            assets
                .module(&ceiling.0)
                .unwrap_or_else(|| panic!("missing ceiling module: {}", ceiling.0))
        });
        let bounds = room.0;
        let width = (bounds.width() / TILE).round() as usize;
        let depth = (bounds.height() / TILE).round() as usize;

        commands
            .entity(entity)
            .insert((Transform::default(), Visibility::default()));

        for x in 0..width {
            for z in 0..depth {
                let transform = Transform::from_xyz(
                    bounds.min.x + (x as f32 + 0.5) * TILE,
                    0.0,
                    bounds.min.y + (z as f32 + 0.5) * TILE,
                );
                commands.spawn((
                    WorldAssetRoot(floor_asset.clone()),
                    transform,
                    ChildOf(entity),
                ));
                if let Some(ceiling_asset) = &ceiling_asset {
                    commands.spawn((
                        WorldAssetRoot((*ceiling_asset).clone()),
                        transform,
                        ChildOf(entity),
                    ));
                }
            }
        }

        for x in 0..width {
            let center = bounds.min.x + (x as f32 + 0.5) * TILE;
            if !openings.contains(&Vec2::new(center, bounds.min.y)) {
                commands.spawn((
                    WorldAssetRoot(wall_asset.clone()),
                    Transform::from_xyz(center, 0.0, bounds.min.y),
                    ChildOf(entity),
                ));
            }
            if !openings.contains(&Vec2::new(center, bounds.max.y)) {
                commands.spawn((
                    WorldAssetRoot(wall_asset.clone()),
                    Transform::from_xyz(center, 0.0, bounds.max.y)
                        .with_rotation(Quat::from_rotation_y(PI)),
                    ChildOf(entity),
                ));
            }
        }
        for z in 0..depth {
            let center = bounds.min.y + (z as f32 + 0.5) * TILE;
            if !openings.contains(&Vec2::new(bounds.min.x, center)) {
                commands.spawn((
                    WorldAssetRoot(wall_asset.clone()),
                    Transform::from_xyz(bounds.min.x, 0.0, center)
                        .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
                    ChildOf(entity),
                ));
            }
            if !openings.contains(&Vec2::new(bounds.max.x, center)) {
                commands.spawn((
                    WorldAssetRoot(wall_asset.clone()),
                    Transform::from_xyz(bounds.max.x, 0.0, center)
                        .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
                    ChildOf(entity),
                ));
            }
        }
        commands.entity(entity).remove::<PendingRoomRender>();
    }
}

fn render_exterior_corridor(
    corridors: Query<Entity, With<PendingExteriorRender>>,
    assets: Option<Res<FacilityAssets>>,
    render_ceilings: Res<RenderCeilings>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    let floor = assets.module("floor_tile").expect("missing exterior floor");
    let wall = assets.module("wall").expect("missing exterior wall");
    let ceiling = render_ceilings.0.then(|| {
        assets
            .module("ceiling_tile")
            .expect("missing exterior ceiling")
    });
    for entity in &corridors {
        for z in 0..3 {
            let center = EXIT_CORRIDOR.max.y - (z as f32 + 0.5) * TILE;
            commands.spawn((
                WorldAssetRoot(floor.clone()),
                Transform::from_xyz(0.0, 0.0, center),
                ChildOf(entity),
            ));
            if let Some(ceiling) = &ceiling {
                commands.spawn((
                    WorldAssetRoot((*ceiling).clone()),
                    Transform::from_xyz(0.0, 0.0, center),
                    ChildOf(entity),
                ));
            }
            for (x, rotation) in [
                (EXIT_CORRIDOR.min.x, FRAC_PI_2),
                (EXIT_CORRIDOR.max.x, -FRAC_PI_2),
            ] {
                commands.spawn((
                    WorldAssetRoot(wall.clone()),
                    Transform::from_xyz(x, 0.0, center)
                        .with_rotation(Quat::from_rotation_y(rotation)),
                    ChildOf(entity),
                ));
            }
        }
        commands.spawn((
            WorldAssetRoot(wall.clone()),
            Transform::from_xyz(0.0, 0.0, EXIT_CORRIDOR.min.y),
            ChildOf(entity),
        ));
        commands.entity(entity).remove::<PendingExteriorRender>();
    }
}

fn render_doors(
    doors: Query<(Entity, &Door, &DoorSwing), With<PendingDoorRender>>,
    assets: Option<Res<FacilityAssets>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, door, swing) in &doors {
        let frame = assets
            .module(&door.frame)
            .unwrap_or_else(|| panic!("missing door frame module: {}", door.frame));
        let panel = assets
            .module(&door.panel)
            .unwrap_or_else(|| panic!("missing door panel module: {}", door.panel));
        commands.entity(entity).insert((
            Transform::from_translation(Vec3::new(door.position.x, 0.0, door.position.y))
                .with_rotation(door.rotation),
            Visibility::default(),
        ));
        commands.spawn((
            WorldAssetRoot(frame.clone()),
            Transform::IDENTITY,
            ChildOf(entity),
        ));
        commands.spawn((
            DoorPanel,
            WorldAssetRoot(panel.clone()),
            panel_transform(swing.0),
            ChildOf(entity),
        ));
        commands.entity(entity).remove::<PendingDoorRender>();
    }
}
