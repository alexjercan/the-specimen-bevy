use bevy::prelude::*;
use game_ui::theme;

use super::{screen_camera, TITLE};
use crate::CoreState;

const PULSE_SECS: f32 = 1.6;

#[derive(Component)]
pub(super) struct LoadingScreen;

#[derive(Component)]
struct LoadingLabel;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(CoreState::Loading), spawn_loading_screen)
        .add_systems(Update, pulse_label.run_if(in_state(CoreState::Loading)));
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn(screen_camera(CoreState::Loading));
    commands.spawn((
        LoadingScreen,
        Name::new("Loading screen"),
        DespawnOnExit(CoreState::Loading),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(24),
            ..default()
        },
        BackgroundColor(theme::BACKGROUND),
        children![
            (
                Text::new(TITLE),
                TextFont::from_font_size(40.0),
                TextColor(theme::TEXT),
            ),
            (
                LoadingLabel,
                Text::new("Loading"),
                TextFont::from_font_size(16.0),
                TextColor(theme::MUTED),
            ),
        ],
    ));
}

fn pulse_label(time: Res<Time<Real>>, mut labels: Query<&mut TextColor, With<LoadingLabel>>) {
    let phase = time.elapsed_secs() / PULSE_SECS * std::f32::consts::TAU;
    let alpha = 0.35 + 0.65 * (0.5 + 0.5 * phase.sin());
    for mut color in &mut labels {
        color.0 = theme::MUTED.with_alpha(alpha);
    }
}
