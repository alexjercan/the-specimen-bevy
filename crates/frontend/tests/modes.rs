use bevy::{prelude::*, ui_widgets::Button};
use game_flow::GameState;
use game_settings::{CustomLevel, GameSettings, PreferredMode};
use gameplay::{
    achievements::{Achievement, AchievementProgress},
    facility::FacilityPower,
    interaction::PickupKind,
    monster::Caught,
    player::{PlayerController, Stamina},
    presets::{
        GameMode, GamePreset, MonsterChaseSpeed, MonsterRunHearing, MonsterWalkHearing,
        MonsterWalkSpeed, PresetRules,
    },
};

mod common;

use common::*;

fn insane_gate_unlocked() -> AchievementProgress {
    AchievementProgress {
        unlocked: Achievement::INSANE_GATE.into_iter().collect(),
    }
}

fn unlock(app: &mut App, achievement: Achievement) {
    app.world_mut()
        .resource_mut::<AchievementProgress>()
        .unlocked
        .insert(achievement);
}

fn mode_label(app: &mut App) -> String {
    let label = named(app, "Mode label").unwrap();
    text_of(app, child(app, label, 0))
}

fn custom_value(app: &mut App, field: &str) -> String {
    let value = named(app, &format!("{field} value")).unwrap();
    text_of(app, child(app, value, 0))
}

fn next_mode(app: &mut App) {
    press(app, "Mode next button");
    app.update();
}

fn previous_mode(app: &mut App) {
    press(app, "Mode previous button");
    app.update();
}

fn adjust_custom(app: &mut App, field: &str, step: isize) {
    let option = named(app, &format!("{field} option")).unwrap();
    let controls = child(app, option, 1);
    let arrow = child(app, controls, if step < 0 { 0 } else { 2 });
    click(app, arrow);
    app.update();
}

fn devices(app: &mut App) -> (usize, usize) {
    let kinds: Vec<PickupKind> = app
        .world_mut()
        .query::<&PickupKind>()
        .iter(app.world())
        .copied()
        .collect();
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

fn stamina(app: &mut App) -> Stamina {
    *app.world_mut()
        .query_filtered::<&Stamina, With<PlayerController>>()
        .single(app.world())
        .unwrap()
}

fn assert_insane_run(app: &mut App) {
    assert_eq!(game_state(app), GameState::Playing);
    assert_eq!(
        app.world().resource::<GamePreset>().mode(),
        GameMode::Insane
    );
    assert_eq!(stamina(app), Stamina::spent());
    assert_eq!(devices(app), (0, 0));
    let power = *app.world().resource::<FacilityPower>();
    assert!(!power.is_on() && !power.repairable());
}

fn assert_normal_run(app: &mut App) {
    assert_eq!(game_state(app), GameState::Playing);
    assert_eq!(
        app.world().resource::<GamePreset>().mode(),
        GameMode::Normal
    );
    assert_eq!(stamina(app), Stamina::default());
    assert_eq!(devices(app), (1, 1));
    assert!(app.world().resource::<FacilityPower>().is_on());
}

#[test]
fn mode_selector_stays_hidden_until_first_escape() {
    let mut almost = insane_gate_unlocked();
    almost.unlocked.remove(&Achievement::EscapeUndetected);
    almost.unlocked.remove(&Achievement::Escaped);
    for progress in [AchievementProgress::default(), almost] {
        let mut app = menu_app();
        app.world_mut().insert_resource(progress);
        ready(&mut app);
        assert!(named(&mut app, "Play button").is_some());
        assert!(named(&mut app, "Mode selector").is_none());
        assert!(named(&mut app, "Custom options").is_none());
        assert_eq!(count::<With<Button>>(&mut app), 5);
        assert!(!has_text(&mut app, "Insane"));
        assert!(!has_text(&mut app, "Normal"));
        press(&mut app, "Play button");
        assert_normal_run(&mut app);
    }
}

#[test]
fn mode_selector_starts_at_normal_and_cycles_both_ways_after_unlock() {
    let mut app = menu_app();
    app.world_mut().insert_resource(insane_gate_unlocked());
    ready(&mut app);
    let row = named(&mut app, "Play row").unwrap();
    let selector = named(&mut app, "Mode selector").unwrap();
    let play = named(&mut app, "Play button").unwrap();
    let row_children = app.world().get::<Children>(row).unwrap();
    assert_eq!(&row_children[..], &[play, selector]);
    assert_eq!(mode_label(&mut app), "Normal");

    next_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Custom");
    next_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Insane");
    next_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Normal");
    previous_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Insane");
    previous_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Custom");
    previous_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Normal");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(
        app.world().resource::<GamePreset>().mode(),
        GameMode::Normal
    );
}

#[test]
fn first_escape_unlocks_custom_controls_and_play_uses_the_adjusted_preset() {
    let mut app = menu_app();
    unlock(&mut app, Achievement::Escaped);
    ready(&mut app);
    assert!(named(&mut app, "Mode selector").is_some());
    let panel = named(&mut app, "Custom options").unwrap();
    assert_eq!(
        app.world().get::<Visibility>(panel),
        Some(&Visibility::Hidden)
    );
    next_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Custom");
    assert_eq!(
        app.world().get::<Visibility>(panel),
        Some(&Visibility::Inherited)
    );
    next_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Normal");
    next_mode(&mut app);
    for field in [
        "Walk speed",
        "Chase speed",
        "Walk hearing",
        "Run hearing",
        "Sprint",
        "Flashbangs",
        "Detector",
        "Blackout",
    ] {
        adjust_custom(&mut app, field, 1);
    }
    assert_eq!(custom_value(&mut app, "Walk speed"), "Medium (0.75 m/s)");
    assert_eq!(custom_value(&mut app, "Chase speed"), "Medium (9.5 m/s)");
    assert_eq!(custom_value(&mut app, "Walk hearing"), "Medium (4.5 m)");
    assert_eq!(custom_value(&mut app, "Run hearing"), "Medium (13.5 m)");
    assert_eq!(custom_value(&mut app, "Sprint"), "Off");
    assert_eq!(custom_value(&mut app, "Flashbangs"), "Off");
    assert_eq!(custom_value(&mut app, "Detector"), "Off");
    assert_eq!(custom_value(&mut app, "Blackout"), "On");

    let mut custom = PresetRules::NORMAL;
    custom.monster.walk_speed = MonsterWalkSpeed::Medium;
    custom.monster.chase_speed = MonsterChaseSpeed::Medium;
    custom.monster.walk_hearing = MonsterWalkHearing::Medium;
    custom.monster.run_hearing = MonsterRunHearing::Medium;
    custom.sprint = false;
    custom.flashbangs = false;
    custom.detector = false;
    custom.blackout = true;
    assert!(!GamePreset::Custom(custom).insane_equivalent());
    press(&mut app, "Play button");
    assert_eq!(
        *app.world().resource::<GamePreset>(),
        GamePreset::Custom(custom)
    );
    assert_eq!(devices(&mut app), (0, 0));
    assert_eq!(stamina(&mut app), Stamina::spent());
    assert!(!app.world().resource::<FacilityPower>().is_on());
    catch(&mut app);
    run_for(&mut app, REVEAL_AT + 0.2);
    press(&mut app, "Retry button");
    assert_eq!(
        *app.world().resource::<GamePreset>(),
        GamePreset::Custom(custom)
    );
}

#[test]
fn hidden_insane_unlock_also_unlocks_the_preset_selector() {
    let mut app = menu_app();
    unlock(&mut app, Achievement::EscapeInsane);
    ready(&mut app);
    assert!(named(&mut app, "Mode selector").is_some());
    previous_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Insane");
}

#[test]
fn play_launches_the_selected_mode_and_keeps_it_selected_on_return() {
    let mut app = menu_app();
    app.world_mut().insert_resource(insane_gate_unlocked());
    ready(&mut app);
    previous_mode(&mut app);
    assert_eq!(mode_label(&mut app), "Insane");
    press(&mut app, "Play button");
    assert_insane_run(&mut app);

    escape(&mut app);
    press(&mut app, "Main menu button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(mode_label(&mut app), "Insane");
    press(&mut app, "Play button");
    assert_insane_run(&mut app);
}

#[test]
fn saved_custom_mode_and_values_restore_on_a_new_menu() {
    let mut first = menu_app();
    persist_settings(&mut first);
    unlock(&mut first, Achievement::Escaped);
    ready(&mut first);
    next_mode(&mut first);
    for field in ["Walk speed", "Run hearing", "Sprint", "Blackout"] {
        adjust_custom(&mut first, field, 1);
    }
    assert_saved(&first);
    let saved = saved_settings(&first).unwrap();
    assert_eq!(saved.preferred_mode, PreferredMode::Custom);

    let mut second = menu_app();
    second.world_mut().insert_resource(saved);
    unlock(&mut second, Achievement::Escaped);
    ready(&mut second);
    assert_eq!(mode_label(&mut second), "Custom");
    assert_eq!(custom_value(&mut second, "Walk speed"), "Medium (0.75 m/s)");
    assert_eq!(custom_value(&mut second, "Run hearing"), "Medium (13.5 m)");
    assert_eq!(custom_value(&mut second, "Sprint"), "Off");
    assert_eq!(custom_value(&mut second, "Blackout"), "On");
    let mut custom = PresetRules::NORMAL;
    custom.monster.walk_speed = MonsterWalkSpeed::Medium;
    custom.monster.run_hearing = MonsterRunHearing::Medium;
    custom.sprint = false;
    custom.blackout = true;
    press(&mut second, "Play button");
    assert_eq!(
        *second.world().resource::<GamePreset>(),
        GamePreset::Custom(custom)
    );
}

#[test]
fn locked_saved_mode_falls_back_to_normal_without_losing_custom_values() {
    let mut app = menu_app();
    let mut settings = GameSettings {
        preferred_mode: PreferredMode::Insane,
        ..Default::default()
    };
    settings.custom_difficulty.run_hearing = CustomLevel::High;
    app.world_mut().insert_resource(settings);
    ready(&mut app);
    assert!(named(&mut app, "Mode selector").is_none());
    press(&mut app, "Play button");
    assert_normal_run(&mut app);
    let settings = app.world().resource::<GameSettings>();
    assert_eq!(settings.preferred_mode, PreferredMode::Insane);
    assert_eq!(settings.custom_difficulty.run_hearing, CustomLevel::High);
}

#[test]
fn retry_after_an_insane_game_over_keeps_insane_and_rebuilds_a_fresh_run() {
    let mut app = menu_app();
    app.world_mut().insert_resource(insane_gate_unlocked());
    ready(&mut app);
    previous_mode(&mut app);
    press(&mut app, "Play button");
    assert_insane_run(&mut app);
    let first = player(&mut app);
    catch(&mut app);
    assert_eq!(game_state(&app), GameState::GameOver);
    run_for(&mut app, REVEAL_AT + 0.2);
    press(&mut app, "Retry button");
    assert_insane_run(&mut app);
    let retry = player(&mut app);
    assert_ne!(first, retry);
    assert!(app.world().get::<Caught>(retry).is_none());
}

#[test]
fn retry_after_a_normal_game_over_keeps_normal() {
    let mut app = menu_app();
    app.world_mut().insert_resource(insane_gate_unlocked());
    ready(&mut app);
    press(&mut app, "Play button");
    assert_normal_run(&mut app);
    catch(&mut app);
    assert_eq!(game_state(&app), GameState::GameOver);
    run_for(&mut app, REVEAL_AT + 0.2);
    press(&mut app, "Retry button");
    assert_normal_run(&mut app);
}
