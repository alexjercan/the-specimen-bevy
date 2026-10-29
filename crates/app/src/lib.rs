mod glue;
mod playable;

use std::{path::PathBuf, time::Duration};

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::{
    app::ScheduleRunnerPlugin,
    input::InputPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
    window::WindowPlugin,
};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::prelude::{ChaCha8Rng, EntropyPlugin, GlobalRng};
use capture::video::VideoProfile;
use game_assets::GameAssetsState;
use game_flow::{CoreState, GameState, PauseState};
use game_frontend::MenuPlugin;
use game_settings::{GameSettings, GameSettingsPlugin};
use gameplay::{
    facility::{ActiveRun, FacilityLightingPlugin, StartRun},
    fuses::FuseSeed,
    presets::GamePreset,
};
use rand_core::Rng;

pub use playable::Specimen;

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";

type MainPlugin = Box<dyn FnOnce(&mut App) + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Frontend {
    Headless,
    Windowed { menu: bool },
}

impl Frontend {
    fn is_headless(self) -> bool {
        self == Frontend::Headless
    }
}

#[derive(Resource)]
struct FrameLimit {
    remaining: u32,
}

#[derive(Resource, Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameStep(Duration);

impl FrameStep {
    pub const DEFAULT: Self = Self(Duration::from_nanos(1_000_000_000 / 60));
    // NOTE: the screenshot tools keep the 30 Hz step they used before the owner moved here.
    pub const SCREENSHOTS: Self = Self(Duration::from_nanos(1_000_000_000 / 30));

    pub fn duration(self) -> Duration {
        self.0
    }
}

pub struct AppBuilder {
    frontend: Frontend,
    stdio: bool,
    seed: Option<u64>,
    recording: Option<(PathBuf, VideoProfile)>,
    frame_limit: Option<u32>,
    main_plugin: Option<MainPlugin>,
}

impl AppBuilder {
    pub fn new(frontend: Frontend) -> Self {
        Self {
            frontend,
            stdio: false,
            seed: None,
            recording: None,
            frame_limit: None,
            main_plugin: None,
        }
    }

    pub fn with_stdio(mut self) -> Self {
        self.stdio = true;
        self
    }

    pub fn bounded(mut self, frames: u32) -> Self {
        self.frame_limit = Some(frames);
        self
    }

    pub fn with_recording(mut self, path: PathBuf, profile: VideoProfile) -> Self {
        self.recording = Some((path, profile));
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_main_plugin(mut self, plugin: impl Plugin) -> Self {
        self.main_plugin = Some(Box::new(|app| {
            app.add_plugins(plugin);
        }));
        self
    }

    fn menu_enabled(&self) -> bool {
        self.frontend == Frontend::Windowed { menu: true } && self.main_plugin.is_none()
    }

    fn facility_lighting_enabled(&self) -> bool {
        !self.frontend.is_headless() && self.main_plugin.is_none()
    }

    pub fn build(self) -> App {
        let headless = self.frontend.is_headless();
        assert!(
            self.recording.is_none() || (!headless && self.stdio),
            "recording needs a windowed stdio run"
        );
        let menu = self.menu_enabled();
        assert!(
            !(menu && self.stdio),
            "stdio drives the game without a menu"
        );
        let facility_lighting = self.facility_lighting_enabled();
        let mut app = App::new();
        let logging = LogPlugin {
            level: Level::INFO,
            filter: LOG_FILTER.into(),
            ..default()
        };
        app.add_plugins(GameSettingsPlugin { persist: menu })
            .add_plugins(glue::SettingsGluePlugin)
            .add_plugins(game_achievements::AchievementStorePlugin {
                store: game_achievements::AchievementStore::persistent(menu),
            });
        if headless {
            app.add_plugins(
                MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
            )
            .add_plugins(logging)
            .add_plugins((InputPlugin, StatesPlugin, TransformPlugin))
            .insert_state(CoreState::Ready);
        } else {
            let mut window = Window::default();
            #[cfg(target_arch = "wasm32")]
            {
                window.canvas = Some("#bevy".into());
                window.fit_canvas_to_parent = true;
            }
            if menu {
                window.mode =
                    game_frontend::window_mode(app.world().resource::<GameSettings>().display_mode);
            }
            let plugins = DefaultPlugins.set(logging).set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            });
            #[cfg(target_arch = "wasm32")]
            let plugins = plugins.set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            });
            app.add_plugins(plugins);
            let fps_overlay = app.world().resource::<GameSettings>().fps_overlay;
            app.init_state::<CoreState>()
                .add_plugins((
                    game_assets::GameAssetsPlugin,
                    game_ui::GameUiPlugin,
                    game_ui::FpsOverlayPlugin {
                        visible: fps_overlay,
                    },
                ))
                .add_systems(OnEnter(GameAssetsState::Ready), core_ready)
                .add_systems(OnEnter(GameAssetsState::Failed), core_failed);
        }
        let seed = seed_entropy(&mut app, self.seed);
        let step = FrameStep::DEFAULT;
        app.insert_resource(step)
            .insert_resource(FuseSeed(seed))
            .add_sub_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_plugins((EnhancedInputPlugin, gameplay::GameplayPlugin));
        if let Some(frames) = self.frame_limit {
            app.insert_resource(FrameLimit { remaining: frames })
                .add_systems(Last, count_frames);
        }
        if headless {
            if self.stdio {
                app.add_plugins(stdio::HeadlessStdio::<Specimen>::new(step.duration()));
            }
        } else {
            app.add_plugins((
                gameplay::player::PlayerViewPlugin,
                gameplay::facility::FacilityViewPlugin,
                gameplay::interaction::InteractionViewPlugin,
                gameplay::devices::DeviceViewPlugin,
                gameplay::fuses::FuseViewPlugin,
                gameplay::monster::MonsterViewPlugin,
                gameplay::hiding::HidingSoundsPlugin,
                gameplay::ambience::AmbienceViewPlugin,
            ))
            .add_plugins((
                glue::FacilityAssetsPlugin,
                glue::MonsterAssetsPlugin,
                glue::InteractionAssetsPlugin,
                glue::FeatureSoundsPlugin,
            ))
            .add_plugins((
                game_audio::GameAudioPlugin,
                glue::SoundGluePlugin,
                glue::MenuAssetsPlugin,
            ));
            #[cfg(feature = "halloween")]
            app.add_plugins(game_halloween::HalloweenPlugin);
            if let Some((path, profile)) = self.recording {
                app.insert_resource(stdio::Recording(path))
                    .add_plugins(capture::CapturePlugin::new(profile, step.duration()));
            }
            if self.stdio {
                app.add_plugins(stdio::RenderedStdio::<Specimen>::new(step.duration()));
            }
            #[cfg(feature = "debug")]
            app.add_plugins(debug::DebugPlugin);
            app.add_plugins(game_frontend::CameraQualityPlugin);
            if facility_lighting {
                app.add_plugins(FacilityLightingPlugin);
            }
        }
        match self.main_plugin {
            Some(main_plugin) => main_plugin(&mut app),
            None if menu => {
                app.add_plugins(MenuPlugin);
            }
            None => {
                app.add_plugins(GamePlugin);
            }
        }
        app
    }
}

fn seed_entropy(app: &mut App, seed: Option<u64>) -> u64 {
    match seed {
        Some(seed) => {
            let mut bytes = [0; 32];
            bytes[..8].copy_from_slice(&seed.to_le_bytes());
            app.add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed(bytes));
            seed
        }
        None => {
            app.add_plugins(EntropyPlugin::<ChaCha8Rng>::default());
            let world = app.world_mut();
            let mut global = world.query_filtered::<&mut ChaCha8Rng, With<GlobalRng>>();
            global
                .single_mut(world)
                .expect("EntropyPlugin spawns the global rng")
                .next_u64()
        }
    }
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(CoreState::Ready), start_playing)
            .add_observer(scope_run);
    }
}

// NOTE: only the run anchor leaves with Playing, so the world stays for snapshots and the power resets.
fn scope_run(added: On<Add<ActiveRun>>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert(DespawnOnExit(GameState::Playing));
}

fn start_playing(
    preset: Res<GamePreset>,
    mut runs: MessageWriter<StartRun>,
    mut next: ResMut<NextState<GameState>>,
) {
    runs.write(StartRun { preset: *preset });
    next.set(GameState::Playing);
}

fn count_frames(mut limit: ResMut<FrameLimit>, mut exit: MessageWriter<AppExit>) {
    limit.remaining = limit.remaining.saturating_sub(1);
    if limit.remaining == 0 {
        exit.write(AppExit::Success);
    }
}

fn core_ready(mut next: ResMut<NextState<CoreState>>) {
    next.set(CoreState::Ready);
}

fn core_failed(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::error());
}
