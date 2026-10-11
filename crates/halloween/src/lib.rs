mod decor;
mod eyes;
mod fuse;
mod lighting;

use bevy::{prelude::*, world_serialization::WorldAsset};
use bevy_asset_loader::prelude::*;
use game_assets::GameAssetsState;

pub use decor::{
    candle_light, spawn_decor, Decor, HalloweenDecor, CANDLE_CLUSTER, DECOR, JACK_O_LANTERN,
    LANTERN, PUMPKIN_SMALL, SIGN_TRIM, SPECIMEN_JAR, SPIDER, SPIDER_WEB,
};
pub use eyes::{EyeHalo, HalloweenEyes, MonsterEyesPlugin, EYE_COLOR, MONSTER_EYES_MESH};
pub use fuse::{CandyFuse, CANDY_FUSE};

pub const HALLOWEEN_MODULES: [&str; 9] = [
    CANDY_FUSE,
    JACK_O_LANTERN,
    PUMPKIN_SMALL,
    SPECIMEN_JAR,
    LANTERN,
    CANDLE_CLUSTER,
    SPIDER_WEB,
    SPIDER,
    SIGN_TRIM,
];

#[derive(AssetCollection, Resource)]
pub struct HalloweenAssets {
    #[asset(path = "halloween/candy_fuse.glb#Scene0")]
    pub candy_fuse: Handle<WorldAsset>,
    #[asset(path = "halloween/jack_o_lantern.glb#Scene0")]
    pub jack_o_lantern: Handle<WorldAsset>,
    #[asset(path = "halloween/pumpkin_small.glb#Scene0")]
    pub pumpkin_small: Handle<WorldAsset>,
    #[asset(path = "halloween/specimen_jar.glb#Scene0")]
    pub specimen_jar: Handle<WorldAsset>,
    #[asset(path = "halloween/lantern.glb#Scene0")]
    pub lantern: Handle<WorldAsset>,
    #[asset(path = "halloween/candle_cluster.glb#Scene0")]
    pub candle_cluster: Handle<WorldAsset>,
    #[asset(path = "halloween/spider_web.glb#Scene0")]
    pub spider_web: Handle<WorldAsset>,
    #[asset(path = "halloween/spider.glb#Scene0")]
    pub spider: Handle<WorldAsset>,
    #[asset(path = "halloween/sign_trim.glb#Scene0")]
    pub sign_trim: Handle<WorldAsset>,
}

impl HalloweenAssets {
    pub fn module(&self, name: &str) -> Option<&Handle<WorldAsset>> {
        match name {
            CANDY_FUSE => Some(&self.candy_fuse),
            JACK_O_LANTERN => Some(&self.jack_o_lantern),
            PUMPKIN_SMALL => Some(&self.pumpkin_small),
            SPECIMEN_JAR => Some(&self.specimen_jar),
            LANTERN => Some(&self.lantern),
            CANDLE_CLUSTER => Some(&self.candle_cluster),
            SPIDER_WEB => Some(&self.spider_web),
            SPIDER => Some(&self.spider),
            SIGN_TRIM => Some(&self.sign_trim),
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
            lighting::HalloweenLightingPlugin,
        ));
    }
}
