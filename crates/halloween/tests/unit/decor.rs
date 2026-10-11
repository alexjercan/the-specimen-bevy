use gameplay::levels::{build_first_floor, build_main_menu_background, PickupKind, FUSE_TABLES};

use super::*;

fn test_assets() -> HalloweenAssets {
    HalloweenAssets {
        candy_fuse: Handle::default(),
        jack_o_lantern: Handle::default(),
        pumpkin_small: Handle::default(),
        specimen_jar: Handle::default(),
        lantern: Handle::default(),
        candle_cluster: Handle::default(),
        spider_web: Handle::default(),
        spider: Handle::default(),
        sign_trim: Handle::default(),
    }
}

fn level_props() -> Vec<(String, Transform)> {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor)
        .add_systems(Startup, |mut commands: Commands| {
            build_main_menu_background(&mut commands);
        });
    app.update();
    let world = app.world_mut();
    world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect()
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
fn decorations_stay_clear_of_pickups() {
    let lit = DECOR
        .iter()
        .filter(|decor| decor.module == JACK_O_LANTERN)
        .count();
    assert!(lit >= 8);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    let pickups: Vec<Vec2> = world
        .query_filtered::<&Transform, With<PickupKind>>()
        .iter(world)
        .map(|transform| transform.translation.xz())
        .chain(FUSE_TABLES.iter().map(|table| table.position.xz()))
        .collect();
    for decor in &DECOR {
        for pickup in &pickups {
            assert!(
                decor.anchor.distance(*pickup) > 1.0,
                "{} at {} crowds a pickup at {pickup}",
                decor.module,
                decor.anchor
            );
        }
    }
}

#[test]
fn decorations_spawn_as_visual_children_without_gameplay_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(test_assets())
        .add_plugins(DecorPlugin)
        .add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    let placed: Vec<Entity> = world
        .query_filtered::<Entity, With<HalloweenDecor>>()
        .iter(world)
        .collect();
    let labels = level_props()
        .iter()
        .filter(|(name, _)| name.starts_with("sign_label_"))
        .count();
    assert!(labels >= 20);
    let webbed = level_props()
        .iter()
        .filter(|(name, transform)| webbed_sign(name, transform))
        .count();
    assert_eq!(webbed, 4);
    assert_eq!(
        placed.len(),
        DECOR.len() - 12 + labels + webbed * SIGN_WEB_DECOR.len()
    );
    let decorated_labels = placed
        .iter()
        .filter(|&&entity| {
            let parent = world.get::<ChildOf>(entity).unwrap().parent();
            world
                .get::<Prop>(parent)
                .is_some_and(|prop| prop.0.starts_with("sign_label_"))
        })
        .count();
    assert_eq!(decorated_labels, labels + webbed * SIGN_WEB_DECOR.len());
    let label_entities: Vec<(Entity, bool)> = world
        .query::<(Entity, &Prop, &Transform)>()
        .iter(world)
        .filter(|(_, prop, _)| prop.0.starts_with("sign_label_"))
        .map(|(entity, prop, transform)| (entity, webbed_sign(&prop.0, transform)))
        .collect();
    for (label, web) in label_entities {
        let modules: Vec<&str> = placed
            .iter()
            .filter(|&&entity| world.get::<ChildOf>(entity).unwrap().parent() == label)
            .map(|&entity| world.get::<Name>(entity).unwrap().as_str())
            .collect();
        for (module, expected) in [
            (SIGN_TRIM, 1),
            (SPIDER_WEB, usize::from(web)),
            (SPIDER, usize::from(web)),
        ] {
            assert_eq!(
                modules
                    .iter()
                    .filter(|name| **name == format!("halloween {module}"))
                    .count(),
                expected,
                "unexpected {module} for {}",
                world.get::<Prop>(label).unwrap().0
            );
        }
    }
    assert!(SIGN_WEB_DECOR.iter().all(|decor| decor.offset.x > 0.3));
    let lights = world
        .query_filtered::<&ChildOf, With<PointLight>>()
        .iter(world)
        .filter(|parent| placed.contains(&parent.parent()))
        .count();
    assert_eq!(
        lights,
        DECOR
            .iter()
            .filter(|decor| {
                has_light(decor.module)
                    && decor.prop != "potted_plant"
                    && decor.anchor != Vec2::new(2.8, -2.4)
                    && decor.anchor != Vec2::new(2.2, -4.6)
                    && decor.anchor != Vec2::new(0.0, -2.5)
            })
            .count()
    );
    for entity in placed {
        let entity = world.entity(entity);
        assert!(entity.get::<Prop>().is_none());
        assert!(entity.get::<gameplay::levels::PropCollider>().is_none());
        assert!(entity.get::<PickupKind>().is_none());
        let parent = entity.get::<ChildOf>().expect("decor follows its prop");
        assert!(world.get::<Prop>(parent.parent()).is_some());
    }
}

#[test]
fn main_menu_decorations_follow_props_and_keep_gameplay_components_untouched() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(test_assets())
        .add_plugins(DecorPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            build_main_menu_background(&mut commands);
        });
    app.update();
    let world = app.world_mut();
    let placed: Vec<Entity> = world
        .query_filtered::<Entity, With<HalloweenDecor>>()
        .iter(world)
        .collect();
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
fn menu_props_spawned_before_halloween_assets_still_receive_decorations() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(DecorPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            build_main_menu_background(&mut commands);
        });
    app.update();
    let count = |app: &mut App| {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<HalloweenDecor>>()
            .iter(world)
            .count()
    };
    assert_eq!(count(&mut app), 0);
    app.insert_resource(test_assets());
    app.update();
    assert_eq!(count(&mut app), 4);
    app.update();
    assert_eq!(count(&mut app), 4);
}

#[test]
fn plant_visuals_become_pumpkins_without_changing_their_prop_identity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(test_assets())
        .add_plugins(DecorPlugin)
        .add_systems(Startup, build_first_floor);
    app.update();
    let plants: Vec<Entity> = {
        let world = app.world_mut();
        world
            .query::<(Entity, &Prop)>()
            .iter(world)
            .filter(|(_, prop)| prop.0 == "potted_plant")
            .map(|(entity, _)| entity)
            .collect()
    };
    assert_eq!(plants.len(), 8);
    for &plant in &plants {
        app.world_mut()
            .spawn((WorldAssetRoot(Handle::default()), ChildOf(plant)));
    }
    app.update();
    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<HalloweenDecor>>()
            .iter(world)
            .count(),
        DECOR.len() - 4
            + level_props()
                .iter()
                .filter(|(name, _)| name.starts_with("sign_label_"))
                .count()
            + level_props()
                .iter()
                .filter(|(name, transform)| webbed_sign(name, transform))
                .count()
                * SIGN_WEB_DECOR.len()
    );
    for plant in plants {
        assert_eq!(world.get::<Prop>(plant).unwrap().0, "potted_plant");
        assert!(world.get::<gameplay::levels::PropCollider>(plant).is_none());
    }
}
