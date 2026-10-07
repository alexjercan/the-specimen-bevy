pub mod controller;
pub mod levels;

pub mod prelude {
    pub use crate::controller::{
        PlayerController, PlayerControllerPlugin, PlayerInput, WASDController, WASDControllerPlugin,
    };
    pub use crate::levels::{build_first_floor, Room};
}
