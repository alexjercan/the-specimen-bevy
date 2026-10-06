use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

use crate::CaptureState;

#[derive(Clone, Copy, Default)]
pub struct ScreenshotCapturePlugin;

impl Plugin for ScreenshotCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaptureState>()
            .init_resource::<CaptureLog>();
    }
}

#[derive(Resource, Default)]
pub struct CaptureLog {
    written: HashSet<PathBuf>,
}

pub fn screenshot_written(world: &World, path: impl AsRef<Path>) -> bool {
    world
        .get_resource::<CaptureLog>()
        .is_some_and(|log| log.written.contains(path.as_ref()))
}

pub fn screenshot_written_at(
    path: impl Into<PathBuf>,
) -> Arc<dyn Fn(&World) -> bool + Send + Sync> {
    let path = path.into();
    Arc::new(move |world| screenshot_written(world, &path))
}

pub fn screenshot_start(world: &mut World, path: impl Into<PathBuf>) {
    let path = path.into();
    world.resource_mut::<CaptureLog>().written.remove(&path);
    world.resource_mut::<CaptureState>().register();
    world.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>,
              mut state: ResMut<CaptureState>,
              mut log: ResMut<CaptureLog>,
              mut exit: MessageWriter<AppExit>| {
            let result = save_screenshot(&path, &event.image);
            record_result(&path, result, &mut state, &mut log, &mut exit);
        },
    );
}

fn save_screenshot(path: &Path, image: &Image) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let image = image
        .clone()
        .try_into_dynamic()
        .map_err(|error| error.to_string())?
        .to_rgb8();
    let background = *image.get_pixel(0, 0);
    if !image.pixels().any(|pixel| *pixel != background) {
        return Err("screenshot is uniform; no rendered scene was captured".to_string());
    }
    image.save(path).map_err(|error| error.to_string())
}

fn record_result(
    path: &Path,
    result: Result<(), String>,
    state: &mut CaptureState,
    log: &mut CaptureLog,
    exit: &mut MessageWriter<AppExit>,
) {
    match result {
        Ok(()) => {
            info!("screenshot saved to {}", path.display());
            log.written.insert(path.to_path_buf());
            state.finish();
        }
        Err(reason) => {
            error!("screenshot {} failed: {reason}", path.display());
            state.fail(format!("{}: {reason}", path.display()));
            exit.write(AppExit::error());
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/screenshot.rs"]
mod tests;
