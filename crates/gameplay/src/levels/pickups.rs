use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use game_assets::FacilityAssets;
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
        app.add_message::<PlaySound>().add_observer(pick_up);
    }
}

fn pick_up(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    pickups: Query<(&PickupKind, &Transform)>,
    assets: Option<Res<FacilityAssets>>,
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
                sounds.write(PlaySound {
                    sound: Sound::FuseSlot(inventory.0),
                    position: None,
                });
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
                sounds.write(PlaySound {
                    sound: Sound::FlashbangPickup,
                    position: None,
                });
            }
            PickupKind::Detector => {
                commands.trigger(AchievementSignal {
                    player,
                    kind: AchievementSignalKind::PickedDetector,
                });
                commands.entity(player).insert_if_new(Detector::default());
                sounds.write(PlaySound {
                    sound: Sound::DetectorPickup,
                    position: None,
                });
            }
        }
        taken.push(pickup);
        if assets.is_some() {
            commands
                .entity(pickup)
                .remove::<PickupKind>()
                .insert(PickupMotion {
                    from: pickup_transform.translation,
                    elapsed: 0.0,
                });
        } else {
            commands.entity(pickup).despawn();
        }
    }
}
