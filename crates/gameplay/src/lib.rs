pub mod controller;
pub mod levels;

pub mod prelude {
    pub use crate::controller::{ControllerPlugin, PlayerController};
    pub use crate::levels::{build_first_floor, Room};
}
