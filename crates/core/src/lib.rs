mod glue;
mod lighting;
mod menu;

use std::{path::PathBuf, time::Duration};

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::MonitorSelection;
use bevy::{
    app::ScheduleRunnerPlugin,
    audio::{GlobalVolume, Volume},
    input::InputPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
    window::{PrimaryWindow, WindowMode, WindowPlugin},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_rand::prelude::{ChaCha8Rng, EntropyPlugin, GlobalRng};
use game_assets::GameAssetsState;
use game_settings::{DisplayMode, GameSettings, GameSettingsPlugin, GraphicsQuality};
use gameplay::levels::{build_first_floor, spawn_first_floor_actors, FacilityPower, FuseSeed};
use rand_core::Rng;

pub use menu::{GameState, MenuPlugin, PauseState};

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";

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
        app.add_plugins(GameSettingsPlugin { persist: menu })
            .add_plugins(glue::ControllerSettingsPlugin)
            .add_plugins(game_achievements::AchievementStorePlugin {
                store: game_achievements::AchievementStore::persistent(menu),
            });
        if self.headless {
            app.add_plugins(
                MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
            )
            .add_plugins(logging)
            .add_plugins((InputPlugin, StatesPlugin))
            .insert_state(CoreState::Ready);
        } else {
            let mut window = Window::default();
            #[cfg(target_arch = "wasm32")]
            {
                window.canvas = Some("#bevy".into());
                window.fit_canvas_to_parent = true;
            }
            if menu {
                window.mode = window_mode(app.world().resource::<GameSettings>().display_mode);
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
            if self.muted_audio {
                app.insert_resource(GlobalVolume::new(Volume::Linear(0.0)));
            }
            app.init_state::<CoreState>()
                .add_plugins((game_assets::GameAssetsPlugin, game_ui::GameUiPlugin))
                .add_systems(OnEnter(GameAssetsState::Ready), core_ready)
                .add_systems(OnEnter(GameAssetsState::Failed), core_failed);
        }
        let seed = seed_entropy(&mut app, self.seed);
        app.insert_resource(FuseSeed(seed))
            .add_sub_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_systems(OnExit(GameState::Playing), reset_facility_power)
            .add_plugins((EnhancedInputPlugin, gameplay::GameplayPlugin));
        if self.headless {
            if self.transport {
                app.add_plugins(transport::TransportPlugin);
            }
        } else {
            app.add_plugins((
                gameplay::controller::PlayerPresentationPlugin,
                gameplay::levels::LevelRenderPlugin,
                gameplay::levels::MonsterRenderPlugin,
            ))
            .add_plugins((
                glue::InteractionHintPlugin,
                glue::FuseHudPlugin,
                glue::FlashlightHudPlugin,
                glue::StaminaHudPlugin,
                glue::DeviceHudPlugin,
                glue::FpsOverlayPlugin,
            ))
            .add_plugins((game_audio::GameAudioPlugin, glue::SoundGluePlugin));
            #[cfg(feature = "halloween")]
            app.add_plugins(game_halloween::HalloweenPlugin);
            if let Some(path) = self.recording {
                app.insert_resource(transport::RecordTransport(path))
                    .add_plugins(capture::CapturePlugin::new(60));
            }
            if self.transport {
                app.add_plugins(transport::RenderedTransportPlugin);
            }
            #[cfg(feature = "debug")]
            app.add_plugins(debug::DebugPlugin);
            if facility_lighting {
                app.add_plugins(lighting::FacilityLightingPlugin)
                    .add_systems(Update, apply_graphics);
            }
            if menu {
                app.add_systems(Update, apply_window_mode);
            }
        }
        if self.transport {
            app.add_systems(OnEnter(CoreState::Ready), transport_ready);
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
        app.add_systems(
            OnEnter(CoreState::Ready),
            (start_playing, build_first_floor, spawn_first_floor_actors).chain(),
        );
    }
}

fn start_playing(mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Playing);
}

fn reset_facility_power(mut commands: Commands) {
    commands.insert_resource(FacilityPower::default());
}

fn window_mode(mode: DisplayMode) -> WindowMode {
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

fn apply_window_mode(
    settings: Res<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !settings.is_changed() {
        return;
    }
    let mode = window_mode(settings.display_mode);
    for mut window in &mut windows {
        if window.mode != mode {
            window.mode = mode;
        }
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

fn core_ready(mut next: ResMut<NextState<CoreState>>) {
    next.set(CoreState::Ready);
}

fn core_failed(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::error());
}

fn transport_ready(mut timeline: ResMut<transport::TransportTimeline>) {
    timeline.ready();
}
