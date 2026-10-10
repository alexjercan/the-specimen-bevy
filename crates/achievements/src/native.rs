use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use gameplay::achievements::Achievement;

use crate::{StoreDirty, StoreUnknown};

#[derive(Resource)]
pub(crate) struct StorePath(pub(crate) Option<PathBuf>);

pub(crate) struct Loaded {
    pub known: HashSet<Achievement>,
    pub unknown: HashSet<String>,
    pub path: Option<PathBuf>,
}

impl Loaded {
    pub(crate) fn disabled() -> Self {
        Self {
            known: HashSet::new(),
            unknown: HashSet::new(),
            path: None,
        }
    }

    fn empty(path: Option<PathBuf>) -> Self {
        Self {
            known: HashSet::new(),
            unknown: HashSet::new(),
            path,
        }
    }
}

pub(crate) fn default_path() -> Option<PathBuf> {
    let root = std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    let Some(root) = root else {
        warn!("no XDG_DATA_HOME or HOME set; achievements will not persist");
        return None;
    };
    Some(root.join("the-specimen-bevy").join("achievements.json"))
}

pub(crate) fn load_from_path(path: &Path) -> Loaded {
    match fs::read(path) {
        Ok(data) => match crate::store::decode(&data) {
            Ok(decoded) => Loaded {
                known: decoded.known,
                unknown: decoded.unknown,
                path: Some(path.to_path_buf()),
            },
            Err(_) => {
                if backup_corrupt(path) {
                    warn!(
                        "achievements file at {} is corrupt; backed up and starting empty",
                        path.display()
                    );
                    Loaded::empty(Some(path.to_path_buf()))
                } else {
                    warn!(
                        "achievements file at {} is corrupt; persistence disabled to preserve it",
                        path.display()
                    );
                    Loaded::disabled()
                }
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Loaded::empty(Some(path.to_path_buf()))
        }
        Err(error) => {
            warn!(
                "achievements file at {} could not be read ({error}); persistence disabled for this session",
                path.display()
            );
            Loaded::disabled()
        }
    }
}

fn backup_corrupt(path: &Path) -> bool {
    let backup = first_available_backup(path);
    match fs::rename(path, &backup) {
        Ok(()) => true,
        Err(error) => {
            warn!("failed to back up corrupt achievements file: {error}");
            false
        }
    }
}

fn first_available_backup(path: &Path) -> PathBuf {
    let base = format!("{}.corrupt", path.display());
    if !Path::new(&base).exists() {
        return PathBuf::from(base);
    }
    let mut suffix = 1u32;
    loop {
        let candidate = format!("{base}.{suffix}");
        if !Path::new(&candidate).exists() {
            return PathBuf::from(candidate);
        }
        suffix += 1;
    }
}

pub(crate) fn save_to_path(
    path: &Path,
    known: &HashSet<Achievement>,
    unknown: &HashSet<String>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let data = crate::store::encode(known, unknown);
    fs::write(&temp, data)?;
    fs::rename(&temp, path)
}

pub(crate) fn save_dirty(
    progress: Res<gameplay::achievements::AchievementProgress>,
    unknown: Res<StoreUnknown>,
    path: Res<StorePath>,
    mut dirty: ResMut<StoreDirty>,
) {
    if !dirty.0 {
        return;
    }
    let Some(path) = &path.0 else {
        dirty.0 = false;
        return;
    };
    match save_to_path(path, &progress.unlocked, &unknown.0) {
        Ok(()) => dirty.0 = false,
        Err(error) => warn!("achievements save failed: {error}"),
    }
}

#[cfg(test)]
#[path = "../tests/unit/native.rs"]
mod tests;
