use bevy::prelude::*;
use game_assets::UiAssets;
use game_ui::{menu_button, text, theme};
use gameplay::{controller::PlayerController, levels::Escaped};

use super::{release_cursor, screen_camera, GameState, MenuAction};

#[derive(Component)]
pub(super) struct CompleteScreen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, finish_run.run_if(in_state(GameState::Playing)))
        .add_systems(
            OnEnter(GameState::Complete),
            (spawn_complete_screen, release_cursor),
        );
}

fn finish_run(
    players: Query<(), (With<PlayerController>, With<Escaped>)>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !players.is_empty() {
        next.set(GameState::Complete);
    }
}

fn spawn_complete_screen(mut commands: Commands, assets: Res<UiAssets>) {
    let font = assets.font.clone();
    commands.spawn(screen_camera(GameState::Complete));
    commands.spawn((
        CompleteScreen,
        Name::new("Complete screen"),
        DespawnOnExit(GameState::Complete),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::left(px(96)),
            row_gap: px(12),
            ..default()
        },
        BackgroundColor(theme::BACKGROUND),
        children![
            (
                text("You escaped", 56.0, theme::TEXT, font.clone()),
                Node {
                    margin: UiRect::bottom(px(28)),
                    ..default()
                },
            ),
            (
                Node {
                    width: px(240),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![
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
            ),
        ],
    ));
}
