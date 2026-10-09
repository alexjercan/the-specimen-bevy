use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

use crate::levels::fuses::{FusePickup, FUSE_COUNT};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FuseZone {
    Far,
    Middle,
    Exit,
}

pub struct FuseTable {
    pub room: &'static str,
    pub position: Vec3,
    pub yaw: f32,
    pub zone: FuseZone,
}

pub const FUSE_TABLES: [FuseTable; 6] = [
    FuseTable {
        room: "hiding",
        position: Vec3::new(13.0, 0.9, 0.0),
        yaw: PI + 0.35,
        zone: FuseZone::Far,
    },
    FuseTable {
        room: "office",
        position: Vec3::new(3.0, 0.9, -20.7),
        yaw: PI - 0.25,
        zone: FuseZone::Exit,
    },
    FuseTable {
        room: "maintenance",
        position: Vec3::new(-13.0, 0.9, -21.5),
        yaw: PI + 0.6,
        zone: FuseZone::Middle,
    },
    FuseTable {
        room: "utility",
        position: Vec3::new(-10.2, 0.9, -15.45),
        yaw: PI + 0.15,
        zone: FuseZone::Middle,
    },
    FuseTable {
        room: "security",
        position: Vec3::new(10.0, 0.9, -30.65),
        yaw: PI - 0.2,
        zone: FuseZone::Exit,
    },
    FuseTable {
        room: "prep",
        position: Vec3::new(7.0, 0.9, -7.3),
        yaw: PI + 0.2,
        zone: FuseZone::Far,
    },
];

pub fn select_fuse_slots(seed: u64) -> [usize; FUSE_COUNT] {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut slots = [0; FUSE_COUNT];
    for (index, zone) in [FuseZone::Far, FuseZone::Middle, FuseZone::Exit]
        .into_iter()
        .enumerate()
    {
        let candidates: Vec<_> = FUSE_TABLES
            .iter()
            .enumerate()
            .filter_map(|(slot, table)| (table.zone == zone).then_some(slot))
            .collect();
        assert!(
            !candidates.is_empty(),
            "fuse zone {zone:?} has no candidates"
        );
        let count = candidates.len() as u64;
        let limit = u64::MAX - u64::MAX % count;
        let pick = loop {
            let value = rng.next_u64();
            if value < limit {
                break (value % count) as usize;
            }
        };
        slots[index] = candidates[pick];
    }
    slots.sort_unstable();
    slots
}

pub(super) fn spawn(commands: &mut Commands, seed: u64) {
    info!("fuse seed: {seed}");
    for slot in select_fuse_slots(seed) {
        let table = &FUSE_TABLES[slot];
        commands.spawn((
            Name::new("fuse"),
            FusePickup { slot },
            Transform::from_translation(table.position)
                .with_rotation(Quat::from_rotation_y(table.yaw)),
        ));
    }
}
