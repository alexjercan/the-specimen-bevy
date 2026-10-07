use bevy::prelude::*;
use game_assets::UiAssets;
use game_ui::{menu_button, panel, text, theme};

use super::{release_cursor, GameState, MenuAction, PauseState};

#[derive(Component)]
pub(super) struct PauseMenu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, toggle_pause.run_if(in_state(GameState::Playing)))
        .add_systems(
            OnEnter(PauseState::Paused),
            (spawn_pause_menu, freeze_clock, release_cursor),
        )
        .add_systems(OnExit(PauseState::Paused), resume_clock)
        .add_systems(OnExit(GameState::Playing), resume_clock);
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<PauseState>>,
    mut next: ResMut<NextState<PauseState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(match state.get() {
            PauseState::Running => PauseState::Paused,
            PauseState::Paused => PauseState::Running,
        });
    }
}

fn freeze_clock(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_clock(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn spawn_pause_menu(mut commands: Commands, assets: Res<UiAssets>) {
    let font = assets.font.clone();
    commands.spawn((
        PauseMenu,
        Name::new("Pause menu"),
        DespawnOnExit(PauseState::Paused),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(theme::SCRIM),
        GlobalZIndex(100),
        children![(
            Name::new("Pause panel"),
            Node {
                width: px(260),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::all(px(20)),
                border: UiRect::all(px(theme::BORDER)),
                border_radius: BorderRadius::all(px(theme::RADIUS)),
                ..default()
            },
            panel(),
            children![
                (
                    text("Paused", 26.0, theme::TEXT, font.clone()),
                    Node {
                        margin: UiRect::bottom(px(8)),
                        ..default()
                    },
                ),
                (
                    Name::new("Resume button"),
                    MenuAction::Resume,
                    menu_button("Resume", font.clone()),
                ),
                (
                    Name::new("Main menu button"),
                    MenuAction::MainMenu,
                    menu_button("Main Menu", font.clone()),
                ),
                (
                    Name::new("Quit button"),
                    MenuAction::Quit,
                    menu_button("Quit", font.clone()),
                ),
            ],
        )],
    ));
}
