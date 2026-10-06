use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use debug::{fps_label, DebugPlugin, DebugSettings, FpsText, INSPECTOR_TOGGLE_KEY};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, DebugPlugin))
        .init_resource::<ButtonInput<KeyCode>>();
    app.update();
    app
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release(key);
    input.clear();
}

#[test]
fn headless_app_skips_egui_and_wireframe() {
    let app = app();
    assert!(!app.is_plugin_added::<EguiPlugin>());
    assert!(!app.is_plugin_added::<WireframePlugin>());
    assert_eq!(
        *app.world().resource::<DebugSettings>(),
        DebugSettings::default()
    );
}

#[test]
fn fps_overlay_spawns_once_with_label() {
    let mut app = app();
    app.update();
    let mut texts = app.world_mut().query_filtered::<&Text, With<FpsText>>();
    let texts: Vec<_> = texts.iter(app.world()).collect();
    assert_eq!(texts.len(), 1);
    assert!(texts[0].0.starts_with("FPS "));
}

#[test]
fn fps_label_formats_rounded_or_placeholder() {
    assert_eq!(fps_label(Some(59.6)), "FPS 60");
    assert_eq!(fps_label(None), "FPS --");
}

#[test]
fn f12_toggles_inspector_only() {
    let mut app = app();
    tap(&mut app, INSPECTOR_TOGGLE_KEY);
    assert!(app.world().resource::<DebugSettings>().inspector);
    tap(&mut app, KeyCode::F11);
    assert!(app.world().resource::<DebugSettings>().inspector);
    tap(&mut app, INSPECTOR_TOGGLE_KEY);
    let settings = *app.world().resource::<DebugSettings>();
    assert!(!settings.inspector);
    assert!(!settings.wireframe);
}

#[test]
fn wireframe_setting_drives_global_config() {
    let mut app = app();
    app.init_resource::<WireframeConfig>();
    app.world_mut().resource_mut::<DebugSettings>().wireframe = true;
    app.update();
    assert!(app.world().resource::<WireframeConfig>().global);
    app.world_mut().resource_mut::<DebugSettings>().wireframe = false;
    app.update();
    assert!(!app.world().resource::<WireframeConfig>().global);
}
