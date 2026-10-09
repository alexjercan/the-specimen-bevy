use bevy::camera::RenderTarget;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::PbrPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_egui::{
    egui, EguiContext, EguiGlobalSettings, EguiMultipassSchedule, EguiPlugin,
    EguiPrimaryContextPass, PrimaryEguiContext,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use game_settings::GameSettings;

#[cfg(test)]
#[path = "../tests/unit/context.rs"]
mod context_tests;

pub const INSPECTOR_TOGGLE_KEY: KeyCode = KeyCode::F12;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSettings {
    pub inspector: bool,
    pub wireframe: bool,
}

#[derive(Component)]
pub struct FpsText;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        if app.is_plugin_added::<PbrPlugin>() && !app.is_plugin_added::<WireframePlugin>() {
            app.add_plugins(WireframePlugin::default());
        }
        if app.is_plugin_added::<WinitPlugin>() {
            if !app.is_plugin_added::<EguiPlugin>() {
                app.add_plugins(EguiPlugin::default());
            }
            app.world_mut()
                .resource_mut::<EguiGlobalSettings>()
                .auto_create_primary_context = false;
            app.add_plugins(DefaultInspectorConfigPlugin)
                .add_systems(Update, keep_inspector_on_window_camera)
                .add_systems(
                    EguiPrimaryContextPass,
                    inspector_ui.run_if(|settings: Res<DebugSettings>| settings.inspector),
                );
        }
        app.init_resource::<DebugSettings>()
            .add_systems(Startup, spawn_fps_text)
            .add_systems(
                Update,
                (
                    toggle_inspector.run_if(input_just_pressed(INSPECTOR_TOGGLE_KEY)),
                    update_fps_text,
                    sync_fps_visibility,
                    sync_wireframe.run_if(
                        resource_exists::<WireframeConfig>
                            .and_then(resource_changed::<DebugSettings>),
                    ),
                ),
            );
    }
}

pub fn fps_label(fps: Option<f64>) -> String {
    match fps {
        Some(fps) => format!("FPS {fps:.0}"),
        None => "FPS --".into(),
    }
}

fn spawn_fps_text(mut commands: Commands) {
    commands.spawn((
        FpsText,
        Visibility::Visible,
        Text::new(fps_label(None)),
        TextFont::from_font_size(16.0),
        TextColor(Color::srgb(0.4, 1.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            right: px(8),
            ..default()
        },
        GlobalZIndex(i32::MAX),
    ));
}

fn sync_fps_visibility(
    settings: Option<Res<GameSettings>>,
    mut texts: Query<&mut Visibility, With<FpsText>>,
) {
    let visible = settings.is_none_or(|settings| settings.fps_overlay);
    for mut visibility in &mut texts {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_fps_text(diagnostics: Res<DiagnosticsStore>, mut texts: Query<&mut Text, With<FpsText>>) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed());
    let label = fps_label(fps);
    for mut text in &mut texts {
        if text.0 != label {
            text.0.clone_from(&label);
        }
    }
}

fn toggle_inspector(mut settings: ResMut<DebugSettings>) {
    settings.inspector = !settings.inspector;
}

fn sync_wireframe(settings: Res<DebugSettings>, mut config: ResMut<WireframeConfig>) {
    if config.global != settings.wireframe {
        config.global = settings.wireframe;
    }
}

fn keep_inspector_on_window_camera(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera, &RenderTarget, Has<PrimaryEguiContext>)>,
) {
    let primary = cameras
        .iter()
        .filter(|(_, camera, target, _)| {
            camera.is_active && matches!(target, RenderTarget::Window(_))
        })
        .max_by_key(|(_, camera, _, _)| camera.order)
        .map(|(entity, _, _, _)| entity);
    for (entity, _, _, has_context) in &cameras {
        if has_context && Some(entity) != primary {
            commands
                .entity(entity)
                .remove::<(PrimaryEguiContext, EguiContext, EguiMultipassSchedule)>();
        }
    }
    if let Some(primary) = primary {
        if cameras
            .get(primary)
            .is_ok_and(|(_, _, _, has_context)| !has_context)
        {
            commands.entity(primary).insert(PrimaryEguiContext);
        }
    }
}

fn inspector_ui(world: &mut World) {
    let Ok(context) = world
        .query_filtered::<&EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut context = context.clone();
    let mut wireframe = world.resource::<DebugSettings>().wireframe;
    egui::Window::new("Inspector").show(context.get_mut(), |ui| {
        ui.checkbox(&mut wireframe, "Wireframe");
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
        });
    });
    let mut settings = world.resource_mut::<DebugSettings>();
    if settings.wireframe != wireframe {
        settings.wireframe = wireframe;
    }
}
