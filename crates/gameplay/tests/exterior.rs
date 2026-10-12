use bevy::prelude::*;
use gameplay::facility::Module;
use gameplay::facility::{build_first_floor, Prop, Room};

mod common;

#[test]
fn exterior_corridor_is_visual_only_and_attached_to_exit_room() {
    let mut app = common::level_app();
    app.add_systems(Startup, build_first_floor);
    app.update();

    let world = app.world_mut();
    let (exit, bounds) = world
        .query::<(Entity, &Name, &Room)>()
        .iter(world)
        .find(|(_, name, _)| name.as_str() == "exit")
        .map(|(entity, _, room)| (entity, room.0))
        .expect("exit room");
    let props: Vec<(Module, Vec3, Entity)> = world
        .query::<(&Prop, &Transform, &ChildOf)>()
        .iter(world)
        .map(|(prop, transform, parent)| (prop.0, transform.translation, parent.parent()))
        .collect();
    let attachments: Vec<Entity> = world
        .query::<(Entity, &ChildOf)>()
        .iter(world)
        .filter(|(_, parent)| parent.parent() == exit)
        .map(|(entity, _)| entity)
        .collect();
    let corridors: Vec<Entity> = attachments
        .into_iter()
        .filter(|&attachment| {
            world.get::<Room>(attachment).is_none()
                && props.iter().any(|(_, _, parent)| *parent == attachment)
        })
        .collect();
    assert_eq!(
        corridors.len(),
        1,
        "one visual-only corridor on the exit room"
    );
    let corridor = corridors[0];
    let details: Vec<_> = props
        .iter()
        .filter(|(_, _, parent)| *parent == corridor)
        .collect();
    assert_eq!(details.len(), 2);
    assert!(details
        .iter()
        .any(|(module, _, _)| *module == Module::WallLampRed));
    assert!(details
        .iter()
        .any(|(module, _, _)| *module == Module::ExitSign));
    for (_, position, _) in details {
        assert!(
            position.z < bounds.min.y,
            "corridor detail must sit outside the exit room"
        );
        assert!(bounds.min.x < position.x && position.x < bounds.max.x);
    }
}
