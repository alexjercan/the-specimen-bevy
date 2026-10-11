mod common;

use bevy::{prelude::*, world_serialization::WorldAssetRoot};
use game_halloween::{
    Decor, HalloweenAssets, HalloweenDecor, CANDLE_CLUSTER, DECOR, JACK_O_LANTERN, LANTERN,
    SIGN_TRIM, SPECIMEN_JAR, SPIDER, SPIDER_WEB,
};
use gameplay::levels::{
    build_first_floor, build_main_menu_background, PickupKind, Prop, PropCollider, FUSE_TABLES,
};

use common::{halloween_app, halloween_assets, level_app};

const ANCHOR_TOLERANCE: f32 = 0.01;
const LIT_MODULES: [&str; 3] = [JACK_O_LANTERN, LANTERN, CANDLE_CLUSTER];

fn anchored(decor: &Decor, prop: &str, transform: &Transform) -> bool {
    decor.prop == prop && decor.anchor.distance(transform.translation.xz()) <= ANCHOR_TOLERANCE
}

fn decor_for<'a>(
    prop: &'a str,
    transform: &'a Transform,
) -> impl Iterator<Item = &'static Decor> + 'a {
    DECOR
        .iter()
        .filter(move |decor| anchored(decor, prop, transform))
}

fn spawn_menu_background(mut commands: Commands) {
    build_main_menu_background(&mut commands);
}

fn level_props() -> Vec<(String, Transform)> {
    let mut app = level_app();
    app.add_systems(Startup, (build_first_floor, spawn_menu_background));
    app.update();
    let world = app.world_mut();
    world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect()
}

fn placed_decor(app: &mut App) -> Vec<Entity> {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<HalloweenDecor>>()
        .iter(world)
        .collect()
}

fn module_name(world: &World, entity: Entity) -> String {
    world
        .get::<Name>(entity)
        .unwrap()
        .as_str()
        .strip_prefix("halloween ")
        .expect("decor is named after its module")
        .to_owned()
}

fn visual_handle(world: &World, decor: Entity) -> Handle<bevy::world_serialization::WorldAsset> {
    let children = world
        .get::<Children>(decor)
        .expect("decor carries its visual");
    let mut roots = children
        .iter()
        .filter_map(|child| world.get::<WorldAssetRoot>(child))
        .map(|root| root.0.clone());
    let handle = roots.next().expect("decor has a module visual");
    assert!(roots.next().is_none());
    handle
}

#[test]
fn every_decoration_anchors_to_exactly_one_authored_prop() {
    let props = level_props();
    for decor in &DECOR {
        let matches = props
            .iter()
            .filter(|(module, transform)| {
                decor_for(module, transform).any(|d| std::ptr::eq(d, decor))
            })
            .count();
        assert_eq!(
            matches, 1,
            "{} on {} at {}",
            decor.module, decor.prop, decor.anchor
        );
    }
}

#[test]
fn office_security_and_storage_have_anchored_shelf_and_table_decor() {
    let props = level_props();
    for (module, x, z, surface) in [
        ("concept_table", 3.75, -18.75, 0.8),
        ("shelf_unit_bins", 8.1, -17.5, 2.005),
        ("file_cabinet_open", 13.4, -26.25, 1.32),
        ("shelf_unit", 13.35, -18.75, 2.005),
        ("shelf_unit_alt", 10.75, -12.5, 2.005),
        ("shelf_unit_bins", 10.75, -14.8, 2.005),
        ("shelf_unit_low", 13.15, -6.25, 1.005),
    ] {
        let prop = props
            .iter()
            .find(|(name, transform)| {
                name == module && transform.translation.xz().distance(Vec2::new(x, z)) < 0.01
            })
            .expect("authored room prop");
        assert!(
            decor_for(&prop.0, &prop.1).any(|decor| (decor.offset.y - surface).abs() < 0.01),
            "missing surface decoration at {x}, {z}"
        );
    }
}

#[test]
fn rooms_beyond_prep_have_surface_decor() {
    let props = level_props();
    for (name, x, z, module) in [
        ("shelf_unit_bins", -9.5, -7.0, LANTERN),
        ("shelf_unit_low", -13.3, -13.2, CANDLE_CLUSTER),
        ("shelf_unit_alt", -10.0, -5.7, SPECIMEN_JAR),
        ("shelf_unit_bins", -7.5, 9.7, SPECIMEN_JAR),
        ("shelf_unit", -12.35, 15.65, LANTERN),
        ("shelf_unit_bins", 7.1, 13.75, LANTERN),
        ("shelf_unit_low", 7.5, 18.38, LANTERN),
        ("shelf_unit", 9.0, 4.25, SPECIMEN_JAR),
    ] {
        let prop = props
            .iter()
            .find(|(prop, transform)| {
                prop == name && transform.translation.xz().distance(Vec2::new(x, z)) < 0.01
            })
            .expect("authored room prop");
        assert!(
            decor_for(&prop.0, &prop.1).any(|decor| decor.module == module),
            "missing {module} at {x}, {z}"
        );
    }
}

#[test]
fn decorations_stay_clear_of_pickups() {
    let lit = DECOR
        .iter()
        .filter(|decor| decor.module == JACK_O_LANTERN)
        .count();
    assert!(lit >= 8);
    let mut app = level_app();
    app.add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    let pickups: Vec<Vec2> = world
        .query_filtered::<&Transform, With<PickupKind>>()
        .iter(world)
        .map(|transform| transform.translation.xz())
        .chain(FUSE_TABLES.iter().map(|table| table.position.xz()))
        .collect();
    let crowded: Vec<_> = DECOR
        .iter()
        .flat_map(|decor| {
            pickups
                .iter()
                .filter(move |pickup| decor.anchor.distance(**pickup) <= 1.0)
                .map(move |pickup| {
                    format!(
                        "{} at {} crowds a pickup at {pickup}",
                        decor.module, decor.anchor
                    )
                })
        })
        .collect();
    assert!(crowded.is_empty(), "{}", crowded.join("\n"));
}

#[test]
fn decorations_spawn_as_visual_children_without_gameplay_components() {
    let mut app = halloween_app();
    app.add_systems(Startup, build_first_floor);
    app.update();
    let placed = placed_decor(&mut app);
    let world = app.world_mut();
    let props: Vec<(Entity, String, Transform)> = world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .map(|(entity, prop, transform)| (entity, prop.0.clone(), *transform))
        .collect();
    let authored: usize = props
        .iter()
        .filter(|(_, name, _)| name != "potted_plant")
        .map(|(_, name, transform)| decor_for(name, transform).count())
        .sum();
    let labels: Vec<Entity> = props
        .iter()
        .filter(|(_, name, _)| name.starts_with("sign_label_"))
        .map(|(entity, _, _)| *entity)
        .collect();
    assert!(labels.len() >= 20);
    let assets = halloween_assets();
    let mut trims = 0;
    let mut webs = 0;
    let mut spiders = 0;
    let mut lit = 0;
    for &entity in &placed {
        let decor = world.entity(entity);
        assert!(decor.get::<Prop>().is_none());
        assert!(decor.get::<PropCollider>().is_none());
        assert!(decor.get::<PickupKind>().is_none());
        let parent = decor
            .get::<ChildOf>()
            .expect("decor follows its prop")
            .parent();
        assert!(world.get::<Prop>(parent).is_some());
        let module = module_name(world, entity);
        assert_eq!(
            visual_handle(world, entity),
            *assets
                .module(&module)
                .expect("decor uses a halloween module")
        );
        let lights = world
            .get::<Children>(entity)
            .unwrap()
            .iter()
            .filter(|child| world.get::<PointLight>(*child).is_some())
            .count();
        assert_eq!(lights, usize::from(LIT_MODULES.contains(&module.as_str())));
        lit += lights;
        let on_label = labels.contains(&parent);
        match module.as_str() {
            SIGN_TRIM => {
                assert!(on_label);
                trims += 1;
            }
            SPIDER_WEB | SPIDER => {
                assert!(on_label);
                assert!(world.get::<Transform>(parent).unwrap().translation.y > 2.5);
                if module == SPIDER_WEB {
                    webs += 1;
                } else {
                    spiders += 1;
                }
            }
            _ => assert!(!on_label),
        }
    }
    assert_eq!(trims, labels.len());
    assert_eq!(webs, 4);
    assert_eq!(spiders, 4);
    assert_eq!(placed.len(), authored + labels.len() + webs + spiders);
    assert!(lit > 0);
}

#[test]
fn main_menu_decorations_follow_props_and_keep_gameplay_components_untouched() {
    let mut app = halloween_app();
    app.add_systems(Startup, spawn_menu_background);
    app.update();
    let placed = placed_decor(&mut app);
    let world = app.world_mut();
    assert_eq!(placed.len(), 4);
    let mut modules: Vec<String> = placed
        .iter()
        .map(|&entity| {
            let parent = world.get::<ChildOf>(entity).unwrap().parent();
            assert!(world.get::<Prop>(parent).is_some());
            assert!(world.get::<Prop>(entity).is_none());
            world.get::<Name>(entity).unwrap().as_str().to_owned()
        })
        .collect();
    modules.sort();
    assert_eq!(
        modules,
        [
            "halloween candle_cluster",
            "halloween jack_o_lantern",
            "halloween lantern",
            "halloween specimen_jar",
        ]
    );
}

#[test]
fn plant_visuals_become_pumpkins_without_changing_their_prop_identity() {
    let mut app = halloween_app();
    app.add_systems(Startup, build_first_floor);
    app.update();
    let before = placed_decor(&mut app).len();
    let plants: Vec<(Entity, Transform)> = {
        let world = app.world_mut();
        world
            .query::<(Entity, &Prop, &Transform)>()
            .iter(world)
            .filter(|(_, prop, _)| prop.0 == "potted_plant")
            .map(|(entity, _, transform)| (entity, *transform))
            .collect()
    };
    assert_eq!(plants.len(), 8);
    let visuals: Vec<Entity> = plants
        .iter()
        .map(|&(plant, _)| {
            app.world_mut()
                .spawn((WorldAssetRoot(Handle::default()), ChildOf(plant)))
                .id()
        })
        .collect();
    app.update();
    assert_eq!(placed_decor(&mut app).len(), before + plants.len());
    let world = app.world();
    let assets: &HalloweenAssets = world.resource::<HalloweenAssets>();
    for (&(plant, transform), &visual) in plants.iter().zip(&visuals) {
        assert_eq!(world.get::<Prop>(plant).unwrap().0, "potted_plant");
        assert!(world.get::<PropCollider>(plant).is_none());
        let decor = decor_for("potted_plant", &transform)
            .next()
            .expect("every plant has a pumpkin");
        assert!(world.get::<HalloweenDecor>(visual).is_some());
        assert_eq!(
            world.get::<WorldAssetRoot>(visual).unwrap().0,
            *assets.module(decor.module).unwrap()
        );
        assert_eq!(module_name(world, visual), decor.module);
    }
}
