mod fuse_tables;
mod lights;
mod props;
mod rooms;
mod signs;

use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use rand_core::{Rng, SeedableRng};

use crate::{
    controller::{PlayerController, Stamina},
    presets::GamePreset,
};

use super::{
    fuses::{run_seed, FuseSeed},
    monster::{Monster, PatrolRng},
    power::FacilityPower,
};

const EYE_HEIGHT: f32 = 1.6;
const MONSTER_SEED_SALT: u64 = 0x4d4f_4e53_5445_5201;

pub use fuse_tables::{select_fuse_slots, FuseTable, FuseZone, FUSE_TABLES, FUSE_ZONES};

pub fn build_first_floor(
    mut commands: Commands,
    seed: Option<Res<FuseSeed>>,
    preset: Option<Res<GamePreset>>,
) {
    let preset = preset.as_deref().copied().unwrap_or_default();
    rooms::spawn(&mut commands);
    lights::spawn(&mut commands);
    props::spawn_fixtures(&mut commands);
    signs::spawn(&mut commands);
    props::spawn_furniture(&mut commands);
    props::spawn_dressing(&mut commands);
    if preset.flashbangs {
        props::spawn_flashbang(&mut commands);
    }
    if preset.detector {
        props::spawn_detector(&mut commands);
    }
    let seed = run_seed(seed.as_deref());
    fuse_tables::spawn(&mut commands, seed);
    commands.insert_resource(if preset.blackout {
        FacilityPower::blackout(seed)
    } else {
        FacilityPower::new(seed)
    });
}

pub fn spawn_first_floor_actors(
    mut commands: Commands,
    seed: Option<Res<FuseSeed>>,
    preset: Option<Res<GamePreset>>,
    mut global: Option<Single<&mut ChaCha8Rng, With<GlobalRng>>>,
) {
    let preset = preset.as_deref().copied().unwrap_or_default();
    let seed = seed
        .map(|seed| seed.0)
        .or_else(|| global.as_mut().map(|rng| rng.next_u64()))
        .unwrap_or_else(|| run_seed(None));
    commands.insert_resource(PatrolRng(ChaCha8Rng::seed_from_u64(
        seed ^ MONSTER_SEED_SALT,
    )));
    commands.spawn((
        Name::new("Human Deer"),
        Monster::default(),
        Transform::from_xyz(-2.5, 0.0, 0.0),
        Visibility::default(),
    ));
    commands.spawn((
        Name::new("Player"),
        PlayerController,
        if preset.sprint {
            Stamina::default()
        } else {
            Stamina::spent()
        },
        Transform::from_xyz(2.5, EYE_HEIGHT, -27.5),
    ));
}
