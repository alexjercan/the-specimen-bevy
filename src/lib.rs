pub use autopilot;
pub use capture;
pub use game_assets;
pub use game_core;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use game_assets::{FacilityAssets, GameAssetsPlugin, GameAssetsState};
    pub use game_core::AppBuilder;

    pub use autopilot::{frames, AutopilotPlugin};
    pub use capture::{
        loops::{loop_end, loop_start, loop_written_at},
        screenshot::{screenshot_start, screenshot_written_at},
        CapturePlugin,
    };
}
