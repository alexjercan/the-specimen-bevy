use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use game_audio::{PlaySound, PlaySourceSound, Sound, SourceSounds};
use rand_core::{Rng, SeedableRng};

use crate::controller::player::{Interact, PlayerController, PlayerControlsEnabled};

use super::{
    fuses::FuseInventory,
    hiding::Hidden,
    interaction::{InteractTarget, InteractTargets},
    sounds::IntermittentSound,
};

pub const MIN_OUTAGE_DELAY_SECS: f32 = 30.0;
pub const MAX_OUTAGE_DELAY_SECS: f32 = 90.0;
const OUTAGE_SEED_SALT: u64 = 0x49b3_756a_bec2_102f;
const OUTAGE_CYCLE_MIX: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct FacilityPower {
    pub on: bool,
    pub outage_pending: bool,
    pub remaining_secs: f32,
    seed: u64,
    cycles: u64,
}

impl FacilityPower {
    pub fn new(seed: u64) -> Self {
        Self {
            on: true,
            outage_pending: true,
            remaining_secs: Self::delay(seed, 0),
            seed,
            cycles: 0,
        }
    }

    fn delay(seed: u64, cycle: u64) -> f32 {
        let mut rng = ChaCha8Rng::seed_from_u64(
            seed ^ OUTAGE_SEED_SALT ^ cycle.wrapping_mul(OUTAGE_CYCLE_MIX),
        );
        MIN_OUTAGE_DELAY_SECS
            + (MAX_OUTAGE_DELAY_SECS - MIN_OUTAGE_DELAY_SECS)
                * (rng.next_u32() as f64 / (u32::MAX as f64 + 1.0)) as f32
    }

    pub fn tick(&mut self, seconds: f32) {
        if !self.outage_pending {
            return;
        }
        self.remaining_secs = (self.remaining_secs - seconds.max(0.0)).max(0.0);
        if self.remaining_secs == 0.0 {
            self.outage();
        }
    }

    pub fn outage(&mut self) {
        if self.outage_pending {
            self.outage_pending = false;
            self.on = false;
        }
    }

    pub fn restore(&mut self) {
        if self.on {
            return;
        }
        self.on = true;
        self.cycles += 1;
        self.remaining_secs = Self::delay(self.seed, self.cycles);
        self.outage_pending = true;
    }
}

#[derive(Message)]
struct RepairBoiler(Entity);

pub struct FacilityPowerPlugin;

impl Plugin for FacilityPowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .add_message::<PlaySourceSound>()
            .add_message::<RepairBoiler>()
            .add_observer(repair_boiler)
            .add_systems(Update, (advance_outage, process_repairs).chain());
    }
}

fn advance_outage(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    players: Query<(), With<PlayerController>>,
    boilers: Query<(Entity, &SourceSounds)>,
    mut power: Option<ResMut<FacilityPower>>,
    mut sounds: MessageWriter<PlaySound>,
    mut source_sounds: MessageWriter<PlaySourceSound>,
) {
    let Some(power) = power.as_deref_mut() else {
        return;
    };
    if !power.outage_pending || !enabled.0 || players.is_empty() {
        return;
    }
    power.tick(time.delta_secs());
    if !power.on {
        if let Some((source, _)) = boilers.iter().find(|(_, bindings)| {
            bindings
                .0
                .iter()
                .any(|(sound, _)| *sound == Sound::BreakerTrip)
        }) {
            source_sounds.write(PlaySourceSound {
                source,
                sound: Sound::BreakerTrip,
            });
        }
        sounds.write(PlaySound {
            sound: Sound::PowerDown,
            position: None,
        });
    }
}

fn repair_boiler(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    players: Query<(&Transform, Option<&FuseInventory>, Option<&Hidden>), With<PlayerController>>,
    power: Option<Res<FacilityPower>>,
    mut repairs: MessageWriter<RepairBoiler>,
) {
    if !enabled.0 || !power.is_some_and(|power| !power.on) {
        return;
    }
    if let Some(entity) = players.iter().find_map(|(player, inventory, hidden)| {
        match targets.aimed(player, inventory, hidden)? {
            InteractTarget::Boiler(entity) => Some(entity),
            _ => None,
        }
    }) {
        repairs.write(RepairBoiler(entity));
    }
}

fn process_repairs(
    mut repairs: MessageReader<RepairBoiler>,
    mut power: Option<ResMut<FacilityPower>>,
    boilers: Query<(), With<IntermittentSound>>,
    mut sounds: MessageWriter<PlaySourceSound>,
) {
    for repair in repairs.read() {
        if let (Some(power), Ok(())) = (power.as_deref_mut(), boilers.get(repair.0)) {
            if !power.on {
                power.restore();
                sounds.write(PlaySourceSound {
                    source: repair.0,
                    sound: Sound::BoilerReset,
                });
                sounds.write(PlaySourceSound {
                    source: repair.0,
                    sound: Sound::BoilerRestart,
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/facility_power.rs"]
mod tests;
