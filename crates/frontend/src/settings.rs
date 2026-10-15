#[cfg(not(target_arch = "wasm32"))]
use std::{
    thread,
    time::{Duration, Instant},
};

use bevy::{
    prelude::*,
    ui::UiScale,
    window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode},
};
use game_audio::Volumes;
#[cfg(not(target_arch = "wasm32"))]
use game_settings::FpsCap;
use game_settings::{DisplayMode, GameSettings, InputBinding, MsaaQuality};
use game_ui::FpsText;
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
        invert_y: settings.invert_y,
    }
}

pub fn volumes(settings: &GameSettings) -> Volumes {
    Volumes {
        master: settings.master,
        world: settings.sfx,
        ui: settings.sfx,
        music: settings.music,
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

pub fn msaa(quality: MsaaQuality) -> Msaa {
    match quality {
        MsaaQuality::Off => Msaa::Off,
        MsaaQuality::Sample2 => Msaa::Sample2,
        MsaaQuality::Sample4 => Msaa::Sample4,
        MsaaQuality::Sample8 => Msaa::Sample8,
    }
}

// NOTE: stacked cameras must share MSAA to share Bevy's render texture; runtime settings sync updates live cameras and new cameras take saved quality.
pub struct CameraQualityPlugin;

impl Plugin for CameraQualityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiScale>()
            .add_observer(quality_for_new_cameras)
            .add_systems(Update, sync_runtime_settings);
        // NOTE: Web builds skip native frame pacing; the browser controls frame scheduling.
        #[cfg(not(target_arch = "wasm32"))]
        app.insert_resource(FramePacing {
            started: Instant::now(),
        })
        .add_systems(First, start_frame)
        .add_systems(Last, limit_frame);
    }
}

fn quality_for_new_cameras(
    added: On<Add, Camera>,
    settings: Res<GameSettings>,
    mut commands: Commands,
) {
    commands.entity(added.entity).insert(msaa(settings.msaa));
}

fn sync_runtime_settings(
    settings: Res<GameSettings>,
    mut synced: Local<bool>,
    mut controller: ResMut<ControllerSettings>,
    mut audio: ResMut<Volumes>,
    mut ui_scale: ResMut<UiScale>,
    window: Option<Single<&mut Window, With<PrimaryWindow>>>,
    mut cameras: Query<(Entity, Option<&mut Msaa>), With<Camera>>,
    mut fps_texts: Query<&mut Visibility, With<FpsText>>,
    mut commands: Commands,
) {
    if *synced && !settings.is_changed() {
        return;
    }
    *synced = true;
    *controller = controller_settings(&settings);
    *audio = volumes(&settings);
    ui_scale.0 = settings.ui_scale;
    let quality = msaa(settings.msaa);
    for (entity, camera) in &mut cameras {
        if let Some(mut camera) = camera {
            camera.set_if_neq(quality);
        } else {
            commands.entity(entity).insert(quality);
        }
    }
    if let Some(mut window) = window {
        let mode = window_mode(settings.display_mode);
        if window.mode != mode {
            window.mode = mode;
        }
        let present_mode = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != present_mode {
            window.present_mode = present_mode;
        }
    }
    let visibility = if settings.fps_overlay {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut text in &mut fps_texts {
        text.set_if_neq(visibility);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct FramePacing {
    started: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
fn start_frame(mut pacing: ResMut<FramePacing>) {
    pacing.started = Instant::now();
}

#[cfg(not(target_arch = "wasm32"))]
fn limit_frame(
    settings: Res<GameSettings>,
    pacing: Res<FramePacing>,
    windows: Query<(), With<PrimaryWindow>>,
) {
    if windows.is_empty() {
        return;
    }
    let Some(period) = frame_period(settings.fps_cap) else {
        return;
    };
    thread::sleep(period.saturating_sub(pacing.started.elapsed()));
}

#[cfg(not(target_arch = "wasm32"))]
fn frame_period(cap: FpsCap) -> Option<Duration> {
    let rate = match cap {
        FpsCap::Uncapped => return None,
        FpsCap::Fps30 => 30,
        FpsCap::Fps60 => 30 * 2,
        FpsCap::Fps120 => 30 * 4,
    };
    Some(Duration::from_secs(1) / rate)
}

fn input(binding: InputBinding) -> Input {
    match binding {
        InputBinding::Key(key) => Input::Key(key),
        InputBinding::Mouse(button) => Input::Mouse(button),
    }
}
