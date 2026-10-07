use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    input::InputPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_assets::GameAssetsState;
use gameplay::{controller::PlayerController, levels::build_first_floor};

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";
const EYE_HEIGHT: f32 = 1.6;

#[derive(States, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CoreState {
    #[default]
    Loading,
    Ready,
    Failed,
}

pub struct AppBuilder {
    headless: bool,
    main_plugin: Box<dyn FnOnce(&mut App) + Send + Sync>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            headless: false,
            main_plugin: Box::new(|app| {
                app.add_plugins(GamePlugin);
            }),
        }
    }
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

    pub fn with_main_plugin(mut self, plugin: impl Plugin) -> Self {
        self.main_plugin = Box::new(|app| {
            app.add_plugins(plugin);
        });
        self
    }

    pub fn build(self) -> App {
        let mut app = App::new();
        let logging = LogPlugin {
            level: Level::INFO,
            filter: LOG_FILTER.into(),
            ..default()
        };
        if self.headless {
            app.add_plugins(
                MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
            )
            .add_plugins((InputPlugin, StatesPlugin))
            .insert_state(CoreState::Ready)
            .add_plugins(EnhancedInputPlugin)
            .add_plugins(gameplay::controller::PlayerControllerPlugin::default().without_camera())
            .add_plugins(transport::TransportPlugin);
        } else {
            app.add_plugins(DefaultPlugins.set(logging))
                .init_state::<CoreState>()
                .add_plugins((
                    game_assets::GameAssetsPlugin,
                    gameplay::levels::LevelRenderPlugin,
                ))
                .add_systems(OnEnter(GameAssetsState::Ready), core_ready)
                .add_systems(OnEnter(GameAssetsState::Failed), core_failed)
                .add_plugins(EnhancedInputPlugin)
                .add_plugins(gameplay::controller::PlayerControllerPlugin::default());
            #[cfg(feature = "debug")]
            app.add_plugins(debug::DebugPlugin);
        }
        (self.main_plugin)(&mut app);
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

fn core_ready(mut next: ResMut<NextState<CoreState>>) {
    next.set(CoreState::Ready);
}

fn core_failed(mut next: ResMut<NextState<CoreState>>) {
    next.set(CoreState::Failed);
}

fn spawn_controller(mut commands: Commands) {
    commands.spawn((PlayerController, Transform::from_xyz(0.0, EYE_HEIGHT, -5.0)));
}
