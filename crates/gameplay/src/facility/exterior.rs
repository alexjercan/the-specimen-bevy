use bevy::{math::Rect, prelude::*};

use super::{
    builder::{prop, LightEffect},
    lights::LightConfig,
    module::Module,
};

pub(crate) const EXIT_CORRIDOR: Rect = Rect::new(-1.25, -38.75, 1.25, -31.25);

#[derive(Component)]
pub(crate) struct ExteriorCorridor;

#[derive(Component)]
pub(crate) struct PendingExteriorRender;

pub(crate) fn spawn(commands: &mut Commands, parent: Entity) {
    let corridor = commands
        .spawn((
            Name::new("Exterior corridor"),
            ExteriorCorridor,
            PendingExteriorRender,
            Transform::IDENTITY,
            Visibility::default(),
            ChildOf(parent),
        ))
        .id();
    spawn_details(commands, corridor);
}

pub(crate) fn spawn_details(commands: &mut Commands, parent: Entity) {
    commands.spawn((
        prop(
            "Exterior corridor lamp",
            Module::WallLampRed,
            Transform::from_xyz(1.0, 2.2, -31.35),
        ),
        LightConfig {
            intensity: Some(18_000.0),
            effect: Some(LightEffect::Pulse),
            ..default()
        },
        ChildOf(parent),
    ));
    commands.spawn((
        prop(
            "Exterior corridor sign",
            Module::ExitSign,
            Transform::from_xyz(0.0, 2.62, -31.35),
        ),
        ChildOf(parent),
    ));
}
