use std::time::Duration;

use bevy::prelude::*;
use gameplay::facility::{
    build_main_menu_background, FacilityLight, FacilityPlugin, FacilityPower, FacilityViewPlugin,
    LightIntensity, Module, Prop,
};

mod common;

fn render_app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins((FacilityPlugin, FacilityViewPlugin));
    app
}

fn spawn_prop(app: &mut App, module: Module) -> Entity {
    app.world_mut()
        .spawn((Prop(module), Transform::default()))
        .id()
}

fn light_of(app: &mut App, prop: Entity) -> (f32, f32) {
    let world = app.world_mut();
    let mut lights =
        world.query_filtered::<(&PointLight, &LightIntensity, &ChildOf), With<FacilityLight>>();
    let (light, base, _) = lights
        .iter(world)
        .find(|(_, _, parent)| parent.parent() == prop)
        .expect("prop light");
    (light.intensity, base.0)
}

#[test]
fn outage_extinguishes_normal_lights_but_not_emergency_lights() {
    let mut app = render_app();
    let normal: Vec<Entity> = [Module::CeilingLightCool, Module::CeilingLightAmber]
        .into_iter()
        .map(|module| spawn_prop(&mut app, module))
        .collect();
    let emergency: Vec<Entity> = [Module::WallLampRed, Module::ExitSign, Module::BoilerUnit]
        .into_iter()
        .map(|module| spawn_prop(&mut app, module))
        .collect();
    let mut power = app.world_mut().resource_mut::<FacilityPower>();
    *power = FacilityPower::new(3);
    power.outage();
    app.update();
    for prop in &normal {
        assert_eq!(light_of(&mut app, *prop).0, 0.0);
    }
    for prop in &emergency {
        assert!(light_of(&mut app, *prop).0 > 0.0);
    }
    app.world_mut().resource_mut::<FacilityPower>().restore();
    app.update();
    for prop in normal.iter().chain(&emergency) {
        let (intensity, base) = light_of(&mut app, *prop);
        assert!(intensity > 0.0 && intensity <= base);
    }
    let (intensity, base) = light_of(&mut app, normal[0]);
    assert_eq!(intensity, base);
}

#[test]
fn menu_lamps_stay_lit_even_when_previous_run_ended_in_blackout() {
    let mut app = render_app();
    app.add_systems(Startup, |mut commands: Commands| {
        build_main_menu_background(&mut commands);
    });
    *app.world_mut().resource_mut::<FacilityPower>() = FacilityPower::blackout(3);
    app.update();
    app.update();
    let world = app.world_mut();
    let mut lights = world.query_filtered::<&PointLight, With<FacilityLight>>();
    let intensities: Vec<f32> = lights.iter(world).map(|light| light.intensity).collect();
    assert_eq!(intensities.len(), 3);
    assert!(intensities.iter().all(|intensity| *intensity > 0.0));
}
