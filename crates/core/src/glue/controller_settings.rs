use bevy::prelude::*;
use game_settings::GameSettings;
use gameplay::controller::PlayerControllerSettings;

pub(crate) struct PlayerControllerSettingsGluePlugin;

impl Plugin for PlayerControllerSettingsGluePlugin {
    fn build(&self, app: &mut App) {
        let look_sensitivity = app.world().resource::<GameSettings>().mouse_sensitivity;
        app.insert_resource(PlayerControllerSettings { look_sensitivity })
            .add_systems(PreUpdate, sync_player_controller_settings);
    }
}

fn sync_player_controller_settings(
    settings: Res<GameSettings>,
    mut controller: ResMut<PlayerControllerSettings>,
) {
    if settings.is_changed() && controller.look_sensitivity != settings.mouse_sensitivity {
        controller.look_sensitivity = settings.mouse_sensitivity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_sensitivity_tracks_loaded_and_changed_settings() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut settings = GameSettings::default();
        settings.mouse_sensitivity = 0.003;
        app.insert_resource(settings)
            .add_plugins(PlayerControllerSettingsGluePlugin);
        assert_eq!(
            app.world()
                .resource::<PlayerControllerSettings>()
                .look_sensitivity,
            0.003
        );
        app.update();
        app.world_mut()
            .resource_mut::<GameSettings>()
            .mouse_sensitivity = 0.004;
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerControllerSettings>()
                .look_sensitivity,
            0.004
        );
    }
}
