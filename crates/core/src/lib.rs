use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    log::{Level, LogPlugin},
    prelude::*,
};

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";

#[derive(Default)]
pub struct AppBuilder {
    headless: bool,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn headless() -> Self {
        Self { headless: true }
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
            .add_plugins(logging);
            return app;
        }
        app.add_plugins(DefaultPlugins.set(logging))
            .add_plugins(game_assets::GameAssetsPlugin);
        #[cfg(feature = "debug")]
        app.add_plugins(debug::DebugPlugin);
        app
    }
}
