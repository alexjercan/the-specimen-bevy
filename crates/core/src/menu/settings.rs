use bevy::prelude::*;
use game_assets::UiAssets;
use game_settings::{parse_key, GameSettings, SettingsDirty, MAX_SENSITIVITY, MIN_SENSITIVITY};
use game_ui::{menu_button, text, theme, MenuButton};

use super::{GameState, PauseState};

#[derive(Component)]
pub(super) struct SettingsOverlay;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsAction {
    Open,
    Back,
    Master,
    Sfx,
    Music,
    Sensitivity,
    Graphics,
    Forward,
    Left,
    Backward,
    Right,
    Interact,
}

const FIELDS: [SettingsAction; 10] = [
    SettingsAction::Master,
    SettingsAction::Sfx,
    SettingsAction::Music,
    SettingsAction::Sensitivity,
    SettingsAction::Graphics,
    SettingsAction::Forward,
    SettingsAction::Left,
    SettingsAction::Backward,
    SettingsAction::Right,
    SettingsAction::Interact,
];

#[derive(Component)]
struct SettingLabel(SettingsAction);

#[derive(Component)]
pub(super) struct Step(pub(super) i8);

#[derive(Resource, Default)]
struct AwaitingKey(Option<SettingsAction>);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AwaitingKey>()
        .add_systems(Update, (activate, capture_key, refresh).chain());
}

fn value(
    action: SettingsAction,
    settings: &GameSettings,
    awaiting: Option<SettingsAction>,
) -> String {
    let (name, value) = match action {
        SettingsAction::Master => ("Master", format!("{}%", (settings.master * 100.0).round())),
        SettingsAction::Sfx => ("SFX", format!("{}%", (settings.sfx * 100.0).round())),
        SettingsAction::Music => (
            "Music (reserved)",
            format!("{}%", (settings.music * 100.0).round()),
        ),
        SettingsAction::Sensitivity => (
            "Mouse sensitivity",
            format!(
                "{:.1}x",
                settings.mouse_sensitivity / game_settings::DEFAULT_SENSITIVITY
            ),
        ),
        SettingsAction::Graphics => ("Graphics", format!("{:?}", settings.graphics)),
        SettingsAction::Forward => ("Forward", settings.keys.forward.clone()),
        SettingsAction::Left => ("Left", settings.keys.left.clone()),
        SettingsAction::Backward => ("Backward", settings.keys.backward.clone()),
        SettingsAction::Right => ("Right", settings.keys.right.clone()),
        SettingsAction::Interact => ("Interact", settings.keys.interact.clone()),
        _ => return String::new(),
    };
    if awaiting == Some(action) {
        format!("{name}: PRESS A KEY (Esc cancels)")
    } else {
        format!("{name}: {value}")
    }
}

fn setting_button(
    action: SettingsAction,
    settings: &GameSettings,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Button,
        MenuButton,
        action,
        Node {
            width: percent(100),
            padding: UiRect::axes(px(14), px(7)),
            border: UiRect::all(px(theme::BORDER)),
            align_items: AlignItems::Center,
            ..default()
        },
        game_ui::button_paint(Interaction::None),
        children![(
            SettingLabel(action),
            text(value(action, settings, None), 16.0, theme::TEXT, font)
        )],
    )
}

fn step_button(action: SettingsAction, step: i8, font: Handle<Font>) -> impl Bundle {
    (
        Button,
        MenuButton,
        action,
        Step(step),
        Node {
            width: px(44),
            padding: UiRect::axes(px(12), px(7)),
            border: UiRect::all(px(theme::BORDER)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        game_ui::button_paint(Interaction::None),
        children![text(
            if step < 0 { "-" } else { "+" },
            18.0,
            theme::TEXT,
            font
        )],
    )
}

fn spawn_overlay(
    commands: &mut Commands,
    font: Handle<Font>,
    settings: &GameSettings,
    state: GameState,
) {
    let mut root = commands.spawn((
        SettingsOverlay,
        Name::new("Settings overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(200),
        BackgroundColor(theme::BACKGROUND),
    ));
    match state {
        GameState::MainMenu => {
            root.insert(DespawnOnExit(GameState::MainMenu));
        }
        GameState::Playing => {
            root.insert(DespawnOnExit(PauseState::Paused));
        }
        GameState::Complete => {
            return;
        }
    }
    root.with_children(|parent| {
        parent
            .spawn((
                Node {
                    width: px(460),
                    max_height: percent(95),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    padding: UiRect::all(px(16)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|list| {
                list.spawn(text("Settings", 28.0, theme::TEXT, font.clone()));
                for action in FIELDS {
                    if matches!(
                        action,
                        SettingsAction::Master
                            | SettingsAction::Sfx
                            | SettingsAction::Music
                            | SettingsAction::Sensitivity
                    ) {
                        list.spawn(Node {
                            width: percent(100),
                            align_items: AlignItems::Center,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                SettingLabel(action),
                                text(
                                    value(action, settings, None),
                                    16.0,
                                    theme::TEXT,
                                    font.clone(),
                                ),
                            ));
                            row.spawn(step_button(action, -1, font.clone()));
                            row.spawn(step_button(action, 1, font.clone()));
                        });
                    } else {
                        list.spawn(setting_button(action, settings, font.clone()));
                    }
                }
                list.spawn((SettingsAction::Back, menu_button("Back", font)));
            });
    });
}

fn activate(
    buttons: Query<(&Interaction, &SettingsAction, Option<&Step>), Changed<Interaction>>,
    overlay: Query<Entity, With<SettingsOverlay>>,
    state: Option<Res<State<GameState>>>,
    settings: ResMut<GameSettings>,
    mut dirty: ResMut<SettingsDirty>,
    mut awaiting: ResMut<AwaitingKey>,
    assets: Option<Res<UiAssets>>,
    mut commands: Commands,
) {
    let mut settings = settings;
    for (interaction, action, step) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            SettingsAction::Open if overlay.is_empty() => {
                if let (Some(state), Some(assets)) = (state.as_ref(), assets.as_ref()) {
                    spawn_overlay(&mut commands, assets.font.clone(), &settings, *state.get());
                }
            }
            SettingsAction::Back => {
                for entity in &overlay {
                    commands.entity(entity).despawn();
                }
                awaiting.0 = None;
            }
            _ if overlay.is_empty() => {}
            SettingsAction::Master | SettingsAction::Sfx | SettingsAction::Music => {
                let current = match action {
                    SettingsAction::Master => &mut settings.master,
                    SettingsAction::Sfx => &mut settings.sfx,
                    _ => &mut settings.music,
                };
                *current =
                    (*current + 0.1 * step.map_or(1.0, |step| step.0 as f32)).clamp(0.0, 1.0);
                dirty.0 = true;
            }
            SettingsAction::Sensitivity => {
                settings.mouse_sensitivity = (settings.mouse_sensitivity
                    + 0.0005 * step.map_or(1.0, |step| step.0 as f32))
                .clamp(MIN_SENSITIVITY, MAX_SENSITIVITY);
                dirty.0 = true;
            }
            SettingsAction::Graphics => {
                settings.graphics = settings.graphics.next();
                dirty.0 = true;
            }
            key @ (SettingsAction::Forward
            | SettingsAction::Left
            | SettingsAction::Backward
            | SettingsAction::Right
            | SettingsAction::Interact) => awaiting.0 = Some(*key),
            _ => {}
        }
    }
}

fn capture_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut awaiting: ResMut<AwaitingKey>,
    mut settings: ResMut<GameSettings>,
    mut dirty: ResMut<SettingsDirty>,
    overlays: Query<Entity, With<SettingsOverlay>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if awaiting.0.is_some() {
            awaiting.0 = None;
        } else {
            for entity in &overlays {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    let Some(action) = awaiting.0 else {
        return;
    };
    let Some(key) = keys.get_just_pressed().copied().next() else {
        return;
    };
    let name = format!("{key:?}");
    if parse_key(&name).is_none() {
        return;
    }
    let mut updated = settings.keys.clone();
    match action {
        SettingsAction::Forward => updated.forward = name,
        SettingsAction::Left => updated.left = name,
        SettingsAction::Backward => updated.backward = name,
        SettingsAction::Right => updated.right = name,
        SettingsAction::Interact => updated.interact = name,
        _ => return,
    }
    if !updated.unique() {
        return;
    }
    settings.keys = updated;
    awaiting.0 = None;
    dirty.0 = true;
}

fn refresh(
    settings: Res<GameSettings>,
    awaiting: Res<AwaitingKey>,
    mut labels: Query<(&SettingLabel, &mut Text)>,
) {
    if !settings.is_changed() && !awaiting.is_changed() {
        return;
    }
    for (label, mut text) in &mut labels {
        **text = value(label.0, &settings, awaiting.0);
    }
}
