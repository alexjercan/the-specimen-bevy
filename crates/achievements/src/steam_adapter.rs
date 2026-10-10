use std::collections::HashSet;

use bevy::prelude::*;
use gameplay::achievements::{AchievementProgress, AchievementUnlocked};

use crate::steam::{self, RemoteAchievements};

#[derive(Resource, Clone)]
struct SteamClient(steamworks::Client);

#[derive(Resource, Default)]
struct SteamStatsReady(bool);

#[derive(Resource, Default)]
struct SteamWarnedMissing(HashSet<String>);

struct SteamRemote<'a>(&'a steamworks::Client);

impl RemoteAchievements for SteamRemote<'_> {
    fn achieved(&self, id: &str) -> Option<bool> {
        self.0.user_stats().achievement(id).get().ok()
    }

    fn unlock(&mut self, id: &str) -> bool {
        self.0.user_stats().achievement(id).set().is_ok()
    }

    fn store(&mut self) -> bool {
        self.0.user_stats().store_stats().is_ok()
    }
}

pub(crate) fn build(app: &mut App) {
    match steamworks::Client::init() {
        Ok(client) => {
            info!("steam client initialized");
            app.insert_resource(SteamClient(client))
                .init_resource::<SteamStatsReady>()
                .init_resource::<SteamWarnedMissing>()
                .add_systems(First, poll_stats_received)
                .add_observer(reconcile_on_unlock);
        }
        Err(error) => {
            info!("Steam unavailable; local achievements only: {error}");
        }
    }
}

fn warn_missing(warned: &mut SteamWarnedMissing, missing: &[gameplay::achievements::Achievement]) {
    for achievement in missing {
        if warned.0.insert(achievement.id().to_string()) {
            warn!(
                "Steam achievement API name {} is unknown to Steam; check the Steamworks configuration",
                achievement.id()
            );
        }
    }
}

fn poll_stats_received(
    client: Res<SteamClient>,
    mut ready: ResMut<SteamStatsReady>,
    mut progress: ResMut<AchievementProgress>,
    mut dirty: ResMut<crate::StoreDirty>,
    mut warned: ResMut<SteamWarnedMissing>,
) {
    let app_id = client.0.utils().app_id();
    let mut became_ready = false;
    client.0.process_callbacks(|callback| {
        if let steamworks::CallbackResult::UserStatsReceived(received) = callback {
            if received.result.is_ok() && received.game_id.app_id() == app_id {
                became_ready = true;
            }
        }
    });
    if ready.0 || !became_ready {
        return;
    }
    ready.0 = true;
    let mut remote = SteamRemote(&client.0);
    let reconciled = steam::reconcile(&mut progress.unlocked, &mut remote);
    if !reconciled.pulled.is_empty() {
        dirty.0 = true;
    }
    warn_missing(&mut warned, &reconciled.missing);
}

fn reconcile_on_unlock(
    _unlocked: On<AchievementUnlocked>,
    client: Res<SteamClient>,
    ready: Res<SteamStatsReady>,
    mut progress: ResMut<AchievementProgress>,
    mut warned: ResMut<SteamWarnedMissing>,
) {
    if !ready.0 {
        return;
    }
    let mut remote = SteamRemote(&client.0);
    let reconciled = steam::reconcile(&mut progress.unlocked, &mut remote);
    warn_missing(&mut warned, &reconciled.missing);
}
