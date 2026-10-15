mod achievements;
mod background;
mod cinematic;
mod complete;
mod credits;
mod game_over;
mod loading;
mod main_menu;
mod music;
mod pause;
mod settings;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    platform::collections::HashMap,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_audio::{Bus, PlaySound};
use game_flow::{GameState, PauseState};

pub(crate) const TITLE: &str = "The Specimen";

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

const UI_GAIN: f32 = 0.6;

#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct MenuSounds {
    pub back: Handle<AudioSource>,
    pub confirm: Handle<AudioSource>,
    pub denied: Handle<AudioSource>,
    pub focus: Handle<AudioSource>,
    pub hover: Handle<AudioSource>,
    pub pause: Handle<AudioSource>,
    pub press: Handle<AudioSource>,
    pub resume: Handle<AudioSource>,
    pub ambient: Handle<AudioSource>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct MenuAssets {
    pub font: Handle<Font>,
    pub key_glyphs: HashMap<String, Handle<Image>>,
    pub achievement_icons: HashMap<String, Handle<Image>>,
}

impl MenuAssets {
    pub fn key_glyph(&self, key: &str) -> Option<Handle<Image>> {
        self.key_glyphs.get(key).cloned()
    }

    pub fn achievement_icon(&self, stem: &str, unlocked: bool) -> Option<Handle<Image>> {
        let suffix = if unlocked { "unlocked" } else { "locked" };
        self.achievement_icons
            .get(format!("{stem}-{suffix}").as_str())
            .cloned()
    }
}

pub(crate) fn ui_sound(handle: &Handle<AudioSource>) -> PlaySound {
    PlaySound::global(handle.clone(), Bus::Ui, UI_GAIN)
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            loading::plugin,
            main_menu::plugin,
            pause::plugin,
            cinematic::plugin,
            complete::plugin,
            credits::plugin,
            game_over::plugin,
            settings::plugin,
            achievements::plugin,
            music::plugin,
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
    preset: Res<gameplay::presets::GamePreset>,
    mut runs: MessageWriter<gameplay::facility::StartRun>,
    mut selection: ResMut<main_menu::ModeSelection>,
    mut settings: ResMut<game_settings::GameSettings>,
    mut settings_dirty: ResMut<game_settings::SettingsDirty>,
    progress: Res<gameplay::achievements::AchievementProgress>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    menu_sounds: Res<MenuSounds>,
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
                runs.write(gameplay::facility::StartRun {
                    preset: selection.preset(),
                });
                game.set(GameState::Playing);
                &menu_sounds.confirm
            }
            MenuAction::PreviousMode => {
                selection.cycle(-1, &progress);
                selection.save_into(&mut settings);
                settings_dirty.0 = true;
                &menu_sounds.press
            }
            MenuAction::NextMode => {
                selection.cycle(1, &progress);
                selection.save_into(&mut settings);
                settings_dirty.0 = true;
                &menu_sounds.press
            }
            MenuAction::AdjustCustom(field, step) => {
                if selection.0 == gameplay::presets::GameMode::Custom {
                    selection.adjust(*field, *step);
                    selection.save_into(&mut settings);
                    settings_dirty.0 = true;
                }
                &menu_sounds.press
            }
            MenuAction::Retry => {
                runs.write(gameplay::facility::StartRun { preset: *preset });
                game.set(GameState::Playing);
                &menu_sounds.confirm
            }
            MenuAction::Resume => {
                pause.set(PauseState::Running);
                &menu_sounds.resume
            }
            MenuAction::MainMenu => {
                game.set(GameState::MainMenu);
                &menu_sounds.back
            }
            MenuAction::Credits => {
                *route = credits::CreditsRoute::MainMenu;
                game.set(GameState::Credits);
                &menu_sounds.confirm
            }
            MenuAction::Achievements => {
                game.set(GameState::Achievements);
                &menu_sounds.confirm
            }
            #[cfg(not(target_arch = "wasm32"))]
            MenuAction::Quit => {
                exit.write(AppExit::Success);
                &menu_sounds.press
            }
        };
        sounds.write(ui_sound(sound));
    }
}

fn hover_buttons(
    buttons: Query<&Interaction, (With<game_ui::MenuButton>, Changed<Interaction>)>,
    menu_sounds: Res<MenuSounds>,
    mut sounds: MessageWriter<PlaySound>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Hovered {
            sounds.write(ui_sound(&menu_sounds.hover));
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
