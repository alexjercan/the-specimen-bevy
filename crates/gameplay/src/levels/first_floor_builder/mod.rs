mod fuse_tables;
mod lights;
mod props;
mod rooms;
mod signs;

use bevy::prelude::*;

use super::{
    fuses::{run_seed, FuseSeed},
    power::FacilityPower,
};

pub use fuse_tables::{FuseTable, FUSE_TABLES};

pub fn build_first_floor(mut commands: Commands, seed: Option<Res<FuseSeed>>) {
    rooms::spawn(&mut commands);
    lights::spawn(&mut commands);
    props::spawn_fixtures(&mut commands);
    signs::spawn(&mut commands);
    props::spawn_furniture(&mut commands);
    let seed = run_seed(seed.as_deref());
    fuse_tables::spawn(&mut commands, seed);
    commands.insert_resource(FacilityPower::new(seed));
}
