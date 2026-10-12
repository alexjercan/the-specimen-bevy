use bevy::prelude::*;
use game_assets::{GameAssetsState, MonsterAssets as MonsterCollection};
use gameplay::monster::MonsterAssets;

pub(crate) struct MonsterAssetsPlugin;

impl Plugin for MonsterAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), fill_monster_assets);
    }
}

fn fill_monster_assets(collection: Res<MonsterCollection>, mut commands: Commands) {
    commands.insert_resource(MonsterAssets {
        scene: collection.scene.clone(),
        idle: collection.idle.clone(),
        walk: collection.walk.clone(),
        chase: collection.chase.clone(),
        attack: collection.attack.clone(),
    });
}
