use bevy::prelude::*;
use game_assets::{GameAssetsState, SoundAssets};
use gameplay::{
    ambience::AmbienceSounds, devices::DeviceSounds, facility::FacilitySounds, fuses::FuseSounds,
    hiding::HidingSounds, interaction::InteractionSounds, monster::MonsterSounds,
    player::PlayerSounds,
};

pub(crate) struct FeatureSoundsPlugin;

impl Plugin for FeatureSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameAssetsState::Ready),
            (
                fill_facility_sounds,
                fill_player_sounds,
                fill_monster_sounds,
                fill_device_sounds,
                fill_interaction_sounds,
                fill_fuse_sounds,
                fill_hiding_sounds,
                fill_ambience_sounds,
            ),
        );
    }
}

fn fill_facility_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(FacilitySounds {
        door_unlatch: assets.door_unlatch.clone(),
        door_swing: assets.door_swing.clone(),
        door_shut: assets.door_shut.clone(),
        door_locked: assets.door_locked.clone(),
        power_down: assets.power_down.clone(),
        breaker_trip: assets.breaker_trip.clone(),
        boiler_reset: assets.boiler_reset.clone(),
        boiler_restart: assets.boiler_restart.clone(),
    });
}

fn fill_player_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(PlayerSounds {
        steps: [
            assets.step_01.clone(),
            assets.step_02.clone(),
            assets.step_04.clone(),
        ],
        sprint_exhausted: assets.sprint_exhausted.clone(),
        flashlight_click: assets.flashlight_click.clone(),
    });
}

fn fill_monster_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(MonsterSounds {
        presence: assets.monster_presence.clone(),
        steps: [
            assets.monster_step_a.clone(),
            assets.monster_step_b.clone(),
            assets.monster_step_c.clone(),
            assets.monster_step_d.clone(),
            assets.monster_step_e.clone(),
            assets.monster_step_f.clone(),
        ],
        detected: assets.monster_detected.clone(),
        attack: assets.monster_attack.clone(),
        heartbeat: assets.monster_heartbeat.clone(),
    });
}

fn fill_device_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(DeviceSounds {
        detector_nearby: assets.detector_nearby.clone(),
        flashbang_throw: assets.flashbang_throw.clone(),
        flashbang_burst: assets.flashbang_burst.clone(),
    });
}

fn fill_interaction_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(InteractionSounds {
        fuse_slots: [
            assets.fuse_slot_1.clone(),
            assets.fuse_slot_2.clone(),
            assets.fuse_slot_3.clone(),
        ],
        flashbang_pickup: assets.flashbang_pickup.clone(),
        detector_pickup: assets.detector_pickup.clone(),
    });
}

fn fill_fuse_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(FuseSounds {
        complete: assets.fuse_complete.clone(),
    });
}

fn fill_hiding_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(HidingSounds {
        locker_open: assets.locker_open.clone(),
        locker_close: assets.locker_close.clone(),
        table_enter: assets.table_enter.clone(),
        table_leave: assets.table_leave.clone(),
    });
}

fn fill_ambience_sounds(assets: Res<SoundAssets>, mut commands: Commands) {
    commands.insert_resource(AmbienceSounds {
        roomtone: assets.roomtone.clone(),
        low_pressure: assets.low_pressure.clone(),
        conduit: assets.conduit_roomtone.clone(),
        boiler: assets.furnace.clone(),
        tank: assets.tank_hum.clone(),
        vent: assets.vent_hvac.clone(),
        vent_wind: assets.vent_wind.clone(),
        cool_buzz: assets.cool_buzz.clone(),
        boiler_tick: assets.boiler_tick.clone(),
        faucet_burst: assets.faucet.clone(),
    });
}
