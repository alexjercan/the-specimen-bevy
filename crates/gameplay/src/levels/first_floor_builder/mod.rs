mod fuse_tables;
mod lights;
mod props;
mod rooms;
mod signs;

use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, GlobalRng};
use rand_core::{Rng, SeedableRng};

use crate::controller::PlayerController;

use super::{
    fuses::{run_seed, FuseSeed},
    monster::{Monster, PatrolRng},
    power::FacilityPower,
};

const EYE_HEIGHT: f32 = 1.6;
const MONSTER_SEED_SALT: u64 = 0x4d4f_4e53_5445_5201;

pub use fuse_tables::{select_fuse_slots, FuseTable, FuseZone, FUSE_TABLES, FUSE_ZONES};

pub fn build_first_floor(mut commands: Commands, seed: Option<Res<FuseSeed>>) {
    rooms::spawn(&mut commands);
    lights::spawn(&mut commands);
    props::spawn_fixtures(&mut commands);
    signs::spawn(&mut commands);
    props::spawn_furniture(&mut commands);
    props::spawn_devices(&mut commands);
    let seed = run_seed(seed.as_deref());
    fuse_tables::spawn(&mut commands, seed);
    commands.insert_resource(FacilityPower::new(seed));
}

pub fn spawn_first_floor_actors(
    mut commands: Commands,
    seed: Option<Res<FuseSeed>>,
    mut global: Option<Single<&mut ChaCha8Rng, With<GlobalRng>>>,
) {
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
        Transform::from_xyz(2.5, EYE_HEIGHT, -27.5),
    ));
}
