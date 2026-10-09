use bevy::{math::Rect, prelude::*};

use super::{
    builder::{passage, prop, room, DoorOf, DoorRef, LevelRoot, LightEffect},
    lights::LightConfig,
};

#[derive(Component)]
pub(crate) struct MenuBackground;

pub fn build_main_menu_background(commands: &mut Commands) -> Entity {
    let root = commands
        .spawn((
            Name::new("Main menu room"),
            LevelRoot,
            MenuBackground,
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .id();
    let entrance = commands
        .spawn((
            passage("Main menu room / hallway", Vec2::new(0.0, -6.25)),
            ChildOf(root),
        ))
        .id();
    commands
        .spawn((
            room(
                "Main menu room shell",
                Rect::new(-3.75, -6.25, 3.75, 1.25),
                "floor_tile",
                "wall_conduit",
                "ceiling_tile",
            ),
            ChildOf(root),
        ))
        .with_related::<DoorOf>((DoorRef(entrance), ChildOf(root)));
    commands
        .spawn((
            room(
                "Main menu hallway",
                Rect::new(-1.25, -41.25, 1.25, -6.25),
                "floor_tile",
                "wall_conduit",
                "ceiling_tile",
            ),
            ChildOf(root),
        ))
        .with_related::<DoorOf>((DoorRef(entrance), ChildOf(root)));

    commands.spawn((
        prop(
            "Main menu flickering lamp",
            "ceiling_light_amber",
            Transform::from_xyz(0.0, 0.0, -2.5),
        ),
        LightConfig {
            intensity: Some(110_000.0),
            effect: Some(LightEffect::FlickerStrong(0.7)),
            ..default()
        },
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu hallway lamp near",
            "ceiling_light_amber",
            Transform::from_xyz(0.0, 0.0, -8.75),
        ),
        LightConfig {
            intensity: Some(95_000.0),
            effect: Some(LightEffect::FlickerStrong(2.1)),
            ..default()
        },
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu hallway lamp far",
            "ceiling_light_amber",
            Transform::from_xyz(0.0, 0.0, -16.25),
        ),
        LightConfig {
            intensity: Some(90_000.0),
            effect: Some(LightEffect::FlickerStrong(4.3)),
            ..default()
        },
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu workbench",
            "workbench",
            Transform::from_xyz(2.2, 0.0, -4.6).with_rotation(Quat::from_rotation_y(-0.4)),
        ),
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu shelves",
            "shelf_unit_bins",
            Transform::from_xyz(-2.7, 0.0, -5.7),
        ),
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu crate",
            "storage_crate",
            Transform::from_xyz(2.8, 0.0, -2.4),
        ),
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu tipped chair",
            "chair_tipped",
            Transform::from_xyz(0.8, 0.0, -3.6).with_rotation(Quat::from_rotation_y(0.65)),
        ),
        ChildOf(root),
    ));
    commands.spawn((
        prop(
            "Main menu wall vent",
            "wall_vent",
            Transform::from_xyz(1.9, 1.9, -6.2),
        ),
        ChildOf(root),
    ));
    root
}
