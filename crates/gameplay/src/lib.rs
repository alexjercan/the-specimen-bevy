pub mod controller;
pub mod facility;

pub mod prelude {
    pub use crate::controller::{ControllerPlugin, PlayerController};
    pub use crate::facility::{
        render::FacilityRenderPlugin, DoorState, Facility, FacilityPlugin, Objective, PlayerStart,
    };
}
