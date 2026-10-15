use bevy::{
    prelude::*,
    window::{MonitorSelection, WindowMode},
};
use game_audio::Volumes;
use game_settings::{DisplayMode, GameSettings, GraphicsQuality, InputBinding};
use gameplay::player::{ControllerSettings, Input, KeyBindings};

pub fn controller_settings(settings: &GameSettings) -> ControllerSettings {
    let keys = settings.keys;
    ControllerSettings {
        keys: KeyBindings {
            forward: keys.forward,
            left: keys.left,
            backward: keys.backward,
            right: keys.right,
            sprint: keys.sprint,
            interact: keys.interact,
            flashlight: input(keys.flashlight),
            flashbang: input(keys.flashbang),
        },
        mouse_sensitivity: settings.mouse_sensitivity,
    }
}

pub fn volumes(settings: &GameSettings) -> Volumes {
    Volumes {
        master: settings.master,
        world: settings.sfx,
        ui: settings.sfx,
    }
}

pub fn window_mode(mode: DisplayMode) -> WindowMode {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = mode;
        WindowMode::Windowed
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match mode {
            DisplayMode::Windowed => WindowMode::Windowed,
            DisplayMode::Fullscreen => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        }
    }
}

pub fn msaa(quality: GraphicsQuality) -> Msaa {
    match quality {
        GraphicsQuality::Low => Msaa::Off,
        GraphicsQuality::Medium => Msaa::Sample2,
        GraphicsQuality::High => Msaa::Sample4,
    }
}

// NOTE: stacked cameras must share MSAA to share Bevy's render texture; new cameras take saved quality and the settings panel rewrites live cameras.
pub struct CameraQualityPlugin;

impl Plugin for CameraQualityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(quality_for_new_cameras);
    }
}

fn quality_for_new_cameras(
    added: On<Add, Camera>,
    settings: Res<GameSettings>,
    mut commands: Commands,
) {
    commands
        .entity(added.entity)
        .insert(msaa(settings.graphics));
}

fn input(binding: InputBinding) -> Input {
    match binding {
        InputBinding::Key(key) => Input::Key(key),
        InputBinding::Mouse(button) => Input::Mouse(button),
    }
}
