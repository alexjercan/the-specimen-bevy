use std::collections::HashSet;

use gameplay::achievements::Achievement;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct StoreData {
    version: u32,
    unlocked: Vec<String>,
}

pub(crate) struct Decoded {
    pub known: HashSet<Achievement>,
    pub unknown: HashSet<String>,
}

pub(crate) fn decode(data: &[u8]) -> serde_json::Result<Decoded> {
    let parsed: StoreData = serde_json::from_slice(data)?;
    let mut known = HashSet::new();
    let mut unknown = HashSet::new();
    for id in parsed.unlocked {
        match Achievement::from_id(&id) {
            Some(achievement) => {
                known.insert(achievement);
            }
            None => {
                unknown.insert(id);
            }
        }
    }
    Ok(Decoded { known, unknown })
}

pub(crate) fn encode(known: &HashSet<Achievement>, unknown: &HashSet<String>) -> Vec<u8> {
    let mut ids: Vec<String> = known
        .iter()
        .map(|achievement| achievement.id().to_string())
        .chain(unknown.iter().cloned())
        .collect();
    ids.sort();
    let data = StoreData {
        version: 1,
        unlocked: ids,
    };
    serde_json::to_vec_pretty(&data).expect("achievement store serializes")
}

#[cfg(test)]
#[path = "../tests/unit/store.rs"]
mod tests;
