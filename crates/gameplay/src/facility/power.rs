use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

use crate::{
    audible::{Audible, Origin},
    fuses::FuseInventory,
    hiding::Hidden,
    interaction::{InteractTarget, InteractTargets},
    monster::Caught,
    player::{Interact, PlayerController, PlayerControlsEnabled},
};

use game_audio::Falloff;

use super::{builder::Prop, menu_background::MenuBackground, module::Module};

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum PowerCue {
    PowerDown,
    BreakerTrip { boiler: Entity },
    BoilerReset { boiler: Entity },
    BoilerRestart { boiler: Entity },
}

impl Audible for PowerCue {
    fn name(&self) -> Option<&'static str> {
        Some(match self {
            Self::PowerDown => "power_down",
            Self::BreakerTrip { .. } => "breaker_trip",
            Self::BoilerReset { .. } => "boiler_reset",
            Self::BoilerRestart { .. } => "boiler_restart",
        })
    }

    fn origin(&self) -> Origin {
        match *self {
            Self::PowerDown => Origin::Listener,
            Self::BreakerTrip { boiler }
            | Self::BoilerReset { boiler }
            | Self::BoilerRestart { boiler } => Origin::From(boiler),
        }
    }

    fn falloff(&self) -> Falloff {
        Falloff::DEFAULT
    }
}

pub const MIN_OUTAGE_DELAY_SECS: f32 = 30.0;
pub const MAX_OUTAGE_DELAY_SECS: f32 = 90.0;
const OUTAGE_SEED_SALT: u64 = 0x49b3_756a_bec2_102f;
const OUTAGE_CYCLE_MIX: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerState {
    On { until_outage: Option<f32> },
    Tripped,
    Blackout,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct FacilityPower {
    state: PowerState,
    seed: u64,
    cycles: u64,
}

impl Default for FacilityPower {
    fn default() -> Self {
        Self {
            state: PowerState::On { until_outage: None },
            seed: 0,
            cycles: 0,
        }
    }
}

impl FacilityPower {
    pub fn new(seed: u64) -> Self {
        Self {
            state: PowerState::On {
                until_outage: Some(Self::delay(seed, 0)),
            },
            seed,
            cycles: 0,
        }
    }

    pub fn blackout(seed: u64) -> Self {
        Self {
            state: PowerState::Blackout,
            seed,
            cycles: 0,
        }
    }

    pub fn state(&self) -> PowerState {
        self.state
    }

    pub fn is_on(&self) -> bool {
        matches!(self.state, PowerState::On { .. })
    }

    pub fn repairable(&self) -> bool {
        self.state == PowerState::Tripped
    }

    pub fn outage_in(&self) -> Option<f32> {
        match self.state {
            PowerState::On { until_outage } => until_outage,
            PowerState::Tripped | PowerState::Blackout => None,
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
        let PowerState::On {
            until_outage: Some(remaining),
        } = self.state
        else {
            return;
        };
        let remaining = (remaining - seconds.max(0.0)).max(0.0);
        self.state = if remaining == 0.0 {
            PowerState::Tripped
        } else {
            PowerState::On {
                until_outage: Some(remaining),
            }
        };
    }

    pub fn outage(&mut self) {
        if let PowerState::On {
            until_outage: Some(_),
        } = self.state
        {
            self.state = PowerState::Tripped;
        }
    }

    pub fn restore(&mut self) {
        if self.is_on() {
            return;
        }
        self.cycles += 1;
        self.state = PowerState::On {
            until_outage: Some(Self::delay(self.seed, self.cycles)),
        };
    }
}

#[derive(Message)]
struct RepairBoiler(Entity);

#[derive(Component)]
pub struct Boiler;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainsPowered;

fn tag_boiler(added: On<Add<Prop>>, props: Query<&Prop>, mut commands: Commands) {
    if props
        .get(added.entity)
        .is_ok_and(|prop| prop.0 == Module::BoilerUnit)
    {
        commands.entity(added.entity).insert(Boiler);
    }
}

fn tag_mains_powered(
    added: On<Add<Prop>>,
    props: Query<(&Prop, Option<&ChildOf>)>,
    menu_roots: Query<(), With<MenuBackground>>,
    mut commands: Commands,
) {
    let Ok((prop, parent)) = props.get(added.entity) else {
        return;
    };
    let menu = parent.is_some_and(|parent| menu_roots.contains(parent.parent()));
    if prop.0.mains_powered() && !menu {
        commands.entity(added.entity).insert(MainsPowered);
    }
}

pub struct FacilityPowerPlugin;

impl Plugin for FacilityPowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FacilityPower>()
            .add_message::<RepairBoiler>()
            .add_message::<PowerCue>()
            .add_observer(tag_boiler)
            .add_observer(tag_mains_powered)
            .add_observer(repair_boiler)
            .add_systems(Update, (advance_outage, process_repairs).chain());
    }
}

fn advance_outage(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    players: Query<(), (With<PlayerController>, Without<Caught>)>,
    boilers: Query<Entity, With<Boiler>>,
    mut power: ResMut<FacilityPower>,
    mut cues: MessageWriter<PowerCue>,
) {
    if power.outage_in().is_none() || !enabled.0 || players.is_empty() {
        return;
    }
    power.tick(time.delta_secs());
    if !power.is_on() {
        if let Some(boiler) = boilers.iter().next() {
            cues.write(PowerCue::BreakerTrip { boiler });
        }
        cues.write(PowerCue::PowerDown);
    }
}

fn repair_boiler(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    players: Query<
        (&Transform, Option<&FuseInventory>, Option<&Hidden>),
        (With<PlayerController>, Without<Caught>),
    >,
    power: Res<FacilityPower>,
    mut repairs: MessageWriter<RepairBoiler>,
) {
    if !enabled.0 || !power.repairable() {
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
    mut power: ResMut<FacilityPower>,
    boilers: Query<(), With<Boiler>>,
    mut cues: MessageWriter<PowerCue>,
) {
    for repair in repairs.read() {
        if !boilers.contains(repair.0) || !power.repairable() {
            continue;
        }
        power.restore();
        cues.write(PowerCue::BoilerReset { boiler: repair.0 });
        cues.write(PowerCue::BoilerRestart { boiler: repair.0 });
    }
}
