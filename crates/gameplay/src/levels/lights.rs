use bevy::prelude::*;

use super::{
    builder::{LightEffect, LightIntensity, Prop},
    module_names::{
        BOILER_UNIT, CEILING_LIGHT_COOL, CONCEPT_CONTAINMENT_TANK, EXIT_SIGN, WALL_LAMP_RED,
    },
};

const COOL: Color = Color::linear_rgb(0.78, 0.88, 1.0);
const AMBER: Color = Color::linear_rgb(1.0, 0.58, 0.2);
const EXIT_GREEN: Color = Color::linear_rgb(0.12, 1.0, 0.35);
const FAULT_RED: Color = Color::linear_rgb(1.0, 0.06, 0.03);
const FIRE: Color = Color::linear_rgb(1.0, 0.35, 0.08);
const SPECIMEN: Color = Color::linear_rgb(0.3, 0.95, 0.85);

#[derive(Component, Clone, Copy, Default)]
pub struct LightConfig {
    pub position: Option<Vec3>,
    pub intensity: Option<f32>,
    pub effect: Option<LightEffect>,
}

impl LightConfig {
    pub const fn intensity(intensity: f32) -> Self {
        Self {
            position: None,
            intensity: Some(intensity),
            effect: None,
        }
    }

    pub const fn flicker(phase: f32) -> Self {
        Self {
            position: None,
            intensity: None,
            effect: Some(LightEffect::Flicker(phase)),
        }
    }
}

struct LightSpec {
    position: Vec3,
    color: Color,
    intensity: f32,
    range: f32,
    effect: Option<LightEffect>,
}

fn light_spec(module: &str) -> Option<LightSpec> {
    let (position, color, intensity, range, effect) = match module {
        CEILING_LIGHT_COOL => (Vec3::Y * 2.7, COOL, 90_000.0, 9.0, None),
        "ceiling_light_amber" => (Vec3::Y * 2.7, AMBER, 140_000.0, 9.0, None),
        EXIT_SIGN => (Vec3::NEG_Z * 0.2, EXIT_GREEN, 6_000.0, 5.0, None),
        WALL_LAMP_RED => (
            Vec3::NEG_Z * 0.25,
            FAULT_RED,
            60_000.0,
            5.0,
            Some(LightEffect::Pulse),
        ),
        BOILER_UNIT => (
            Vec3::new(0.0, 0.5, -0.85),
            FIRE,
            45_000.0,
            7.0,
            Some(LightEffect::Flicker(-7.0)),
        ),
        CONCEPT_CONTAINMENT_TANK => (
            Vec3::new(0.0, 2.1, 0.0),
            SPECIMEN,
            12_000.0,
            4.5,
            Some(LightEffect::Flicker(0.0)),
        ),
        _ => return None,
    };
    Some(LightSpec {
        position,
        color,
        intensity,
        range,
        effect,
    })
}

fn light(transform: Transform, color: Color, intensity: f32, range: f32) -> impl Bundle {
    (
        Name::new("light"),
        PointLight {
            color,
            intensity,
            range,
            radius: 0.08,
            shadow_maps_enabled: false,
            ..default()
        },
        LightIntensity(intensity),
        transform,
    )
}

pub struct PropLightsPlugin;

impl Plugin for PropLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_prop_light);
    }
}

fn attach_prop_light(
    added: On<Add, Prop>,
    props: Query<(&Prop, Option<&LightConfig>)>,
    mut commands: Commands,
) {
    let Ok((prop, config)) = props.get(added.entity) else {
        return;
    };
    let Some(spec) = light_spec(&prop.0) else {
        return;
    };
    let position = config
        .and_then(|config| config.position)
        .unwrap_or(spec.position);
    let intensity = config
        .and_then(|config| config.intensity)
        .unwrap_or(spec.intensity);
    let effect = config.and_then(|config| config.effect).or(spec.effect);
    let mut entity = commands.entity(added.entity);
    if let Some(effect) = effect {
        entity.insert(effect);
    }
    entity.with_children(|children| {
        let mut child = children.spawn(light(
            Transform::from_translation(position),
            spec.color,
            intensity,
            spec.range,
        ));
        if let Some(effect) = effect {
            child.insert(effect);
        }
    });
}
