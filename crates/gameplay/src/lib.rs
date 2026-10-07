pub mod controller;
pub mod facility;
pub mod levels;

pub mod prelude {
    pub use crate::controller::{ControllerPlugin, PlayerController};
    pub use crate::facility::{DoorState, Facility, FacilityPlugin, Objective, PlayerStart};
    pub use crate::levels::first_floor;
}
