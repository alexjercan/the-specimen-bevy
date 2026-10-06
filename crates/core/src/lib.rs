use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    asset::AssetPlugin,
    log::{Level, LogPlugin},
    prelude::*,
};

const LOG_FILTER: &str = "wgpu=error,naga=warn,bevy_ecs=warn,bevy_time=warn";

#[derive(Default)]
pub struct AppBuilder {
    asset_path: Option<String>,
    headless: bool,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn headless() -> Self {
        Self {
            headless: true,
            ..Self::default()
        }
    }

    pub fn with_asset_path(mut self, path: impl Into<String>) -> Self {
        self.asset_path = Some(path.into());
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
            .add_plugins(logging);
            return app;
        }
        let plugins = DefaultPlugins.set(logging);
        if let Some(path) = self.asset_path {
            app.add_plugins(plugins.set(AssetPlugin {
                file_path: path,
                ..default()
            }));
        } else {
            app.add_plugins(plugins);
        }
        #[cfg(feature = "debug")]
        app.add_plugins(debug::DebugPlugin);
        app
    }
}
