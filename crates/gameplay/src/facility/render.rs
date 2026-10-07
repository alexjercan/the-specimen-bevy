use bevy::{
    ecs::system::SystemParam,
    gltf::GltfMaterialName,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use game_assets::{FacilityAssets, GameAssetsState};

use super::{
    layout::{door_panel, Door, Glow, Part},
    Boundary, DoorState, Doorway, Facility, FacilityLayout, FacilityPart, Room,
};

pub const GLOWING_MATERIALS: [&str; 4] = ["lamp_cool", "lamp_red", "lamp_fire", "specimen_fluid"];

pub struct FacilityRenderPlugin;

impl Plugin for FacilityRenderPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Facility, Visibility>()
            .register_required_components::<Room, Visibility>()
            .register_required_components::<FacilityPart, Visibility>()
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb(0.55, 0.6, 0.7),
                brightness: 20.0,
                ..default()
            })
            .add_observer(render_added_part)
            .add_observer(attach_glow_surfaces)
            .add_systems(OnEnter(GameAssetsState::Ready), render_pending_parts)
            .add_systems(
                Update,
                (sync_door_panels, animate_glow).run_if(in_state(GameAssetsState::Ready)),
            );
    }

    fn finish(&self, app: &mut App) {
        assert!(
            app.world().contains_resource::<State<GameAssetsState>>(),
            "FacilityRenderPlugin needs GameAssetsState; add GameAssetsPlugin"
        );
    }
}

#[derive(Component, Debug)]
pub struct Rendered;

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

#[derive(SystemParam)]
struct PartRenderer<'w, 's> {
    assets: Option<Res<'w, FacilityAssets>>,
    parts: Query<'w, 's, (&'static FacilityPart, Option<&'static DoorState>), Without<Rendered>>,
    doorways: Query<'w, 's, (&'static Boundary, &'static Doorway)>,
    parents: Query<'w, 's, &'static ChildOf>,
    layouts: Query<'w, 's, &'static FacilityLayout>,
}

impl PartRenderer<'_, '_> {
    fn render(&self, commands: &mut Commands, entity: Entity) {
        let Some(assets) = &self.assets else {
            return;
        };
        let Ok((FacilityPart(owner), state)) = self.parts.get(entity) else {
            return;
        };
        let Some(facility) = self
            .parents
            .iter_ancestors(entity)
            .find_map(|ancestor| self.layouts.get(ancestor).ok())
        else {
            warn!("facility part {entity} has no facility layout ancestor");
            return;
        };
        let to_local = facility.anchor(*owner).compute_affine().inverse();
        let local = |transform: Transform| {
            Transform::from_matrix((to_local * transform.compute_affine()).into())
        };
        let door = self.doorways.get(entity).ok().zip(state);

        commands.entity(entity).insert(Rendered);
        for piece in facility.pieces(*owner) {
            let Some(scene) = assets.module(piece.module) else {
                error!("facility module {} is not loaded", piece.module);
                continue;
            };
            let transform = match (piece.part, door) {
                (Part::Door, Some((doorway, state))) => {
                    let (translation, yaw) = panel(doorway, *state);
                    Transform::from_translation(translation)
                        .with_rotation(Quat::from_rotation_y(yaw))
                }
                _ => Transform::from_translation(piece.translation)
                    .with_rotation(piece.rotation())
                    .with_scale(piece.scale),
            };
            let mut child = commands.spawn((
                Name::new(piece.module),
                WorldAssetRoot(scene.clone()),
                local(transform),
                ChildOf(entity),
            ));
            if piece.part == Part::Door {
                child.insert(DoorPanel);
            }
            if matches!(piece.part, Part::Ceiling | Part::Light) {
                child.insert(Overhead);
            }
            if let Some(glow @ (Glow::Flicker(_) | Glow::Pulse)) = piece.glow {
                child.insert(Glowing(glow));
            }
        }
        for light in facility.lights(*owner) {
            let mut child = commands.spawn((
                PointLight {
                    color: Color::linear_rgb(light.color[0], light.color[1], light.color[2]),
                    intensity: light.intensity,
                    range: light.range,
                    radius: 0.08,
                    shadow_maps_enabled: light.shadows,
                    ..default()
                },
                local(Transform::from_translation(light.position)),
                ChildOf(entity),
            ));
            if matches!(light.glow, Glow::Flicker(_) | Glow::Pulse) {
                child.insert(GlowLight {
                    glow: light.glow,
                    base: light.intensity,
                });
            }
        }
    }
}

fn panel((boundary, doorway): (&Boundary, &Doorway), state: DoorState) -> (Vec3, f32) {
    let door = match state {
        DoorState::Open => Door::Open(doorway.swing),
        DoorState::Closed | DoorState::Locked => Door::Closed,
    };
    door_panel(boundary.key, boundary.facing, door)
}

fn render_added_part(
    add: On<Add, FacilityPart>,
    state: Res<State<GameAssetsState>>,
    renderer: PartRenderer,
    mut commands: Commands,
) {
    if *state.get() == GameAssetsState::Ready {
        renderer.render(&mut commands, add.entity);
    }
}

fn render_pending_parts(
    pending: Query<Entity, (With<FacilityPart>, Without<Rendered>)>,
    renderer: PartRenderer,
    mut commands: Commands,
) {
    for entity in &pending {
        renderer.render(&mut commands, entity);
    }
}

fn sync_door_panels(
    doors: Query<(&Boundary, &Doorway, &DoorState, &Transform, &Children), Changed<DoorState>>,
    mut panels: Query<&mut Transform, (With<DoorPanel>, Without<Boundary>)>,
) {
    for (boundary, doorway, state, anchor, children) in &doors {
        let (translation, yaw) = panel((boundary, doorway), *state);
        let world = Transform::from_translation(translation)
            .with_rotation(Quat::from_rotation_y(yaw))
            .compute_affine();
        let local = Transform::from_matrix((anchor.compute_affine().inverse() * world).into());
        let mut iter = panels.iter_many_mut(children);
        while let Some(mut transform) = iter.fetch_next() {
            *transform = local;
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

pub fn glow_factor(glow: Glow, t: f32) -> f32 {
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
