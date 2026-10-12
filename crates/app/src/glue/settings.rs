use bevy::prelude::*;
use game_frontend::{controller_settings, volumes};
use game_settings::GameSettings;

pub(crate) struct SettingsGluePlugin;

impl Plugin for SettingsGluePlugin {
    fn build(&self, app: &mut App) {
        let settings = app.world().resource::<GameSettings>();
        let controller = controller_settings(settings);
        let volumes = volumes(settings);
        app.insert_resource(controller).insert_resource(volumes);
    }
}
