#![allow(dead_code)]

use bevy::{
    asset::{uuid::Uuid, AssetPath},
    platform::collections::HashMap,
    prelude::*,
    state::app::StatesPlugin,
    world_serialization::WorldAsset,
};
use bevy_asset_loader::{
    loading_state::{LoadingState, LoadingStateAppExt},
    mapped::{AssetFileStem, MapKey},
};
use game_assets::{FacilityAssets, GameAssetsState};
use game_halloween::{HalloweenAssets, HalloweenPlugin};
use gameplay::{
    levels::{FuseSeed, FUSE_MODULE},
    presets::GamePreset,
};

pub fn world_asset(n: u128) -> Handle<WorldAsset> {
    Handle::Uuid(Uuid::from_u128(n), default())
}

pub fn fuse_module() -> Handle<WorldAsset> {
    world_asset(1)
}

pub fn halloween_assets() -> HalloweenAssets {
    HalloweenAssets {
        candy_fuse: world_asset(101),
        jack_o_lantern: world_asset(102),
        pumpkin_small: world_asset(103),
        specimen_jar: world_asset(104),
        lantern: world_asset(105),
        candle_cluster: world_asset(106),
        spider_web: world_asset(107),
        spider: world_asset(108),
        sign_trim: world_asset(109),
    }
}

fn facility_assets() -> FacilityAssets {
    let path = AssetPath::from(format!("facility/modules/{FUSE_MODULE}.glb"));
    FacilityAssets {
        modules: HashMap::from_iter([(AssetFileStem::from_asset_path(&path), fuse_module())]),
    }
}

pub fn level_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FuseSeed(7))
        .insert_resource(GamePreset::default());
    app
}

pub fn halloween_app() -> App {
    let mut app = level_app();
    app.add_plugins((AssetPlugin::default(), StatesPlugin))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .insert_state(GameAssetsState::Ready)
        .add_loading_state(LoadingState::new(GameAssetsState::Loading))
        .insert_resource(facility_assets())
        .insert_resource(halloween_assets())
        .add_plugins(HalloweenPlugin);
    app
}
