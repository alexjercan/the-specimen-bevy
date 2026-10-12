use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use gameplay::facility::{Module, Prop};

use crate::HalloweenAssets;

pub const JACK_O_LANTERN: &str = "jack_o_lantern";
pub const PUMPKIN_SMALL: &str = "pumpkin_small";
pub const SPECIMEN_JAR: &str = "specimen_jar";
pub const LANTERN: &str = "lantern";
pub const CANDLE_CLUSTER: &str = "candle_cluster";
pub const SPIDER_WEB: &str = "spider_web";
pub const SPIDER: &str = "spider";
pub const SIGN_TRIM: &str = "sign_trim";

const DRUM_SPOT: Vec3 = Vec3::new(-0.06, 0.89, 0.0);
const CRATE_TOP: f32 = 0.7;
const ANCHOR_TOLERANCE: f32 = 0.01;
const CANDLE_COLOR: Color = Color::srgb(1.0, 0.45, 0.1);
const CANDLE_INTENSITY: f32 = 4_000.0;
const CANDLE_RANGE: f32 = 2.5;
const CANDLE_HEIGHT: f32 = 0.11;

#[derive(Clone, Copy, Debug)]
pub struct Decor {
    pub prop: Option<Module>,
    pub anchor: Vec2,
    pub module: &'static str,
    pub offset: Vec3,
    pub yaw: f32,
}

const fn decor(
    prop: Module,
    x: f32,
    z: f32,
    module: &'static str,
    offset: Vec3,
    yaw: f32,
) -> Decor {
    Decor {
        prop: Some(prop),
        anchor: Vec2::new(x, z),
        module,
        offset,
        yaw,
    }
}

pub static DECOR: [Decor; 62] = [
    decor(
        Module::SteelDrum,
        -4.4,
        -23.1,
        JACK_O_LANTERN,
        DRUM_SPOT,
        PI,
    ),
    decor(
        Module::SteelDrum,
        -12.5,
        -2.5,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_2,
    ),
    decor(
        Module::SteelDrum,
        7.0,
        10.5,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_2,
    ),
    decor(
        Module::SteelDrum,
        -12.8,
        18.2,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -FRAC_PI_4,
    ),
    decor(
        Module::StorageCrate,
        2.8,
        2.6,
        PUMPKIN_SMALL,
        Vec3::new(0.22, CRATE_TOP, 0.05),
        0.4,
    ),
    decor(
        Module::StorageCrate,
        12.8,
        18.1,
        PUMPKIN_SMALL,
        Vec3::new(-0.2, CRATE_TOP, 0.0),
        -0.6,
    ),
    decor(
        Module::StorageCrate,
        2.8,
        -2.4,
        JACK_O_LANTERN,
        Vec3::new(-0.15, CRATE_TOP, 0.1),
        PI - 0.5,
    ),
    decor(
        Module::PottedPlant,
        -5.75,
        -14.7,
        JACK_O_LANTERN,
        Vec3::ZERO,
        0.4,
    ),
    decor(
        Module::PottedPlant,
        5.75,
        -14.8,
        JACK_O_LANTERN,
        Vec3::ZERO,
        -0.2,
    ),
    decor(
        Module::PottedPlant,
        -5.7,
        -6.9,
        PUMPKIN_SMALL,
        Vec3::ZERO,
        1.0,
    ),
    decor(
        Module::PottedPlant,
        5.75,
        -6.8,
        JACK_O_LANTERN,
        Vec3::ZERO,
        0.7,
    ),
    decor(
        Module::PottedPlant,
        -5.7,
        2.8,
        PUMPKIN_SMALL,
        Vec3::ZERO,
        -0.5,
    ),
    decor(
        Module::PottedPlant,
        5.7,
        2.7,
        JACK_O_LANTERN,
        Vec3::ZERO,
        0.3,
    ),
    decor(
        Module::PottedPlant,
        -5.7,
        18.25,
        JACK_O_LANTERN,
        Vec3::ZERO,
        -0.4,
    ),
    decor(
        Module::PottedPlant,
        5.7,
        18.25,
        PUMPKIN_SMALL,
        Vec3::ZERO,
        0.2,
    ),
    decor(Module::SteelDrum, 13.2, -9.7, PUMPKIN_SMALL, DRUM_SPOT, 0.3),
    decor(
        Module::StorageCrate,
        2.5,
        -26.25,
        JACK_O_LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        PI,
    ),
    decor(
        Module::StorageCrate,
        12.5,
        -30.0,
        JACK_O_LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        PI,
    ),
    decor(
        Module::StorageCrate,
        12.0,
        -8.0,
        JACK_O_LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        PI,
    ),
    decor(
        Module::SteelDrum,
        12.6,
        11.8,
        JACK_O_LANTERN,
        DRUM_SPOT,
        -0.6,
    ),
    decor(
        Module::StorageCrate,
        -10.3,
        -22.9,
        PUMPKIN_SMALL,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.2,
    ),
    decor(
        Module::StorageCrate,
        11.0,
        6.2,
        PUMPKIN_SMALL,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        -0.3,
    ),
    decor(
        Module::StorageCrate,
        7.8,
        15.6,
        JACK_O_LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.5,
    ),
    decor(
        Module::ConceptTable,
        -1.5,
        12.5,
        SPECIMEN_JAR,
        Vec3::new(-0.4, 0.8, 0.0),
        0.3,
    ),
    decor(
        Module::ConceptTable,
        9.0,
        0.0,
        SPECIMEN_JAR,
        Vec3::new(-0.45, 0.8, 0.0),
        -0.2,
    ),
    decor(
        Module::StorageCrate,
        -7.0,
        4.4,
        LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.0,
    ),
    decor(
        Module::StorageCrate,
        -7.4,
        2.6,
        LANTERN,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        -7.5,
        13.75,
        CANDLE_CLUSTER,
        Vec3::new(-0.35, 0.8, 0.0),
        0.0,
    ),
    decor(
        Module::Workbench,
        2.2,
        -4.6,
        CANDLE_CLUSTER,
        Vec3::new(-0.35, 0.9, 0.0),
        0.0,
    ),
    decor(
        Module::Workbench,
        2.2,
        -4.6,
        SPECIMEN_JAR,
        Vec3::new(0.3, 0.9, 0.0),
        0.0,
    ),
    decor(
        Module::CeilingLightAmber,
        0.0,
        -2.5,
        LANTERN,
        Vec3::new(0.0, 1.9, -1.8),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        -2.5,
        -10.0,
        SPECIMEN_JAR,
        Vec3::new(0.5, 0.8, 0.0),
        0.2,
    ),
    decor(
        Module::ConceptTable,
        -7.5,
        -22.9,
        LANTERN,
        Vec3::new(0.5, 0.8, 0.0),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        6.8,
        -25.7,
        CANDLE_CLUSTER,
        Vec3::new(0.5, 0.8, 0.0),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        9.0,
        0.0,
        CANDLE_CLUSTER,
        Vec3::new(0.5, 0.8, 0.0),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        -1.5,
        12.5,
        LANTERN,
        Vec3::new(0.5, 0.8, 0.0),
        0.0,
    ),
    decor(
        Module::ClutterTools,
        -12.4,
        -11.5,
        LANTERN,
        Vec3::new(0.85, 0.0, 0.0),
        0.0,
    ),
    decor(
        Module::ClutterTools,
        12.3,
        -4.2,
        CANDLE_CLUSTER,
        Vec3::new(-0.85, 0.0, 0.0),
        0.0,
    ),
    decor(
        Module::ConceptTable,
        3.75,
        -18.75,
        SPECIMEN_JAR,
        Vec3::new(-0.42, 0.8, 0.1),
        0.0,
    ),
    decor(
        Module::ShelfUnit,
        13.35,
        -18.75,
        SPECIMEN_JAR,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitBins,
        8.1,
        -17.5,
        SPECIMEN_JAR,
        Vec3::new(-0.35, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::FileCabinetOpen,
        13.4,
        -26.25,
        SPECIMEN_JAR,
        Vec3::new(-0.18, 1.32, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        8.3,
        -23.5,
        LANTERN,
        Vec3::new(0.35, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitAlt,
        10.75,
        -12.5,
        SPECIMEN_JAR,
        Vec3::new(-0.35, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitBins,
        10.75,
        -14.8,
        LANTERN,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        13.15,
        -6.25,
        CANDLE_CLUSTER,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        13.15,
        -6.25,
        SPECIMEN_JAR,
        Vec3::new(-0.36, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        13.15,
        -6.25,
        LANTERN,
        Vec3::new(0.36, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::FileCabinetOpen,
        13.4,
        -26.25,
        LANTERN,
        Vec3::new(0.18, 1.32, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitBins,
        -9.5,
        -7.0,
        LANTERN,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        -13.1,
        -9.4,
        SPECIMEN_JAR,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        -12.8,
        -3.75,
        CANDLE_CLUSTER,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitAlt,
        -10.0,
        -5.7,
        SPECIMEN_JAR,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::StorageCrate,
        -7.4,
        2.6,
        PUMPKIN_SMALL,
        Vec3::new(0.0, CRATE_TOP, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        -13.3,
        -13.2,
        CANDLE_CLUSTER,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitBins,
        -7.5,
        9.7,
        SPECIMEN_JAR,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnit,
        -12.35,
        15.65,
        LANTERN,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        -2.5,
        -15.85,
        CANDLE_CLUSTER,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnit,
        9.0,
        4.25,
        SPECIMEN_JAR,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitBins,
        7.1,
        13.75,
        LANTERN,
        Vec3::new(0.0, 2.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        -7.5,
        18.38,
        CANDLE_CLUSTER,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
    decor(
        Module::ShelfUnitLow,
        7.5,
        18.38,
        LANTERN,
        Vec3::new(0.0, 1.005, 0.0),
        0.0,
    ),
];

const SIGN_TRIM_DECOR: Decor = sign_decor(SIGN_TRIM, Vec3::ZERO);
const SIGN_WEB_DECOR: [Decor; 2] = [
    sign_decor(SPIDER_WEB, Vec3::new(0.48, 0.02, -0.06)),
    sign_decor(SPIDER, Vec3::new(0.48, 0.01, -0.065)),
];

const fn sign_decor(module: &'static str, offset: Vec3) -> Decor {
    Decor {
        prop: None,
        anchor: Vec2::ZERO,
        module,
        offset,
        yaw: 0.0,
    }
}

fn webbed_sign(prop: Module, transform: &Transform) -> bool {
    transform.translation.y > 2.5
        && matches!(
            prop,
            Module::SignLabelBoilerRoom
                | Module::SignLabelLab
                | Module::SignLabelSecurity
                | Module::SignLabelPrep
        )
}

fn sign_label(prop: Module) -> bool {
    matches!(
        prop,
        Module::SignLabelBoilerRoom
            | Module::SignLabelStorage
            | Module::SignLabelMaintenance
            | Module::SignLabelUtility
            | Module::SignLabelOffice
            | Module::SignLabelLab
            | Module::SignLabelSecurity
            | Module::SignLabelPrep
            | Module::SignLabelExit
    )
}

#[derive(Component)]
pub struct HalloweenDecor;

#[derive(Component)]
struct MenuLanternCable;

pub(crate) struct DecorPlugin;

impl Plugin for DecorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(decorate_prop)
            .add_observer(replace_plant_visual)
            .add_systems(
                Update,
                (
                    decorate_preloaded_props.run_if(resource_added::<HalloweenAssets>),
                    add_menu_lantern_cable,
                ),
            );
    }
}

pub fn candle_light() -> impl Bundle {
    pumpkin_light(JACK_O_LANTERN)
}

fn pumpkin_light(module: &str) -> impl Bundle {
    let (height, intensity) = match module {
        LANTERN => (0.18, 2_300.0),
        CANDLE_CLUSTER => (0.21, 1_900.0),
        _ => (CANDLE_HEIGHT, CANDLE_INTENSITY),
    };
    (
        Name::new("halloween candle"),
        PointLight {
            color: CANDLE_COLOR,
            intensity,
            range: CANDLE_RANGE,
            radius: 0.04,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, height, 0.0),
    )
}

fn has_light(module: &str) -> bool {
    matches!(module, JACK_O_LANTERN | LANTERN | CANDLE_CLUSTER)
}

pub(crate) fn decor_for<'a>(
    prop: Module,
    transform: &Transform,
) -> impl Iterator<Item = &'static Decor> + 'a {
    let at = transform.translation.xz();
    DECOR.iter().filter(move |decor| {
        decor.prop == Some(prop) && decor.anchor.distance(at) <= ANCHOR_TOLERANCE
    })
}

fn decorate_preloaded_props(
    props: Query<(Entity, &Prop, &Transform, Option<&Children>)>,
    decorated: Query<(), With<HalloweenDecor>>,
    assets: Res<HalloweenAssets>,
    mut commands: Commands,
) {
    for (entity, prop, transform, children) in &props {
        if prop.0 == Module::PottedPlant
            || children
                .is_some_and(|children| children.iter().any(|child| decorated.contains(child)))
        {
            continue;
        }
        spawn_prop_decor(&mut commands, entity, prop, transform, &assets);
    }
}

fn decorate_prop(
    added: On<Add, Prop>,
    props: Query<(&Prop, &Transform)>,
    assets: Res<HalloweenAssets>,
    mut commands: Commands,
) {
    let Ok((prop, transform)) = props.get(added.entity) else {
        return;
    };
    if prop.0 == Module::PottedPlant {
        return;
    }
    spawn_prop_decor(&mut commands, added.entity, prop, transform, &assets);
}

fn spawn_prop_decor(
    commands: &mut Commands,
    entity: Entity,
    prop: &Prop,
    transform: &Transform,
    assets: &HalloweenAssets,
) {
    for decor in decor_for(prop.0, transform) {
        if let Some(module) = assets.module(decor.module) {
            spawn_decor(commands, entity, transform.rotation, decor, module.clone());
        }
    }
    if sign_label(prop.0) {
        if let Some(module) = assets.module(SIGN_TRIM) {
            spawn_decor(
                commands,
                entity,
                transform.rotation,
                &SIGN_TRIM_DECOR,
                module.clone(),
            );
        }
        if webbed_sign(prop.0, transform) {
            for decor in &SIGN_WEB_DECOR {
                if let Some(module) = assets.module(decor.module) {
                    spawn_decor(commands, entity, transform.rotation, decor, module.clone());
                }
            }
        }
    }
}

fn replace_plant_visual(
    added: On<Add, WorldAssetRoot>,
    roots: Query<&ChildOf, With<WorldAssetRoot>>,
    props: Query<(&Prop, &Transform)>,
    assets: Res<HalloweenAssets>,
    mut commands: Commands,
) {
    let Ok(parent) = roots.get(added.entity) else {
        return;
    };
    let Ok((prop, transform)) = props.get(parent.parent()) else {
        return;
    };
    if prop.0 != Module::PottedPlant {
        return;
    }
    if let Some(decor) = decor_for(prop.0, transform).next() {
        if let Some(module) = assets.module(decor.module) {
            commands.entity(added.entity).insert((
                Name::new(format!("halloween {}", decor.module)),
                HalloweenDecor,
                WorldAssetRoot(module.clone()),
                Transform::from_rotation(
                    transform.rotation.inverse() * Quat::from_rotation_y(decor.yaw),
                ),
            ));
            if has_light(decor.module) {
                commands
                    .entity(added.entity)
                    .with_child(pumpkin_light(decor.module));
            }
        }
    }
}

fn add_menu_lantern_cable(
    lanterns: Query<Entity, Added<MenuLanternCable>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for lantern in &lanterns {
        commands.entity(lantern).with_child((
            Name::new("halloween lantern suspension"),
            Mesh3d(meshes.add(Cylinder::new(0.008, 0.66))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.1, 0.08),
                metallic: 0.8,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.665, 0.0),
        ));
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
        Transform::from_translation(decor.offset).with_rotation(
            if decor.module == SIGN_TRIM || decor.module == SPIDER_WEB || decor.module == SPIDER {
                Quat::IDENTITY
            } else {
                prop_rotation.inverse() * Quat::from_rotation_y(decor.yaw)
            },
        ),
        Visibility::default(),
        ChildOf(prop),
    ));
    if decor.prop == Some(Module::CeilingLightAmber) {
        placed.insert(MenuLanternCable);
    }
    placed.with_child((WorldAssetRoot(module), Transform::IDENTITY));
    if has_light(decor.module) {
        placed.with_child(pumpkin_light(decor.module));
    }
}
