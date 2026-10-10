use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use game_settings::GameSettings;

#[derive(Component)]
pub struct FpsText;

pub struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        app.add_systems(Startup, spawn_fps_text)
            .add_systems(Update, (update_fps_text, sync_fps_visibility));
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
        Visibility::Hidden,
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
    let visible = settings.is_some_and(|settings| settings.fps_overlay);
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
