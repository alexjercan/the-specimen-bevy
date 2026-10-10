use bevy::prelude::*;
use gameplay::levels::build_main_menu_background;

use super::GameState;

#[cfg(test)]
#[path = "../../tests/unit/menu_background.rs"]
mod tests;

pub(super) fn spawn(commands: &mut Commands, state: GameState) {
    let root = build_main_menu_background(commands);
    commands.entity(root).insert(DespawnOnExit(state));

    commands.spawn((
        Name::new("Main menu room camera"),
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.002, 0.004, 0.008)),
            order: 0,
            ..default()
        },
        Transform::from_xyz(0.0, 1.65, 0.9).looking_at(Vec3::new(0.0, 1.3, -3.6), Vec3::Y),
        DespawnOnExit(state),
    ));
    commands.spawn((
        Name::new("Main menu UI camera"),
        Camera2d,
        IsDefaultUiCamera,
        Camera {
            clear_color: ClearColorConfig::None,
            order: 1,
            ..default()
        },
        DespawnOnExit(state),
    ));
    commands.spawn((
        Name::new("Main menu flashlight accent"),
        SpotLight {
            color: Color::srgb(0.72, 0.82, 1.0),
            intensity: 18_000.0,
            range: 10.0,
            inner_angle: 0.18,
            outer_angle: 0.38,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.6, 1.65, 0.7).looking_at(Vec3::new(1.4, 1.0, -4.0), Vec3::Y),
        ChildOf(root),
    ));
}
