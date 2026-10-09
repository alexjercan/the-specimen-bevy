use bevy::{
    prelude::*,
    ui::FocusPolicy,
    ui_widgets::{SliderDragState, SliderValue, ValueChange},
};
use game_assets::UiAssets;
use game_settings::{
    parse_key, GameSettings, SettingsDirty, DEFAULT_SENSITIVITY, MAX_SENSITIVITY, MIN_SENSITIVITY,
};
use game_ui::{menu_button, text, theme, MenuButton};

use super::{GameState, PauseState};

const GLYPH_SIZE: f32 = 30.0;
const CHIP_WIDTH: f32 = 230.0;
const CHIP_HEIGHT: f32 = 40.0;
const AWAITING_TEXT: &str = "PRESS A KEY (Esc cancels)";

#[derive(Component)]
pub(super) struct SettingsOverlay;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SettingsTab {
    #[default]
    Audio,
    Controls,
    Graphics,
}

impl SettingsTab {
    const ALL: [Self; 3] = [Self::Audio, Self::Controls, Self::Graphics];

    fn title(self) -> &'static str {
        match self {
            Self::Audio => "Audio",
            Self::Controls => "Controls",
            Self::Graphics => "Graphics",
        }
    }

    fn groups(self) -> &'static [SettingsGroup] {
        match self {
            Self::Audio => &[SettingsGroup::Volume],
            Self::Controls => &[
                SettingsGroup::Mouse,
                SettingsGroup::Movement,
                SettingsGroup::Interaction,
            ],
            Self::Graphics => &[SettingsGroup::Quality],
        }
    }
}

#[derive(Resource, Default)]
struct ActiveTab(SettingsTab);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsAction {
    Open,
    Back,
    Tab(SettingsTab),
    Graphics,
    Forward,
    Left,
    Backward,
    Right,
    Interact,
}

impl SettingsAction {
    fn name(self) -> &'static str {
        match self {
            Self::Graphics => "Quality preset",
            Self::Forward => "Forward",
            Self::Left => "Left",
            Self::Backward => "Backward",
            Self::Right => "Right",
            Self::Interact => "Interact",
            Self::Open | Self::Back | Self::Tab(_) => "",
        }
    }

    fn key(self, settings: &GameSettings) -> Option<&str> {
        let keys = &settings.keys;
        let key = match self {
            Self::Forward => &keys.forward,
            Self::Left => &keys.left,
            Self::Backward => &keys.backward,
            Self::Right => &keys.right,
            Self::Interact => &keys.interact,
            _ => return None,
        };
        Some(key.as_str())
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsGroup {
    Volume,
    Mouse,
    Movement,
    Interaction,
    Quality,
}

impl SettingsGroup {
    fn title(self) -> &'static str {
        match self {
            Self::Volume => "VOLUME",
            Self::Mouse => "MOUSE",
            Self::Movement => "MOVEMENT",
            Self::Interaction => "INTERACTION",
            Self::Quality => "QUALITY",
        }
    }

    fn rows(self) -> &'static [Row] {
        match self {
            Self::Volume => &[
                Row::Slider(SettingSlider::Master),
                Row::Slider(SettingSlider::Sfx),
                Row::Slider(SettingSlider::Music),
            ],
            Self::Mouse => &[Row::Slider(SettingSlider::Sensitivity)],
            Self::Movement => &[
                Row::Key(SettingsAction::Forward),
                Row::Key(SettingsAction::Left),
                Row::Key(SettingsAction::Backward),
                Row::Key(SettingsAction::Right),
            ],
            Self::Interaction => &[Row::Key(SettingsAction::Interact)],
            Self::Quality => &[Row::Choice(SettingsAction::Graphics)],
        }
    }
}

enum Row {
    Slider(SettingSlider),
    Key(SettingsAction),
    Choice(SettingsAction),
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingSlider {
    Master,
    Sfx,
    Music,
    Sensitivity,
}

impl SettingSlider {
    fn name(self) -> &'static str {
        match self {
            Self::Master => "Master",
            Self::Sfx => "SFX",
            Self::Music => "Music (reserved)",
            Self::Sensitivity => "Mouse sensitivity",
        }
    }

    fn range(self) -> (f32, f32) {
        match self {
            Self::Sensitivity => (MIN_SENSITIVITY, MAX_SENSITIVITY),
            _ => (0.0, 1.0),
        }
    }

    fn step(self) -> f32 {
        match self {
            Self::Sensitivity => 0.0005,
            _ => 0.05,
        }
    }

    fn get(self, settings: &GameSettings) -> f32 {
        match self {
            Self::Master => settings.master,
            Self::Sfx => settings.sfx,
            Self::Music => settings.music,
            Self::Sensitivity => settings.mouse_sensitivity,
        }
    }

    fn set(self, settings: &mut GameSettings, value: f32) {
        match self {
            Self::Master => settings.master = value,
            Self::Sfx => settings.sfx = value,
            Self::Music => settings.music = value,
            Self::Sensitivity => settings.mouse_sensitivity = value,
        }
    }

    pub(super) fn readout(self, value: f32) -> String {
        match self {
            Self::Sensitivity => format!("{:.1}x", value / DEFAULT_SENSITIVITY),
            _ => format!("{}%", (value * 100.0).round()),
        }
    }
}

#[derive(Component)]
struct QualityLabel;

#[derive(Component)]
pub(super) struct SliderReadout(pub(super) SettingSlider);

#[derive(Component)]
struct KeyChip {
    action: SettingsAction,
    shown: String,
}

#[derive(Component)]
struct TabTitle(SettingsTab);

#[derive(Component)]
struct TabMarker(SettingsTab);

#[derive(Resource, Default)]
struct AwaitingKey(Option<SettingsAction>);

#[derive(Resource, Default)]
struct PendingSave(bool);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AwaitingKey>()
        .init_resource::<PendingSave>()
        .init_resource::<ActiveTab>()
        .add_observer(change_slider)
        .add_systems(
            Update,
            (
                activate,
                capture_key,
                sync_sliders,
                save_sliders,
                show_tab,
                refresh,
            )
                .chain(),
        );
}

fn key_label(key: &str) -> &str {
    key.strip_prefix("Key")
        .or_else(|| key.strip_prefix("Digit"))
        .unwrap_or(key)
}

fn chip_state(
    action: SettingsAction,
    settings: &GameSettings,
    awaiting: Option<SettingsAction>,
) -> String {
    if awaiting == Some(action) {
        AWAITING_TEXT.to_owned()
    } else {
        action.key(settings).unwrap_or_default().to_owned()
    }
}

fn chip_content(chip: &mut ChildSpawnerCommands, shown: &str, assets: &UiAssets) {
    if shown == AWAITING_TEXT {
        chip.spawn(text(shown, 14.0, theme::TEXT, assets.font.clone()));
        return;
    }
    match assets.key_glyph(shown) {
        Some(image) => {
            chip.spawn((
                Name::new(format!("{shown} glyph")),
                ImageNode::new(image),
                Node {
                    width: px(GLYPH_SIZE),
                    height: px(GLYPH_SIZE),
                    ..default()
                },
            ));
        }
        None => {
            chip.spawn(text(
                key_label(shown),
                16.0,
                theme::TEXT,
                assets.font.clone(),
            ));
        }
    }
}

fn control_button(action: SettingsAction) -> impl Bundle {
    (
        Button,
        MenuButton,
        action,
        Node {
            width: px(CHIP_WIDTH),
            height: px(CHIP_HEIGHT),
            padding: UiRect::axes(px(10), px(4)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        game_ui::button_paint(Interaction::None),
    )
}

fn row(name: &'static str, font: Handle<Font>) -> impl Bundle {
    (
        Name::new(format!("{name} row")),
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(10),
            padding: UiRect::vertical(px(3)),
            ..default()
        },
        children![(
            text(name, 16.0, theme::TEXT, font),
            Node {
                flex_grow: 1.0,
                ..default()
            },
        )],
    )
}

fn slider_row(field: SettingSlider, settings: &GameSettings, font: Handle<Font>) -> impl Bundle {
    let value = field.get(settings);
    let (start, end) = field.range();
    (
        Name::new(format!("{} row", field.name())),
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(10),
            padding: UiRect::vertical(px(4)),
            ..default()
        },
        children![
            (
                text(field.name(), 16.0, theme::TEXT, font.clone()),
                Node {
                    width: px(150),
                    ..default()
                },
            ),
            (
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                children![(
                    Name::new(format!("{} slider", field.name())),
                    field,
                    game_ui::slider(value, start, end, field.step()),
                )],
            ),
            (
                SliderReadout(field),
                text(field.readout(value), 16.0, theme::TEXT, font),
                Node {
                    width: px(52),
                    ..default()
                },
            ),
        ],
    )
}

fn spawn_row(
    list: &mut ChildSpawnerCommands,
    item: &Row,
    settings: &GameSettings,
    assets: &UiAssets,
) {
    let font = assets.font.clone();
    match *item {
        Row::Slider(field) => {
            list.spawn(slider_row(field, settings, font));
        }
        Row::Key(action) => {
            let shown = chip_state(action, settings, None);
            list.spawn(row(action.name(), font)).with_children(|row| {
                row.spawn((
                    control_button(action),
                    KeyChip {
                        action,
                        shown: shown.clone(),
                    },
                ))
                .with_children(|chip| chip_content(chip, &shown, assets));
            });
        }
        Row::Choice(action) => {
            list.spawn(row(action.name(), font.clone()))
                .with_children(|row| {
                    row.spawn((
                        control_button(action),
                        children![(
                            QualityLabel,
                            text(format!("{:?}", settings.graphics), 16.0, theme::TEXT, font),
                        )],
                    ));
                });
        }
    }
}

fn group(kind: SettingsGroup, font: Handle<Font>) -> impl Bundle {
    (
        kind,
        Name::new(format!("{} settings", kind.title())),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            ..default()
        },
        BorderColor::all(theme::PANEL_BORDER),
        children![(
            text(kind.title(), 14.0, theme::MUTED, font),
            Node {
                margin: UiRect::bottom(px(2)),
                ..default()
            },
        )],
    )
}

fn tab_button(tab: SettingsTab, selected: bool, font: Handle<Font>) -> impl Bundle {
    (
        Button,
        MenuButton,
        SettingsAction::Tab(tab),
        Name::new(format!("{} tab", tab.title())),
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            padding: UiRect::axes(px(12), px(9)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        game_ui::button_paint(Interaction::None),
        children![
            (
                TabTitle(tab),
                text(tab.title(), 18.0, tab_color(selected), font),
            ),
            (
                TabMarker(tab),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    bottom: px(0),
                    height: px(3),
                    display: tab_display(selected),
                    ..default()
                },
                BackgroundColor(theme::ACCENT),
            ),
        ],
    )
}

fn tab_color(selected: bool) -> Color {
    if selected {
        theme::TEXT
    } else {
        theme::MUTED
    }
}

fn tab_display(selected: bool) -> Display {
    if selected {
        Display::Flex
    } else {
        Display::None
    }
}

fn spawn_overlay(
    commands: &mut Commands,
    assets: &UiAssets,
    settings: &GameSettings,
    state: GameState,
    active: SettingsTab,
) {
    let font = assets.font.clone();
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
        FocusPolicy::Block,
        BackgroundColor(theme::BACKGROUND),
    ));
    match state {
        GameState::MainMenu => {
            root.insert(DespawnOnExit(GameState::MainMenu));
        }
        GameState::Playing => {
            root.insert(DespawnOnExit(PauseState::Paused));
        }
        GameState::Complete | GameState::GameOver => {
            return;
        }
    }
    root.with_children(|parent| {
        parent
            .spawn((
                Node {
                    width: px(560),
                    max_height: percent(95),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    padding: UiRect::all(px(16)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|list| {
                list.spawn(text("Settings", 28.0, theme::TEXT, font.clone()));
                list.spawn((
                    Name::new("Settings tabs"),
                    Node {
                        width: percent(100),
                        column_gap: px(6),
                        ..default()
                    },
                ))
                .with_children(|bar| {
                    for tab in SettingsTab::ALL {
                        bar.spawn(tab_button(tab, tab == active, font.clone()));
                    }
                });
                list.spawn((
                    Name::new("Settings tab body"),
                    Node {
                        width: percent(100),
                        min_height: px(410),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|body| {
                    for tab in SettingsTab::ALL {
                        body.spawn((
                            tab,
                            Name::new(format!("{} page", tab.title())),
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(8),
                                display: tab_display(tab == active),
                                ..default()
                            },
                        ))
                        .with_children(|page| {
                            for kind in tab.groups() {
                                page.spawn(group(*kind, font.clone()))
                                    .with_children(|group| {
                                        for item in kind.rows() {
                                            spawn_row(group, item, settings, assets);
                                        }
                                    });
                            }
                        });
                    }
                });
                list.spawn((SettingsAction::Back, menu_button("Back", font)));
            });
    });
}

fn change_slider(
    change: On<ValueChange<f32>>,
    fields: Query<&SettingSlider>,
    mut settings: ResMut<GameSettings>,
    mut pending: ResMut<PendingSave>,
) {
    let Ok(field) = fields.get(change.source) else {
        return;
    };
    let (start, end) = field.range();
    let value = change.value.clamp(start, end);
    if field.get(&settings) == value {
        return;
    }
    field.set(&mut settings, value);
    pending.0 = true;
}

fn save_sliders(
    mut pending: ResMut<PendingSave>,
    mut dirty: ResMut<SettingsDirty>,
    sliders: Query<&SliderDragState, With<SettingSlider>>,
) {
    if pending.0 && !sliders.iter().any(|slider| slider.dragging) {
        pending.0 = false;
        dirty.0 = true;
    }
}

fn sync_sliders(
    settings: Res<GameSettings>,
    sliders: Query<(Entity, &SettingSlider, &SliderValue)>,
    mut commands: Commands,
) {
    if !settings.is_changed() {
        return;
    }
    for (entity, field, current) in &sliders {
        let value = field.get(&settings);
        if current.0 != value {
            commands.entity(entity).insert(SliderValue(value));
        }
    }
}

fn activate(
    buttons: Query<(&Interaction, &SettingsAction), Changed<Interaction>>,
    overlay: Query<Entity, With<SettingsOverlay>>,
    state: Option<Res<State<GameState>>>,
    mut settings: ResMut<GameSettings>,
    mut dirty: ResMut<SettingsDirty>,
    mut awaiting: ResMut<AwaitingKey>,
    mut active: ResMut<ActiveTab>,
    assets: Option<Res<UiAssets>>,
    mut commands: Commands,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            SettingsAction::Open if overlay.is_empty() => {
                if let (Some(state), Some(assets)) = (state.as_ref(), assets.as_ref()) {
                    spawn_overlay(&mut commands, assets, &settings, *state.get(), active.0);
                }
            }
            SettingsAction::Back => {
                for entity in &overlay {
                    commands.entity(entity).despawn();
                }
                awaiting.0 = None;
            }
            _ if overlay.is_empty() => {}
            SettingsAction::Tab(tab) => {
                active.0 = *tab;
                awaiting.0 = None;
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
            SettingsAction::Open => {}
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

fn show_tab(
    active: Res<ActiveTab>,
    mut pages: Query<(&SettingsTab, &mut Node), Without<TabMarker>>,
    mut markers: Query<(&TabMarker, &mut Node), Without<SettingsTab>>,
    mut titles: Query<(&TabTitle, &mut TextColor)>,
) {
    if !active.is_changed() {
        return;
    }
    for (tab, mut node) in &mut pages {
        let display = tab_display(*tab == active.0);
        if node.display != display {
            node.display = display;
        }
    }
    for (marker, mut node) in &mut markers {
        let display = tab_display(marker.0 == active.0);
        if node.display != display {
            node.display = display;
        }
    }
    for (title, mut color) in &mut titles {
        color.set_if_neq(TextColor(tab_color(title.0 == active.0)));
    }
}

fn refresh(
    settings: Res<GameSettings>,
    awaiting: Res<AwaitingKey>,
    assets: Option<Res<UiAssets>>,
    mut labels: Query<&mut Text, (With<QualityLabel>, Without<SliderReadout>)>,
    mut readouts: Query<(&SliderReadout, &mut Text), Without<QualityLabel>>,
    mut chips: Query<(Entity, &mut KeyChip)>,
    mut commands: Commands,
) {
    if !settings.is_changed() && !awaiting.is_changed() {
        return;
    }
    for mut text in &mut labels {
        **text = format!("{:?}", settings.graphics);
    }
    for (readout, mut text) in &mut readouts {
        **text = readout.0.readout(readout.0.get(&settings));
    }
    let Some(assets) = assets else {
        return;
    };
    for (entity, mut chip) in &mut chips {
        let shown = chip_state(chip.action, &settings, awaiting.0);
        if chip.shown == shown {
            continue;
        }
        commands
            .entity(entity)
            .despawn_children()
            .with_children(|content| chip_content(content, &shown, &assets));
        chip.shown = shown;
    }
}

#[cfg(test)]
#[path = "../../tests/unit/settings_menu.rs"]
mod tests;
