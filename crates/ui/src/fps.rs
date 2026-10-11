use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

pub struct FpsOverlayPlugin;

impl Plugin for FpsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_text)
            .add_systems(Update, update_fps_text);
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
