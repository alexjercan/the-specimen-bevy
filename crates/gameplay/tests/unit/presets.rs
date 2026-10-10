use bevy::prelude::*;

use super::{
    GameMode, GamePreset, MonsterChaseSpeed, MonsterRunHearing, MonsterWalkHearing,
    MonsterWalkSpeed,
};
use crate::{
    controller::{PlayerController, RUN_SPEED, Stamina},
    levels::{FacilityPower, FuseSeed, PickupKind, build_first_floor, spawn_first_floor_actors},
};

#[test]
fn tuning_variants_resolve_to_the_requested_values() {
    assert_eq!(MonsterWalkSpeed::Slow.value(), 0.5);
    assert_eq!(MonsterWalkSpeed::Medium.value(), 0.75);
    assert_eq!(MonsterWalkSpeed::Fast.value(), 1.0);
    assert_eq!(MonsterChaseSpeed::Normal.value(), RUN_SPEED + 1.0);
    assert_eq!(MonsterChaseSpeed::Medium.value(), 9.5);
    assert_eq!(MonsterChaseSpeed::Fast.value(), RUN_SPEED * 2.0);
    assert_eq!(MonsterWalkHearing::Short.value(), 3.0);
    assert_eq!(MonsterWalkHearing::Medium.value(), 4.5);
    assert_eq!(MonsterWalkHearing::Long.value(), 6.0);
    assert_eq!(MonsterRunHearing::Short.value(), 9.0);
    assert_eq!(MonsterRunHearing::Medium.value(), 13.5);
    assert_eq!(MonsterRunHearing::Long.value(), 18.0);
}

#[test]
fn presets_choose_the_requested_variants() {
    let normal = GamePreset::default();
    assert_eq!(normal, GamePreset::of(GameMode::Normal));
    assert_eq!(normal.mode, GameMode::Normal);
    assert_eq!(normal.monster.walk_speed, MonsterWalkSpeed::Slow);
    assert_eq!(normal.monster.chase_speed, MonsterChaseSpeed::Normal);
    assert_eq!(normal.monster.walk_hearing, MonsterWalkHearing::Short);
    assert_eq!(normal.monster.run_hearing, MonsterRunHearing::Short);
    assert!(normal.sprint && normal.flashbangs && normal.detector && !normal.blackout);

    let custom = GamePreset::of(GameMode::Custom);
    assert_eq!(custom.mode, GameMode::Custom);
    assert_eq!(custom.monster, normal.monster);
    assert!(!custom.insane_equivalent());

    let insane = GamePreset::of(GameMode::Insane);
    assert_eq!(insane.mode, GameMode::Insane);
    assert_eq!(insane.monster.walk_speed, MonsterWalkSpeed::Fast);
    assert_eq!(insane.monster.chase_speed, MonsterChaseSpeed::Fast);
    assert_eq!(insane.monster.walk_hearing, MonsterWalkHearing::Long);
    assert_eq!(insane.monster.run_hearing, MonsterRunHearing::Long);
    assert_ne!(
        insane.monster.chase_speed.value(),
        normal.monster.chase_speed.value() * 2.0
    );
    assert!(!insane.sprint && !insane.flashbangs && !insane.detector && insane.blackout);
    assert!(insane.insane_equivalent());
    let mut max_custom = insane;
    max_custom.mode = GameMode::Custom;
    assert!(max_custom.insane_equivalent());
    for easier in [
        GamePreset {
            sprint: true,
            ..max_custom
        },
        GamePreset {
            flashbangs: true,
            ..max_custom
        },
        GamePreset {
            detector: true,
            ..max_custom
        },
        GamePreset {
            blackout: false,
            ..max_custom
        },
        GamePreset {
            monster: normal.monster,
            ..max_custom
        },
    ] {
        assert!(!easier.insane_equivalent());
    }
}

fn build(mode: GameMode) -> World {
    build_with(GamePreset::of(mode))
}

fn build_with(preset: GamePreset) -> World {
    let mut world = World::new();
    world.insert_resource(FuseSeed(7));
    world.insert_resource(preset);
    world.run_system_cached(build_first_floor).unwrap();
    world.run_system_cached(spawn_first_floor_actors).unwrap();
    world
}

fn devices(world: &mut World) -> (usize, usize) {
    let kinds: Vec<PickupKind> = world.query::<&PickupKind>().iter(world).copied().collect();
    (
        kinds
            .iter()
            .filter(|kind| **kind == PickupKind::Flashbang)
            .count(),
        kinds
            .iter()
            .filter(|kind| **kind == PickupKind::Detector)
            .count(),
    )
}

fn stamina(world: &mut World) -> Stamina {
    *world
        .query_filtered::<&Stamina, With<PlayerController>>()
        .single(world)
        .unwrap()
}

#[test]
fn level_build_applies_the_selected_preset() {
    let mut normal = build(GameMode::Normal);
    assert_eq!(devices(&mut normal), (1, 1));
    assert_eq!(*normal.resource::<FacilityPower>(), FacilityPower::new(7));
    assert_eq!(stamina(&mut normal), Stamina::default());

    let mut insane = build(GameMode::Insane);
    assert_eq!(devices(&mut insane), (0, 0));
    let power = *insane.resource::<FacilityPower>();
    assert!(!power.on && !power.outage_pending && !power.repairable());
    assert_eq!(stamina(&mut insane), Stamina::spent());
}

#[test]
fn level_build_without_a_preset_uses_normal() {
    let mut world = World::new();
    world.insert_resource(FuseSeed(7));
    world.run_system_cached(build_first_floor).unwrap();
    world.run_system_cached(spawn_first_floor_actors).unwrap();
    assert_eq!(devices(&mut world), (1, 1));
    assert!(world.resource::<FacilityPower>().on);
    assert_eq!(stamina(&mut world), Stamina::default());
}

#[test]
fn flashbang_and_detector_pickups_are_controlled_separately() {
    let normal = GamePreset::default();
    let mut flashbang_only = build_with(GamePreset {
        detector: false,
        ..normal
    });
    assert_eq!(devices(&mut flashbang_only), (1, 0));
    let mut detector_only = build_with(GamePreset {
        flashbangs: false,
        ..normal
    });
    assert_eq!(devices(&mut detector_only), (0, 1));
}
