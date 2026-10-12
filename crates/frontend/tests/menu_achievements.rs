use bevy::{platform::collections::HashMap, prelude::*};
use game_assets::FacilityAssets;
use game_flow::{GameState, PauseState};
use game_frontend::MenuAssets;
use game_ui::theme;
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

mod common;

use common::*;

const TOAST_SECS: f32 = 4.0;

fn progress(unlocked: &[Achievement]) -> AchievementProgress {
    AchievementProgress {
        unlocked: unlocked.iter().copied().collect(),
    }
}

fn row(app: &mut App, achievement: Achievement) -> Entity {
    named(app, &format!("{} row", achievement.name())).unwrap()
}

fn row_icon(app: &mut App, achievement: Achievement) -> Handle<Image> {
    let row = row(app, achievement);
    let icon = child(app, row, 0);
    app.world().get::<ImageNode>(icon).unwrap().image.clone()
}

fn row_status(app: &mut App, achievement: Achievement) -> String {
    let row = row(app, achievement);
    assert!(app.world().get::<ImageNode>(child(app, row, 0)).is_some());
    text_of(app, child(app, row, 2))
}

fn row_name_and_criteria(app: &mut App, achievement: Achievement) -> (String, String) {
    let row = row(app, achievement);
    let column = child(app, row, 1);
    (
        text_of(app, child(app, column, 0)),
        text_of(app, child(app, column, 1)),
    )
}

fn packaged_icons() -> (
    HashMap<String, Handle<Image>>,
    HashMap<Handle<Image>, String>,
) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/achievements");
    let mut stems: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
        .collect();
    stems.sort();
    let mut icons = HashMap::default();
    let mut names = HashMap::default();
    for (index, stem) in stems.iter().enumerate() {
        let handle = image_handle(1000 + index as u128);
        icons.insert(stem.clone(), handle.clone());
        names.insert(handle, stem.clone());
    }
    (icons, names)
}

#[test]
fn achievements_screen_lists_all_entries_with_counts_and_status() {
    let mut app = ready_menu_app();
    let unlocked = [Achievement::RestoreBoiler, Achievement::EscapeUndetected];
    app.insert_resource(progress(&unlocked));

    press(&mut app, "Achievements button");
    assert_eq!(game_state(&app), GameState::Achievements);
    assert_eq!(named_count(&mut app, "Achievements screen"), 1);
    let screen = named(&mut app, "Achievements screen").unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(screen).unwrap().0,
        Color::NONE
    );
    let panel = named(&mut app, "Achievements panel").unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(panel).unwrap().0,
        theme::PANEL
    );
    assert_eq!(app.world().get::<Node>(panel).unwrap().width, px(560));
    assert!(has_text(&mut app, "2 / 9 unlocked"));

    for achievement in Achievement::ALL {
        let expected = if unlocked.contains(&achievement) {
            "UNLOCKED"
        } else {
            "LOCKED"
        };
        assert_eq!(row_status(&mut app, achievement), expected);
        if !achievement.hidden() {
            assert!(has_text(&mut app, achievement.name()));
        }
    }
}

#[test]
fn every_achievement_row_and_toast_uses_a_packaged_icon() {
    let (achievement_icons, stems) = packaged_icons();
    assert_eq!(stems.len(), Achievement::ALL.len() * 2);
    let mut app = menu_app_with(MenuAssets {
        achievement_icons,
        ..menu_assets()
    });
    ready(&mut app);
    let unlocked = [Achievement::Escaped, Achievement::EscapeInsane];
    app.insert_resource(progress(&unlocked));
    press(&mut app, "Achievements button");

    let mut seen = Vec::new();
    for achievement in Achievement::ALL {
        let handle = row_icon(&mut app, achievement);
        let stem = stems
            .get(&handle)
            .unwrap_or_else(|| panic!("{} row icon is not a packaged icon", achievement.name()));
        let suffix = if unlocked.contains(&achievement) {
            "-unlocked"
        } else {
            "-locked"
        };
        assert!(stem.ends_with(suffix), "{} uses {stem}", achievement.name());
        assert!(!seen.contains(stem), "{stem} is used twice");
        seen.push(stem.clone());
    }

    app.world_mut()
        .trigger(AchievementUnlocked(Achievement::RestoreBoiler));
    app.update();
    let toast = named(&mut app, "Achievement toast").unwrap();
    let icon = child(&app, toast, 0);
    let handle = app.world().get::<ImageNode>(icon).unwrap().image.clone();
    let stem = stems.get(&handle).unwrap();
    assert!(stem.ends_with("-unlocked"));
    let row_stem = stems[&row_icon(&mut app, Achievement::RestoreBoiler)].clone();
    assert!(row_stem.ends_with("-locked"));
    assert_eq!(*stem, row_stem.replace("-locked", "-unlocked"));
}

#[test]
fn achievements_reuses_the_menu_room_when_facility_assets_are_loaded() {
    let mut app = menu_app();
    app.insert_resource(FacilityAssets {
        modules: Default::default(),
    });
    ready(&mut app);
    press(&mut app, "Achievements button");
    assert_eq!(game_state(&app), GameState::Achievements);
    assert_eq!(count::<With<Camera3d>>(&mut app), 1);
    assert_eq!(count::<With<IsDefaultUiCamera>>(&mut app), 1);
    assert_eq!(named_count(&mut app, "Achievements screen"), 1);
    press(&mut app, "Back button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert_eq!(count::<With<Camera3d>>(&mut app), 1);
}

#[test]
fn back_button_and_escape_both_return_to_main_menu_and_clean_up() {
    let mut app = ready_menu_app();
    let menu = entities(&mut app);

    press(&mut app, "Achievements button");
    assert_eq!(game_state(&app), GameState::Achievements);
    assert!(played_sounds(&mut app).contains(&menu_sounds().confirm));
    press(&mut app, "Back button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert!(played_sounds(&mut app).contains(&menu_sounds().back));
    assert!(named(&mut app, "Achievements screen").is_none());
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(entities(&mut app), menu);

    press(&mut app, "Achievements button");
    assert_eq!(game_state(&app), GameState::Achievements);
    escape(&mut app);
    assert_eq!(game_state(&app), GameState::MainMenu);
    assert!(played_sounds(&mut app).contains(&menu_sounds().back));
    assert!(named(&mut app, "Achievements screen").is_none());
    assert_eq!(count::<With<Camera2d>>(&mut app), 1);
    assert_eq!(entities(&mut app), menu);
}

#[test]
fn locked_hidden_achievement_hides_its_name_and_criteria() {
    let mut app = ready_menu_app();
    press(&mut app, "Achievements button");
    assert_eq!(row_status(&mut app, Achievement::EscapeInsane), "LOCKED");
    let (name, criteria) = row_name_and_criteria(&mut app, Achievement::EscapeInsane);
    assert_eq!(name, "?");
    assert_eq!(criteria, "");
    assert!(!has_text(&mut app, "Out of your mind"));
    assert!(!has_text(&mut app, "Escape in Insane mode."));
}

#[test]
fn unlocked_hidden_achievement_reveals_its_name_and_criteria() {
    let mut app = ready_menu_app();
    app.insert_resource(progress(&[Achievement::EscapeInsane]));
    press(&mut app, "Achievements button");
    assert_eq!(row_status(&mut app, Achievement::EscapeInsane), "UNLOCKED");
    let (name, criteria) = row_name_and_criteria(&mut app, Achievement::EscapeInsane);
    assert_eq!(name, "Out of your mind");
    assert_eq!(criteria, "Escape in Insane mode.");
}

#[test]
fn achievement_unlock_spawns_and_then_clears_a_toast() {
    let mut app = ready_menu_app();
    played_sounds(&mut app);

    app.world_mut()
        .trigger(AchievementUnlocked(Achievement::RestoreBoiler));
    app.update();
    assert_eq!(named_count(&mut app, "Achievement toast"), 1);
    assert_eq!(named_count(&mut app, "Achievement toast container"), 1);
    assert!(has_text(&mut app, "ACHIEVEMENT UNLOCKED"));
    assert!(has_text(&mut app, Achievement::RestoreBoiler.name()));
    let toast = named(&mut app, "Achievement toast").unwrap();
    assert!(app
        .world()
        .get::<ImageNode>(child(&app, toast, 0))
        .is_some());
    assert!(played_sounds(&mut app).contains(&menu_sounds().confirm));
    let container = named(&mut app, "Achievement toast container").unwrap();
    assert_eq!(
        app.world().get::<Pickable>(container),
        Some(&Pickable::IGNORE)
    );

    run_for(&mut app, TOAST_SECS + 0.2);
    assert_eq!(named_count(&mut app, "Achievement toast"), 0);
}

#[test]
fn achievements_from_pause_hide_controls_and_return_without_resuming() {
    let mut app = ready_menu_app();
    press(&mut app, "Play button");
    escape(&mut app);
    assert_eq!(pause_state(&app), PauseState::Paused);
    let pause_menu = named(&mut app, "Pause menu").unwrap();

    press(&mut app, "Achievements button");
    assert_eq!(game_state(&app), GameState::Playing);
    assert_eq!(pause_state(&app), PauseState::Paused);
    assert_eq!(named_count(&mut app, "Achievements screen"), 1);
    assert_eq!(display_of(&app, pause_menu), Display::None);
    assert!(has_text(&mut app, "0 / 9 unlocked"));

    press(&mut app, "Back button");
    assert!(named(&mut app, "Achievements screen").is_none());
    assert_eq!(display_of(&app, pause_menu), Display::Flex);
    assert_eq!(pause_state(&app), PauseState::Paused);

    press(&mut app, "Achievements button");
    assert_eq!(named_count(&mut app, "Achievements screen"), 1);
    assert_eq!(display_of(&app, pause_menu), Display::None);
    escape(&mut app);
    assert!(named(&mut app, "Achievements screen").is_none());
    assert_eq!(display_of(&app, pause_menu), Display::Flex);
    assert_eq!(pause_state(&app), PauseState::Paused);
    assert_eq!(game_state(&app), GameState::Playing);

    press(&mut app, "Resume button");
    assert_eq!(pause_state(&app), PauseState::Running);
}
