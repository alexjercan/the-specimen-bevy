use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::LineBreak,
};
use game_assets::UiAssets;
use game_audio::{PlaySound, Sound};
use game_ui::{menu_button, text, theme};
use gameplay::achievements::{Achievement, AchievementProgress, AchievementUnlocked};

use super::{release_cursor, screen_camera, GameState, MenuAction};

const TOAST_SECS: f32 = 4.0;

#[cfg(test)]
#[path = "../../tests/unit/menu_achievements.rs"]
mod tests;

#[derive(Component)]
pub(super) struct AchievementsScreen;

#[derive(Component)]
struct AchievementList;

#[derive(Component)]
struct ToastContainer;

#[derive(Component)]
struct Toast {
    elapsed: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_toast)
        .add_systems(
            OnEnter(GameState::Achievements),
            (spawn_achievements, release_cursor),
        )
        .add_systems(
            Update,
            (back_on_escape, scroll_achievements).run_if(in_state(GameState::Achievements)),
        )
        .add_systems(Update, advance_toasts);
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
        Achievement::EscapeWithoutFlashbang => "empty-handed",
        Achievement::EscapeWithoutDetector => "trust-your-ears",
        Achievement::FlashbangHitMonster => "buy-some-time",
        Achievement::EscapeWithoutBoiler => "in-the-dark",
        Achievement::RestoreBoiler => "let-there-be-light",
        Achievement::CaughtAfterExitOpen => "so-close",
        Achievement::EscapeUndetected => "unseen",
    }
}

fn achievement_row(
    achievement: Achievement,
    unlocked: bool,
    font: Handle<Font>,
    icon: Handle<Image>,
) -> impl Bundle {
    let (status, color) = status_text(unlocked);
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
                    text(achievement.name(), 17.0, theme::TEXT, font.clone()),
                    (
                        text(achievement.criteria(), 13.0, theme::MUTED, font.clone()),
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
    assets: Res<UiAssets>,
    progress: Res<AchievementProgress>,
) {
    let font = assets.font.clone();
    commands.spawn(screen_camera(GameState::Achievements));
    let unlocked = progress.unlocked.len();
    let total = Achievement::ALL.len();
    commands
        .spawn((
            AchievementsScreen,
            Name::new("Achievements screen"),
            DespawnOnExit(GameState::Achievements),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|parent| {
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
                                list.spawn(achievement_row(
                                    achievement,
                                    unlocked,
                                    font.clone(),
                                    icon,
                                ));
                            }
                        });
                    panel.spawn((
                        Name::new("Back button"),
                        MenuAction::MainMenu,
                        menu_button("Back", font),
                    ));
                });
        });
}

fn scroll_achievements(
    mut wheel: MessageReader<MouseWheel>,
    mut lists: Query<&mut ScrollPosition, With<AchievementList>>,
) {
    let mut delta = 0.0;
    for event in wheel.read() {
        delta -= event.y
            * match event.unit {
                MouseScrollUnit::Line => 38.0,
                MouseScrollUnit::Pixel => 1.0,
            };
    }
    for mut position in &mut lists {
        position.0.y = (position.0.y + delta).max(0.0);
    }
}

fn back_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::MainMenu);
        sounds.write(PlaySound {
            sound: Sound::UiBack,
            position: None,
        });
    }
}

fn spawn_toast(
    unlock: On<AchievementUnlocked>,
    assets: Option<Res<UiAssets>>,
    containers: Query<Entity, With<ToastContainer>>,
    mut commands: Commands,
    mut sounds: MessageWriter<PlaySound>,
) {
    let Some(assets) = assets else {
        return;
    };
    let font = assets.font.clone();
    let achievement = unlock.0;
    let icon = assets
        .achievement_icon(icon_stem(achievement), true)
        .unwrap_or_default();
    sounds.write(PlaySound {
        sound: Sound::UiConfirm,
        position: None,
    });
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
