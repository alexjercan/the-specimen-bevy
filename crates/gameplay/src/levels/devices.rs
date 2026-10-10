use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use game_assets::FacilityAssets;
use game_audio::{PlaySound, Sound};

use crate::{
    achievements::{AchievementSignal, AchievementSignalKind},
    controller::player::{apply_input, PlayerController, PlayerControlsEnabled, UseFlashbang},
};

use super::{
    interaction::StructuralSight,
    monster::{Caught, Monster},
    objective::Escaped,
    pickups::PickupPlugin,
    render::THROW_DISTANCE,
};

pub const FLASHBANG_DURATION: f32 = 5.0;
pub const FLASHBANG_BURST_DELAY: f32 = 0.6;
pub const FLASHBANG_HIT_RADIUS: f32 = 4.0;
pub const DETECTOR_RANGE: f32 = 25.0;
pub const PULSE_NEAR: f32 = 2.0;
pub const PULSE_FAST: f32 = 0.2;
pub const PULSE_SLOW: f32 = 1.6;

#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component)]
pub struct Flashbangs(pub usize);

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Flashed {
    pub remaining: f32,
}

impl Default for Flashed {
    fn default() -> Self {
        Self {
            remaining: FLASHBANG_DURATION,
        }
    }
}

impl Flashed {
    pub fn elapsed(&self) -> f32 {
        FLASHBANG_DURATION - self.remaining
    }
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
#[require(PulseClock)]
pub struct Detector {
    pub reading: Option<DetectorReading>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectorReading {
    pub distance: f32,
    pub bearing: f32,
}

#[derive(Component, Default)]
pub(crate) struct PulseClock(f32);

#[derive(Component, Clone, Debug, Default)]
pub(crate) struct FlashTargets(pub Vec<Entity>);

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct PendingBurst(f32);

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct ThrownFlashbang {
    pub elapsed: f32,
    pub landing_y: f32,
}

pub struct DevicePlugin;

impl Plugin for DevicePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PickupPlugin>() {
            app.add_plugins(PickupPlugin);
        }
        app.register_type::<Flashbangs>()
            .add_message::<PlaySound>()
            .add_observer(use_flashbang)
            .add_systems(
                Update,
                (
                    tick_flash.before(apply_input),
                    tick_burst.after(tick_flash),
                    (read_detector, pulse_detector).chain().after(apply_input),
                ),
            );
    }
}

fn use_flashbang(
    _: On<Start<UseFlashbang>>,
    enabled: Res<PlayerControlsEnabled>,
    mut players: Query<
        (Entity, &Transform, &mut Flashbangs),
        (With<PlayerController>, Without<Caught>, Without<Escaped>),
    >,
    mut sounds: MessageWriter<PlaySound>,
    assets: Option<Res<FacilityAssets>>,
    mut commands: Commands,
) {
    if !enabled.0 {
        return;
    }
    for (player, transform, mut flashbangs) in &mut players {
        if flashbangs.0 == 0 {
            continue;
        }
        flashbangs.0 -= 1;
        commands
            .entity(player)
            .remove::<FlashTargets>()
            .insert((Flashed::default(), PendingBurst(0.0)));
        if assets.is_some() {
            commands.entity(player).with_children(|children| {
                children.spawn((
                    Name::new("Thrown flashbang"),
                    ThrownFlashbang {
                        elapsed: 0.0,
                        landing_y: 0.08 - transform.translation.y,
                    },
                    Transform::from_xyz(0.18, -0.2, -0.35),
                    Visibility::default(),
                ));
            });
        }
        sounds.write(PlaySound {
            sound: Sound::FlashbangThrow,
            position: None,
        });
    }
}

pub(super) fn tick_burst(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut players: Query<(Entity, &Transform, &mut PendingBurst, Option<&Caught>), Without<Monster>>,
    monsters: Query<(Entity, &Transform), (With<Monster>, Without<PlayerController>)>,
    sight: StructuralSight,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (player, pose, mut burst, caught) in &mut players {
        burst.0 += time.delta_secs();
        if burst.0 >= FLASHBANG_BURST_DELAY {
            sounds.write(PlaySound {
                sound: Sound::FlashbangBurst,
                position: None,
            });
            if caught.is_some() {
                commands.entity(player).remove::<PendingBurst>();
                continue;
            }
            let origin =
                (pose.translation + pose.rotation * Vec3::new(0.0, 0.0, -THROW_DISTANCE)).xz();
            let targets: Vec<_> = monsters
                .iter()
                .filter_map(|(monster, transform)| {
                    let target = transform.translation.xz();
                    (origin.distance(target) <= FLASHBANG_HIT_RADIUS
                        && sight.clear_sight(origin, target))
                    .then_some(monster)
                })
                .collect();
            if !targets.is_empty() {
                commands.entity(player).insert(FlashTargets(targets));
                commands.trigger(AchievementSignal {
                    player,
                    kind: AchievementSignalKind::FlashbangHitMonster,
                });
            }
            commands.entity(player).remove::<PendingBurst>();
        }
    }
}

pub(crate) fn tick_flash(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut players: Query<(Entity, &mut Flashed)>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (player, mut flashed) in &mut players {
        flashed.remaining -= time.delta_secs();
        if flashed.remaining <= 0.0 {
            commands.entity(player).remove::<(Flashed, FlashTargets)>();
        }
    }
}

pub(crate) fn unseen(
    flashed: Option<&Flashed>,
    targets: Option<&FlashTargets>,
    monster: Entity,
) -> bool {
    flashed.is_some_and(|flashed| flashed.remaining > 0.0)
        && targets.is_some_and(|targets| targets.0.contains(&monster))
}

pub fn detector_reading(player: &Transform, monster: Vec3) -> Option<DetectorReading> {
    let offset = (monster - player.translation).xz();
    let distance = offset.length();
    if distance > DETECTOR_RANGE {
        return None;
    }
    let (yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
    let local = Quat::from_rotation_y(-yaw) * Vec3::new(offset.x, 0.0, offset.y);
    Some(DetectorReading {
        distance,
        bearing: local.x.atan2(-local.z),
    })
}

fn read_detector(
    mut players: Query<(&Transform, &mut Detector), With<PlayerController>>,
    monsters: Query<&Transform, (With<Monster>, Without<PlayerController>)>,
) {
    for (player, mut detector) in &mut players {
        let reading = monsters
            .iter()
            .filter_map(|monster| detector_reading(player, monster.translation))
            .min_by(|a, b| a.distance.total_cmp(&b.distance));
        detector.set_if_neq(Detector { reading });
    }
}

pub fn pulse_interval(distance: f32) -> f32 {
    let t = ((distance - PULSE_NEAR) / (DETECTOR_RANGE - PULSE_NEAR)).clamp(0.0, 1.0);
    PULSE_FAST + (PULSE_SLOW - PULSE_FAST) * t
}

fn pulse_detector(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut players: Query<(&Detector, &mut PulseClock), (With<PlayerController>, Without<Caught>)>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (detector, mut clock) in &mut players {
        let Some(reading) = detector.reading else {
            clock.0 = 0.0;
            continue;
        };
        clock.0 += time.delta_secs();
        if clock.0 >= pulse_interval(reading.distance) {
            clock.0 = 0.0;
            sounds.write(PlaySound {
                sound: Sound::DetectorNearby,
                position: None,
            });
        }
    }
}
