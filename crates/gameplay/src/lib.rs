pub mod facility;

pub mod prelude {
    pub use crate::facility::{
        render::FacilityRenderPlugin, DoorState, Facility, FacilityPlugin, Objective, PlayerStart,
    };
}
