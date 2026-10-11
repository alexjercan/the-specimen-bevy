use bevy::prelude::*;
use gameplay::{
    controller::{PlayerController, Stamina},
    levels::{build_first_floor, spawn_first_floor_actors, FacilityPower, FuseSeed, PickupKind},
    presets::{GameMode, GamePreset},
};

mod common;

#[test]
fn insane_equivalence_needs_every_handicap_of_the_insane_preset() {
    let normal = GamePreset::default();
    assert_eq!(normal, GamePreset::of(GameMode::Normal));
    assert!(!normal.insane_equivalent());
    assert!(normal.sprint && normal.flashbangs && normal.detector && !normal.blackout);

    let custom = GamePreset::of(GameMode::Custom);
    assert_eq!(custom.monster, normal.monster);
    assert!(!custom.insane_equivalent());

    let insane = GamePreset::of(GameMode::Insane);
    assert_ne!(insane.monster, normal.monster);
    assert!(!insane.sprint && !insane.flashbangs && !insane.detector && insane.blackout);
    assert!(insane.insane_equivalent());
    let max_custom = GamePreset {
        mode: GameMode::Custom,
        ..insane
    };
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
    assert!(!power.on && !power.outage_pending && !power.repairable());
    assert_eq!(stamina(&mut insane), Stamina::spent());
}

#[test]
fn flashbang_and_detector_pickups_are_controlled_separately() {
    let normal = GamePreset::default();
    let mut flashbang_only = build(GamePreset {
        detector: false,
        ..normal
    });
    assert_eq!(devices(&mut flashbang_only), (1, 0));
    let mut detector_only = build(GamePreset {
        flashbangs: false,
        ..normal
    });
    assert_eq!(devices(&mut detector_only), (0, 1));
}
