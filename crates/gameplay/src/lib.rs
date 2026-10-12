pub mod achievements;
pub mod ambience;
pub mod audible;
pub mod devices;
pub mod facility;
pub mod fuses;
pub mod hiding;
pub mod interaction;
pub mod monster;
pub mod player;
pub mod presets;

use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            presets::PresetsPlugin,
            player::PlayerPlugin,
            achievements::AchievementPlugin,
            facility::FacilityPlugin,
            interaction::InteractionPlugin,
            fuses::FusePlugin,
            hiding::HidingPlugin,
            ambience::AmbiencePlugin,
            monster::MonsterPlugin,
            devices::DevicePlugin,
        ));
    }
}

pub mod prelude {
    pub use crate::facility::{build_first_floor, Room};
    pub use crate::player::{PlayerController, PlayerInput, PlayerPlugin};
    pub use crate::GameplayPlugin;
}
