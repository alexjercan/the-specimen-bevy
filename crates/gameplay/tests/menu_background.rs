use bevy::prelude::*;
use gameplay::levels::{
    build_main_menu_background, DoorOf, DoorRef, Doors, LevelRoot, LightConfig, LightEffect,
    Passage, Prop, PropLightsPlugin, Room,
};

#[test]
fn menu_room_uses_authored_room_and_prop_light_pipeline() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PropLightsPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            build_main_menu_background(&mut commands);
        });
    app.update();

    let world = app.world_mut();
    let roots = world.query::<&LevelRoot>().iter(world).count();
    let rooms = world.query::<&Room>().iter(world).count();
    let props = world.query::<&Prop>().iter(world).count();
    assert_eq!(roots, 1);
    assert_eq!(rooms, 2);
    assert_eq!(props, 8);
    let mut openings = world.query::<(Entity, &Passage)>();
    let (passage, position) = openings.single(world).expect("one hallway passage");
    assert_eq!(position.0, Vec2::new(0.0, -6.25));
    let mut rooms = world.query::<(Entity, &Room, &Doors)>();
    let authored: Vec<_> = rooms
        .iter(world)
        .map(|(entity, room, links)| {
            (
                entity,
                room.0,
                links.iter().next().expect("room must link to passage"),
            )
        })
        .collect();
    let bounds: Vec<_> = authored
        .iter()
        .map(|(entity, bounds, link)| {
            assert_eq!(world.get::<DoorRef>(*link).unwrap().0, passage);
            assert_eq!(world.get::<DoorOf>(*link).unwrap().0, *entity);
            *bounds
        })
        .collect();
    assert!(bounds.contains(&Rect::new(-3.75, -6.25, 3.75, 1.25)));
    assert!(bounds.contains(&Rect::new(-1.25, -41.25, 1.25, -6.25)));
    let mut lights = world.query::<(&Prop, &LightConfig, &LightEffect)>();
    let lamps: Vec<_> = lights
        .iter(world)
        .filter(|(prop, _, _)| prop.0 == "ceiling_light_amber")
        .collect();
    assert_eq!(lamps.len(), 3);
    assert!(lamps.iter().all(|(_, config, effect)| {
        config.intensity.unwrap() >= 90_000.0 && matches!(effect, LightEffect::FlickerStrong(_))
    }));
    let mut distant_lamps = world.query::<(&Prop, &Transform)>();
    let last_lamp_z = distant_lamps
        .iter(world)
        .filter(|(prop, _)| prop.0 == "ceiling_light_amber")
        .map(|(_, transform)| transform.translation.z)
        .reduce(f32::min)
        .expect("menu lamps");
    assert_eq!(last_lamp_z, -16.25);
    assert!(-41.25 < last_lamp_z - 9.0);
}
