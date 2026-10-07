use bevy::prelude::*;
use game_ui::{button, label};

#[test]
fn reusable_widgets_have_bevy_ui_components() {
    let mut app = App::new();
    let text = app.world_mut().spawn(label("TEST")).id();
    let action = app.world_mut().spawn(button()).id();
    assert_eq!(app.world().get::<Text>(text).unwrap().0, "TEST");
    assert!(app.world().get::<TextFont>(text).is_some());
    assert!(app.world().get::<Button>(action).is_some());
    assert!(app.world().get::<Node>(action).is_some());
}
