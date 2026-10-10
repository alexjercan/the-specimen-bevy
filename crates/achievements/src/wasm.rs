use std::collections::HashSet;

use bevy::prelude::*;
use gameplay::achievements::Achievement;

use crate::{StoreDirty, StoreUnknown};

pub(crate) const KEY: &str = "the-specimen-bevy.achievements";
pub(crate) const CORRUPT_KEY: &str = "the-specimen-bevy.achievements.corrupt";

#[derive(Resource)]
pub(crate) struct StoreEnabled(pub(crate) bool);

pub(crate) struct Loaded {
    pub known: HashSet<Achievement>,
    pub unknown: HashSet<String>,
    pub enabled: bool,
}

pub(crate) fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub(crate) fn load() -> Loaded {
    let Some(storage) = storage() else {
        warn!("local storage unavailable; achievements will not persist");
        return Loaded {
            known: HashSet::new(),
            unknown: HashSet::new(),
            enabled: false,
        };
    };
    match storage.get_item(KEY) {
        Ok(Some(value)) => match crate::store::decode(value.as_bytes()) {
            Ok(decoded) => Loaded {
                known: decoded.known,
                unknown: decoded.unknown,
                enabled: true,
            },
            Err(_) => {
                let backed_up = backup_corrupt(&storage, &value);
                warn!("achievements data corrupt; backup succeeded: {backed_up}");
                Loaded {
                    known: HashSet::new(),
                    unknown: HashSet::new(),
                    enabled: backed_up,
                }
            }
        },
        Ok(None) => Loaded {
            known: HashSet::new(),
            unknown: HashSet::new(),
            enabled: true,
        },
        Err(_) => {
            warn!("local storage read failed; achievements will not persist");
            Loaded {
                known: HashSet::new(),
                unknown: HashSet::new(),
                enabled: false,
            }
        }
    }
}

fn backup_corrupt(storage: &web_sys::Storage, value: &str) -> bool {
    let mut suffix = 0u32;
    loop {
        let key = if suffix == 0 {
            CORRUPT_KEY.to_string()
        } else {
            format!("{CORRUPT_KEY}.{suffix}")
        };
        match storage.get_item(&key) {
            Ok(None) => return storage.set_item(&key, value).is_ok(),
            Ok(Some(_)) => suffix += 1,
            Err(_) => return false,
        }
    }
}

pub(crate) fn save(
    storage: &web_sys::Storage,
    known: &HashSet<Achievement>,
    unknown: &HashSet<String>,
) -> Result<(), ()> {
    let data = crate::store::encode(known, unknown);
    let text = String::from_utf8(data).expect("achievement store json is utf8");
    storage.set_item(KEY, &text).map_err(|_| ())
}

pub(crate) fn save_dirty(
    progress: Res<gameplay::achievements::AchievementProgress>,
    unknown: Res<StoreUnknown>,
    enabled: Res<StoreEnabled>,
    mut dirty: ResMut<StoreDirty>,
) {
    if !dirty.0 {
        return;
    }
    if !enabled.0 {
        dirty.0 = false;
        return;
    }
    let Some(storage) = storage() else {
        warn!("local storage unavailable; achievements save skipped");
        return;
    };
    if save(&storage, &progress.unlocked, &unknown.0).is_err() {
        warn!("achievements save failed");
    } else {
        dirty.0 = false;
    }
}
