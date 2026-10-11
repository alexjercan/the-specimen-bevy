use bevy::prelude::*;
use bevy_enhanced_input::prelude::Binding;
use game_settings::{GameSettings, InputBinding};
use gameplay::controller::{ControllerSettings, KeyBindings};

pub(crate) struct ControllerSettingsPlugin;

impl Plugin for ControllerSettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = controller_settings(app.world().resource::<GameSettings>());
        app.insert_resource(settings).add_systems(
            PreUpdate,
            sync_controller_settings.run_if(resource_changed::<GameSettings>),
        );
    }
}

fn sync_controller_settings(
    settings: Res<GameSettings>,
    mut controller: ResMut<ControllerSettings>,
) {
    controller.set_if_neq(controller_settings(&settings));
}

fn controller_settings(settings: &GameSettings) -> ControllerSettings {
    let keys = settings.keys;
    ControllerSettings {
        keys: KeyBindings {
            forward: keys.forward,
            left: keys.left,
            backward: keys.backward,
            right: keys.right,
            sprint: keys.sprint,
            interact: keys.interact,
            flashlight: binding(keys.flashlight),
            flashbang: binding(keys.flashbang),
        },
        mouse_sensitivity: settings.mouse_sensitivity,
    }
}

fn binding(binding: InputBinding) -> Binding {
    match binding {
        InputBinding::Key(key) => key.into(),
        InputBinding::Mouse(button) => button.into(),
    }
}
