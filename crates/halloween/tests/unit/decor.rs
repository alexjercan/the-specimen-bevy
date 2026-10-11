use gameplay::levels::{build_first_floor, build_main_menu_background, PickupKind, FUSE_TABLES};

use super::*;

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
        .insert_resource(HalloweenAssets {
            candy_fuse: Handle::default(),
            jack_o_lantern: Handle::default(),
            pumpkin_small: Handle::default(),
        })
        .add_plugins(DecorPlugin)
        .add_systems(Startup, build_first_floor);
    app.update();
    let world = app.world_mut();
    let placed: Vec<Entity> = world
        .query_filtered::<Entity, With<HalloweenDecor>>()
        .iter(world)
        .collect();
    assert_eq!(placed.len(), DECOR.len() - 9);
    let lights = world
        .query_filtered::<&ChildOf, With<PointLight>>()
        .iter(world)
        .filter(|parent| placed.contains(&parent.parent()))
        .count();
    assert_eq!(lights, 6);
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
fn plant_visuals_become_pumpkins_without_changing_their_prop_identity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(HalloweenAssets {
            candy_fuse: Handle::default(),
            jack_o_lantern: Handle::default(),
            pumpkin_small: Handle::default(),
        })
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
        DECOR.len() - 1
    );
    for plant in plants {
        assert_eq!(world.get::<Prop>(plant).unwrap().0, "potted_plant");
        assert!(world.get::<gameplay::levels::PropCollider>(plant).is_none());
    }
}
