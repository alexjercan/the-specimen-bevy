use bevy::prelude::*;
use game_assets::{GameAssetsState, UiAssets, KEY_GLYPHS};
use game_frontend::MenuAssets;

pub(crate) struct MenuAssetsPlugin;

impl Plugin for MenuAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), fill_menu_assets);
    }
}

fn fill_menu_assets(collection: Res<UiAssets>, mut commands: Commands) {
    commands.insert_resource(MenuAssets {
        font: collection.font.clone(),
        key_glyphs: KEY_GLYPHS
            .iter()
            .filter_map(|(key, _)| Some(((*key).to_owned(), collection.key_glyph(key)?)))
            .collect(),
        achievement_icons: collection
            .achievement_icons
            .iter()
            .map(|(stem, icon)| (stem.as_ref().to_owned(), icon.clone()))
            .collect(),
    });
}
