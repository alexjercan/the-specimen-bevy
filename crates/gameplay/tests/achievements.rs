use std::time::Duration;

use bevy::prelude::*;
use gameplay::{
    achievements::{
        Achievement, AchievementPlugin, AchievementProgress, AchievementUnlocked, RunAchievements,
    },
    devices::{DevicePlugin, FlashbangHit},
    facility::{DoorPlugin, Escaped, ExitOpened, FacilityPower, FacilityPowerPlugin, PowerCue},
    interaction::{PickupCue, PickupPlugin},
    monster::{Caught, MonsterCue, MonsterCueKind, MonsterPlugin},
    player::PlayerController,
    presets::{GameMode, GamePreset},
};

mod common;

fn app() -> (App, Entity) {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((
        DoorPlugin,
        FacilityPowerPlugin,
        PickupPlugin,
        DevicePlugin,
        MonsterPlugin,
        AchievementPlugin,
    ));
    app.finish();
    app.cleanup();
    app.update();
    let player = app.world_mut().spawn(PlayerController).id();
    (app, player)
}

fn happened<M: Message>(app: &mut App, message: M) {
    app.world_mut().write_message(message);
    app.update();
}

fn restored_boiler(app: &mut App) {
    let boiler = app.world_mut().spawn_empty().id();
    happened(app, PowerCue::BoilerRestart { boiler });
}

fn flashbang_hit(app: &mut App) {
    let monster = app.world_mut().spawn_empty().id();
    happened(app, FlashbangHit { monster });
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
    let monster = app.world_mut().spawn_empty().id();
    happened(&mut app, PickupCue::Flashbang);
    happened(
        &mut app,
        MonsterCue {
            monster,
            kind: MonsterCueKind::Detected,
        },
    );
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(has(&app, Achievement::Escaped));
    assert!(!has(&app, Achievement::EscapeWithoutFlashbang));
    assert!(has(&app, Achievement::EscapeWithoutDetector));
    assert!(!has(&app, Achievement::EscapeWithoutBoiler));
    assert!(!has(&app, Achievement::EscapeUndetected));
}

#[test]
fn escaping_before_outage_or_after_repair_does_not_unlock_in_the_dark() {
    let (mut app, player) = app();
    *app.world_mut().resource_mut::<FacilityPower>() = FacilityPower::new(42);
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(!has(&app, Achievement::EscapeWithoutBoiler));

    app.world_mut().despawn(player);
    let replay = app.world_mut().spawn(PlayerController).id();
    app.world_mut().resource_mut::<FacilityPower>().outage();
    restored_boiler(&mut app);
    app.world_mut().resource_mut::<FacilityPower>().restore();
    app.world_mut().resource_mut::<FacilityPower>().outage();
    app.world_mut().entity_mut(replay).insert(Escaped);
    assert!(!has(&app, Achievement::EscapeWithoutBoiler));
}

#[test]
fn restored_boiler_and_confirmed_hit_unlock_once_across_replays() {
    let (mut app, player) = app();
    restored_boiler(&mut app);
    flashbang_hit(&mut app);
    assert!(has(&app, Achievement::RestoreBoiler));
    assert!(has(&app, Achievement::FlashbangHitMonster));
    app.world_mut().despawn(player);
    let replay = app.world_mut().spawn(PlayerController).id();
    assert!(
        !app.world()
            .get::<RunAchievements>(replay)
            .unwrap()
            .restored_boiler
    );
    let mut power = app.world_mut().resource_mut::<FacilityPower>();
    *power = FacilityPower::new(42);
    power.outage();
    app.world_mut().entity_mut(replay).insert(Escaped);
    assert!(has(&app, Achievement::EscapeWithoutBoiler));
    assert_eq!(
        app.world().resource::<AchievementProgress>().unlocked.len(),
        7
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
    app.world_mut().despawn(player);
    let replay = app.world_mut().spawn(PlayerController).id();
    let door = app.world_mut().spawn_empty().id();
    happened(&mut app, ExitOpened { door });
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
    flashbang_hit(&mut app);
    flashbang_hit(&mut app);
    let mut power = app.world_mut().resource_mut::<FacilityPower>();
    *power = FacilityPower::new(42);
    power.outage();
    app.world_mut().entity_mut(player).insert(Escaped);
    let mut unlocks = app.world().resource::<Unlocks>().0.clone();
    unlocks.sort_by_key(|achievement| achievement.id());
    assert_eq!(
        unlocks,
        [
            Achievement::Escaped,
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
fn insane_gate_needs_all_visible_achievements() {
    let mut progress = AchievementProgress::default();
    assert!(!progress.insane_unlocked());
    for achievement in Achievement::INSANE_GATE {
        assert!(!achievement.hidden());
        assert!(!progress.insane_unlocked());
        progress.unlocked.insert(achievement);
    }
    assert!(progress.insane_unlocked());
    assert!(!Achievement::INSANE_GATE.contains(&Achievement::EscapeInsane));
    assert!(Achievement::EscapeInsane.hidden());
    assert_eq!(
        Achievement::ALL
            .iter()
            .filter(|achievement| !achievement.hidden())
            .count(),
        Achievement::INSANE_GATE.len()
    );

    let mut hidden_only = AchievementProgress::default();
    hidden_only.unlocked.insert(Achievement::EscapeInsane);
    assert!(hidden_only.insane_unlocked());
}

#[test]
fn insane_or_maximum_custom_escape_unlocks_the_hidden_achievement() {
    let (mut app, player) = self::app();
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(!has(&app, Achievement::EscapeInsane));

    app.insert_resource(GamePreset::of(GameMode::Insane));
    let monster = app.world_mut().spawn_empty().id();
    let caught = app.world_mut().spawn(PlayerController).id();
    app.world_mut()
        .entity_mut(caught)
        .insert(Caught::new(monster));
    assert!(!has(&app, Achievement::EscapeInsane));

    let replay = app.world_mut().spawn(PlayerController).id();
    app.world_mut().entity_mut(replay).insert(Escaped);
    assert!(has(&app, Achievement::EscapeInsane));

    let (mut app, player) = self::app();
    let mut custom = GamePreset::of(GameMode::Insane).rules();
    app.insert_resource(GamePreset::Custom(custom));
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(has(&app, Achievement::EscapeInsane));

    let (mut app, player) = self::app();
    custom.flashbangs = true;
    app.insert_resource(GamePreset::Custom(custom));
    app.world_mut().entity_mut(player).insert(Escaped);
    assert!(!has(&app, Achievement::EscapeInsane));
}
