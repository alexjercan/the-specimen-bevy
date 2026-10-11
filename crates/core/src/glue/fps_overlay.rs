use bevy::prelude::*;
use game_settings::GameSettings;
use game_ui::FpsText;

pub(crate) struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_fps_visibility);
    }
}

fn sync_fps_visibility(
    settings: Res<GameSettings>,
    mut texts: Query<&mut Visibility, With<FpsText>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut visibility in &mut texts {
        *visibility = if settings.fps_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
