use std::f32::consts::PI;

use bevy::prelude::*;

use crate::levels::fuses::{select_fuse_slots, FusePickup};

pub struct FuseTable {
    pub room: &'static str,
    pub position: Vec3,
    pub yaw: f32,
}

pub const FUSE_TABLES: [FuseTable; 5] = [
    FuseTable {
        room: "hiding",
        position: Vec3::new(8.8, 0.8, 0.0),
        yaw: PI + 0.35,
    },
    FuseTable {
        room: "office",
        position: Vec3::new(3.45, 0.8, -18.85),
        yaw: PI - 0.25,
    },
    FuseTable {
        room: "maintenance",
        position: Vec3::new(-7.2, 0.8, -22.9),
        yaw: PI + 0.6,
    },
    FuseTable {
        room: "utility",
        position: Vec3::new(-10.2, 0.9, -15.45),
        yaw: PI + 0.15,
    },
    FuseTable {
        room: "security",
        position: Vec3::new(9.8, 0.9, -29.95),
        yaw: PI - 0.2,
    },
];

pub(super) fn spawn(commands: &mut Commands, seed: u64) {
    info!("fuse seed: {seed}");
    for slot in select_fuse_slots(seed, FUSE_TABLES.len()) {
        let table = &FUSE_TABLES[slot];
        commands.spawn((
            Name::new("fuse"),
            FusePickup { slot },
            Transform::from_translation(table.position)
                .with_rotation(Quat::from_rotation_y(table.yaw)),
        ));
    }
}
