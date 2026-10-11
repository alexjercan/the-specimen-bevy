pub mod achievements;
pub mod controller;
pub mod levels;
pub mod presets;

use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<presets::GamePreset>().add_plugins((
            controller::PlayerControllerPlugin,
            achievements::AchievementPlugin,
            levels::DoorPlugin,
            levels::PickupPlugin,
            levels::FusePlugin,
            levels::HidingPlugin,
            levels::ObjectivePlugin,
            levels::FacilityPowerPlugin,
            levels::PropSoundsPlugin,
            levels::PropLightsPlugin,
            levels::MonsterPlugin,
            levels::DevicePlugin,
        ));
    }
}

pub mod prelude {
    pub use crate::controller::{PlayerController, PlayerControllerPlugin, PlayerInput};
    pub use crate::levels::{build_first_floor, Room};
    pub use crate::GameplayPlugin;
}
