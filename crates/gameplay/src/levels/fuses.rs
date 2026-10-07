use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::controller::player::{Interact, PlayerController, PlayerControlsEnabled};

use super::{
    doors::INTERACT_RANGE,
    interaction::{InteractTarget, InteractTargets},
};

pub const FUSE_COUNT: usize = 3;
pub const FUSE_MODULE: &str = "fuse_pickup";
pub(crate) const FUSE_RADIUS: f32 = 0.03;
pub(crate) const FUSE_LENGTH: f32 = 0.2;
pub(crate) const FUSE_AIM_RADIUS: f32 = 0.15;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub struct FusePickup {
    pub slot: usize,
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseInventory(pub usize);

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseSeed(pub u64);

pub struct FusePlugin;

impl Plugin for FusePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_inventory).add_observer(pick_up);
    }
}

fn attach_inventory(added: On<Add, PlayerController>, mut commands: Commands) {
    commands.entity(added.entity).insert_if_new(FuseInventory::default());
}

fn pick_up(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    mut players: Query<(&Transform, &mut FuseInventory), With<PlayerController>>,
    mut commands: Commands,
) {
    if !enabled.0 {
        return;
    }
    let mut taken = Vec::new();
    for (player, mut inventory) in &mut players {
        let Some(InteractTarget::Fuse(fuse)) = targets.aimed(player) else {
            continue;
        };
        if taken.contains(&fuse) || inventory.0 >= FUSE_COUNT {
            continue;
        }
        taken.push(fuse);
        inventory.0 += 1;
        commands.entity(fuse).despawn();
    }
}

pub fn select_fuse_slots(seed: u64, pool: usize) -> [usize; FUSE_COUNT] {
    assert!(pool >= FUSE_COUNT, "fuse pool is smaller than {FUSE_COUNT}");
    let mut state = seed;
    let mut indices: Vec<usize> = (0..pool).collect();
    let mut slots = [0; FUSE_COUNT];
    for (index, slot) in slots.iter_mut().enumerate() {
        let pick = index + (splitmix(&mut state) % (pool - index) as u64) as usize;
        indices.swap(index, pick);
        *slot = indices[index];
    }
    slots.sort_unstable();
    slots
}

pub(crate) fn run_seed(seed: Option<&FuseSeed>) -> u64 {
    seed.map(|seed| seed.0).unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or_default()
    })
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(crate) fn fuse_center(transform: &Transform) -> Vec3 {
    transform.translation + Vec3::Y * FUSE_RADIUS
}

pub(crate) fn fuse_hit(origin: Vec3, forward: Vec3, center: Vec3) -> Option<f32> {
    let offset = origin - center;
    let b = offset.dot(forward);
    let c = offset.length_squared() - FUSE_AIM_RADIUS * FUSE_AIM_RADIUS;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    if root - b < 0.0 {
        return None;
    }
    let distance = (-b - root).max(0.0);
    (distance <= INTERACT_RANGE).then_some(distance)
}
