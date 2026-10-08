use bevy::prelude::*;

use super::{
    animate_lights, animate_surfaces, needs_mains_power, GlowSurface, LightEffect, LightIntensity,
    Prop, GLOWING_MATERIALS,
};
use crate::levels::{
    menu_background::MenuBackground,
    module_names::{BOILER_UNIT, EXIT_SIGN, WALL_LAMP_RED},
    power::FacilityPower,
};

#[test]
fn emergency_fixtures_remain_lit() {
    for module in [WALL_LAMP_RED, EXIT_SIGN, BOILER_UNIT] {
        assert!(!needs_mains_power(module));
    }
    for module in [
        "ceiling_light_cool",
        "ceiling_light_amber",
        "concept_containment_tank",
    ] {
        assert!(needs_mains_power(module));
    }
}

#[test]
fn amber_and_cool_ceiling_materials_are_power_controlled() {
    for material in ["lamp_amber", "lamp_cool"] {
        assert!(GLOWING_MATERIALS.contains(&material));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(Update, animate_surfaces);
    let emissive = LinearRgba::rgb(3.0, 2.0, 1.0);
    let materials = ["lamp_amber", "lamp_cool", "lamp_red"];
    let surfaces: Vec<_> = materials
        .into_iter()
        .map(|material| {
            let handle = app
                .world_mut()
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    emissive,
                    ..default()
                });
            let entity = app
                .world_mut()
                .spawn((
                    GlowSurface {
                        effect: None,
                        base: emissive,
                        needs_power: material != "lamp_red",
                    },
                    MeshMaterial3d(handle.clone()),
                ))
                .id();
            (entity, handle)
        })
        .collect();
    let mut power = FacilityPower::new(3);
    power.outage();
    app.insert_resource(power);
    app.update();
    for (index, (_, handle)) in surfaces.iter().enumerate() {
        let actual = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(handle)
            .unwrap()
            .emissive;
        assert_eq!(actual, if index < 2 { emissive * 0.0 } else { emissive });
    }
    app.world_mut().resource_mut::<FacilityPower>().restore();
    app.update();
    for (_, handle) in &surfaces {
        assert_eq!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(handle)
                .unwrap()
                .emissive,
            emissive
        );
    }
}

#[test]
fn menu_lamp_flickers_even_when_previous_run_ended_in_blackout() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, animate_lights);
    let root = app
        .world_mut()
        .spawn((MenuBackground, Transform::IDENTITY))
        .id();
    let lamp = app
        .world_mut()
        .spawn((Prop("ceiling_light_amber".into()), ChildOf(root)))
        .id();
    let bulb = app
        .world_mut()
        .spawn((
            PointLight::default(),
            LightIntensity(100.0),
            LightEffect::Flicker(0.7),
            ChildOf(lamp),
        ))
        .id();
    let mut power = FacilityPower::new(3);
    power.outage();
    app.insert_resource(power);
    app.update();
    assert!(app.world().get::<PointLight>(bulb).unwrap().intensity > 0.0);
}

#[test]
fn outage_extinguishes_normal_lights_but_not_emergency_lights() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, animate_lights);
    let normal = app
        .world_mut()
        .spawn((
            Prop("ceiling_light_cool".into()),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let emergency = app
        .world_mut()
        .spawn((
            Prop(WALL_LAMP_RED.into()),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let normal_light = app
        .world_mut()
        .spawn((
            PointLight::default(),
            LightIntensity(100.0),
            Transform::default(),
            ChildOf(normal),
        ))
        .id();
    let emergency_light = app
        .world_mut()
        .spawn((
            PointLight::default(),
            LightIntensity(100.0),
            LightEffect::Pulse,
            Transform::default(),
            ChildOf(emergency),
        ))
        .id();
    let mut power = FacilityPower::new(3);
    power.outage();
    app.insert_resource(power);
    app.update();
    assert_eq!(
        app.world()
            .get::<PointLight>(normal_light)
            .unwrap()
            .intensity,
        0.0
    );
    assert!(
        app.world()
            .get::<PointLight>(emergency_light)
            .unwrap()
            .intensity
            > 0.0
    );
    app.world_mut().resource_mut::<FacilityPower>().restore();
    app.update();
    assert_eq!(
        app.world()
            .get::<PointLight>(normal_light)
            .unwrap()
            .intensity,
        100.0
    );
}
