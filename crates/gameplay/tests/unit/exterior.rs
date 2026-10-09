use bevy::prelude::*;

use crate::levels::{
    builder::{Prop, Room},
    first_floor_builder::build_first_floor,
};

use super::{ExteriorCorridor, PendingExteriorRender, EXIT_CORRIDOR};

#[test]
fn exterior_corridor_is_visual_only_and_attached_to_exit_room() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let mut corridors = world.query_filtered::<(Entity, &ChildOf), With<ExteriorCorridor>>();
    let (corridor, parent) = corridors.single(world).expect("one exterior corridor");
    assert!(world.get::<PendingExteriorRender>(corridor).is_some());
    assert!(world.get::<Room>(corridor).is_none());
    let exit = world.get::<Room>(parent.0).expect("exit room parent");
    assert_eq!(exit.0.min.y, EXIT_CORRIDOR.max.y);
    assert_eq!(EXIT_CORRIDOR.width(), 2.5);
    assert_eq!(EXIT_CORRIDOR.height(), 7.5);
    assert_eq!(EXIT_CORRIDOR.min.x, -1.25);
    assert_eq!(EXIT_CORRIDOR.max.x, 1.25);
    let mut details = world.query::<(&Prop, &ChildOf)>();
    let modules: Vec<_> = details
        .iter(world)
        .filter(|(_, child)| child.0 == corridor)
        .map(|(prop, _)| prop.0.as_str())
        .collect();
    assert_eq!(modules.len(), 2);
    assert!(modules.contains(&"wall_lamp_red"));
    assert!(modules.contains(&"exit_sign"));
}
