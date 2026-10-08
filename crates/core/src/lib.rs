mod glue;
mod menu;

use std::{path::PathBuf, time::Duration};

#[cfg(feature = "debug")]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::{
    app::ScheduleRunnerPlugin,
    audio::{GlobalVolume, Volume},
    input::InputPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::prelude::{ChaCha8Rng, EntropyPlugin};
use game_assets::GameAssetsState;
use game_settings::{GameSettings, GameSettingsPlugin, GraphicsQuality};
#[cfg(feature = "debug")]
use gameplay::controller::PlayerControlsEnabled;
use gameplay::{
    controller::PlayerController,
    levels::{build_first_floor, FacilityPower, FuseSeed, LightIntensity},
};

pub use menu::{GameState, MenuPlugin, PauseState};

#[cfg(all(test, feature = "debug"))]
#[path = "../tests/unit/debug_controls.rs"]
mod debug_controls_tests;

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";
const EYE_HEIGHT: f32 = 1.6;
const WINDOWED_AMBIENT_BRIGHTNESS: f32 = 6.0;
const WINDOWED_LIGHT_SCALE: f32 = 0.45;

#[derive(States, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CoreState {
    #[default]
    Loading,
    Ready,
}

type MainPlugin = Box<dyn FnOnce(&mut App) + Send + Sync>;

#[derive(Default)]
pub struct AppBuilder {
    headless: bool,
    transport: bool,
    menu: bool,
    seed: Option<u64>,
    recording: Option<PathBuf>,
    muted_audio: bool,
    main_plugin: Option<MainPlugin>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn headless() -> Self {
        Self {
            headless: true,
            ..default()
        }
    }

    pub fn with_transport(mut self) -> Self {
        self.transport = true;
        self
    }

    pub fn with_menu(mut self) -> Self {
        self.menu = true;
        self
    }

    pub fn with_recording(mut self, path: PathBuf) -> Self {
        self.recording = Some(path);
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_muted_audio(mut self) -> Self {
        self.muted_audio = true;
        self
    }

    pub fn with_main_plugin(mut self, plugin: impl Plugin) -> Self {
        self.main_plugin = Some(Box::new(|app| {
            app.add_plugins(plugin);
        }));
        self
    }

    fn menu_enabled(&self) -> bool {
        self.menu && !self.headless && !self.transport && self.main_plugin.is_none()
    }

    fn facility_lighting_enabled(&self) -> bool {
        !self.headless && self.main_plugin.is_none()
    }

    pub fn build(self) -> App {
        assert!(self.recording.is_none() || (!self.headless && self.transport));
        let menu = self.menu_enabled();
        let facility_lighting = self.facility_lighting_enabled();
        let mut app = App::new();
        let logging = LogPlugin {
            level: Level::INFO,
            filter: LOG_FILTER.into(),
            ..default()
        };
        app.add_plugins(GameSettingsPlugin { persist: menu });
        if self.headless {
            app.add_plugins(
                MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
            )
            .add_plugins(logging)
            .add_plugins((InputPlugin, StatesPlugin))
            .insert_state(CoreState::Ready)
            .add_plugins(EnhancedInputPlugin)
            .add_plugins(gameplay::controller::PlayerControllerPlugin::default().without_camera())
            .add_plugins((
                gameplay::levels::DoorPlugin,
                gameplay::levels::FusePlugin,
                gameplay::levels::HidingPlugin,
                gameplay::levels::ObjectivePlugin,
                gameplay::levels::FacilityPowerPlugin,
                gameplay::levels::PropSoundsPlugin,
                gameplay::levels::PropLightsPlugin,
            ));
            if self.transport {
                app.add_plugins(transport::TransportPlugin);
            }
        } else {
            app.add_plugins(DefaultPlugins.set(logging));
            if self.muted_audio {
                app.insert_resource(GlobalVolume::new(Volume::Linear(0.0)));
            }
            if let Some(seed) = self.seed {
                let mut bytes = [0; 32];
                bytes[..8].copy_from_slice(&seed.to_le_bytes());
                app.add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed(bytes));
            } else {
                app.add_plugins(EntropyPlugin::<ChaCha8Rng>::default());
            }
            app.init_state::<CoreState>()
                .add_plugins((
                    game_assets::GameAssetsPlugin,
                    gameplay::levels::LevelRenderPlugin,
                ))
                .add_systems(OnEnter(GameAssetsState::Ready), core_ready)
                .add_systems(OnEnter(GameAssetsState::Failed), core_failed)
                .add_plugins(EnhancedInputPlugin)
                .add_plugins(gameplay::controller::PlayerControllerPlugin::default())
                .add_plugins((
                    gameplay::levels::DoorPlugin,
                    gameplay::levels::FusePlugin,
                    gameplay::levels::HidingPlugin,
                    gameplay::levels::ObjectivePlugin,
                    gameplay::levels::FacilityPowerPlugin,
                    gameplay::levels::PropSoundsPlugin,
                    gameplay::levels::PropLightsPlugin,
                ))
                .add_plugins((
                    glue::InteractionHintPlugin,
                    glue::FuseHudPlugin,
                    glue::FlashlightHudPlugin,
                    glue::StaminaHudPlugin,
                ))
                .add_plugins((game_audio::GameAudioPlugin, glue::SoundGluePlugin));
            if let Some(path) = self.recording {
                app.insert_resource(transport::RecordTransport(path))
                    .add_plugins(capture::CapturePlugin::new(60));
            }
            if self.transport {
                app.add_plugins(transport::RenderedTransportPlugin);
            }
            #[cfg(feature = "debug")]
            app.add_plugins(debug::DebugPlugin)
                .add_systems(PostUpdate, sync_debug_controls);
            if facility_lighting {
                app.insert_resource(GlobalAmbientLight {
                    color: Color::WHITE,
                    brightness: WINDOWED_AMBIENT_BRIGHTNESS,
                    ..default()
                })
                .add_systems(
                    Update,
                    (dim_new_lights, dim_ambient_on_outage, apply_graphics),
                );
            }
        }
        if self.transport {
            app.add_systems(OnEnter(CoreState::Ready), transport_ready);
        }
        if let Some(seed) = self.seed {
            app.insert_resource(FuseSeed(seed));
        }
        match self.main_plugin {
            Some(main_plugin) => main_plugin(&mut app),
            None if menu => {
                app.add_plugins(menu::MenuPlugin);
            }
            None => {
                app.add_plugins(GamePlugin);
            }
        }
        app
    }
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_controller)
            .add_systems(OnEnter(CoreState::Ready), build_first_floor);
    }
}

#[cfg(feature = "debug")]
fn sync_debug_controls(
    debug: Res<debug::DebugSettings>,
    pause: Option<Res<State<PauseState>>>,
    players: Query<(), With<PlayerController>>,
    mut controls: ResMut<PlayerControlsEnabled>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let paused = pause.is_some_and(|state| *state.get() == PauseState::Paused);
    controls.0 = !debug.inspector && !paused;
    let capture = controls.0 && !players.is_empty();
    for mut cursor in &mut cursors {
        cursor.visible = !capture;
        cursor.grab_mode = if capture {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

fn apply_graphics(
    settings: Res<GameSettings>,
    cameras: Query<(Entity, Option<&Msaa>), With<Camera3d>>,
    mut commands: Commands,
) {
    let quality = match settings.graphics {
        GraphicsQuality::Low => Msaa::Off,
        GraphicsQuality::Medium => Msaa::Sample2,
        GraphicsQuality::High => Msaa::Sample4,
    };
    for (entity, current) in &cameras {
        if current != Some(&quality) {
            commands.entity(entity).insert(quality);
        }
    }
}

fn dim_new_lights(
    mut lights: Query<(&mut PointLight, &mut LightIntensity), Added<LightIntensity>>,
) {
    for (mut light, mut base) in &mut lights {
        base.0 *= WINDOWED_LIGHT_SCALE;
        light.intensity = base.0;
    }
}

fn dim_ambient_on_outage(
    power: Option<Res<FacilityPower>>,
    game: Option<Res<State<GameState>>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let playing = game.is_none_or(|game| *game.get() == GameState::Playing);
    let brightness = if playing && power.is_some_and(|power| !power.on) {
        0.35
    } else {
        WINDOWED_AMBIENT_BRIGHTNESS
    };
    if ambient.brightness != brightness {
        ambient.brightness = brightness;
    }
}

fn core_ready(mut next: ResMut<NextState<CoreState>>) {
    next.set(CoreState::Ready);
}

fn core_failed(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::error());
}

fn transport_ready(mut timeline: ResMut<transport::TransportTimeline>) {
    timeline.ready();
}

fn player() -> impl Bundle {
    (PlayerController, Transform::from_xyz(0.0, EYE_HEIGHT, -5.0))
}

fn spawn_controller(mut commands: Commands) {
    commands.spawn(player());
}

#[cfg(test)]
#[path = "../tests/unit/dark_lighting.rs"]
mod tests;
