mod background;
mod cinematic;
mod complete;
mod game_over;
#[cfg(test)]
#[path = "../../tests/unit/menu_hover.rs"]
mod hover_tests;
mod loading;
mod main_menu;
mod pause;
mod settings;
#[cfg(test)]
mod tests;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_audio::{PlaySound, Sound};

use crate::CoreState;

pub(crate) const TITLE: &str = "HORROR GAME";

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(CoreState = CoreState::Ready)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Complete,
    GameOver,
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
    Retry,
    Resume,
    MainMenu,
    Quit,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<game_ui::GameUiPlugin>() {
            app.add_plugins(game_ui::GameUiPlugin);
        }
        app.init_resource::<game_settings::GameSettings>()
            .init_resource::<game_settings::SettingsDirty>()
            .add_message::<PlaySound>()
            .add_sub_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_plugins((
                loading::plugin,
                main_menu::plugin,
                pause::plugin,
                cinematic::plugin,
                complete::plugin,
                game_over::plugin,
                settings::plugin,
            ))
            .add_systems(Update, (activate_buttons, hover_buttons));
    }
}

fn activate_buttons(
    buttons: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    overlay: Query<(), With<settings::SettingsOverlay>>,
    mut game: ResMut<NextState<GameState>>,
    mut pause: ResMut<NextState<PauseState>>,
    mut exit: MessageWriter<AppExit>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if !overlay.is_empty() {
        return;
    }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let sound = match action {
            MenuAction::Play => {
                game.set(GameState::Playing);
                Sound::UiConfirm
            }
            MenuAction::Retry => {
                game.set(GameState::Playing);
                Sound::UiConfirm
            }
            MenuAction::Resume => {
                pause.set(PauseState::Running);
                Sound::UiResume
            }
            MenuAction::MainMenu => {
                game.set(GameState::MainMenu);
                Sound::UiBack
            }
            MenuAction::Quit => {
                exit.write(AppExit::Success);
                Sound::UiPress
            }
        };
        sounds.write(PlaySound {
            sound,
            position: None,
        });
    }
}

fn hover_buttons(
    buttons: Query<&Interaction, (With<game_ui::MenuButton>, Changed<Interaction>)>,
    mut sounds: MessageWriter<PlaySound>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Hovered {
            sounds.write(PlaySound {
                sound: Sound::UiHover,
                position: None,
            });
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
