use bevy::prelude::*;
use game_ui::{flashlight_meter, FlashlightMeter, GameUiPlugin};

#[test]
fn meter_hides_only_when_full_and_off() {
    assert!(!FlashlightMeter::default().visible());
    assert!(FlashlightMeter {
        charge: 1.0,
        on: true
    }
    .visible());
    assert!(FlashlightMeter {
        charge: 0.8,
        on: false
    }
    .visible());
}

#[test]
fn charge_bar_tracks_meter() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let meter = app.world_mut().spawn(flashlight_meter()).id();
    app.update();
    app.world_mut().entity_mut(meter).insert(FlashlightMeter {
        charge: 0.25,
        on: false,
    });
    app.update();
    let world = app.world_mut();
    let root = world.get::<Node>(meter).unwrap();
    assert_eq!(root.flex_direction, FlexDirection::Row);
    assert_eq!(root.align_items, AlignItems::Center);
    let mut nodes = world.query::<(&Name, &Node)>();
    let fill = nodes
        .iter(world)
        .find(|(name, _)| name.as_str() == "Flashlight fill")
        .unwrap()
        .1;
    assert_eq!(fill.width, percent(25.0));
    let track = nodes
        .iter(world)
        .find(|(name, _)| name.as_str() == "Flashlight charge track")
        .unwrap()
        .1;
    assert_eq!(track.flex_grow, 1.0);
}
