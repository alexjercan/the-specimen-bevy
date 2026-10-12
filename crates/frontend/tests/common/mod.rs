#![allow(dead_code)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use bevy::{
    asset::uuid::Uuid,
    audio::{AudioPlayer, PlaybackMode},
    ecs::{query::QueryFilter, resource::IsResource, system::SystemIdMarker},
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::MouseButtonInput,
        ButtonState, InputPlugin,
    },
    platform::collections::HashMap,
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{GameAudioPlugin, PlaySound};
use game_flow::{CoreState, GameState, PauseState};
use game_frontend::{controller_settings, volumes, MenuAssets, MenuPlugin, MenuSounds};
use game_settings::{GameSettings, GameSettingsPlugin, SettingsStore};
use gameplay::{
    ambience::{AmbienceSounds, AmbienceViewPlugin},
    facility::Escaped,
    fuses::FuseSeed,
    monster::{Caught, Monster},
    player::PlayerController,
    GameplayPlugin,
};

pub const SEED: u64 = 7;
pub const REVEAL_AT: f32 = 4.0;

pub fn image_handle(index: u128) -> Handle<Image> {
    Handle::Uuid(Uuid::from_u128(index + 1), default())
}

fn audio_handle(index: u128) -> Handle<AudioSource> {
    Handle::Uuid(Uuid::from_u128(index + 1), default())
}

pub fn menu_sounds() -> MenuSounds {
    MenuSounds {
        back: audio_handle(0),
        confirm: audio_handle(1),
        denied: audio_handle(2),
        focus: audio_handle(3),
        hover: audio_handle(4),
        pause: audio_handle(5),
        press: audio_handle(6),
        resume: audio_handle(7),
    }
}

pub fn menu_assets() -> MenuAssets {
    MenuAssets {
        key_glyphs: HashMap::default(),
        achievement_icons: HashMap::default(),
        font: Handle::Uuid(Uuid::from_u128(u128::MAX), default()),
    }
}

fn base_app() -> App {
    app_with(true)
}

fn app_with(fps_overlay_plugin: bool) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .add_plugins(GameSettingsPlugin { persist: false });
    let settings = app.world().resource::<GameSettings>();
    let controller = controller_settings(settings);
    let bus = volumes(settings);
    let fps_overlay = settings.fps_overlay;
    app.insert_resource(controller)
        .insert_resource(bus)
        .insert_resource(FuseSeed(SEED))
        .init_state::<CoreState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<PauseState>()
        .add_plugins((GameplayPlugin, GameAudioPlugin, game_ui::GameUiPlugin));
    if fps_overlay_plugin {
        app.add_plugins(game_ui::FpsOverlayPlugin {
            visible: fps_overlay,
        });
    }
    app.add_plugins(MenuPlugin).insert_resource(menu_sounds());
    #[cfg(feature = "debug")]
    app.init_resource::<debug::DebugSettings>();
    app.world_mut()
        .spawn((Window::default(), CursorOptions::default(), PrimaryWindow));
    app
}

fn start(mut app: App) -> App {
    app.finish();
    app.cleanup();
    app.update();
    app
}

pub fn loading_app() -> App {
    start(base_app())
}

pub fn menu_app_with(assets: MenuAssets) -> App {
    let mut app = base_app();
    app.insert_resource(assets);
    start(app)
}

pub fn menu_app() -> App {
    menu_app_with(menu_assets())
}

pub fn ready(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<CoreState>>()
        .set(CoreState::Ready);
    app.update();
}

pub fn ready_menu_app() -> App {
    let mut app = menu_app();
    ready(&mut app);
    app
}

fn ambience_sounds() -> AmbienceSounds {
    AmbienceSounds {
        roomtone: audio_handle(20),
        low_pressure: audio_handle(21),
        conduit: audio_handle(22),
        boiler: audio_handle(23),
        tank: audio_handle(24),
        vent: audio_handle(25),
        vent_wind: audio_handle(26),
        cool_buzz: audio_handle(27),
        boiler_tick: audio_handle(28),
        faucet_burst: audio_handle(29),
    }
}

pub fn ready_menu_app_with_ambience() -> App {
    let mut app = base_app();
    app.add_plugins(AmbienceViewPlugin)
        .insert_resource(ambience_sounds())
        .insert_resource(menu_assets());
    let mut app = start(app);
    ready(&mut app);
    app
}

pub fn ready_menu_app_without_fps_overlay() -> App {
    let mut app = app_with(false);
    app.insert_resource(menu_assets());
    let mut app = start(app);
    ready(&mut app);
    app
}

pub fn count<F: QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}

pub fn entities(app: &mut App) -> usize {
    count::<(
        Without<IsResource>,
        Without<SystemIdMarker>,
        Without<AudioPlayer>,
    )>(app)
}

// NOTE: one-shots end on their own, so only voices that keep playing count as leaks.
pub fn voices(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<&PlaybackSettings, With<AudioPlayer>>()
        .iter(app.world())
        .filter(|settings| !matches!(settings.mode, PlaybackMode::Despawn | PlaybackMode::Remove))
        .count()
}

pub fn named(app: &mut App, name: &str) -> Option<Entity> {
    app.world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (candidate.as_str() == name).then_some(entity))
}

pub fn named_count(app: &mut App, name: &str) -> usize {
    app.world_mut()
        .query::<&Name>()
        .iter(app.world())
        .filter(|candidate| candidate.as_str() == name)
        .count()
}

pub fn child(app: &App, entity: Entity, index: usize) -> Entity {
    app.world().get::<Children>(entity).unwrap()[index]
}

pub fn text_of(app: &App, entity: Entity) -> String {
    app.world().get::<Text>(entity).unwrap().0.clone()
}

pub fn display_of(app: &App, entity: Entity) -> Display {
    app.world().get::<Node>(entity).unwrap().display
}

pub fn has_text(app: &mut App, value: &str) -> bool {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0 == value)
}

pub fn has_text_containing(app: &mut App, value: &str) -> bool {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0.contains(value))
}

pub fn button_names(app: &mut App) -> Vec<String> {
    let mut names: Vec<String> = app
        .world_mut()
        .query_filtered::<&Name, With<Button>>()
        .iter(app.world())
        .map(|name| name.as_str().to_owned())
        .collect();
    names.sort();
    names
}

pub fn click(app: &mut App, entity: Entity) {
    app.world_mut()
        .entity_mut(entity)
        .insert(Interaction::Pressed);
    app.update();
    app.update();
}

pub fn press(app: &mut App, name: &str) {
    let button = named(app, name).unwrap_or_else(|| panic!("missing {name}"));
    click(app, button);
}

pub fn press_label(app: &mut App, label: &str) {
    let button = app
        .world_mut()
        .query_filtered::<(Entity, &Children), With<Button>>()
        .iter(app.world())
        .find_map(|(entity, children)| {
            children
                .iter()
                .any(|child| {
                    app.world()
                        .get::<Text>(child)
                        .is_some_and(|text| text.0 == label)
                })
                .then_some(entity)
        })
        .unwrap_or_else(|| panic!("missing button {label}"));
    click(app, button);
}

pub fn key_tap(app: &mut App, key_code: KeyCode, logical_key: Key) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: logical_key.clone(),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

pub fn escape(app: &mut App) {
    key_tap(app, KeyCode::Escape, Key::Escape);
}

pub fn mouse_click(app: &mut App, button: MouseButton) {
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(MouseButtonInput {
            button,
            state,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }
}

pub fn game_state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

pub fn pause_state(app: &App) -> PauseState {
    *app.world().resource::<State<PauseState>>().get()
}

pub fn cursor(app: &mut App) -> (CursorGrabMode, bool) {
    let cursor = app
        .world_mut()
        .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    (cursor.grab_mode, cursor.visible)
}

pub fn played_sounds(app: &mut App) -> Vec<Handle<AudioSource>> {
    app.world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .map(|cue| cue.handle)
        .collect()
}

pub fn run_for(app: &mut App, secs: f32) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for _ in 0..(secs * 10.0).round() as usize {
        app.update();
    }
    app.insert_resource(TimeUpdateStrategy::Automatic);
}

pub fn player(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world())
        .unwrap()
}

pub fn single<F: QueryFilter>(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, F>()
        .single(app.world())
        .unwrap()
}

pub fn win(app: &mut App) {
    let player = player(app);
    app.world_mut().entity_mut(player).insert(Escaped);
    app.update();
    app.update();
    assert_eq!(game_state(app), GameState::Complete);
    run_for(app, REVEAL_AT + 0.2);
    assert_eq!(game_state(app), GameState::Credits);
}

pub fn catch(app: &mut App) {
    let player = player(app);
    let monster = single::<With<Monster>>(app);
    app.world_mut().entity_mut(player).insert(Caught {
        monster,
        remaining: 0.0,
    });
    app.update();
    app.update();
}

static STORES: AtomicUsize = AtomicUsize::new(0);

pub fn persist_settings(app: &mut App) -> PathBuf {
    let index = STORES.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!(
        "the-specimen-core-tests-{}-{index}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    let file = path.join("settings.json");
    app.insert_resource(SettingsStore::File(file.clone()));
    file
}

pub fn saved_settings(app: &App) -> Option<GameSettings> {
    app.world().resource::<SettingsStore>().load().unwrap()
}

pub fn assert_saved(app: &App) {
    assert_eq!(
        saved_settings(app).as_ref(),
        Some(app.world().resource::<GameSettings>())
    );
}

pub fn forget_saved(file: &std::path::Path) {
    let _ = std::fs::remove_file(file);
}
