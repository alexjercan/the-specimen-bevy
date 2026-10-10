use std::collections::HashMap;

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
    render::{BURST_LIFETIME, THROW_DISTANCE, THROW_GRAVITY},
};

pub const FLASHBANG_DURATION: f32 = 5.0;
pub const FLASHBANG_BURST_DELAY: f32 = 0.6;
pub const FLASHBANG_HIT_RADIUS: f32 = 4.0;
pub const FLASH_EXPOSURE_DURATION: f32 = 0.9;
pub const DETECTOR_RANGE: f32 = 25.0;
pub const PULSE_NEAR: f32 = 2.0;
pub const PULSE_FAST: f32 = 0.2;
pub const PULSE_SLOW: f32 = 1.6;

#[cfg(test)]
#[path = "../../tests/unit/thrown_flashbang.rs"]
mod tests;

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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct FlashExposure {
    pub remaining: f32,
}

impl Default for FlashExposure {
    fn default() -> Self {
        Self {
            remaining: FLASH_EXPOSURE_DURATION,
        }
    }
}

impl FlashExposure {
    pub fn elapsed(&self) -> f32 {
        FLASH_EXPOSURE_DURATION - self.remaining
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
pub(crate) struct ThrownFlashbang {
    pub owner: Entity,
    pub elapsed: f32,
    pub start: Vec3,
    pub velocity: Vec3,
    pub landing: Vec3,
    pub burst: bool,
}

impl ThrownFlashbang {
    pub fn new(owner: Entity, player: &Transform) -> Self {
        let start = player.translation + player.rotation * Vec3::new(0.18, -0.2, -0.35);
        let forward = (player.rotation * Vec3::NEG_Z).xz().normalize_or_zero();
        let landing = Vec3::new(
            player.translation.x + forward.x * THROW_DISTANCE,
            0.08,
            player.translation.z + forward.y * THROW_DISTANCE,
        );
        let gravity = Vec3::NEG_Y * THROW_GRAVITY;
        let velocity =
            (landing - start) / FLASHBANG_BURST_DELAY - gravity * (FLASHBANG_BURST_DELAY / 2.0);
        Self {
            owner,
            elapsed: 0.0,
            start,
            velocity,
            landing,
            burst: false,
        }
    }

    pub fn position(&self) -> Vec3 {
        let flight = self.elapsed.min(FLASHBANG_BURST_DELAY);
        if self.elapsed >= FLASHBANG_BURST_DELAY {
            self.landing
        } else {
            self.start + self.velocity * flight - Vec3::Y * (THROW_GRAVITY * flight * flight / 2.0)
        }
    }
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
                    (tick_flash, tick_exposure).before(apply_input),
                    tick_burst.after(tick_flash).after(tick_exposure),
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
        let thrown = ThrownFlashbang::new(player, transform);
        let mut projectile = commands.spawn((
            Name::new("Thrown flashbang"),
            super::LevelRoot,
            Transform::from_translation(thrown.start),
            thrown,
        ));
        if assets.is_some() {
            projectile.insert(Visibility::default());
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
    mut throws: Query<
        (Entity, &mut ThrownFlashbang, &mut Transform),
        (Without<Monster>, Without<PlayerController>),
    >,
    players: Query<
        (
            Has<Caught>,
            Has<Escaped>,
            Option<&Flashed>,
            Option<&FlashTargets>,
            &Transform,
        ),
        (With<PlayerController>, Without<ThrownFlashbang>),
    >,
    monsters: Query<
        (Entity, &Transform),
        (
            With<Monster>,
            Without<ThrownFlashbang>,
            Without<PlayerController>,
        ),
    >,
    sight: StructuralSight,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        for (entity, thrown, _) in &throws {
            if players.get(thrown.owner).is_err() {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    let mut hits: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (entity, mut thrown, mut pose) in &mut throws {
        if players.get(thrown.owner).is_err() {
            commands.entity(entity).despawn();
            continue;
        }
        thrown.elapsed += time.delta_secs();
        pose.translation = thrown.position();
        pose.rotation = Quat::from_rotation_x(thrown.elapsed.min(FLASHBANG_BURST_DELAY) * 9.0);
        if !thrown.burst && thrown.elapsed >= FLASHBANG_BURST_DELAY {
            thrown.burst = true;
            sounds.write(PlaySound {
                sound: Sound::FlashbangBurst,
                position: None,
            });
            let Ok((caught, escaped, flashed, old_targets, player_transform)) =
                players.get(thrown.owner)
            else {
                continue;
            };
            if caught || escaped {
                continue;
            }
            let player = thrown.owner;
            let origin = pose.translation.xz();
            let eye = player_transform.translation.xz();
            let towards_burst = origin - eye;
            let facing = (player_transform.rotation * Vec3::NEG_Z).xz();
            if towards_burst.length() <= FLASHBANG_HIT_RADIUS
                && facing
                    .normalize_or_zero()
                    .dot(towards_burst.normalize_or_zero())
                    >= 0.5
                && sight.clear_sight(eye, origin)
            {
                commands.entity(player).insert(FlashExposure::default());
            }
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
                let entry = hits.entry(player).or_insert_with(|| {
                    if flashed.is_some_and(|flashed| flashed.remaining > 0.0) {
                        old_targets.map_or_else(Vec::new, |targets| targets.0.clone())
                    } else {
                        Vec::new()
                    }
                });
                for target in targets {
                    if !entry.contains(&target) {
                        entry.push(target);
                    }
                }
                commands.trigger(AchievementSignal {
                    player,
                    kind: AchievementSignalKind::FlashbangHitMonster,
                });
            }
        }
        if thrown.elapsed >= FLASHBANG_BURST_DELAY + BURST_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
    for (player, targets) in hits {
        commands
            .entity(player)
            .insert((FlashTargets(targets), Flashed::default()));
    }
}

fn tick_exposure(
    time: Res<Time>,
    enabled: Option<Res<PlayerControlsEnabled>>,
    mut players: Query<(Entity, &mut FlashExposure)>,
    mut commands: Commands,
) {
    if enabled.is_some_and(|enabled| !enabled.0) {
        return;
    }
    for (player, mut exposure) in &mut players {
        exposure.remaining -= time.delta_secs();
        if exposure.remaining <= 0.0 {
            commands.entity(player).remove::<FlashExposure>();
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
