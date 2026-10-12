use bevy::prelude::*;
use gameplay::{
    facility::{build_first_floor, spawn_first_floor_actors, FacilityPower},
    fuses::FuseSeed,
    interaction::PickupKind,
    player::{PlayerController, Stamina},
    presets::{GameMode, GamePreset, PresetRules},
};

mod common;

#[test]
fn insane_equivalence_needs_every_handicap_of_the_insane_preset() {
    let normal = GamePreset::default();
    assert_eq!(normal, GamePreset::of(GameMode::Normal));
    assert!(!normal.insane_equivalent());
    let rules = normal.rules();
    assert!(rules.sprint && rules.flashbangs && rules.detector && !rules.blackout);

    let custom = GamePreset::of(GameMode::Custom);
    assert_eq!(custom.rules().monster, normal.rules().monster);
    assert!(!custom.insane_equivalent());

    let insane = GamePreset::of(GameMode::Insane);
    let rules = insane.rules();
    assert_ne!(rules.monster, normal.rules().monster);
    assert!(!rules.sprint && !rules.flashbangs && !rules.detector && rules.blackout);
    assert!(insane.insane_equivalent());
    let max_custom = insane.rules();
    assert!(GamePreset::Custom(max_custom).insane_equivalent());
    for easier in [
        PresetRules {
            sprint: true,
            ..max_custom
        },
        PresetRules {
            flashbangs: true,
            ..max_custom
        },
        PresetRules {
            detector: true,
            ..max_custom
        },
        PresetRules {
            blackout: false,
            ..max_custom
        },
        PresetRules {
            monster: normal.rules().monster,
            ..max_custom
        },
    ] {
        assert!(!GamePreset::Custom(easier).insane_equivalent());
    }
}

fn build(preset: GamePreset) -> World {
    let mut world = World::new();
    world.insert_resource(FuseSeed(common::SEED));
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
    let mut normal = build(GamePreset::default());
    assert_eq!(devices(&mut normal), (1, 1));
    assert_eq!(
        *normal.resource::<FacilityPower>(),
        FacilityPower::new(common::SEED)
    );
    assert_eq!(stamina(&mut normal), Stamina::default());

    let mut insane = build(GamePreset::of(GameMode::Insane));
    assert_eq!(devices(&mut insane), (0, 0));
    let power = *insane.resource::<FacilityPower>();
    assert!(!power.is_on() && power.outage_in().is_none() && !power.repairable());
    assert_eq!(stamina(&mut insane), Stamina::spent());
}

#[test]
fn flashbang_and_detector_pickups_are_controlled_separately() {
    let normal = PresetRules::NORMAL;
    let mut flashbang_only = build(GamePreset::Custom(PresetRules {
        detector: false,
        ..normal
    }));
    assert_eq!(devices(&mut flashbang_only), (1, 0));
    let mut detector_only = build(GamePreset::Custom(PresetRules {
        flashbangs: false,
        ..normal
    }));
    assert_eq!(devices(&mut detector_only), (0, 1));
}
