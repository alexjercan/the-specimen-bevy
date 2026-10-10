use std::collections::HashSet;

use gameplay::achievements::Achievement;

pub trait RemoteAchievements {
    fn achieved(&self, id: &str) -> Option<bool>;
    fn unlock(&mut self, id: &str) -> bool;
    fn store(&mut self) -> bool;
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Reconciled {
    pub pulled: Vec<Achievement>,
    pub pushed: Vec<Achievement>,
    pub missing: Vec<Achievement>,
}

pub fn reconcile(local: &mut HashSet<Achievement>, remote: &mut impl RemoteAchievements) -> Reconciled {
    let mut reconciled = Reconciled::default();
    for achievement in Achievement::ALL {
        match remote.achieved(achievement.id()) {
            Some(true) => {
                if local.insert(achievement) {
                    reconciled.pulled.push(achievement);
                }
            }
            Some(false) => {
                if local.contains(&achievement) && remote.unlock(achievement.id()) {
                    reconciled.pushed.push(achievement);
                }
            }
            None => {
                if local.contains(&achievement) {
                    reconciled.missing.push(achievement);
                }
            }
        }
    }
    if !reconciled.pushed.is_empty() {
        remote.store();
    }
    reconciled
}

#[cfg(test)]
#[path = "../tests/unit/steam.rs"]
mod tests;
