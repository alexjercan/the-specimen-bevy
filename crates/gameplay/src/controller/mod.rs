mod collision;
mod flashlight;
pub mod player;
mod stamina;
pub mod wasd_camera;

pub use collision::{door_frames, door_panel, wall_obstacles};
pub use flashlight::{Flashlight, FlashlightBeam, DRAIN_SECONDS, RECHARGE_SECONDS, RESTART_CHARGE};
pub use player::{
    PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, PlayerInput, LOOK_SENSITIVITY,
    PITCH_LIMIT as PLAYER_PITCH_LIMIT, RUN_SPEED, WALK_SPEED,
};
pub use stamina::{
    SprintExhausted, Stamina, DRAIN_SECONDS as SPRINT_DRAIN_SECONDS,
    RECHARGE_SECONDS as SPRINT_RECHARGE_SECONDS, RESTART_CHARGE as SPRINT_RESTART_CHARGE,
};

pub use wasd_camera::{
    ControllerState, WASDController, WASDControllerPlugin, DOWN_KEY, HOLD_KEY, LOOK_BUTTON,
    MOVE_SPEED, PITCH_LIMIT, RELEASE_KEY, RESUME_BUTTON, UP_KEY,
};
