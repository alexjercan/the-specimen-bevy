use bevy::prelude::*;
use gameplay::levels::{
    build_main_menu_background, FacilityLight, FacilityPower, LevelRenderPlugin, LightIntensity,
    Prop, PropLightsPlugin, BOILER_UNIT,
};

mod common;

fn render_app() -> App {
    let mut app = common::level_app();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins((PropLightsPlugin, LevelRenderPlugin));
    app
}

fn spawn_prop(app: &mut App, module: &str) -> Entity {
    app.world_mut()
        .spawn((Prop(module.to_owned()), Transform::default()))
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
    let normal: Vec<Entity> = ["ceiling_light_cool", "ceiling_light_amber"]
        .into_iter()
        .map(|module| spawn_prop(&mut app, module))
        .collect();
    let emergency: Vec<Entity> = ["wall_lamp_red", "exit_sign", BOILER_UNIT]
        .into_iter()
        .map(|module| spawn_prop(&mut app, module))
        .collect();
    let mut power = FacilityPower::new(3);
    power.outage();
    app.insert_resource(power);
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
    app.insert_resource(FacilityPower::blackout(3));
    app.update();
    app.update();
    let world = app.world_mut();
    let mut lights = world.query_filtered::<&PointLight, With<FacilityLight>>();
    let intensities: Vec<f32> = lights.iter(world).map(|light| light.intensity).collect();
    assert_eq!(intensities.len(), 3);
    assert!(intensities.iter().all(|intensity| *intensity > 0.0));
}
