mod collision;
mod flashlight;
pub mod player;
pub mod wasd_camera;

pub use flashlight::{Flashlight, FlashlightBeam, DRAIN_SECONDS, RECHARGE_SECONDS, RESTART_CHARGE};
pub use player::{
    PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, PlayerInput, LOOK_SENSITIVITY,
    PITCH_LIMIT as PLAYER_PITCH_LIMIT, RUN_SPEED, WALK_SPEED,
};

pub use wasd_camera::{
    ControllerState, WASDController, WASDControllerPlugin, DOWN_KEY, HOLD_KEY, LOOK_BUTTON,
    MOVE_SPEED, PITCH_LIMIT, RELEASE_KEY, RESUME_BUTTON, UP_KEY,
};
