mod loading;
mod main_menu;
mod pause;
#[cfg(test)]
mod tests;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::CoreState;

pub(crate) const TITLE: &str = "HORROR GAME";

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(CoreState = CoreState::Ready)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
}

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
enum MenuAction {
    Play,
    Resume,
    MainMenu,
    Quit,
}

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<game_ui::GameUiPlugin>() {
            app.add_plugins(game_ui::GameUiPlugin);
        }
        app.add_sub_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_plugins((loading::plugin, main_menu::plugin, pause::plugin))
            .add_systems(Update, activate_buttons);
    }
}

fn activate_buttons(
    buttons: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut game: ResMut<NextState<GameState>>,
    mut pause: ResMut<NextState<PauseState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuAction::Play => game.set(GameState::Playing),
            MenuAction::Resume => pause.set(PauseState::Running),
            MenuAction::MainMenu => game.set(GameState::MainMenu),
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

fn screen_camera<S: States>(state: S) -> impl Bundle {
    (
        Name::new("Screen camera"),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(game_ui::theme::BACKGROUND),
            ..default()
        },
        DespawnOnExit(state),
    )
}

fn release_cursor(mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut cursor in &mut cursors {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}
