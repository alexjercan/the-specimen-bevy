use bevy::prelude::*;
use game_ui::{flashlight_meter, stamina_meter, GameUiPlugin, StaminaMeter};

#[test]
fn meter_hides_at_full_charge_when_idle() {
    assert!(!StaminaMeter::default().visible());
    assert!(StaminaMeter {
        charge: 1.0,
        sprinting: true,
    }
    .visible());
    assert!(StaminaMeter {
        charge: 0.5,
        sprinting: false,
    }
    .visible());
}

#[test]
fn charge_bar_tracks_stamina_below_flashlight() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let flashlight = app.world_mut().spawn(flashlight_meter()).id();
    let meter = app.world_mut().spawn(stamina_meter()).id();
    app.update();
    app.world_mut().entity_mut(meter).insert(StaminaMeter {
        charge: 0.25,
        sprinting: false,
    });
    app.update();
    let world = app.world_mut();
    let root = world.get::<Node>(meter).unwrap();
    assert_eq!(root.bottom, px(24));
    assert_eq!(root.left, px(24));
    assert_eq!(root.width, px(184));
    let flashlight_node = world.get::<Node>(flashlight).unwrap();
    assert_eq!(flashlight_node.bottom, px(80));
    assert_eq!(flashlight_node.left, root.left);
    assert_eq!(flashlight_node.width, root.width);
    assert_eq!(root.flex_direction, FlexDirection::Row);
    assert_eq!(root.align_items, AlignItems::Center);
    let mut nodes = world.query::<(&Name, &Node)>();
    let fill = nodes
        .iter(world)
        .find(|(name, _)| name.as_str() == "Sprint fill")
        .unwrap()
        .1;
    assert_eq!(fill.width, percent(25.0));
}
