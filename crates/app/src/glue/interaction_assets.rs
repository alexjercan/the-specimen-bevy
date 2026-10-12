use bevy::prelude::*;
use game_assets::{GameAssetsState, UiAssets};
use gameplay::interaction::InteractionAssets;

pub(crate) struct InteractionAssetsPlugin;

impl Plugin for InteractionAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), fill_interaction_assets);
    }
}

fn fill_interaction_assets(collection: Res<UiAssets>, mut commands: Commands) {
    commands.insert_resource(InteractionAssets {
        font: collection.font.clone(),
        interact_glyph: collection.key_glyph("KeyF").expect("KeyF glyph"),
    });
}
