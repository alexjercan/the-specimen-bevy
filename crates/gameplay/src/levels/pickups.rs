use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use game_audio::{PlaySound, Sound};

use crate::controller::player::{Interact, PlayerController, PlayerControlsEnabled};

use crate::achievements::{AchievementSignal, AchievementSignalKind};

use super::{
    devices::{Detector, Flashbangs},
    fuses::{aim_hit, FuseInventory, FUSE_COUNT, FUSE_RADIUS},
    hiding::Hidden,
    interaction::{InteractTarget, InteractTargets},
    monster::Caught,
};

const FUSE_AIM_RADIUS: f32 = 0.15;
const DEVICE_AIM_RADIUS: f32 = 0.2;
const DEVICE_CENTER_HEIGHT: f32 = 0.06;
const PICKUP_MOTION_SECS: f32 = 0.55;
const PICKUP_ARC_HEIGHT: f32 = 0.15;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[require(Transform, Visibility)]
pub enum PickupKind {
    Fuse { slot: usize },
    Flashbang,
    Detector,
}

impl PickupKind {
    pub fn center(self, transform: &Transform) -> Vec3 {
        let height = match self {
            PickupKind::Fuse { .. } => FUSE_RADIUS,
            PickupKind::Flashbang | PickupKind::Detector => DEVICE_CENTER_HEIGHT,
        };
        transform.translation + Vec3::Y * height
    }

    pub(crate) fn hit(self, origin: Vec3, forward: Vec3, center: Vec3) -> Option<f32> {
        let radius = match self {
            PickupKind::Fuse { .. } => FUSE_AIM_RADIUS,
            PickupKind::Flashbang | PickupKind::Detector => DEVICE_AIM_RADIUS,
        };
        aim_hit(origin, forward, center, radius)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PickupMotion {
    pub from: Vec3,
    pub elapsed: f32,
}

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .add_observer(pick_up)
            .add_systems(Update, animate_pickups);
    }
}

fn pickup_position(from: Vec3, hand: Vec3, elapsed: f32) -> Vec3 {
    let u = (elapsed / PICKUP_MOTION_SECS).clamp(0.0, 1.0);
    from.lerp(hand, u) + Vec3::Y * PICKUP_ARC_HEIGHT * (u * PI).sin()
}

fn animate_pickups(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    player: Query<&Transform, (With<PlayerController>, Without<PickupMotion>)>,
    mut pickups: Query<(Entity, &mut Transform, &mut PickupMotion)>,
    mut commands: Commands,
) {
    if !enabled.0 {
        return;
    }
    let Ok(player) = player.single() else { return };
    let hand = player.translation + player.forward() * 0.35 - player.up() * 0.22;
    for (entity, mut transform, mut motion) in &mut pickups {
        motion.elapsed += time.delta_secs();
        if motion.elapsed >= PICKUP_MOTION_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = pickup_position(motion.from, hand, motion.elapsed);
    }
}

fn pick_up(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    pickups: Query<(&PickupKind, &Transform)>,
    mut players: Query<
        (
            Entity,
            &Transform,
            Option<&mut FuseInventory>,
            Option<&Hidden>,
            Option<&mut Flashbangs>,
        ),
        (With<PlayerController>, Without<Caught>),
    >,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    if !enabled.0 {
        return;
    }
    let mut taken = Vec::new();
    for (player, transform, mut inventory, hidden, flashbangs) in &mut players {
        let Some(InteractTarget::Pickup(pickup)) =
            targets.aimed(transform, inventory.as_deref(), hidden)
        else {
            continue;
        };
        let Ok((&kind, pickup_transform)) = pickups.get(pickup) else {
            continue;
        };
        if taken.contains(&pickup) {
            continue;
        }
        match kind {
            PickupKind::Fuse { .. } => {
                let Some(inventory) = inventory.as_deref_mut() else {
                    continue;
                };
                if inventory.0 >= FUSE_COUNT {
                    continue;
                }
                inventory.0 += 1;
                sounds.write(PlaySound::global(Sound::FuseSlot(inventory.0)));
            }
            PickupKind::Flashbang => {
                commands.trigger(AchievementSignal {
                    player,
                    kind: AchievementSignalKind::PickedFlashbang,
                });
                match flashbangs {
                    Some(mut flashbangs) => flashbangs.0 += 1,
                    None => {
                        commands.entity(player).insert(Flashbangs(1));
                    }
                }
                sounds.write(PlaySound::global(Sound::FlashbangPickup));
            }
            PickupKind::Detector => {
                commands.trigger(AchievementSignal {
                    player,
                    kind: AchievementSignalKind::PickedDetector,
                });
                commands.entity(player).insert_if_new(Detector::default());
                sounds.write(PlaySound::global(Sound::DetectorPickup));
            }
        }
        taken.push(pickup);
        commands
            .entity(pickup)
            .remove::<PickupKind>()
            .insert(PickupMotion {
                from: pickup_transform.translation,
                elapsed: 0.0,
            });
    }
}
