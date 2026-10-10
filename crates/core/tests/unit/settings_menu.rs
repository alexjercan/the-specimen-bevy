use bevy::{
    asset::AssetPath,
    input::touch::TouchPhase,
    input::{
        ButtonState, InputPlugin,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
    },
    platform::collections::HashMap,
    state::app::StatesPlugin,
};
use bevy_asset_loader::mapped::{AssetFileStem, MapKey};
use game_assets::key_glyph_stem;

use super::*;
use crate::{CoreState, menu::MenuPlugin};

fn glyphs(stems: &[&str]) -> (HashMap<AssetFileStem, Handle<Image>>, Vec<Handle<Image>>) {
    let mut map = HashMap::default();
    let mut handles = Vec::new();
    for (index, stem) in stems.iter().enumerate() {
        let handle = Handle::<Image>::from(bevy::asset::uuid::Uuid::from_u128(index as u128 + 1));
        let path = AssetPath::from(format!("ui/input-prompts/{stem}.png"));
        map.insert(AssetFileStem::from_asset_path(&path), handle.clone());
        handles.push(handle);
    }
    (map, handles)
}

fn app(assets: UiAssets) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, StatesPlugin))
        .init_state::<CoreState>()
        .insert_resource(assets)
        .add_plugins(MenuPlugin);
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .resource_mut::<NextState<CoreState>>()
        .set(CoreState::Ready);
    app.update();
    app
}

fn button(app: &mut App, action: SettingsAction) -> Entity {
    app.world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (*candidate == action).then_some(entity))
        .unwrap()
}

fn click(app: &mut App, action: SettingsAction) {
    let entity = button(app, action);
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
}

fn key(app: &mut App, key_code: KeyCode, logical_key: Key) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: logical_key.clone(),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

fn mouse(app: &mut App, button: MouseButton) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(MouseButtonInput {
            button,
            state,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

fn ancestor<C: Component + Copy>(app: &App, entity: Entity) -> Option<C> {
    let mut current = entity;
    loop {
        if let Some(found) = app.world().get::<C>(current) {
            return Some(*found);
        }
        current = app.world().get::<ChildOf>(current)?.parent();
    }
}

fn visible_tabs(app: &mut App) -> Vec<SettingsTab> {
    let mut tabs = app
        .world_mut()
        .query::<(&SettingsTab, &Node)>()
        .iter(app.world())
        .filter(|(_, node)| node.display != Display::None)
        .map(|(tab, _)| *tab)
        .collect::<Vec<_>>();
    tabs.sort_by_key(|tab| *tab as u8);
    tabs
}

fn marked_tabs(app: &mut App) -> Vec<SettingsTab> {
    app.world_mut()
        .query::<(&TabMarker, &Node)>()
        .iter(app.world())
        .filter(|(_, node)| node.display != Display::None)
        .map(|(marker, _)| marker.0)
        .collect()
}

enum Shown {
    Glyph(Handle<Image>),
    Text(String),
}

fn chip(app: &mut App, action: SettingsAction) -> Shown {
    let entity = app
        .world_mut()
        .query::<(Entity, &KeyChip)>()
        .iter(app.world())
        .find_map(|(entity, chip)| (chip.action == action).then_some(entity))
        .unwrap();
    let children = app.world().get::<Children>(entity).unwrap();
    assert_eq!(children.len(), 1);
    let child = children[0];
    if let Some(image) = app.world().get::<ImageNode>(child) {
        Shown::Glyph(image.image.clone())
    } else {
        Shown::Text(app.world().get::<Text>(child).unwrap().0.clone())
    }
}

fn played_sounds(app: &mut App) -> Vec<Sound> {
    app.world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .map(|cue| cue.sound)
        .collect()
}

fn chip_text(app: &mut App, action: SettingsAction) -> String {
    match chip(app, action) {
        Shown::Text(text) => text,
        Shown::Glyph(_) => panic!("{action:?} shows a glyph"),
    }
}

fn chip_glyph(app: &mut App, action: SettingsAction) -> Handle<Image> {
    match chip(app, action) {
        Shown::Glyph(image) => image,
        Shown::Text(text) => panic!("{action:?} shows text {text}"),
    }
}

#[test]
fn settings_tabs_switch_pages_and_hold_labeled_groups() {
    let mut app = app(UiAssets::default());
    click(&mut app, SettingsAction::Open);
    assert_eq!(visible_tabs(&mut app), vec![SettingsTab::Audio]);
    assert_eq!(marked_tabs(&mut app), vec![SettingsTab::Audio]);

    let mut titles = app
        .world_mut()
        .query::<(Entity, &SettingsGroup, &Children)>()
        .iter(app.world())
        .map(|(entity, group, children)| {
            let title = app.world().get::<Text>(children[0]).unwrap().0.clone();
            (*group as u8, title, ancestor::<SettingsTab>(&app, entity))
        })
        .collect::<Vec<_>>();
    titles.sort_by_key(|(group, ..)| *group);
    assert_eq!(
        titles,
        vec![
            (0, "VOLUME".into(), Some(SettingsTab::Audio)),
            (1, "MOUSE".into(), Some(SettingsTab::Controls)),
            (2, "MOVEMENT".into(), Some(SettingsTab::Controls)),
            (3, "INTERACTION".into(), Some(SettingsTab::Controls)),
            (4, "QUALITY".into(), Some(SettingsTab::Graphics)),
        ]
    );

    let sliders = app
        .world_mut()
        .query::<(Entity, &SettingSlider)>()
        .iter(app.world())
        .map(|(entity, field)| (entity, *field))
        .collect::<Vec<_>>();
    assert_eq!(sliders.len(), 4);
    for (entity, field) in sliders {
        let expected = match field {
            SettingSlider::Sensitivity => SettingsGroup::Mouse,
            _ => SettingsGroup::Volume,
        };
        assert_eq!(ancestor::<SettingsGroup>(&app, entity), Some(expected));
    }

    assert_eq!(chip_text(&mut app, SettingsAction::Flashlight), "MouseLeft");
    assert_eq!(chip_text(&mut app, SettingsAction::Flashbang), "MouseRight");

    let actions = app
        .world_mut()
        .query::<(Entity, &SettingsAction)>()
        .iter(app.world())
        .map(|(entity, action)| (entity, *action))
        .collect::<Vec<_>>();
    let mut tabs = 0;
    for (entity, action) in actions {
        let expected = match action {
            SettingsAction::Open | SettingsAction::Back => None,
            SettingsAction::Tab(_) => {
                tabs += 1;
                None
            }
            SettingsAction::Graphics | SettingsAction::DisplayMode | SettingsAction::FpsOverlay => {
                Some(SettingsGroup::Quality)
            }
            SettingsAction::Interact | SettingsAction::Flashlight | SettingsAction::Flashbang => {
                Some(SettingsGroup::Interaction)
            }
            SettingsAction::Forward
            | SettingsAction::Left
            | SettingsAction::Backward
            | SettingsAction::Right => Some(SettingsGroup::Movement),
        };
        assert_eq!(
            ancestor::<SettingsGroup>(&app, entity),
            expected,
            "{action:?}"
        );
    }
    assert_eq!(tabs, 3);

    click(&mut app, SettingsAction::Tab(SettingsTab::Controls));
    assert_eq!(visible_tabs(&mut app), vec![SettingsTab::Controls]);
    assert_eq!(marked_tabs(&mut app), vec![SettingsTab::Controls]);
    click(&mut app, SettingsAction::Forward);
    assert_eq!(
        app.world().resource::<AwaitingKey>().0,
        Some(SettingsAction::Forward)
    );
    click(&mut app, SettingsAction::Tab(SettingsTab::Graphics));
    assert_eq!(visible_tabs(&mut app), vec![SettingsTab::Graphics]);
    let display_label = app
        .world_mut()
        .query::<(&ChoiceLabel, &Text)>()
        .iter(app.world())
        .find_map(|(label, text)| (label.0 == SettingsAction::DisplayMode).then(|| text.0.clone()))
        .unwrap();
    assert_eq!(display_label, "Fullscreen");
    click(&mut app, SettingsAction::DisplayMode);
    assert_eq!(
        app.world().resource::<GameSettings>().display_mode,
        game_settings::DisplayMode::Windowed
    );
    assert!(app.world().resource::<SettingsDirty>().0);
    let display_label = app
        .world_mut()
        .query::<(&ChoiceLabel, &Text)>()
        .iter(app.world())
        .find_map(|(label, text)| (label.0 == SettingsAction::DisplayMode).then(|| text.0.clone()))
        .unwrap();
    assert_eq!(display_label, "Windowed");
    assert_eq!(app.world().resource::<AwaitingKey>().0, None);
    assert_eq!(chip_text(&mut app, SettingsAction::Forward), "W");

    click(&mut app, SettingsAction::Back);
    assert!(
        app.world_mut()
            .query::<&SettingsOverlay>()
            .iter(app.world())
            .next()
            .is_none()
    );
    click(&mut app, SettingsAction::Open);
    assert_eq!(visible_tabs(&mut app), vec![SettingsTab::Graphics]);
}

#[test]
fn settings_panel_keeps_menu_visible_and_scrolls_controls() {
    let mut app = app(UiAssets::default());
    click(&mut app, SettingsAction::Open);
    let menu = app
        .world_mut()
        .query_filtered::<Entity, With<MainMenu>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        app.world().get::<Node>(menu).unwrap().display,
        Display::None
    );
    let overlay = app
        .world_mut()
        .query_filtered::<Entity, With<SettingsOverlay>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        app.world().get::<BackgroundColor>(overlay).unwrap().0,
        Color::NONE
    );
    let panel = app.world().get::<Children>(overlay).unwrap()[0];
    let panel_node = app.world().get::<Node>(panel).unwrap();
    assert_eq!(panel_node.height, px(620));
    assert_eq!(panel_node.max_height, percent(90));
    let scroll = app
        .world_mut()
        .query_filtered::<Entity, With<SettingsScroll>>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        app.world().get::<Node>(scroll).unwrap().overflow.y,
        OverflowAxis::Scroll
    );
    let bar = app
        .world_mut()
        .query_filtered::<Entity, With<SettingsScrollbar>>()
        .single(app.world())
        .unwrap();
    assert_eq!(app.world().get::<Scrollbar>(bar).unwrap().target, scroll);
    assert_eq!(
        app.world().get::<Visibility>(bar),
        Some(&Visibility::Hidden)
    );
    let thumb = app.world().get::<Children>(bar).unwrap()[0];
    assert!(app.world().get::<ScrollbarThumb>(thumb).is_some());
    click(&mut app, SettingsAction::Tab(SettingsTab::Controls));
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
    click(&mut app, SettingsAction::Tab(SettingsTab::Audio));
    assert_eq!(app.world().get::<ScrollPosition>(scroll).unwrap().0.y, 0.0);
    click(&mut app, SettingsAction::Back);
    assert_eq!(
        app.world().get::<Node>(menu).unwrap().display,
        Display::Flex
    );
    click(&mut app, SettingsAction::Open);
    key(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(
        app.world().get::<Node>(menu).unwrap().display,
        Display::Flex
    );
}

#[test]
fn fps_setting_is_available_and_off_by_default() {
    let mut app = app(UiAssets::default());
    assert!(!app.world().resource::<GameSettings>().fps_overlay);
    click(&mut app, SettingsAction::Open);
    click(&mut app, SettingsAction::Tab(SettingsTab::Graphics));
    click(&mut app, SettingsAction::FpsOverlay);
    assert!(app.world().resource::<GameSettings>().fps_overlay);
}

#[test]
fn settings_buttons_and_slider_changes_play_feedback() {
    let mut app = app(UiAssets::default());
    click(&mut app, SettingsAction::Open);
    assert!(played_sounds(&mut app).contains(&Sound::UiConfirm));

    click(&mut app, SettingsAction::Tab(SettingsTab::Graphics));
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    for action in [
        SettingsAction::Graphics,
        SettingsAction::DisplayMode,
        SettingsAction::FpsOverlay,
    ] {
        click(&mut app, action);
        assert!(played_sounds(&mut app).contains(&Sound::UiPress));
    }
    click(&mut app, SettingsAction::Tab(SettingsTab::Controls));
    played_sounds(&mut app);
    click(&mut app, SettingsAction::Flashlight);
    assert!(played_sounds(&mut app).contains(&Sound::UiFocus));
    mouse(&mut app, MouseButton::Middle);
    assert!(played_sounds(&mut app).contains(&Sound::UiConfirm));

    click(&mut app, SettingsAction::Back);
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
    let mut app = app(UiAssets {
        key_glyphs,
        ..default()
    });
    click(&mut app, SettingsAction::Open);
    click(&mut app, SettingsAction::Tab(SettingsTab::Controls));
    assert_eq!(chip_glyph(&mut app, SettingsAction::Forward), handles[0]);
    assert_eq!(chip_glyph(&mut app, SettingsAction::Interact), handles[1]);
    assert_eq!(chip_text(&mut app, SettingsAction::Left), "A");
    assert_eq!(chip_glyph(&mut app, SettingsAction::Flashlight), handles[2]);
    assert_eq!(chip_glyph(&mut app, SettingsAction::Flashbang), handles[3]);

    click(&mut app, SettingsAction::Forward);
    assert_eq!(chip_text(&mut app, SettingsAction::Forward), AWAITING_TEXT);
    key(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(chip_glyph(&mut app, SettingsAction::Forward), handles[0]);
    assert!(!app.world().resource::<SettingsDirty>().0);

    click(&mut app, SettingsAction::Forward);
    key(&mut app, KeyCode::ArrowUp, Key::ArrowUp);
    assert_eq!(
        app.world().resource::<GameSettings>().keys.forward,
        "ArrowUp"
    );
    assert_eq!(chip_text(&mut app, SettingsAction::Forward), "ArrowUp");

    click(&mut app, SettingsAction::Left);
    key(&mut app, KeyCode::KeyW, Key::Character("w".into()));
    assert_eq!(app.world().resource::<GameSettings>().keys.left, "KeyW");
    assert_eq!(chip_glyph(&mut app, SettingsAction::Left), handles[0]);
    assert!(app.world().resource::<SettingsDirty>().0);

    click(&mut app, SettingsAction::Flashlight);
    mouse(&mut app, MouseButton::Middle);
    assert_eq!(chip_glyph(&mut app, SettingsAction::Flashlight), handles[4]);
}

#[test]
fn flashlight_and_flashbang_bindings_capture_mouse_and_keyboard_without_the_opening_click() {
    let mut app = app(UiAssets::default());
    click(&mut app, SettingsAction::Open);
    click(&mut app, SettingsAction::Flashlight);
    assert_eq!(
        app.world().resource::<GameSettings>().keys.flashlight,
        "MouseLeft"
    );
    mouse(&mut app, MouseButton::Middle);
    assert_eq!(
        app.world().resource::<GameSettings>().keys.flashlight,
        "MouseMiddle"
    );
    click(&mut app, SettingsAction::Flashbang);
    key(&mut app, KeyCode::KeyQ, Key::Character("q".into()));
    assert_eq!(
        app.world().resource::<GameSettings>().keys.flashbang,
        "KeyQ"
    );
    click(&mut app, SettingsAction::Flashbang);
    mouse(&mut app, MouseButton::Middle);
    assert_eq!(
        app.world().resource::<GameSettings>().keys.flashbang,
        "KeyQ"
    );
    assert_eq!(
        app.world().resource::<AwaitingKey>().0,
        Some(SettingsAction::Flashbang)
    );
    assert!(app.world().resource::<SettingsDirty>().0);
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
    for name in candidates {
        if parse_key(&name).is_some() {
            accepted += 1;
            assert!(key_glyph_stem(&name).is_some(), "no glyph for {name}");
        }
    }
    assert_eq!(accepted, 31);
    assert_eq!(key_label("KeyQ"), "Q");
    assert_eq!(key_label("Digit4"), "4");
    assert_eq!(key_label("Space"), "Space");
    assert_eq!(key_label("MouseRight"), "MouseRight");
}
