use bevy::prelude::*;

use super::{
    builder::{LightIntensity, Prop},
    lights::{FacilityLight, LightConfig},
};

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
        FacilityLight,
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
    let Some(spec) = prop.0.light_spec() else {
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
