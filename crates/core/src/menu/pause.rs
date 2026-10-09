#[cfg(target_arch = "wasm32")]
use std::cell::Cell;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use game_assets::UiAssets;
use game_ui::{menu_button, panel, text, theme};
use gameplay::controller::PlayerControlsEnabled;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

use super::{release_cursor, settings::SettingsOverlay, GameState, MenuAction, PauseState};

#[derive(Component)]
pub(super) struct PauseMenu;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static POINTER_LOCK_LOST: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn plugin(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    watch_pointer_lock(app);
    app.add_systems(Update, toggle_pause.run_if(in_state(GameState::Playing)))
        .add_systems(
            OnEnter(PauseState::Paused),
            (
                spawn_pause_menu,
                freeze_clock,
                disable_controls,
                release_cursor,
            ),
        )
        .add_systems(OnExit(PauseState::Paused), (resume_clock, enable_controls))
        .add_systems(OnExit(GameState::Playing), resume_clock);
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<PauseState>>,
    overlay: Query<(), With<SettingsOverlay>>,
    mut next: ResMut<NextState<PauseState>>,
) {
    if overlay.is_empty() && keys.just_pressed(KeyCode::Escape) {
        next.set(match state.get() {
            PauseState::Running => PauseState::Paused,
            PauseState::Paused => PauseState::Running,
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn watch_pointer_lock(app: &mut App) {
    let document = web_sys::window().unwrap().document().unwrap();
    let listener_document = document.clone();
    let listener = Closure::<dyn FnMut()>::new(move || {
        if listener_document.pointer_lock_element().is_none() {
            POINTER_LOCK_LOST.with(|lost| lost.set(true));
        }
    });
    document
        .add_event_listener_with_callback("pointerlockchange", listener.as_ref().unchecked_ref())
        .unwrap();
    listener.forget();
    app.add_systems(Update, pause_on_pointer_unlock);
}

#[cfg(target_arch = "wasm32")]
fn pause_on_pointer_unlock(
    game: Option<Res<State<GameState>>>,
    state: Option<Res<State<PauseState>>>,
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    next: Option<ResMut<NextState<PauseState>>>,
) {
    if !POINTER_LOCK_LOST.with(|lost| lost.replace(false))
        || !game.is_some_and(|game| *game.get() == GameState::Playing)
        || !state.is_some_and(|state| *state.get() == PauseState::Running)
    {
        return;
    }
    if cursors
        .iter()
        .any(|cursor| cursor.grab_mode == CursorGrabMode::Locked)
    {
        if let Some(mut next) = next {
            next.set(PauseState::Paused);
        }
    }
}

fn disable_controls(mut enabled: ResMut<PlayerControlsEnabled>) {
    enabled.0 = false;
}

fn enable_controls(mut enabled: ResMut<PlayerControlsEnabled>) {
    enabled.0 = true;
}

fn freeze_clock(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_clock(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn spawn_pause_menu(mut commands: Commands, assets: Res<UiAssets>) {
    let font = assets.font.clone();
    let pause_menu = commands
        .spawn((
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
        ))
        .id();
    let pause_panel = commands
        .spawn((
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
                    Name::new("Settings button"),
                    super::settings::SettingsAction::Open,
                    menu_button("Settings", font.clone()),
                ),
                (
                    Name::new("Main menu button"),
                    MenuAction::MainMenu,
                    menu_button("Main Menu", font.clone()),
                ),
            ],
        ))
        .id();
    commands.entity(pause_menu).add_child(pause_panel);
    #[cfg(not(target_arch = "wasm32"))]
    commands.entity(pause_panel).with_children(|parent| {
        parent.spawn((
            Name::new("Quit button"),
            MenuAction::Quit,
            menu_button("Quit", font.clone()),
        ));
    });
}
