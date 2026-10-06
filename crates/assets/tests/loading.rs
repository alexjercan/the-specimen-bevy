use bevy::{asset::AssetPlugin, prelude::*, state::app::StatesPlugin};
use game_assets::{GameAssetsPlugin, GameAssetsState};

#[test]
fn unloadable_mandatory_asset_enters_failed_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: "assets/this-facility-does-not-exist".into(),
            ..default()
        })
        .add_plugins(GameAssetsPlugin);

    for _ in 0..1000 {
        app.update();
        if *app.world().resource::<State<GameAssetsState>>().get() == GameAssetsState::Failed {
            return;
        }
        std::thread::yield_now();
    }
    panic!("unloadable facility assets did not enter Failed");
}
