use bevy::prelude::*;
use game_assets::{GameAssetsState, SoundAssets};
use game_frontend::MenuSounds;

pub(crate) struct SoundGluePlugin;

impl Plugin for SoundGluePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuSounds>()
            .add_systems(OnEnter(GameAssetsState::Ready), fill_menu_sounds);
    }
}

fn fill_menu_sounds(assets: Res<SoundAssets>, mut sounds: ResMut<MenuSounds>) {
    *sounds = MenuSounds {
        back: assets.ui_back.clone(),
        confirm: assets.ui_confirm.clone(),
        denied: assets.ui_denied.clone(),
        focus: assets.ui_focus.clone(),
        hover: assets.ui_hover.clone(),
        pause: assets.ui_pause.clone(),
        press: assets.ui_press.clone(),
        resume: assets.ui_resume.clone(),
        ambient: assets.menu_ambient.clone(),
    };
}
