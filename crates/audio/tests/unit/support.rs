use bevy::{asset::uuid::Uuid, prelude::*};
use game_assets::SoundAssets;

/// Distinct placeholder handles, so tests can check which clip each cue uses.
pub(crate) fn sound_assets() -> SoundAssets {
    let mut next = 0;
    let mut handle = || {
        next += 1;
        Handle::<AudioSource>::Uuid(Uuid::from_u128(next), default())
    };
    SoundAssets {
        furnace: handle(),
        faucet: handle(),
        vent_wind: handle(),
        roomtone: handle(),
        low_pressure: handle(),
        conduit_roomtone: handle(),
        boiler_tick: handle(),
        power_down: handle(),
        breaker_trip: handle(),
        boiler_reset: handle(),
        boiler_restart: handle(),
        vent_hvac: handle(),
        tank_hum: handle(),
        cool_buzz: handle(),
        door_unlatch: handle(),
        door_swing: handle(),
        door_shut: handle(),
        door_locked: handle(),
        locker_open: handle(),
        locker_close: handle(),
        table_enter: handle(),
        table_leave: handle(),
        fuse_slot_1: handle(),
        fuse_slot_2: handle(),
        fuse_slot_3: handle(),
        fuse_complete: handle(),
        flashlight_click: handle(),
        sprint_exhausted: handle(),
        step_01: handle(),
        step_02: handle(),
        step_04: handle(),
        ui_back: handle(),
        ui_confirm: handle(),
        ui_denied: handle(),
        ui_focus: handle(),
        ui_hover: handle(),
        ui_pause: handle(),
        ui_press: handle(),
        ui_resume: handle(),
    }
}
