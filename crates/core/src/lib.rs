mod glue;
mod menu;

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

pub use menu::{GameState, PauseState};

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";
const EYE_HEIGHT: f32 = 1.6;

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

    pub fn with_main_plugin(mut self, plugin: impl Plugin) -> Self {
        self.main_plugin = Some(Box::new(|app| {
            app.add_plugins(plugin);
        }));
        self
    }

    fn menu_enabled(&self) -> bool {
        self.menu && !self.headless && !self.transport && self.main_plugin.is_none()
    }

    pub fn build(self) -> App {
        let menu = self.menu_enabled();
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
            .add_plugins(logging)
            .add_plugins((InputPlugin, StatesPlugin))
            .insert_state(CoreState::Ready)
            .add_plugins(EnhancedInputPlugin)
            .add_plugins(gameplay::controller::PlayerControllerPlugin::default().without_camera())
            .add_plugins(gameplay::levels::DoorPlugin);
            if self.transport {
                app.add_plugins(transport::TransportPlugin);
            }
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
                .add_plugins(gameplay::controller::PlayerControllerPlugin::default())
                .add_plugins(gameplay::levels::DoorPlugin)
                .add_plugins(glue::DoorHintPlugin);
            if self.transport {
                app.add_plugins(transport::RenderedTransportPlugin);
            }
            #[cfg(feature = "debug")]
            app.add_plugins(debug::DebugPlugin);
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
