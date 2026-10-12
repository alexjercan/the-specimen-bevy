use bevy::{
    gltf::GltfMeshName,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    world_serialization::WorldAssetRoot,
};
use gameplay::monster::MONSTER_HEAD_JOINT;

pub const MONSTER_EYES_MESH: &str = "Eyes_low_Deer_head1_0";
pub const EYE_COLOR: Color = Color::srgb(1.0, 0.42, 0.04);
const EYE_GLOW: f32 = 6.0;
const HALO_STRENGTH: f32 = 0.9;
const HALO_RADIUS: f32 = 100.0;
const EYE_SOCKETS: [Vec3; 2] = [
    Vec3::new(145.9, 518.0, -376.0),
    Vec3::new(148.7, 520.4, 309.3),
];

#[derive(Component)]
pub struct HalloweenEyes;

#[derive(Component)]
pub struct EyeHalo;

#[derive(Resource)]
struct EyeLook {
    eyes: Handle<StandardMaterial>,
    halo: Handle<StandardMaterial>,
    halo_mesh: Handle<Mesh>,
}

pub struct MonsterEyesPlugin;

impl Plugin for MonsterEyesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_eye_look)
            .add_systems(PostUpdate, light_eyes);
    }
}

fn prepare_eye_look(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.insert_resource(EyeLook {
        eyes: materials.add(StandardMaterial {
            base_color: EYE_COLOR,
            emissive: LinearRgba::from(EYE_COLOR) * EYE_GLOW,
            perceptual_roughness: 0.3,
            ..default()
        }),
        halo: materials.add(StandardMaterial {
            base_color: Color::from(LinearRgba::from(EYE_COLOR) * HALO_STRENGTH),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..default()
        }),
        halo_mesh: meshes.add(Sphere::new(HALO_RADIUS).mesh().uv(16, 8)),
    });
}

fn light_eyes(
    added: Query<(Entity, &GltfMeshName), Added<GltfMeshName>>,
    parents: Query<&ChildOf>,
    roots: Query<(), With<WorldAssetRoot>>,
    children: Query<&Children>,
    names: Query<&Name>,
    look: Res<EyeLook>,
    mut commands: Commands,
) {
    for (entity, name) in &added {
        if name.0 != MONSTER_EYES_MESH {
            continue;
        }
        commands
            .entity(entity)
            .insert((HalloweenEyes, MeshMaterial3d(look.eyes.clone())));
        let Some(root) = parents
            .iter_ancestors(entity)
            .find(|parent| roots.contains(*parent))
        else {
            continue;
        };
        let Some(head) = children.iter_descendants(root).find(|joint| {
            names
                .get(*joint)
                .is_ok_and(|name| name.as_str() == MONSTER_HEAD_JOINT)
        }) else {
            continue;
        };
        for socket in EYE_SOCKETS {
            commands.spawn((
                Name::new("halloween eye halo"),
                EyeHalo,
                Mesh3d(look.halo_mesh.clone()),
                MeshMaterial3d(look.halo.clone()),
                Transform::from_translation(socket),
                NotShadowCaster,
                NotShadowReceiver,
                ChildOf(head),
            ));
        }
    }
}
