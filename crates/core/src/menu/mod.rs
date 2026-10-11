mod achievements;
mod background;
mod cinematic;
mod complete;
mod credits;
mod game_over;
mod loading;
mod main_menu;
mod pause;
mod settings;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_audio::{PlaySound, Sound};

use crate::CoreState;

pub(crate) const TITLE: &str = "The Specimen";

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(CoreState = CoreState::Ready)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Complete,
    GameOver,
    Credits,
    Achievements,
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
    PreviousMode,
    NextMode,
    AdjustCustom(main_menu::CustomField, isize),
    Retry,
    Resume,
    MainMenu,
    Credits,
    Achievements,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .add_plugins((
                loading::plugin,
                main_menu::plugin,
                pause::plugin,
                cinematic::plugin,
                complete::plugin,
                credits::plugin,
                game_over::plugin,
                settings::plugin,
                achievements::plugin,
            ))
            .add_systems(Update, (activate_buttons, hover_buttons));
    }
}

fn activate_buttons(
    buttons: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    overlay: Query<(), With<settings::SettingsOverlay>>,
    achievements: Query<(), With<achievements::PauseAchievements>>,
    mut game: ResMut<NextState<GameState>>,
    mut pause: ResMut<NextState<PauseState>>,
    mut route: ResMut<credits::CreditsRoute>,
    mut preset: ResMut<gameplay::presets::GamePreset>,
    mut selection: ResMut<main_menu::ModeSelection>,
    mut settings: ResMut<game_settings::GameSettings>,
    mut settings_dirty: ResMut<game_settings::SettingsDirty>,
    progress: Res<gameplay::achievements::AchievementProgress>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if !overlay.is_empty() || !achievements.is_empty() {
        return;
    }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let sound = match action {
            MenuAction::Play => {
                *preset = if selection.0 == gameplay::presets::GameMode::Custom {
                    selection.1
                } else {
                    gameplay::presets::GamePreset::of(selection.0)
                };
                game.set(GameState::Playing);
                Sound::UiConfirm
            }
            MenuAction::PreviousMode => {
                selection.cycle(-1, &progress);
                selection.save_into(&mut settings);
                settings_dirty.0 = true;
                Sound::UiPress
            }
            MenuAction::NextMode => {
                selection.cycle(1, &progress);
                selection.save_into(&mut settings);
                settings_dirty.0 = true;
                Sound::UiPress
            }
            MenuAction::AdjustCustom(field, step) => {
                if selection.0 == gameplay::presets::GameMode::Custom {
                    selection.adjust(*field, *step);
                    selection.save_into(&mut settings);
                    settings_dirty.0 = true;
                }
                Sound::UiPress
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
            MenuAction::Credits => {
                *route = credits::CreditsRoute::MainMenu;
                game.set(GameState::Credits);
                Sound::UiConfirm
            }
            MenuAction::Achievements => {
                game.set(GameState::Achievements);
                Sound::UiConfirm
            }
            #[cfg(not(target_arch = "wasm32"))]
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
            sounds.write(PlaySound::global(Sound::UiHover));
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

fn scroll_list<M: Component>(
    mut wheel: MessageReader<MouseWheel>,
    mut lists: Query<&mut ScrollPosition, With<M>>,
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
