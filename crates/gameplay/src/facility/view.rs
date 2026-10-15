use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use game_settings::GameSettings;

use super::{
    animation::DoorSwing,
    assets::FacilityAssets,
    builder::{
        Ceiling, Door, DoorOf, DoorRef, Doors, Floor, LightEffect, LightIntensity, Passage, Prop,
        Room, Walls, TILE,
    },
    doors::{panel_transform, DoorPanel},
    exterior::{PendingExteriorRender, EXIT_CORRIDOR},
    lights_view::PropLightsPlugin,
    module::Module,
    power::{FacilityPower, MainsPowered},
    sounds::FacilitySoundsPlugin,
};

const GLOWING_MATERIALS: [&str; 5] = [
    "lamp_cool",
    "lamp_amber",
    "lamp_red",
    "lamp_fire",
    "specimen_fluid",
];

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
    effect: Option<LightEffect>,
    base: LinearRgba,
    needs_power: bool,
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

// NOTE: app glue fills FacilityAssets once the loader is ready; this plugin never inits it.
pub struct FacilityViewPlugin;

impl Plugin for FacilityViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderCeilings>()
            .add_plugins((PropLightsPlugin, FacilitySoundsPlugin))
            .add_observer(mark_room_for_render)
            .add_observer(mark_door_for_render)
            .add_observer(mark_prop_for_render)
            .add_observer(attach_prop_glow)
            .add_systems(
                PostUpdate,
                (
                    render_rooms,
                    render_exterior_corridor,
                    render_doors,
                    render_props,
                )
                    .run_if(resource_exists::<FacilityAssets>),
            )
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
    assets: Res<FacilityAssets>,
    mut commands: Commands,
) {
    for (entity, prop, effect) in &props {
        let module = assets
            .module(prop.0)
            .unwrap_or_else(|| panic!("missing prop module: {:?}", prop.0));
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
    props: Query<Has<MainsPowered>, With<Prop>>,
    children: Query<&Children>,
    surfaces: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((effect, parent)) = roots.get(ready.entity) else {
        return;
    };
    let Ok(needs_power) = props.get(parent.parent()) else {
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
    power: Res<FacilityPower>,
    settings: Res<GameSettings>,
    mains: Query<(), With<MainsPowered>>,
    mut lights: Query<(
        Option<&LightEffect>,
        &LightIntensity,
        &ChildOf,
        &mut PointLight,
    )>,
) {
    for (effect, base, parent, mut light) in &mut lights {
        let powered = power.is_on() || !mains.contains(parent.parent());
        light.intensity = if powered {
            base.0
                * if settings.reduce_flicker {
                    1.0
                } else {
                    effect.map_or(1.0, |effect| effect.factor(time.elapsed_secs()))
                }
        } else {
            0.0
        };
    }
}

fn animate_surfaces(
    time: Res<Time>,
    power: Res<FacilityPower>,
    settings: Res<GameSettings>,
    surfaces: Query<(&GlowSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (surface, material) in &surfaces {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = surface.base
                * if surface.needs_power && !power.is_on() {
                    0.0
                } else if settings.reduce_flicker {
                    1.0
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
    assets: Res<FacilityAssets>,
    render_ceilings: Res<RenderCeilings>,
    mut commands: Commands,
) {
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
            .module(floor.0)
            .unwrap_or_else(|| panic!("missing floor module: {:?}", floor.0));
        let wall_asset = assets
            .module(walls.0)
            .unwrap_or_else(|| panic!("missing wall module: {:?}", walls.0));
        let ceiling_asset = render_ceilings.0.then(|| {
            assets
                .module(ceiling.0)
                .unwrap_or_else(|| panic!("missing ceiling module: {:?}", ceiling.0))
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
    assets: Res<FacilityAssets>,
    render_ceilings: Res<RenderCeilings>,
    mut commands: Commands,
) {
    let floor = assets
        .module(Module::FloorTile)
        .expect("missing exterior floor");
    let wall = assets.module(Module::Wall).expect("missing exterior wall");
    let ceiling = render_ceilings.0.then(|| {
        assets
            .module(Module::CeilingTile)
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
    assets: Res<FacilityAssets>,
    mut commands: Commands,
) {
    for (entity, door, swing) in &doors {
        let frame = assets
            .module(door.frame)
            .unwrap_or_else(|| panic!("missing door frame module: {:?}", door.frame));
        let panel = assets
            .module(door.panel)
            .unwrap_or_else(|| panic!("missing door panel module: {:?}", door.panel));
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
