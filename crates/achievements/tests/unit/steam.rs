use std::collections::{HashMap, HashSet};

use gameplay::achievements::Achievement;

use super::{reconcile, RemoteAchievements};

#[derive(Default)]
struct FakeRemote {
    state: HashMap<String, bool>,
    store_calls: u32,
}

impl FakeRemote {
    fn with(mut self, id: &str, achieved: bool) -> Self {
        self.state.insert(id.to_string(), achieved);
        self
    }
}

impl RemoteAchievements for FakeRemote {
    fn achieved(&self, id: &str) -> Option<bool> {
        self.state.get(id).copied()
    }

    fn unlock(&mut self, id: &str) -> bool {
        match self.state.get_mut(id) {
            Some(value) => {
                *value = true;
                true
            }
            None => false,
        }
    }

    fn store(&mut self) -> bool {
        self.store_calls += 1;
        true
    }
}

#[test]
fn pulls_remote_only_achievements_into_local() {
    let mut local = HashSet::new();
    let mut remote = FakeRemote::default().with(Achievement::RestoreBoiler.id(), true);
    let result = reconcile(&mut local, &mut remote);
    assert_eq!(result.pulled, vec![Achievement::RestoreBoiler]);
    assert!(local.contains(&Achievement::RestoreBoiler));
    assert_eq!(remote.store_calls, 0);
}

#[test]
fn pushes_local_only_achievements_and_stores_once() {
    let mut local = HashSet::from([Achievement::RestoreBoiler, Achievement::EscapeUndetected]);
    let mut remote = FakeRemote::default()
        .with(Achievement::RestoreBoiler.id(), false)
        .with(Achievement::EscapeUndetected.id(), false);
    let result = reconcile(&mut local, &mut remote);
    assert_eq!(result.pushed.len(), 2);
    assert_eq!(remote.store_calls, 1);
}

#[test]
fn keeps_unknown_remote_ids_local_without_clearing_or_storing() {
    let mut local = HashSet::from([Achievement::RestoreBoiler]);
    let mut remote = FakeRemote::default();
    let result = reconcile(&mut local, &mut remote);
    assert_eq!(result.missing, vec![Achievement::RestoreBoiler]);
    assert!(local.contains(&Achievement::RestoreBoiler));
    assert_eq!(remote.store_calls, 0);
}

#[test]
fn never_unlocks_remote_for_ids_not_locally_unlocked() {
    let mut local = HashSet::new();
    let mut remote = FakeRemote::default().with(Achievement::RestoreBoiler.id(), false);
    let result = reconcile(&mut local, &mut remote);
    assert!(result.pushed.is_empty());
    assert!(!local.contains(&Achievement::RestoreBoiler));
    assert_eq!(remote.store_calls, 0);
}
