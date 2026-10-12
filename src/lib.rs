pub use autopilot;
pub use capture;
pub use game_app;
pub use game_assets;
pub use game_flow;
pub use game_frontend;
#[cfg(feature = "halloween")]
pub use game_halloween;
pub use gameplay;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use game_app::{AppBuilder, FrameStep, Frontend};
    pub use game_assets::{FacilityAssets, GameAssetsPlugin, GameAssetsState};
    pub use game_flow::CoreState;
    pub use gameplay::prelude::*;

    pub use autopilot::{frames, AutopilotPlugin};
    pub use capture::{
        screenshot::{screenshot_start, screenshot_written_at},
        video::{video_end, video_start, video_written_at, VideoProfile},
        CapturePlugin,
    };
}
