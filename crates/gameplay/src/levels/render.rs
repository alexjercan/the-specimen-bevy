use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use game_assets::FacilityAssets;

use super::builder::{
    Ceiling, Door, DoorOf, DoorRef, Doors, Floor, LightEffect, LightIntensity, Passage, Prop, Room,
    Walls,
};

const TILE: f32 = 2.5;
const GLOWING_MATERIALS: [&str; 4] = ["lamp_cool", "lamp_red", "lamp_fire", "specimen_fluid"];

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
struct GlowSurface {
    effect: LightEffect,
    base: LinearRgba,
}

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
            .add_observer(attach_prop_glow)
            .add_systems(PostUpdate, (render_rooms, render_doors, render_props))
            .add_systems(Update, (animate_lights, animate_surfaces));
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
    roots: Query<&LightEffect>,
    children: Query<&Children>,
    surfaces: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(&effect) = roots.get(ready.entity) else {
        return;
    };
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
            GlowSurface { effect, base },
        ));
    }
}

fn animate_lights(
    time: Res<Time>,
    mut lights: Query<(&LightEffect, &LightIntensity, &mut PointLight)>,
) {
    for (effect, base, mut light) in &mut lights {
        light.intensity = base.0 * effect.factor(time.elapsed_secs());
    }
}

fn animate_surfaces(
    time: Res<Time>,
    surfaces: Query<(&GlowSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (surface, material) in &surfaces {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = surface.base * surface.effect.factor(time.elapsed_secs());
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

fn render_doors(
    doors: Query<(Entity, &Door), With<PendingDoorRender>>,
    assets: Option<Res<FacilityAssets>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, door) in &doors {
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
            WorldAssetRoot(panel.clone()),
            Transform::from_xyz(-0.59, 0.0, 0.0),
            ChildOf(entity),
        ));
        commands.entity(entity).remove::<PendingDoorRender>();
    }
}
