pub(crate) mod collision;
mod flashlight;
pub mod player;
mod settings;
mod stamina;

pub use collision::{door_frames, door_panel, wall_obstacles};
pub use flashlight::{Flashlight, FlashlightBeam, DRAIN_SECONDS, RECHARGE_SECONDS, RESTART_CHARGE};
pub use player::{
    PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, PlayerInput,
    PlayerPresentationPlugin, PITCH_LIMIT as PLAYER_PITCH_LIMIT, RUN_SPEED, WALK_SPEED,
};
pub use settings::{ControllerSettings, KeyBindings};
pub use stamina::{
    SprintExhausted, Stamina, DRAIN_SECONDS as SPRINT_DRAIN_SECONDS,
    RECHARGE_SECONDS as SPRINT_RECHARGE_SECONDS, RESTART_CHARGE as SPRINT_RESTART_CHARGE,
};
