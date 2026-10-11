mod decor;
mod eyes;
mod fuse;
mod tint;

use bevy::{prelude::*, world_serialization::WorldAsset};
use bevy_asset_loader::prelude::*;
use game_assets::GameAssetsState;

pub use decor::{
    candle_light, spawn_decor, Decor, HalloweenDecor, DECOR, JACK_O_LANTERN, PUMPKIN_SMALL,
};
pub use eyes::{EyeHalo, HalloweenEyes, MonsterEyesPlugin, EYE_COLOR, MONSTER_EYES_MESH};
pub use fuse::{CandyFuse, CANDY_FUSE};

pub const HALLOWEEN_MODULES: [&str; 3] = [CANDY_FUSE, JACK_O_LANTERN, PUMPKIN_SMALL];

#[derive(AssetCollection, Resource)]
pub struct HalloweenAssets {
    #[asset(path = "halloween/candy_fuse.glb#Scene0")]
    pub candy_fuse: Handle<WorldAsset>,
    #[asset(path = "halloween/jack_o_lantern.glb#Scene0")]
    pub jack_o_lantern: Handle<WorldAsset>,
    #[asset(path = "halloween/pumpkin_small.glb#Scene0")]
    pub pumpkin_small: Handle<WorldAsset>,
}

impl HalloweenAssets {
    pub fn module(&self, name: &str) -> Option<&Handle<WorldAsset>> {
        match name {
            CANDY_FUSE => Some(&self.candy_fuse),
            JACK_O_LANTERN => Some(&self.jack_o_lantern),
            PUMPKIN_SMALL => Some(&self.pumpkin_small),
            _ => None,
        }
    }
}

pub struct HalloweenPlugin;

impl Plugin for HalloweenPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameAssetsState::Loading).load_collection::<HalloweenAssets>(),
        )
        .add_plugins((
            MonsterEyesPlugin,
            fuse::CandyFusePlugin,
            decor::DecorPlugin,
            tint::TintPlugin,
        ));
    }
}
