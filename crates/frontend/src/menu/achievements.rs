use bevy::{prelude::*, text::LineBreak, ui_widgets::Activate};
use game_audio::PlaySound;

use super::{ui_sound, MenuSounds};
use crate::MenuAssets;
use game_ui::{menu_button, text, theme};
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

use game_flow::{CoreState, GameState, PauseState};

use super::{background, pause::PauseMenu, release_cursor, MenuAction};

const TOAST_SECS: f32 = 4.0;

#[derive(Component)]
pub(super) struct AchievementsScreen;

#[derive(Component)]
pub(super) struct PauseAchievements;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum AchievementAction {
    Open,
    Back,
}

#[derive(Message)]
struct AchievementActivated(AchievementAction);

#[derive(Component)]
struct AchievementList;

#[derive(Component)]
struct ToastContainer;

#[derive(Component)]
struct Toast {
    elapsed: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_message::<AchievementActivated>()
        .add_observer(spawn_toast)
        .add_observer(queue_achievement_activation)
        .add_systems(
            OnEnter(GameState::Achievements),
            (spawn_achievements, release_cursor),
        )
        .add_systems(
            Update,
            back_on_escape.run_if(in_state(GameState::Achievements)),
        )
        .add_systems(Update, back_from_pause.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            (
                activate_pause_achievements,
                advance_toasts,
                super::scroll_list::<AchievementList>,
            )
                .run_if(in_state(CoreState::Ready)),
        );
}

fn status_text(unlocked: bool) -> (&'static str, Color) {
    if unlocked {
        ("UNLOCKED", theme::ACCENT)
    } else {
        ("LOCKED", theme::MUTED)
    }
}

fn icon_stem(achievement: Achievement) -> &'static str {
    match achievement {
        Achievement::Escaped => "first-escape",
        Achievement::EscapeWithoutFlashbang => "empty-handed",
        Achievement::EscapeWithoutDetector => "trust-your-ears",
        Achievement::FlashbangHitMonster => "buy-some-time",
        Achievement::EscapeWithoutBoiler => "in-the-dark",
        Achievement::RestoreBoiler => "let-there-be-light",
        Achievement::CaughtAfterExitOpen => "so-close",
        Achievement::EscapeUndetected => "unseen",
        Achievement::EscapeInsane => "out-of-your-mind",
    }
}

fn achievement_row(
    achievement: Achievement,
    unlocked: bool,
    font: Handle<Font>,
    icon: Handle<Image>,
) -> impl Bundle {
    let (status, color) = status_text(unlocked);
    let hidden = achievement.hidden() && !unlocked;
    let name = if hidden { "?" } else { achievement.name() };
    let criteria = if hidden { "" } else { achievement.criteria() };
    (
        Name::new(format!("{} row", achievement.name())),
        Node {
            width: percent(100),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(12),
            padding: UiRect::all(px(6)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            ..default()
        },
        BackgroundColor(theme::BACKGROUND),
        children![
            (
                ImageNode::new(icon),
                Node {
                    width: px(48),
                    height: px(48),
                    flex_shrink: 0.0,
                    ..default()
                },
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    min_width: px(0),
                    row_gap: px(2),
                    ..default()
                },
                children![
                    text(name, 17.0, theme::TEXT, font.clone()),
                    (
                        text(criteria, 13.0, theme::MUTED, font.clone()),
                        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                        Node {
                            max_width: percent(100),
                            ..default()
                        },
                    ),
                ],
            ),
            (
                text(status, 13.0, color, font),
                Node {
                    width: px(86),
                    ..default()
                },
            ),
        ],
    )
}

fn spawn_achievements(
    mut commands: Commands,
    assets: Res<MenuAssets>,
    progress: Res<AchievementProgress>,
) {
    background::spawn(&mut commands, GameState::Achievements);
    spawn_panel(&mut commands, &assets, &progress, false);
}

fn spawn_panel(
    commands: &mut Commands,
    assets: &MenuAssets,
    progress: &AchievementProgress,
    paused: bool,
) {
    let font = assets.font.clone();
    let unlocked = progress.unlocked.len();
    let total = Achievement::ALL.len();
    let mut screen = commands.spawn((
        AchievementsScreen,
        Name::new("Achievements screen"),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    if paused {
        screen.insert((
            PauseAchievements,
            DespawnOnExit(PauseState::Paused),
            GlobalZIndex(101),
        ));
    } else {
        screen.insert(DespawnOnExit(GameState::Achievements));
    }
    screen.with_children(|parent| {
        parent
            .spawn((
                Name::new("Achievements panel"),
                Node {
                    width: px(560),
                    max_width: percent(95),
                    max_height: percent(95),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    padding: UiRect::all(px(16)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|panel| {
                panel.spawn(text("Achievements", 28.0, theme::TEXT, font.clone()));
                panel.spawn(text(
                    format!("{unlocked} / {total} unlocked"),
                    16.0,
                    theme::MUTED,
                    font.clone(),
                ));
                panel
                    .spawn((
                        AchievementList,
                        ScrollPosition::default(),
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 1.0,
                            row_gap: px(6),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|list| {
                        for achievement in Achievement::ALL {
                            let unlocked = progress.unlocked.contains(&achievement);
                            let icon = assets
                                .achievement_icon(icon_stem(achievement), unlocked)
                                .unwrap_or_default();
                            list.spawn(achievement_row(achievement, unlocked, font.clone(), icon));
                        }
                    });
                if paused {
                    panel.spawn((
                        Name::new("Back button"),
                        AchievementAction::Back,
                        menu_button("Back", font),
                    ));
                } else {
                    panel.spawn((
                        Name::new("Back button"),
                        MenuAction::MainMenu,
                        menu_button("Back", font),
                    ));
                }
            });
    });
}

fn queue_achievement_activation(
    activated: On<Activate>,
    actions: Query<&AchievementAction>,
    mut events: MessageWriter<AchievementActivated>,
) {
    if let Ok(action) = actions.get(activated.entity) {
        events.write(AchievementActivated(*action));
    }
}

fn activate_pause_achievements(
    mut buttons: MessageReader<AchievementActivated>,
    screens: Query<Entity, With<PauseAchievements>>,
    mut pause_menus: Query<&mut Node, With<PauseMenu>>,
    assets: Res<MenuAssets>,
    progress: Res<AchievementProgress>,
    menu_sounds: Res<MenuSounds>,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    for AchievementActivated(action) in buttons.read() {
        match action {
            AchievementAction::Open if screens.is_empty() => {
                spawn_panel(&mut commands, &assets, &progress, true);
                for mut node in &mut pause_menus {
                    node.display = Display::None;
                }
                sounds.write(ui_sound(&menu_sounds.confirm));
            }
            AchievementAction::Back if !screens.is_empty() => {
                close_pause_achievements(&screens, &mut pause_menus, &mut commands);
                sounds.write(ui_sound(&menu_sounds.back));
            }
            _ => {}
        }
    }
}

fn close_pause_achievements(
    screens: &Query<Entity, With<PauseAchievements>>,
    pause_menus: &mut Query<&mut Node, With<PauseMenu>>,
    commands: &mut Commands,
) {
    for entity in screens {
        commands.entity(entity).despawn();
    }
    for mut node in pause_menus {
        node.display = Display::Flex;
    }
}

fn back_from_pause(
    keys: Res<ButtonInput<KeyCode>>,
    screens: Query<Entity, With<PauseAchievements>>,
    mut pause_menus: Query<&mut Node, With<PauseMenu>>,
    menu_sounds: Res<MenuSounds>,
    mut sounds: MessageWriter<PlaySound>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape) && !screens.is_empty() {
        close_pause_achievements(&screens, &mut pause_menus, &mut commands);
        sounds.write(ui_sound(&menu_sounds.back));
    }
}

fn back_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
    menu_sounds: Res<MenuSounds>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::MainMenu);
        sounds.write(ui_sound(&menu_sounds.back));
    }
}

fn spawn_toast(
    unlock: On<AchievementUnlocked>,
    assets: Res<MenuAssets>,
    containers: Query<Entity, With<ToastContainer>>,
    mut commands: Commands,
    menu_sounds: Res<MenuSounds>,
    mut sounds: MessageWriter<PlaySound>,
) {
    let font = assets.font.clone();
    let achievement = unlock.0;
    let icon = assets
        .achievement_icon(icon_stem(achievement), true)
        .unwrap_or_default();
    sounds.write(ui_sound(&menu_sounds.confirm));
    let container = containers.iter().next().unwrap_or_else(|| {
        commands
            .spawn((
                ToastContainer,
                Name::new("Achievement toast container"),
                Pickable::IGNORE,
                GlobalZIndex(1000),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(16),
                    right: px(16),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    row_gap: px(8),
                    ..default()
                },
            ))
            .id()
    });
    commands.entity(container).with_children(|parent| {
        parent.spawn((
            Toast { elapsed: 0.0 },
            Name::new("Achievement toast"),
            Pickable::IGNORE,
            Node {
                width: px(340),
                max_width: percent(90),
                align_items: AlignItems::Center,
                column_gap: px(12),
                padding: UiRect::all(px(12)),
                border: UiRect::all(px(theme::BORDER)),
                border_radius: BorderRadius::all(px(theme::RADIUS)),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::ACCENT),
            children![
                (
                    ImageNode::new(icon),
                    Node {
                        width: px(64),
                        height: px(64),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_width: px(0),
                        row_gap: px(4),
                        ..default()
                    },
                    children![
                        text("ACHIEVEMENT UNLOCKED", 12.0, theme::ACCENT, font.clone()),
                        text(achievement.name(), 18.0, theme::TEXT, font.clone()),
                        (
                            text(achievement.criteria(), 12.0, theme::MUTED, font),
                            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                        ),
                    ],
                ),
            ],
        ));
    });
}

fn advance_toasts(
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut Toast)>,
    mut commands: Commands,
) {
    for (entity, mut toast) in &mut toasts {
        toast.elapsed += time.delta_secs();
        if toast.elapsed >= TOAST_SECS {
            commands.entity(entity).despawn();
        }
    }
}
