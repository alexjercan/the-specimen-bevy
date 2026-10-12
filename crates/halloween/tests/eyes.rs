use bevy::{gltf::GltfMeshName, prelude::*, world_serialization::WorldAssetRoot};
use game_halloween::{EyeHalo, HalloweenEyes, MonsterEyesPlugin, EYE_COLOR, MONSTER_EYES_MESH};
use gameplay::monster::MONSTER_HEAD_JOINT;

fn spawn_monster(
    app: &mut App,
    eyes_name: &str,
    base: &Handle<StandardMaterial>,
) -> (Entity, Entity) {
    let world = app.world_mut();
    let root = world.spawn(WorldAssetRoot::default()).id();
    let head = world
        .spawn((Name::new(MONSTER_HEAD_JOINT), ChildOf(root)))
        .id();
    let eyes = world
        .spawn((
            GltfMeshName(eyes_name.into()),
            Mesh3d::default(),
            MeshMaterial3d(base.clone()),
            ChildOf(root),
        ))
        .id();
    (head, eyes)
}

#[test]
fn only_monster_eye_meshes_get_the_orange_glow_and_halos() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<StandardMaterial>()
        .init_asset::<Mesh>()
        .add_plugins(MonsterEyesPlugin);
    let base = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    let (head, eyes) = spawn_monster(&mut app, MONSTER_EYES_MESH, &base);
    let (other_head, jaw) = spawn_monster(&mut app, "Jaw_low_Deer_head1_0", &base);
    app.update();
    let world = app.world_mut();
    let glow = world
        .get::<MeshMaterial3d<StandardMaterial>>(eyes)
        .unwrap()
        .0
        .clone();
    assert_ne!(glow, base);
    assert!(world.get::<HalloweenEyes>(eyes).is_some());
    let material = world
        .resource::<Assets<StandardMaterial>>()
        .get(&glow)
        .unwrap();
    assert_eq!(material.base_color, EYE_COLOR);
    assert!(material.emissive.red > material.emissive.green);
    assert!(material.emissive.green > material.emissive.blue);
    assert_eq!(
        world
            .get::<MeshMaterial3d<StandardMaterial>>(jaw)
            .unwrap()
            .0,
        base
    );
    assert!(world.get::<HalloweenEyes>(jaw).is_none());
    let halos: Vec<Entity> = world
        .query_filtered::<&ChildOf, With<EyeHalo>>()
        .iter(world)
        .map(ChildOf::parent)
        .collect();
    assert_eq!(halos, vec![head, head]);
    assert!(!halos.contains(&other_head));
}
