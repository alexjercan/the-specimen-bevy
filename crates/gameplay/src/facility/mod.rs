pub mod grid;
pub mod level;

use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use game_assets::{FacilityAssets, GameAssetsState};

use grid::{cell_center, half_point, DOOR_W, FRAME_DEPTH};
use level::{EdgeKind, Glow, Level};

const GLOWING_MATERIALS: [&str; 4] = ["lamp_cool", "lamp_red", "lamp_fire", "specimen_fluid"];
const CEILING: &str = "ceiling_tile";
const DOORWAY: &str = "wall_doorway";
const DOOR_PANEL: &str = "door_panel";
const WALL_POST: &str = "wall_post";

pub struct FacilityPlugin {
    level: Level,
}

impl FacilityPlugin {
    pub fn new(level: Level) -> Self {
        Self { level }
    }
}

impl Plugin for FacilityPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let level = self.level.clone();
        if !app.world().contains_resource::<State<GameAssetsState>>() {
            app.add_systems(Startup, move |mut commands: Commands| {
                spawn_level(&mut commands, &level, None);
            });
            return;
        }
        let fallback = level.clone();
        app.insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.6, 0.7),
            brightness: 20.0,
            ..default()
        })
        .add_observer(attach_glow_surfaces)
        .add_systems(
            OnEnter(GameAssetsState::Ready),
            move |mut commands: Commands, assets: Res<FacilityAssets>| {
                spawn_level(&mut commands, &level, Some(&assets));
            },
        )
        .add_systems(
            OnEnter(GameAssetsState::Failed),
            move |mut commands: Commands| {
                error!("facility assets failed; building the level without visuals");
                spawn_level(&mut commands, &fallback, None);
            },
        )
        .add_systems(
            Update,
            (sync_door_panels, animate_glow).run_if(in_state(GameAssetsState::Ready)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Facility;

#[derive(Component, Debug)]
pub struct Room {
    pub name: &'static str,
    pub min: IVec2,
    pub max: IVec2,
}

#[derive(Component, Debug)]
pub struct FloorCell {
    pub cell: IVec2,
}

#[derive(Component, Debug)]
pub struct Boundary {
    pub key: IVec2,
    pub facing: grid::Side,
}

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct Passage;

#[derive(Component, Debug)]
pub struct Doorway {
    pub swing: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorState {
    Open,
    Closed,
    Locked,
}

impl DoorState {
    pub fn passable(self) -> bool {
        self == DoorState::Open
    }
}

#[derive(Component, Debug)]
pub struct WallPost;

#[derive(Component, Debug)]
pub struct Prop;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Objective {
    Exit,
    FusePanel,
}

#[derive(Component, Debug)]
pub struct PlayerStart;

#[derive(Component, Debug)]
pub struct Overhead;

#[derive(Component, Debug)]
pub struct DoorPanel;

#[derive(Component, Debug)]
pub struct Glowing(pub Glow);

#[derive(Component, Debug)]
pub struct GlowSurface {
    pub glow: Glow,
    pub base: LinearRgba,
}

#[derive(Component, Debug)]
pub struct GlowLight {
    pub glow: Glow,
    pub base: f32,
}

pub fn door_panel(state: DoorState, swing: f32) -> Transform {
    let hinge = Vec3::X * -(DOOR_W / 2.0 - 0.01);
    if !state.passable() {
        return Transform::from_translation(hinge);
    }
    Transform::from_translation(hinge + Vec3::NEG_Z * (FRAME_DEPTH + 0.035))
        .with_rotation(Quat::from_rotation_y(swing.to_radians()))
}

struct Build<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    assets: Option<&'a FacilityAssets>,
}

impl Build<'_, '_, '_> {
    fn node(&mut self, bundle: impl Bundle) -> Entity {
        let mut entity = self.commands.spawn(bundle);
        if self.assets.is_some() {
            entity.insert(Visibility::default());
        }
        entity.id()
    }

    fn scene(
        &mut self,
        parent: Entity,
        scene: &'static str,
        transform: Transform,
    ) -> Option<Entity> {
        let assets = self.assets?;
        let Some(handle) = assets.module(scene) else {
            error!("facility module {scene} is not loaded");
            return None;
        };
        let entity = self
            .commands
            .spawn((
                Name::new(scene),
                WorldAssetRoot(handle.clone()),
                transform,
                ChildOf(parent),
            ))
            .id();
        Some(entity)
    }

    fn mark(&mut self, entity: Option<Entity>, bundle: impl Bundle) {
        if let Some(entity) = entity {
            self.commands.entity(entity).insert(bundle);
        }
    }
}

pub fn spawn_level(
    commands: &mut Commands,
    level: &Level,
    assets: Option<&FacilityAssets>,
) -> Entity {
    let mut build = Build { commands, assets };
    let cells = level.cells();
    let edges = level.edges(&cells);
    let root = build.node((Facility, Name::new("facility"), Transform::default()));
    let rooms: Vec<_> = level
        .rooms
        .iter()
        .map(|area| {
            build.node((
                Room {
                    name: area.name,
                    min: area.min,
                    max: area.max,
                },
                Name::new(area.name),
                Transform::default(),
                ChildOf(root),
            ))
        })
        .collect();
    let room_of = |cell: IVec2| rooms[cells[&cell]];

    for (&cell, &area) in &cells {
        let entity = build.node((
            FloorCell { cell },
            Transform::from_translation(cell_center(cell)),
            ChildOf(rooms[area]),
        ));
        build.scene(entity, level.rooms[area].floor, Transform::IDENTITY);
        let ceiling = build.scene(entity, CEILING, Transform::IDENTITY);
        build.mark(ceiling, Overhead);
    }

    for (&key, edge) in &edges {
        let entity = build.node((
            Boundary {
                key,
                facing: edge.facing,
            },
            Transform::from_translation(half_point(key))
                .with_rotation(Quat::from_rotation_y(edge.facing.yaw())),
            ChildOf(root),
        ));
        match edge.kind {
            EdgeKind::Wall(scene) => {
                build.commands.entity(entity).insert(Wall);
                build.scene(entity, scene, Transform::IDENTITY);
            }
            EdgeKind::Passage => {
                build.commands.entity(entity).insert(Passage);
            }
            EdgeKind::Door(door) => {
                let mut doorway = build.commands.entity(entity);
                doorway.insert((Doorway { swing: door.swing }, door.state));
                if let Some(objective) = door.objective {
                    doorway.insert(objective);
                }
                build.scene(entity, DOORWAY, Transform::IDENTITY);
                let panel = build.scene(entity, DOOR_PANEL, door_panel(door.state, door.swing));
                build.mark(panel, DoorPanel);
            }
        }
    }

    for post in Level::posts(&edges) {
        let entity = build.node((
            WallPost,
            Transform::from_translation(half_point(post)),
            ChildOf(root),
        ));
        build.scene(entity, WALL_POST, Transform::IDENTITY);
    }

    for placement in &level.placements {
        let name = placement
            .pieces
            .first()
            .map_or("placement", |piece| piece.scene);
        let entity = build.node((
            Prop,
            Name::new(name),
            placement.transform,
            ChildOf(room_of(placement.cell)),
        ));
        if let Some(objective) = placement.objective {
            build.commands.entity(entity).insert(objective);
        }
        let animated = placement.glow != Glow::Steady;
        for piece in &placement.pieces {
            let child = build.scene(entity, piece.scene, piece.transform);
            if placement.overhead {
                build.mark(child, Overhead);
            }
            if animated {
                build.mark(child, Glowing(placement.glow));
            }
        }
        if build.assets.is_none() {
            continue;
        }
        for light in &placement.lights {
            let mut child = build.commands.spawn((
                PointLight {
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    radius: 0.08,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(light.position),
                ChildOf(entity),
            ));
            if animated {
                child.insert(GlowLight {
                    glow: placement.glow,
                    base: light.intensity,
                });
            }
        }
    }

    if let Some((cell, facing)) = level.start {
        build.commands.spawn((
            PlayerStart,
            Name::new("player_start"),
            Transform::from_translation(cell_center(cell))
                .with_rotation(Quat::from_rotation_y(facing.yaw())),
            ChildOf(room_of(cell)),
        ));
    }
    root
}

fn sync_door_panels(
    doors: Query<(&Doorway, &DoorState, &Children), Changed<DoorState>>,
    mut panels: Query<&mut Transform, With<DoorPanel>>,
) {
    for (doorway, state, children) in &doors {
        let mut iter = panels.iter_many_mut(children);
        while let Some(mut transform) = iter.fetch_next() {
            *transform = door_panel(*state, doorway.swing);
        }
    }
}

fn attach_glow_surfaces(
    ready: On<WorldInstanceReady>,
    roots: Query<&Glowing>,
    children: Query<&Children>,
    surfaces: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(Glowing(glow)) = roots.get(ready.entity) else {
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
            GlowSurface { glow: *glow, base },
        ));
    }
}

fn glow_factor(glow: Glow, t: f32) -> f32 {
    match glow {
        Glow::Steady => 1.0,
        Glow::Flicker(phase) => {
            let s = (t * 1.3 + phase).sin() + 0.6 * (t * 4.1 + phase * 2.0).sin();
            if s > 1.1 {
                0.06
            } else if s > 0.9 {
                0.5
            } else {
                1.0
            }
        }
        Glow::Pulse => 0.55 + 0.45 * (t * 2.0).sin(),
    }
}

fn animate_glow(
    time: Res<Time>,
    mut lights: Query<(&GlowLight, &mut PointLight)>,
    surfaces: Query<(&GlowSurface, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for (glow, mut light) in &mut lights {
        light.intensity = glow.base * glow_factor(glow.glow, t);
    }
    for (surface, material) in &surfaces {
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.emissive = surface.base * glow_factor(surface.glow, t);
        }
    }
}
