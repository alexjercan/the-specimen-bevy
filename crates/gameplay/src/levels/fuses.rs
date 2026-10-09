use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use game_audio::{PlaySound, Sound};
use rand_core::Rng;

use crate::controller::player::{Interact, PlayerController, PlayerControlsEnabled};

use super::{
    doors::{DoorLock, ExitDoor, INTERACT_RANGE},
    hiding::Hidden,
    interaction::{InteractTarget, InteractTargets},
    monster::Caught,
    pickups::PickupPlugin,
};

pub const FUSE_COUNT: usize = 3;
pub const FUSE_MODULE: &str = "fuse_pickup";
pub(crate) const FUSE_RADIUS: f32 = 0.03;
pub(crate) const FUSE_LENGTH: f32 = 0.2;
pub(crate) const FUSE_PANEL_AIM_RADIUS: f32 = 0.5;

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseInventory(pub usize);

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Transform)]
pub struct FusePanel {
    pub installed: usize,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallFuses {
    pub player: Entity,
    pub panel: Entity,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseSeed(pub u64);

pub struct FusePlugin;

impl Plugin for FusePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PickupPlugin>() {
            app.add_plugins(PickupPlugin);
        }
        app.add_message::<InstallFuses>()
            .add_message::<PlaySound>()
            .add_observer(attach_inventory)
            .add_observer(use_panel)
            .add_systems(Update, install_fuses);
    }
}

fn attach_inventory(added: On<Add, PlayerController>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert_if_new(FuseInventory::default());
}

fn use_panel(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    players: Query<
        (Entity, &Transform, &FuseInventory, Option<&Hidden>),
        (With<PlayerController>, Without<Caught>),
    >,
    mut installs: MessageWriter<InstallFuses>,
) {
    if !enabled.0 {
        return;
    }
    for (player, transform, inventory, hidden) in &players {
        if let Some(InteractTarget::Panel(panel)) =
            targets.aimed(transform, Some(inventory), hidden)
        {
            installs.write(InstallFuses { player, panel });
        }
    }
}

fn install_fuses(
    mut installs: MessageReader<InstallFuses>,
    mut players: Query<&mut FuseInventory, Without<Caught>>,
    mut panels: Query<&mut FusePanel>,
    exits: Query<Entity, (With<ExitDoor>, With<DoorLock>)>,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    for install in installs.read() {
        let (Ok(mut inventory), Ok(mut panel)) = (
            players.get_mut(install.player),
            panels.get_mut(install.panel),
        ) else {
            continue;
        };
        if panel.installed != 0 || inventory.0 < FUSE_COUNT {
            continue;
        }
        inventory.0 -= FUSE_COUNT;
        panel.installed = FUSE_COUNT;
        sounds.write(PlaySound {
            sound: Sound::FuseComplete,
            position: None,
        });
        for door in &exits {
            commands.entity(door).remove::<DoorLock>();
        }
        info!("fuses installed; exit unlocked");
    }
}

pub(crate) fn run_seed(seed: Option<&FuseSeed>) -> u64 {
    seed.map(|seed| seed.0)
        .unwrap_or_else(|| ChaCha8Rng::default().next_u64())
}

pub(crate) fn fuse_panel_hit(origin: Vec3, forward: Vec3, center: Vec3) -> Option<f32> {
    aim_hit(origin, forward, center, FUSE_PANEL_AIM_RADIUS)
}

pub(crate) fn aim_hit(origin: Vec3, forward: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let offset = origin - center;
    let b = offset.dot(forward);
    let c = offset.length_squared() - radius * radius;
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
