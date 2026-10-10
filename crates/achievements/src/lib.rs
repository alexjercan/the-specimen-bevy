use std::collections::HashSet;

use bevy::prelude::*;
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

mod store;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub mod steam;
#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
mod steam_adapter;

#[derive(Resource, Default)]
pub(crate) struct StoreUnknown(HashSet<String>);

#[derive(Resource, Default)]
pub(crate) struct StoreDirty(bool);

pub struct AchievementStorePlugin {
    pub persist: bool,
}

impl Plugin for AchievementStorePlugin {
    fn build(&self, app: &mut App) {
        if !self.persist {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        build_native(app, None);
        #[cfg(target_arch = "wasm32")]
        build_wasm(app);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub fn install_native_for_test(app: &mut App, path: std::path::PathBuf) {
    build_native(app, Some(path));
}

#[cfg(not(target_arch = "wasm32"))]
fn build_native(app: &mut App, path_override: Option<std::path::PathBuf>) {
    let path = path_override.or_else(native::default_path);
    let loaded = match path {
        Some(path) => native::load_from_path(&path),
        None => native::Loaded::disabled(),
    };
    install(app, loaded.known, loaded.unknown);
    app.insert_resource(native::StorePath(loaded.path))
        .add_systems(Last, native::save_dirty);

    #[cfg(feature = "steam")]
    steam_adapter::build(app);
}

#[cfg(target_arch = "wasm32")]
fn build_wasm(app: &mut App) {
    let loaded = wasm::load();
    install(app, loaded.known, loaded.unknown);
    app.insert_resource(wasm::StoreEnabled(loaded.enabled))
        .add_systems(Last, wasm::save_dirty);
}

fn install(app: &mut App, known: HashSet<Achievement>, unknown: HashSet<String>) {
    app.world_mut()
        .get_resource_or_init::<AchievementProgress>()
        .unlocked
        .extend(known);
    app.insert_resource(StoreUnknown(unknown))
        .init_resource::<StoreDirty>()
        .add_observer(mark_dirty);
}

fn mark_dirty(_unlocked: On<AchievementUnlocked>, mut dirty: ResMut<StoreDirty>) {
    dirty.0 = true;
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "../tests/unit/plugin.rs"]
mod tests;
