use bevy::prelude::*;

use super::{animate_lights, needs_mains_power, LightEffect, LightIntensity, Prop};
use crate::levels::{
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
        app.world().get::<PointLight>(normal_light).unwrap().intensity,
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
        app.world().get::<PointLight>(normal_light).unwrap().intensity,
        100.0
    );
}
