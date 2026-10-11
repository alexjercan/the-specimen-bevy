use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use gameplay::levels::Prop;

use crate::HalloweenAssets;

pub const JACK_O_LANTERN: &str = "jack_o_lantern";
pub const PUMPKIN_SMALL: &str = "pumpkin_small";

const DRUM_SPOT: Vec3 = Vec3::new(-0.06, 0.89, 0.0);
const CRATE_TOP: f32 = 0.7;
const ANCHOR_TOLERANCE: f32 = 0.01;
const CANDLE_COLOR: Color = Color::srgb(1.0, 0.45, 0.1);
const CANDLE_INTENSITY: f32 = 4_000.0;
const CANDLE_RANGE: f32 = 2.5;
const CANDLE_HEIGHT: f32 = 0.11;

#[derive(Clone, Copy, Debug)]
pub struct Decor {
    pub prop: &'static str,
    pub anchor: Vec2,
    pub module: &'static str,
    pub offset: Vec3,
    pub yaw: f32,
}

const fn decor(
    prop: &'static str,
    x: f32,
    z: f32,
    module: &'static str,
    offset: Vec3,
    yaw: f32,
) -> Decor {
    Decor {
        prop,
        anchor: Vec2::new(x, z),
        module,
        offset,
        yaw,
    }
}

pub static DECOR: [Decor; 20] = [
    decor("steel_drum", -4.4, -23.1, JACK_O_LANTERN, DRUM_SPOT, PI),
    decor(
        "steel_drum",
        -12.5,
        -2.5,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_2,
    ),
    decor(
        "steel_drum",
        7.0,
        10.5,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_2,
    ),
    decor(
        "steel_drum",
        -12.8,
        18.2,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_4,
    ),
    decor(
        "storage_crate",
        2.8,
        2.6,
        PUMPKIN_SMALL,
        Vec3::new(0.22, CRATE_TOP, 0.05),
        0.4,
    ),
    decor(
        "storage_crate",
        12.8,
        18.2,
        PUMPKIN_SMALL,
        Vec3::new(-0.2, CRATE_TOP, 0.0),
        -0.6,
    ),
    decor(
        "storage_crate",
        2.8,
        -2.4,
        JACK_O_LANTERN,
        Vec3::new(-0.15, CRATE_TOP, 0.1),
        PI - 0.5,
    ),
    decor(
        "potted_plant",
        -5.75,
        -14.7,
        JACK_O_LANTERN,
        Vec3::ZERO,
        0.4,
    ),
    decor(
        "potted_plant",
        5.75,
        -14.8,
        JACK_O_LANTERN,
        Vec3::ZERO,
        -0.2,
    ),
    decor("potted_plant", -5.7, -6.9, PUMPKIN_SMALL, Vec3::ZERO, 1.0),
    decor("potted_plant", 5.75, -6.8, JACK_O_LANTERN, Vec3::ZERO, 0.7),
    decor("potted_plant", -5.7, 2.8, PUMPKIN_SMALL, Vec3::ZERO, -0.5),
    decor("potted_plant", 5.7, 2.7, JACK_O_LANTERN, Vec3::ZERO, 0.3),
    decor(
        "potted_plant",
        -5.7,
        18.25,
        JACK_O_LANTERN,
        Vec3::ZERO,
        -0.4,
    ),
    decor("potted_plant", 5.7, 18.25, PUMPKIN_SMALL, Vec3::ZERO, 0.2),
    decor("steel_drum", 13.2, -9.7, PUMPKIN_SMALL, DRUM_SPOT, 0.3),
    decor("steel_drum", 12.6, 11.8, JACK_O_LANTERN, DRUM_SPOT, -0.6),
    decor(
        "storage_crate",
        -10.3,
        -22.9,
        PUMPKIN_SMALL,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.2,
    ),
    decor(
        "storage_crate",
        11.0,
        6.2,
        PUMPKIN_SMALL,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        -0.3,
    ),
    decor(
        "storage_crate",
        7.8,
        15.6,
        JACK_O_LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.5,
    ),
];

#[derive(Component)]
pub struct HalloweenDecor;

pub(crate) struct DecorPlugin;

impl Plugin for DecorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(decorate_prop)
            .add_observer(replace_plant_visual);
    }
}

pub fn candle_light() -> impl Bundle {
    (
        Name::new("halloween candle"),
        PointLight {
            color: CANDLE_COLOR,
            intensity: CANDLE_INTENSITY,
            range: CANDLE_RANGE,
            radius: 0.04,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, CANDLE_HEIGHT, 0.0),
    )
}

pub(crate) fn decor_for<'a>(
    prop: &'a str,
    transform: &Transform,
) -> impl Iterator<Item = &'static Decor> + 'a {
    let at = transform.translation.xz();
    DECOR
        .iter()
        .filter(move |decor| decor.prop == prop && decor.anchor.distance(at) <= ANCHOR_TOLERANCE)
}

fn decorate_prop(
    added: On<Add, Prop>,
    props: Query<(&Prop, &Transform)>,
    assets: Option<Res<HalloweenAssets>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    let Ok((prop, transform)) = props.get(added.entity) else {
        return;
    };
    if prop.0 == "potted_plant" {
        return;
    }
    for decor in decor_for(&prop.0, transform) {
        if let Some(module) = assets.module(decor.module) {
            spawn_decor(
                &mut commands,
                added.entity,
                transform.rotation,
                decor,
                module.clone(),
            );
        }
    }
}

fn replace_plant_visual(
    added: On<Add, WorldAssetRoot>,
    roots: Query<&ChildOf, With<WorldAssetRoot>>,
    props: Query<(&Prop, &Transform)>,
    assets: Option<Res<HalloweenAssets>>,
    mut commands: Commands,
) {
    let Some(assets) = assets else {
        return;
    };
    let Ok(parent) = roots.get(added.entity) else {
        return;
    };
    let Ok((prop, transform)) = props.get(parent.parent()) else {
        return;
    };
    if prop.0 != "potted_plant" {
        return;
    }
    if let Some(decor) = decor_for(&prop.0, transform).next() {
        if let Some(module) = assets.module(decor.module) {
            commands.entity(added.entity).insert((
                Name::new(format!("halloween {}", decor.module)),
                HalloweenDecor,
                WorldAssetRoot(module.clone()),
                Transform::from_rotation(
                    transform.rotation.inverse() * Quat::from_rotation_y(decor.yaw),
                ),
            ));
            if decor.module == JACK_O_LANTERN {
                commands.entity(added.entity).with_child(candle_light());
            }
        }
    }
}

pub fn spawn_decor(
    commands: &mut Commands,
    prop: Entity,
    prop_rotation: Quat,
    decor: &Decor,
    module: Handle<WorldAsset>,
) {
    let mut placed = commands.spawn((
        Name::new(format!("halloween {}", decor.module)),
        HalloweenDecor,
        Transform::from_translation(decor.offset)
            .with_rotation(prop_rotation.inverse() * Quat::from_rotation_y(decor.yaw)),
        Visibility::default(),
        ChildOf(prop),
    ));
    placed.with_child((WorldAssetRoot(module), Transform::IDENTITY));
    if decor.module == JACK_O_LANTERN {
        placed.with_child(candle_light());
    }
}

#[cfg(test)]
#[path = "../tests/unit/decor.rs"]
mod tests;
