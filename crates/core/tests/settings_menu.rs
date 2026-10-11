use bevy::{
    asset::AssetPath,
    input::{
        keyboard::Key,
        mouse::{MouseScrollUnit, MouseWheel},
        touch::TouchPhase,
    },
    platform::collections::HashMap,
    prelude::*,
    ui_widgets::{Scrollbar, ScrollbarThumb, SliderDragState, SliderValue, ValueChange},
};
use bevy_asset_loader::mapped::{AssetFileStem, MapKey};
use game_assets::{key_glyph_stem, UiAssets};
use game_audio::Sound;
use game_core::{GameState, PauseState};
use game_settings::{
    parse_key, DisplayMode, GameSettings, InputBinding, MAX_SENSITIVITY,
};
use game_ui::SliderFill;

mod common;

use common::*;

const AWAITING_TEXT: &str = "PRESS KEY OR MOUSE (Esc cancels)";
const TABS: [&str; 3] = ["Audio", "Controls", "Graphics"];

fn glyphs(stems: &[&str]) -> (HashMap<AssetFileStem, Handle<Image>>, Vec<Handle<Image>>) {
    let mut map = HashMap::default();
    let mut handles = Vec::new();
    for (index, stem) in stems.iter().enumerate() {
        let handle = image_handle(index as u128);
        let path = AssetPath::from(format!("ui/input-prompts/{stem}.png"));
        map.insert(AssetFileStem::from_asset_path(&path), handle.clone());
        handles.push(handle);
    }
    (map, handles)
}

fn open_settings(app: &mut App) {
    press(app, "Settings button");
}

fn back(app: &mut App) {
    press_label(app, "Back");
}

fn tab(app: &mut App, tab: &str) {
    press(app, &format!("{tab} tab"));
}

fn row_control(app: &mut App, row: &str) -> Entity {
    let row = named(app, &format!("{row} row")).unwrap();
    child(app, row, 1)
}

fn press_control(app: &mut App, row: &str) {
    let control = row_control(app, row);
    click(app, control);
}

fn slider(app: &mut App, name: &str) -> Entity {
    named(app, &format!("{name} slider")).unwrap()
}

fn slide(app: &mut App, name: &str, value: f32) {
    let source = slider(app, name);
    app.world_mut().trigger(ValueChange {
        source,
        value,
        is_final: false,
    });
    app.update();
}

fn slider_value(app: &mut App, name: &str) -> f32 {
    let entity = slider(app, name);
    app.world().get::<SliderValue>(entity).unwrap().0
}

fn fill(app: &mut App, name: &str) -> f32 {
    let entity = slider(app, name);
    let fill = child(app, entity, 0);
    assert!(app.world().get::<SliderFill>(fill).is_some());
    match app.world().get::<Node>(fill).unwrap().width {
        Val::Percent(value) => value,
        other => panic!("unexpected fill width {other:?}"),
    }
}

fn readout(app: &mut App, name: &str) -> String {
    let row = named(app, &format!("{name} row")).unwrap();
    text_of(app, child(app, row, 2))
}

fn choice_label(app: &mut App, row: &str) -> String {
    let control = row_control(app, row);
    text_of(app, child(app, control, 0))
}

enum Shown {
    Glyph(Handle<Image>),
    Text(String),
}

fn chip(app: &mut App, row: &str) -> Shown {
    let control = row_control(app, row);
    let children = app.world().get::<Children>(control).unwrap();
    assert_eq!(children.len(), 1);
    let content = children[0];
    if let Some(image) = app.world().get::<ImageNode>(content) {
        Shown::Glyph(image.image.clone())
    } else {
        Shown::Text(text_of(app, content))
    }
}

fn chip_text(app: &mut App, row: &str) -> String {
    match chip(app, row) {
        Shown::Text(text) => text,
        Shown::Glyph(_) => panic!("{row} shows a glyph"),
    }
}

fn chip_glyph(app: &mut App, row: &str) -> Handle<Image> {
    match chip(app, row) {
        Shown::Glyph(image) => image,
        Shown::Text(text) => panic!("{row} shows text {text}"),
    }
}

fn ancestor_named(app: &App, entity: Entity, suffix: &str) -> Option<String> {
    let mut current = entity;
    loop {
        if let Some(name) = app.world().get::<Name>(current) {
            if name.as_str().ends_with(suffix) {
                return Some(name.as_str().to_owned());
            }
        }
        current = app.world().get::<ChildOf>(current)?.parent();
    }
}

fn visible_pages(app: &mut App) -> Vec<String> {
    TABS.iter()
        .map(|tab| format!("{tab} page"))
        .filter(|page| {
            let entity = named(app, page).unwrap();
            display_of(app, entity) != Display::None
        })
        .collect()
}

fn marked_tabs(app: &mut App) -> Vec<String> {
    TABS.iter()
        .map(|tab| format!("{tab} tab"))
        .filter(|name| {
            let button = named(app, name).unwrap();
            let marker = child(app, button, 1);
            display_of(app, marker) != Display::None
        })
        .collect()
}

fn keys(app: &App) -> game_settings::MovementKeys {
    app.world().resource::<GameSettings>().keys
}

#[test]
fn settings_wait_for_ui_assets_during_loading() {
    let mut app = loading_app();
    assert_eq!(named_count(&mut app, "Loading screen"), 1);
    assert!(named(&mut app, "Settings overlay").is_none());

    app.insert_resource(ui_assets());
    ready(&mut app);
    open_settings(&mut app);
    assert_eq!(named_count(&mut app, "Settings overlay"), 1);
}

#[test]
fn settings_sliders_change_persist_and_sync_in_menu_and_pause() {
    let mut app = ready_menu_app();
    let file = persist_settings(&mut app);
    assert_eq!(saved_settings(&app), None);
    app.world_mut().resource_mut::<GameSettings>().master = 0.0;
    open_settings(&mut app);
    assert_eq!(named_count(&mut app, "Settings overlay"), 1);
    assert_eq!(count::<With<SliderValue>>(&mut app), 4);
    assert_eq!(slider_value(&mut app, "Master"), 0.0);
    assert_eq!(readout(&mut app, "Master"), "0%");

    slide(&mut app, "Master", 0.35);
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    assert!((app.world().resource::<GameSettings>().master - 0.35).abs() < 0.001);
    assert_saved(&app);
    assert!((slider_value(&mut app, "Master") - 0.35).abs() < 0.001);
    assert!((fill(&mut app, "Master") - 35.0).abs() < 0.01);
    assert_eq!(readout(&mut app, "Master"), "35%");

    slide(&mut app, "SFX", 0.7);
    assert_eq!(app.world().resource::<GameSettings>().sfx, 0.7);
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    slide(&mut app, "Music (reserved)", 0.5);
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    assert_eq!(app.world().resource::<GameSettings>().music, 0.5);
    assert_eq!(readout(&mut app, "Music (reserved)"), "50%");
    assert_eq!(
        readout(&mut app, "Mouse sensitivity"),
        "1.5"
    );
    slide(&mut app, "Mouse sensitivity", 0.0);
    assert_eq!(
        app.world().resource::<GameSettings>().mouse_sensitivity,
        0.0
    );
    assert_eq!(readout(&mut app, "Mouse sensitivity"), "0.0");
    slide(&mut app, "Mouse sensitivity", 1.0);
    assert_eq!(
        app.world().resource::<GameSettings>().mouse_sensitivity,
        MAX_SENSITIVITY
    );
    assert_eq!(
        readout(&mut app, "Mouse sensitivity"),
        "4.0"
    );
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));

    forget_saved(&file);
    slide(&mut app, "Master", 0.35);
    assert_eq!(saved_settings(&app), None);
    assert!(!played_sounds(&mut app).contains(&Sound::UiFocus));

    let master = slider(&mut app, "Master");
    app.world_mut()
        .get_mut::<SliderDragState>(master)
        .unwrap()
        .dragging = true;
    slide(&mut app, "Master", 0.4);
    slide(&mut app, "Master", 0.45);
    assert!((app.world().resource::<GameSettings>().master - 0.45).abs() < 0.001);
    assert_eq!(readout(&mut app, "Master"), "45%");
    assert_eq!(saved_settings(&app), None);
    app.world_mut()
        .get_mut::<SliderDragState>(master)
        .unwrap()
        .dragging = false;
    app.update();
    assert_saved(&app);
    slide(&mut app, "Master", 0.35);

    app.world_mut().resource_mut::<GameSettings>().sfx = 0.2;
    app.update();
    app.update();
    assert!((slider_value(&mut app, "SFX") - 0.2).abs() < 0.001);
    assert!((fill(&mut app, "SFX") - 20.0).abs() < 0.01);
    assert_eq!(readout(&mut app, "SFX"), "20%");
    assert_eq!(app.world().resource::<GameSettings>().sfx, 0.2);

    back(&mut app);
    assert!(named(&mut app, "Settings overlay").is_none());
    assert_eq!(count::<With<SliderValue>>(&mut app), 0);
    assert_eq!(game_state(&app), GameState::MainMenu);

    press(&mut app, "Play button");
    escape(&mut app);
    assert_eq!(pause_state(&app), PauseState::Paused);
    open_settings(&mut app);
    assert_eq!(named_count(&mut app, "Settings overlay"), 1);
    let pause_menu = named(&mut app, "Pause menu").unwrap();
    assert_eq!(display_of(&app, pause_menu), Display::None);
    assert!((slider_value(&mut app, "Master") - 0.35).abs() < 0.001);
    assert_eq!(readout(&mut app, "Master"), "35%");
    assert!((slider_value(&mut app, "SFX") - 0.2).abs() < 0.001);

    slide(&mut app, "Master", 0.6);
    assert!((app.world().resource::<GameSettings>().master - 0.6).abs() < 0.001);
    assert_saved(&app);
    assert_eq!(readout(&mut app, "Master"), "60%");
    assert_eq!(pause_state(&app), PauseState::Paused);

    escape(&mut app);
    assert!(named(&mut app, "Settings overlay").is_none());
    assert_eq!(display_of(&app, pause_menu), Display::Flex);
    assert_eq!(pause_state(&app), PauseState::Paused);

    open_settings(&mut app);
    assert_eq!(display_of(&app, pause_menu), Display::None);
    back(&mut app);
    assert!(named(&mut app, "Settings overlay").is_none());
    assert_eq!(display_of(&app, pause_menu), Display::Flex);
}

#[test]
fn settings_rebind_and_overlay_block_play() {
    let mut app = ready_menu_app();
    persist_settings(&mut app);
    open_settings(&mut app);
    press(&mut app, "Play button");
    assert_eq!(game_state(&app), GameState::MainMenu);
    press_control(&mut app, "Forward");
    key_tap(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    assert_eq!(keys(&app).forward, KeyCode::ArrowUp);
    assert_saved(&app);
    assert_eq!(saved_settings(&app).unwrap().keys.forward, KeyCode::ArrowUp);
}

#[test]
fn settings_tabs_switch_pages_and_hold_labeled_groups() {
    let mut app = ready_menu_app();
    persist_settings(&mut app);
    open_settings(&mut app);
    assert_eq!(visible_pages(&mut app), vec!["Audio page"]);
    assert_eq!(marked_tabs(&mut app), vec!["Audio tab"]);

    for (group, page) in [
        ("VOLUME", "Audio page"),
        ("MOUSE", "Controls page"),
        ("MOVEMENT", "Controls page"),
        ("INTERACTION", "Controls page"),
        ("QUALITY", "Graphics page"),
    ] {
        let entity = named(&mut app, &format!("{group} settings")).unwrap();
        assert_eq!(text_of(&app, child(&app, entity, 0)), group);
        assert_eq!(ancestor_named(&app, entity, " page").as_deref(), Some(page));
    }

    for (slider_name, group) in [
        ("Master", "VOLUME settings"),
        ("SFX", "VOLUME settings"),
        ("Music (reserved)", "VOLUME settings"),
        ("Mouse sensitivity", "MOUSE settings"),
    ] {
        let entity = slider(&mut app, slider_name);
        assert_eq!(
            ancestor_named(&app, entity, " settings").as_deref(),
            Some(group)
        );
    }

    assert_eq!(chip_text(&mut app, "Flashlight"), "LMB");
    assert_eq!(chip_text(&mut app, "Flashbang"), "RMB");

    for (row, group) in [
        ("Quality preset", "QUALITY settings"),
        ("Display mode", "QUALITY settings"),
        ("FPS overlay", "QUALITY settings"),
        ("Interact", "INTERACTION settings"),
        ("Flashlight", "INTERACTION settings"),
        ("Flashbang", "INTERACTION settings"),
        ("Forward", "MOVEMENT settings"),
        ("Left", "MOVEMENT settings"),
        ("Backward", "MOVEMENT settings"),
        ("Right", "MOVEMENT settings"),
    ] {
        let control = row_control(&mut app, row);
        assert!(app.world().get::<Button>(control).is_some(), "{row}");
        assert_eq!(
            ancestor_named(&app, control, " settings").as_deref(),
            Some(group),
            "{row}"
        );
    }
    for name in TABS {
        assert_eq!(named_count(&mut app, &format!("{name} tab")), 1);
    }

    tab(&mut app, "Controls");
    assert_eq!(visible_pages(&mut app), vec!["Controls page"]);
    assert_eq!(marked_tabs(&mut app), vec!["Controls tab"]);
    press_control(&mut app, "Forward");
    assert_eq!(chip_text(&mut app, "Forward"), AWAITING_TEXT);
    tab(&mut app, "Graphics");
    assert_eq!(visible_pages(&mut app), vec!["Graphics page"]);
    assert_eq!(choice_label(&mut app, "Display mode"), "Fullscreen");
    press_control(&mut app, "Display mode");
    assert_eq!(
        app.world().resource::<GameSettings>().display_mode,
        DisplayMode::Windowed
    );
    assert_saved(&app);
    assert_eq!(choice_label(&mut app, "Display mode"), "Windowed");
    assert_eq!(chip_text(&mut app, "Forward"), "W");

    back(&mut app);
    assert!(named(&mut app, "Settings overlay").is_none());
    open_settings(&mut app);
    assert_eq!(visible_pages(&mut app), vec!["Graphics page"]);
}

#[test]
fn settings_panel_hides_menu_and_scrolls_controls() {
    let mut app = ready_menu_app();
    open_settings(&mut app);
    let menu = named(&mut app, "Main menu").unwrap();
    assert_eq!(display_of(&app, menu), Display::None);
    let overlay = named(&mut app, "Settings overlay").unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(overlay).unwrap().0,
        Color::NONE
    );
    let panel = child(&app, overlay, 0);
    let panel_node = app.world().get::<Node>(panel).unwrap();
    assert_eq!(panel_node.height, px(620));
    assert_eq!(panel_node.max_height, percent(90));
    let scroll = named(&mut app, "Settings tab body").unwrap();
    assert_eq!(
        app.world().get::<Node>(scroll).unwrap().overflow.y,
        OverflowAxis::Scroll
    );
    let bar = named(&mut app, "Settings scrollbar").unwrap();
    assert_eq!(app.world().get::<Scrollbar>(bar).unwrap().target, scroll);
    assert_eq!(
        app.world().get::<Visibility>(bar),
        Some(&Visibility::Hidden)
    );
    let thumb = child(&app, bar, 0);
    assert!(app.world().get::<ScrollbarThumb>(thumb).is_some());
    tab(&mut app, "Controls");
    app.world_mut().write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        x: 0.0,
        y: -3.0,
        window: Entity::PLACEHOLDER,
        phase: TouchPhase::Moved,
    });
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(scroll).unwrap().0.y,
        114.0
    );
    tab(&mut app, "Audio");
    assert_eq!(app.world().get::<ScrollPosition>(scroll).unwrap().0.y, 0.0);
    back(&mut app);
    assert_eq!(display_of(&app, menu), Display::Flex);
    open_settings(&mut app);
    assert_eq!(display_of(&app, menu), Display::None);
    escape(&mut app);
    assert!(named(&mut app, "Settings overlay").is_none());
    assert_eq!(display_of(&app, menu), Display::Flex);
}

#[test]
fn fps_setting_is_available_and_off_by_default() {
    let mut app = ready_menu_app();
    assert!(!app.world().resource::<GameSettings>().fps_overlay);
    open_settings(&mut app);
    tab(&mut app, "Graphics");
    assert_eq!(choice_label(&mut app, "FPS overlay"), "Off");
    press_control(&mut app, "FPS overlay");
    assert!(app.world().resource::<GameSettings>().fps_overlay);
    assert_eq!(choice_label(&mut app, "FPS overlay"), "On");
}

#[test]
fn settings_buttons_and_slider_changes_play_feedback() {
    let mut app = ready_menu_app();
    open_settings(&mut app);
    assert!(played_sounds(&mut app).contains(&Sound::UiConfirm));

    tab(&mut app, "Graphics");
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    for row in ["Quality preset", "Display mode", "FPS overlay"] {
        press_control(&mut app, row);
        assert!(played_sounds(&mut app).contains(&Sound::UiPress));
    }
    tab(&mut app, "Controls");
    played_sounds(&mut app);
    press_control(&mut app, "Flashlight");
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    mouse_click(&mut app, MouseButton::Middle);
    assert!(played_sounds(&mut app).contains(&Sound::UiConfirm));

    back(&mut app);
    assert!(played_sounds(&mut app).contains(&Sound::UiBack));
}

#[test]
fn key_chips_show_bound_glyphs_with_text_fallback() {
    let (key_glyphs, handles) = glyphs(&[
        "T_W_Key_Alt",
        "T_F_Key_Alt",
        "T_Mouse_Left_Key_Alt",
        "T_Mouse_Right_Key_Alt",
        "T_Mouse_Middle_Key_Alt",
    ]);
    let mut app = menu_app_with(UiAssets {
        key_glyphs,
        ..ui_assets()
    });
    persist_settings(&mut app);
    ready(&mut app);
    open_settings(&mut app);
    tab(&mut app, "Controls");
    assert_eq!(chip_glyph(&mut app, "Forward"), handles[0]);
    assert_eq!(chip_glyph(&mut app, "Interact"), handles[1]);
    assert_eq!(chip_text(&mut app, "Left"), "A");
    assert_eq!(chip_glyph(&mut app, "Flashlight"), handles[2]);
    assert_eq!(chip_glyph(&mut app, "Flashbang"), handles[3]);

    press_control(&mut app, "Forward");
    assert_eq!(chip_text(&mut app, "Forward"), AWAITING_TEXT);
    escape(&mut app);
    assert_eq!(chip_glyph(&mut app, "Forward"), handles[0]);
    assert_eq!(saved_settings(&app), None);

    press_control(&mut app, "Forward");
    key_tap(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    assert_eq!(keys(&app).forward, KeyCode::ArrowUp);
    assert_eq!(chip_text(&mut app, "Forward"), "ArrowUp");

    press_control(&mut app, "Left");
    key_tap(&mut app, KeyCode::KeyW, Key::Character("w".into()));
    assert_eq!(keys(&app).left, KeyCode::KeyW);
    assert_eq!(chip_glyph(&mut app, "Left"), handles[0]);
    assert_saved(&app);

    press_control(&mut app, "Flashlight");
    mouse_click(&mut app, MouseButton::Middle);
    assert_eq!(chip_glyph(&mut app, "Flashlight"), handles[4]);
}

#[test]
fn flashlight_and_flashbang_bindings_capture_mouse_and_keyboard_without_the_opening_click() {
    let mut app = ready_menu_app();
    persist_settings(&mut app);
    open_settings(&mut app);
    press_control(&mut app, "Flashlight");
    assert_eq!(
        keys(&app).flashlight,
        InputBinding::Mouse(MouseButton::Left)
    );
    mouse_click(&mut app, MouseButton::Middle);
    assert_eq!(
        keys(&app).flashlight,
        InputBinding::Mouse(MouseButton::Middle)
    );
    press_control(&mut app, "Flashbang");
    key_tap(&mut app, KeyCode::KeyQ, Key::Character("q".into()));
    assert_eq!(keys(&app).flashbang, InputBinding::Key(KeyCode::KeyQ));
    press_control(&mut app, "Flashbang");
    mouse_click(&mut app, MouseButton::Middle);
    assert_eq!(keys(&app).flashbang, InputBinding::Key(KeyCode::KeyQ));
    assert_eq!(chip_text(&mut app, "Flashbang"), AWAITING_TEXT);
    assert_saved(&app);
    assert_eq!(
        saved_settings(&app).unwrap().keys.flashbang,
        InputBinding::Key(KeyCode::KeyQ)
    );
}

#[test]
fn every_bindable_key_has_a_glyph() {
    let candidates = (0..26)
        .map(|index| format!("Key{}", (b'A' + index) as char))
        .chain((0..10).map(|digit| format!("Digit{digit}")))
        .chain((1..=24).map(|index| format!("F{index}")))
        .chain(
            [
                "ArrowUp",
                "ArrowDown",
                "ArrowLeft",
                "ArrowRight",
                "Space",
                "Tab",
                "Enter",
                "Escape",
                "Backspace",
                "ShiftLeft",
                "ShiftRight",
                "ControlLeft",
                "ControlRight",
                "AltLeft",
                "AltRight",
                "Minus",
                "Equal",
                "Comma",
                "Period",
                "Slash",
                "Backslash",
                "Semicolon",
                "Quote",
                "Backquote",
                "BracketLeft",
                "BracketRight",
                "Insert",
                "Delete",
                "Home",
                "End",
                "PageUp",
                "PageDown",
                "Numpad0",
                "NumpadEnter",
            ]
            .map(str::to_owned),
        );
    let mut accepted = 0;
    let mut missing = Vec::new();
    for name in candidates {
        if parse_key(&name).is_some() {
            accepted += 1;
            if key_glyph_stem(&name).is_none() {
                missing.push(name);
            }
        }
    }
    assert!(accepted > 60, "{accepted}");
    assert_eq!(
        missing,
        vec![
            "F13",
            "F14",
            "F15",
            "F16",
            "F17",
            "F18",
            "F19",
            "F20",
            "F21",
            "F22",
            "F23",
            "F24",
            "Equal",
            "Comma",
            "Period",
            "Backslash",
            "Numpad0",
            "NumpadEnter",
        ]
    );
}
