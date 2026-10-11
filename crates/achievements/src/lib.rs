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

pub enum AchievementStore {
    Disabled,
    #[cfg(not(target_arch = "wasm32"))]
    File(std::path::PathBuf),
    #[cfg(target_arch = "wasm32")]
    Browser,
}

impl AchievementStore {
    pub fn persistent(persist: bool) -> Self {
        if !persist {
            return Self::Disabled;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::default_path().map_or(Self::Disabled, Self::File)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::Browser
        }
    }
}

pub struct AchievementStorePlugin {
    pub store: AchievementStore,
}

impl Plugin for AchievementStorePlugin {
    fn build(&self, app: &mut App) {
        match &self.store {
            AchievementStore::Disabled => {}
            #[cfg(not(target_arch = "wasm32"))]
            AchievementStore::File(path) => build_native(app, path),
            #[cfg(target_arch = "wasm32")]
            AchievementStore::Browser => build_wasm(app),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn build_native(app: &mut App, path: &std::path::Path) {
    let loaded = native::load_from_path(path);
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
