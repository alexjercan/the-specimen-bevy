use bevy::prelude::*;

use super::{
    Achievement, AchievementPlugin, AchievementProgress, AchievementSignal, AchievementSignalKind,
    AchievementUnlocked, RunAchievements,
};
use crate::{
    controller::PlayerController,
    levels::{Caught, Escaped},
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AchievementPlugin));
    let player = app.world_mut().spawn(PlayerController).id();
    (app, player)
}

fn signal(app: &mut App, player: Entity, kind: AchievementSignalKind) {
    app.world_mut().trigger(AchievementSignal { player, kind });
}

fn has(app: &App, achievement: Achievement) -> bool {
    app.world()
        .resource::<AchievementProgress>()
        .unlocked
        .contains(&achievement)
}

#[test]
fn escape_checks_pickup_history_detection_and_repair_not_current_inventory() {
    let (mut app, player) = app();
    signal(&mut app, player, AchievementSignalKind::PickedFlashbang);
    signal(&mut app, player, AchievementSignalKind::Detected);
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(!has(&app, Achievement::EscapeWithoutFlashbang));
    assert!(has(&app, Achievement::EscapeWithoutDetector));
    assert!(has(&app, Achievement::EscapeWithoutBoiler));
    assert!(!has(&app, Achievement::EscapeUndetected));
}

#[test]
fn restored_boiler_and_confirmed_hit_unlock_once_across_replays() {
    let (mut app, player) = app();
    signal(&mut app, player, AchievementSignalKind::RestoredBoiler);
    signal(&mut app, player, AchievementSignalKind::FlashbangHitMonster);
    assert!(has(&app, Achievement::RestoreBoiler));
    assert!(has(&app, Achievement::FlashbangHitMonster));
    app.world_mut().despawn(player);
    let replay = app.world_mut().spawn(PlayerController).id();
    assert_eq!(
        app.world()
            .get::<RunAchievements>(replay)
            .unwrap()
            .restored_boiler,
        false
    );
    app.world_mut().entity_mut(replay).insert(Escaped);
    assert!(has(&app, Achievement::EscapeWithoutBoiler));
    assert_eq!(
        app.world().resource::<AchievementProgress>().unlocked.len(),
        6
    );
}

#[test]
fn caught_requires_player_opened_exit() {
    let (mut app, player) = app();
    let monster = app.world_mut().spawn_empty().id();
    app.world_mut()
        .entity_mut(player)
        .insert(Caught::new(monster));
    assert!(!has(&app, Achievement::CaughtAfterExitOpen));
    let replay = app.world_mut().spawn(PlayerController).id();
    signal(&mut app, replay, AchievementSignalKind::ExitOpened);
    app.world_mut()
        .entity_mut(replay)
        .insert(Caught::new(monster));
    assert!(has(&app, Achievement::CaughtAfterExitOpen));
}

#[derive(Resource, Default)]
struct Unlocks(Vec<Achievement>);

#[test]
fn unlock_event_fires_once_for_new_unlocks_only() {
    let (mut app, player) = app();
    app.init_resource::<Unlocks>().add_observer(
        |unlocked: On<AchievementUnlocked>, mut unlocks: ResMut<Unlocks>| {
            unlocks.0.push(unlocked.0);
        },
    );
    app.world_mut()
        .resource_mut::<AchievementProgress>()
        .unlocked
        .insert(Achievement::EscapeUndetected);
    signal(&mut app, player, AchievementSignalKind::FlashbangHitMonster);
    signal(&mut app, player, AchievementSignalKind::FlashbangHitMonster);
    app.world_mut().entity_mut(player).insert(Escaped);
    let mut unlocks = app.world().resource::<Unlocks>().0.clone();
    unlocks.sort_by_key(|achievement| achievement.id());
    assert_eq!(
        unlocks,
        [
            Achievement::EscapeWithoutBoiler,
            Achievement::EscapeWithoutDetector,
            Achievement::EscapeWithoutFlashbang,
            Achievement::FlashbangHitMonster,
        ]
    );
}

#[test]
fn every_achievement_has_text_and_round_trips_its_id() {
    for achievement in Achievement::ALL {
        assert_eq!(Achievement::from_id(achievement.id()), Some(achievement));
        assert!(!achievement.name().is_empty());
        assert!(!achievement.criteria().is_empty());
    }
    assert_eq!(Achievement::from_id("UNKNOWN"), None);
}

#[test]
fn ids_match_preview_definitions() {
    assert_eq!(Achievement::ALL.len(), 7);
    assert_eq!(
        Achievement::FlashbangHitMonster.id(),
        "FLASHBANG_HIT_MONSTER"
    );
    assert_eq!(Achievement::EscapeUndetected.id(), "ESCAPE_UNDETECTED");
}
