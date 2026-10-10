use bevy::prelude::*;
use game_assets::{FacilityAssets, UiAssets};
use game_settings::{CustomDifficulty, GameSettings, PreferredMode};
use game_ui::{menu_button, text, theme};
use gameplay::achievements::AchievementProgress;
use gameplay::controller::PlayerController;
use gameplay::levels::{
    build_first_floor, spawn_first_floor_actors, Door, DoorRef, FusePanel, LevelRoot, Monster,
    Passage, PickupKind, Prop, Room,
};
use gameplay::presets::{
    GameMode, GamePreset, MonsterChaseSpeed, MonsterRunHearing, MonsterWalkHearing,
    MonsterWalkSpeed,
};

use super::{background, release_cursor, screen_camera, GameState, MenuAction, TITLE};

const MODES: [GameMode; 3] = [GameMode::Normal, GameMode::Custom, GameMode::Insane];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CustomField {
    WalkSpeed,
    ChaseSpeed,
    WalkHearing,
    RunHearing,
    Sprint,
    Flashbangs,
    Detector,
    Blackout,
}

impl CustomField {
    const ALL: [Self; 8] = [
        Self::WalkSpeed,
        Self::ChaseSpeed,
        Self::WalkHearing,
        Self::RunHearing,
        Self::Sprint,
        Self::Flashbangs,
        Self::Detector,
        Self::Blackout,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::WalkSpeed => "Walk speed",
            Self::ChaseSpeed => "Chase speed",
            Self::WalkHearing => "Walk hearing",
            Self::RunHearing => "Run hearing",
            Self::Sprint => "Sprint",
            Self::Flashbangs => "Flashbangs",
            Self::Detector => "Detector",
            Self::Blackout => "Blackout",
        }
    }

    fn value(self, preset: GamePreset) -> String {
        match self {
            Self::WalkSpeed => format!(
                "{:?} ({:.2} m/s)",
                preset.monster.walk_speed,
                preset.monster.walk_speed.value()
            ),
            Self::ChaseSpeed => format!(
                "{:?} ({:.1} m/s)",
                preset.monster.chase_speed,
                preset.monster.chase_speed.value()
            ),
            Self::WalkHearing => format!(
                "{:?} ({:.1} m)",
                preset.monster.walk_hearing,
                preset.monster.walk_hearing.value()
            ),
            Self::RunHearing => format!(
                "{:?} ({:.1} m)",
                preset.monster.run_hearing,
                preset.monster.run_hearing.value()
            ),
            Self::Sprint => on_off(preset.sprint).to_string(),
            Self::Flashbangs => on_off(preset.flashbangs).to_string(),
            Self::Detector => on_off(preset.detector).to_string(),
            Self::Blackout => on_off(preset.blackout).to_string(),
        }
    }
}

fn on_off(enabled: bool) -> &'static str {
    if enabled {
        "On"
    } else {
        "Off"
    }
}

fn cycle_value<T: Copy + PartialEq>(value: &mut T, choices: &[T], step: isize) {
    let index = choices.iter().position(|item| item == value).unwrap_or(0) as isize;
    *value = choices[(index + step).rem_euclid(choices.len() as isize) as usize];
}

#[derive(Component)]
pub(super) struct MainMenu;

#[derive(Component)]
struct ModeLabel;

#[derive(Component)]
struct CustomPanel;

#[derive(Component)]
struct CustomValue(CustomField);

#[derive(Resource)]
pub(super) struct ModeSelection(pub(super) GameMode, pub(super) GamePreset);

impl Default for ModeSelection {
    fn default() -> Self {
        Self(GameMode::Normal, GamePreset::of(GameMode::Custom))
    }
}

impl ModeSelection {
    pub(super) fn from_settings(settings: &GameSettings, progress: &AchievementProgress) -> Self {
        let custom = settings.custom_difficulty;
        let mut preset = GamePreset::of(GameMode::Custom);
        preset.monster.walk_speed = [
            MonsterWalkSpeed::Slow,
            MonsterWalkSpeed::Medium,
            MonsterWalkSpeed::Fast,
        ]
        .get(custom.walk_speed as usize)
        .copied()
        .unwrap_or(MonsterWalkSpeed::Slow);
        preset.monster.chase_speed = [
            MonsterChaseSpeed::Normal,
            MonsterChaseSpeed::Medium,
            MonsterChaseSpeed::Fast,
        ]
        .get(custom.chase_speed as usize)
        .copied()
        .unwrap_or(MonsterChaseSpeed::Normal);
        preset.monster.walk_hearing = [
            MonsterWalkHearing::Short,
            MonsterWalkHearing::Medium,
            MonsterWalkHearing::Long,
        ]
        .get(custom.walk_hearing as usize)
        .copied()
        .unwrap_or(MonsterWalkHearing::Short);
        preset.monster.run_hearing = [
            MonsterRunHearing::Short,
            MonsterRunHearing::Medium,
            MonsterRunHearing::Long,
        ]
        .get(custom.run_hearing as usize)
        .copied()
        .unwrap_or(MonsterRunHearing::Short);
        preset.sprint = custom.sprint;
        preset.flashbangs = custom.flashbangs;
        preset.detector = custom.detector;
        preset.blackout = custom.blackout;
        let mode = match settings.preferred_mode {
            PreferredMode::Normal => GameMode::Normal,
            PreferredMode::Custom
                if progress
                    .unlocked
                    .contains(&gameplay::achievements::Achievement::Escaped)
                    || progress.insane_unlocked() =>
            {
                GameMode::Custom
            }
            PreferredMode::Insane if progress.insane_unlocked() => GameMode::Insane,
            _ => GameMode::Normal,
        };
        Self(mode, preset)
    }

    pub(super) fn save_into(&self, settings: &mut GameSettings) {
        settings.preferred_mode = match self.0 {
            GameMode::Normal => PreferredMode::Normal,
            GameMode::Custom => PreferredMode::Custom,
            GameMode::Insane => PreferredMode::Insane,
        };
        settings.custom_difficulty = CustomDifficulty {
            walk_speed: self.1.monster.walk_speed as u8,
            chase_speed: self.1.monster.chase_speed as u8,
            walk_hearing: self.1.monster.walk_hearing as u8,
            run_hearing: self.1.monster.run_hearing as u8,
            sprint: self.1.sprint,
            flashbangs: self.1.flashbangs,
            detector: self.1.detector,
            blackout: self.1.blackout,
        };
    }

    pub(super) fn cycle(&mut self, step: isize, progress: &AchievementProgress) {
        let modes: Vec<_> = MODES
            .iter()
            .copied()
            .filter(|mode| match mode {
                GameMode::Normal => true,
                GameMode::Custom => {
                    progress
                        .unlocked
                        .contains(&gameplay::achievements::Achievement::Escaped)
                        || progress.insane_unlocked()
                }
                GameMode::Insane => progress.insane_unlocked(),
            })
            .collect();
        let index = modes.iter().position(|mode| *mode == self.0).unwrap_or(0) as isize;
        self.0 = modes[(index + step).rem_euclid(modes.len() as isize) as usize];
    }

    pub(super) fn adjust(&mut self, field: CustomField, step: isize) {
        let preset = &mut self.1;
        match field {
            CustomField::WalkSpeed => cycle_value(
                &mut preset.monster.walk_speed,
                &[
                    MonsterWalkSpeed::Slow,
                    MonsterWalkSpeed::Medium,
                    MonsterWalkSpeed::Fast,
                ],
                step,
            ),
            CustomField::ChaseSpeed => cycle_value(
                &mut preset.monster.chase_speed,
                &[
                    MonsterChaseSpeed::Normal,
                    MonsterChaseSpeed::Medium,
                    MonsterChaseSpeed::Fast,
                ],
                step,
            ),
            CustomField::WalkHearing => cycle_value(
                &mut preset.monster.walk_hearing,
                &[
                    MonsterWalkHearing::Short,
                    MonsterWalkHearing::Medium,
                    MonsterWalkHearing::Long,
                ],
                step,
            ),
            CustomField::RunHearing => cycle_value(
                &mut preset.monster.run_hearing,
                &[
                    MonsterRunHearing::Short,
                    MonsterRunHearing::Medium,
                    MonsterRunHearing::Long,
                ],
                step,
            ),
            CustomField::Sprint => preset.sprint = !preset.sprint,
            CustomField::Flashbangs => preset.flashbangs = !preset.flashbangs,
            CustomField::Detector => preset.detector = !preset.detector,
            CustomField::Blackout => preset.blackout = !preset.blackout,
        }
    }
}

fn mode_name(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Normal => "Normal",
        GameMode::Custom => "Custom",
        GameMode::Insane => "Insane",
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ModeSelection>()
        .add_systems(
            OnEnter(GameState::MainMenu),
            (reset_mode, spawn_main_menu, release_cursor).chain(),
        )
        .add_systems(
            Update,
            sync_mode_label.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnEnter(GameState::Playing), enter_world);
}

fn reset_mode(
    mut selection: ResMut<ModeSelection>,
    settings: Res<GameSettings>,
    progress: Res<AchievementProgress>,
) {
    *selection = ModeSelection::from_settings(&settings, &progress);
}

fn sync_mode_label(
    selection: Res<ModeSelection>,
    mut labels: Query<&mut Text, With<ModeLabel>>,
    mut panels: Query<&mut Visibility, With<CustomPanel>>,
    mut values: Query<(&CustomValue, &mut Text), Without<ModeLabel>>,
) {
    if !selection.is_changed() {
        return;
    }
    for mut label in &mut labels {
        label.0 = mode_name(selection.0).to_string();
    }
    for mut panel in &mut panels {
        *panel = if selection.0 == GameMode::Custom {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (field, mut text) in &mut values {
        text.0 = field.0.value(selection.1);
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    assets: Res<UiAssets>,
    facility: Option<Res<FacilityAssets>>,
    progress: Res<AchievementProgress>,
) {
    let font = assets.font.clone();
    if facility.is_some() {
        background::spawn(&mut commands, GameState::MainMenu);
    } else {
        commands.spawn(screen_camera(GameState::MainMenu));
    }
    let menu = commands
        .spawn((
            MainMenu,
            Name::new("Main menu"),
            DespawnOnExit(GameState::MainMenu),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::left(px(96)),
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(if facility.is_some() {
                Color::srgba(0.002, 0.004, 0.008, 0.10)
            } else {
                theme::BACKGROUND
            }),
            children![(
                text(TITLE, 56.0, theme::TEXT, font.clone()),
                Node {
                    margin: UiRect::bottom(px(28)),
                    ..default()
                },
            )],
        ))
        .id();
    let menu_content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: px(24),
            ..default()
        })
        .id();
    let buttons = commands
        .spawn(Node {
            width: px(240),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        })
        .id();
    commands.entity(buttons).with_children(|parent| {
        if progress
            .unlocked
            .contains(&gameplay::achievements::Achievement::Escaped)
            || progress.insane_unlocked()
        {
            parent
                .spawn((
                    Name::new("Play row"),
                    Node {
                        width: percent(100),
                        column_gap: px(8),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        Name::new("Play button"),
                        MenuAction::Play,
                        menu_button("Play", font.clone()),
                    ))
                    .insert(Node {
                        flex_grow: 1.0,
                        padding: UiRect::axes(px(18), px(10)),
                        border: UiRect::all(px(theme::BORDER)),
                        border_radius: BorderRadius::all(px(theme::RADIUS)),
                        align_items: AlignItems::Center,
                        ..default()
                    });
                    spawn_mode_selector(row, font.clone());
                });
        } else {
            parent.spawn((
                Name::new("Play button"),
                MenuAction::Play,
                menu_button("Play", font.clone()),
            ));
        }
        parent.spawn((
            Name::new("Settings button"),
            super::settings::SettingsAction::Open,
            menu_button("Settings", font.clone()),
        ));
        parent.spawn((
            Name::new("Credits button"),
            MenuAction::Credits,
            menu_button("Credits", font.clone()),
        ));
        parent.spawn((
            Name::new("Achievements button"),
            MenuAction::Achievements,
            menu_button("Achievements", font.clone()),
        ));
    });
    commands.entity(menu_content).add_child(buttons);
    commands.entity(menu).add_child(menu_content);
    if progress
        .unlocked
        .contains(&gameplay::achievements::Achievement::Escaped)
        || progress.insane_unlocked()
    {
        spawn_custom_panel(&mut commands, menu_content, font.clone());
    }
    #[cfg(not(target_arch = "wasm32"))]
    commands.entity(buttons).with_children(|parent| {
        parent.spawn((
            Name::new("Quit button"),
            MenuAction::Quit,
            menu_button("Quit", font.clone()),
        ));
    });
}

fn spawn_custom_panel(commands: &mut Commands, container: Entity, font: Handle<Font>) {
    let panel = commands
        .spawn((
            Name::new("Custom options"),
            CustomPanel,
            Visibility::Hidden,
            Node {
                width: px(390),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(theme::BORDER)),
                border_radius: BorderRadius::all(px(theme::RADIUS)),
                ..default()
            },
            BackgroundColor(theme::PANEL),
        ))
        .id();
    commands.entity(container).add_child(panel);
    commands.entity(panel).with_children(|parent| {
        parent.spawn(text("CUSTOM DIFFICULTY", 18.0, theme::ACCENT, font.clone()));
        for field in CustomField::ALL {
            parent
                .spawn((
                    Name::new(format!("{} option", field.name())),
                    Node {
                        width: percent(100),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        text(field.name(), 14.0, theme::TEXT, font.clone()),
                        Node {
                            width: px(116),
                            ..default()
                        },
                    ));
                    row.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: px(4),
                        ..default()
                    })
                    .with_children(|controls| {
                        spawn_arrow(
                            controls,
                            "Previous custom value",
                            MenuAction::AdjustCustom(field, -1),
                            "<",
                            font.clone(),
                        );
                        controls.spawn((
                            Name::new(format!("{} value", field.name())),
                            Node {
                                width: px(176),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![(
                                CustomValue(field),
                                text(
                                    field.value(GamePreset::of(GameMode::Custom)),
                                    13.0,
                                    theme::TEXT,
                                    font.clone()
                                )
                            )],
                        ));
                        spawn_arrow(
                            controls,
                            "Next custom value",
                            MenuAction::AdjustCustom(field, 1),
                            ">",
                            font.clone(),
                        );
                    });
                });
        }
    });
}

fn spawn_mode_selector(row: &mut ChildSpawnerCommands, font: Handle<Font>) {
    row.spawn((
        Name::new("Mode selector"),
        Node {
            align_items: AlignItems::Center,
            column_gap: px(4),
            ..default()
        },
    ))
    .with_children(|selector| {
        spawn_arrow(
            selector,
            "Mode previous button",
            MenuAction::PreviousMode,
            "<",
            font.clone(),
        );
        selector.spawn((
            Name::new("Mode label"),
            Node {
                width: px(64),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                ModeLabel,
                text(mode_name(GameMode::Normal), 18.0, theme::TEXT, font.clone()),
            )],
        ));
        spawn_arrow(
            selector,
            "Mode next button",
            MenuAction::NextMode,
            ">",
            font,
        );
    });
}

fn spawn_arrow(
    selector: &mut ChildSpawnerCommands,
    name: &'static str,
    action: MenuAction,
    glyph: &'static str,
    font: Handle<Font>,
) {
    selector
        .spawn((Name::new(name), action, menu_button(glyph, font)))
        .insert(Node {
            width: px(32),
            padding: UiRect::vertical(px(10)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        });
}

fn enter_world(world: &mut World) {
    let build = world.register_system_cached(build_first_floor);
    if let Err(error) = world.run_system(build) {
        error!("first floor build failed: {error}");
        return;
    }
    let spawn_actors = world.register_system_cached(spawn_first_floor_actors);
    if let Err(error) = world.run_system(spawn_actors) {
        error!("first floor actor spawn failed: {error}");
        return;
    }
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, (
            Without<ChildOf>,
            Or<(
                With<Room>,
                With<LevelRoot>,
                With<Door>,
                With<DoorRef>,
                With<Passage>,
                With<Prop>,
                With<FusePanel>,
                With<PickupKind>,
                With<Monster>,
                With<PlayerController>,
            )>,
        )>()
        .iter(world)
        .collect();
    for entity in roots {
        world
            .entity_mut(entity)
            .insert(DespawnOnExit(GameState::Playing));
    }
}
