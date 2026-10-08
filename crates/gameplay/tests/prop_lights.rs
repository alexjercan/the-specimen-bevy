use bevy::prelude::*;
use gameplay::levels::{LightConfig, LightEffect, LightIntensity, Prop, PropLightsPlugin};

#[test]
fn registered_props_get_light_children_and_unlit_props_do_not() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PropLightsPlugin));
    let cool = app
        .world_mut()
        .spawn((Prop("ceiling_light_cool".into()), Transform::IDENTITY))
        .id();
    let dead = app
        .world_mut()
        .spawn((Prop("ceiling_light_dead".into()), Transform::IDENTITY))
        .id();
    let boiler = app
        .world_mut()
        .spawn((Prop("boiler_unit".into()), Transform::IDENTITY))
        .id();
    let red = app
        .world_mut()
        .spawn((Prop("wall_lamp_red".into()), Transform::IDENTITY))
        .id();
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &ChildOf,
        &PointLight,
        &LightIntensity,
        &Transform,
        Option<&LightEffect>,
    )>();
    let lights: Vec<_> = query.iter(world).collect();
    assert_eq!(lights.len(), 3);
    let (_, cool_light, base, transform, effect) = lights
        .iter()
        .find(|(parent, ..)| parent.parent() == cool)
        .unwrap();
    assert_eq!(cool_light.intensity, 90_000.0);
    assert_eq!(base.0, 90_000.0);
    assert_eq!(transform.translation, Vec3::Y * 2.7);
    assert!(effect.is_none());
    assert!(lights.iter().all(|(parent, ..)| parent.parent() != dead));
    assert!(matches!(
        world.get::<LightEffect>(boiler),
        Some(LightEffect::Flicker(-7.0))
    ));
    assert!(matches!(
        world.get::<LightEffect>(red),
        Some(LightEffect::Pulse)
    ));
    assert!(lights.iter().any(|(parent, _, _, transform, effect)| {
        parent.parent() == boiler
            && transform.translation == Vec3::new(0.0, 0.5, -0.85)
            && matches!(effect, Some(LightEffect::Flicker(-7.0)))
    }));
}

#[test]
fn authored_config_overrides_fixture_defaults_once() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PropLightsPlugin));
    let parent = app
        .world_mut()
        .spawn((
            Prop("ceiling_light_cool".into()),
            Transform::IDENTITY,
            LightConfig {
                position: Some(Vec3::new(1.0, 2.0, 3.0)),
                intensity: Some(123.0),
                effect: Some(LightEffect::Flicker(1.2)),
            },
        ))
        .id();
    app.update();
    app.update();
    let world = app.world_mut();
    assert!(matches!(
        world.get::<LightEffect>(parent),
        Some(LightEffect::Flicker(1.2))
    ));
    let mut query = world.query::<(
        &ChildOf,
        &PointLight,
        &LightIntensity,
        &Transform,
        &LightEffect,
    )>();
    let children: Vec<_> = query
        .iter(world)
        .filter(|(child, ..)| child.parent() == parent)
        .collect();
    assert_eq!(children.len(), 1);
    let (_, light, base, transform, _) = children[0];
    assert_eq!(light.intensity, 123.0);
    assert_eq!(base.0, 123.0);
    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
}
