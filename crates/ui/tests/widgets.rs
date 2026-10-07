use bevy::prelude::*;
use game_ui::{button, button_paint, label, menu_button, GameUiPlugin};

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

#[test]
fn menu_buttons_repaint_on_interaction() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let action = app
        .world_mut()
        .spawn(menu_button("PLAY", Handle::default()))
        .id();
    app.update();
    let (idle, _) = button_paint(Interaction::None);
    assert_eq!(
        app.world().get::<BackgroundColor>(action).unwrap().0,
        idle.0
    );
    assert_eq!(app.world().get::<Children>(action).unwrap().len(), 1);
    app.world_mut()
        .entity_mut(action)
        .insert(Interaction::Hovered);
    app.update();
    let (hovered, border) = button_paint(Interaction::Hovered);
    assert_eq!(
        app.world().get::<BackgroundColor>(action).unwrap().0,
        hovered.0
    );
    assert_eq!(
        app.world().get::<BorderColor>(action).unwrap().top,
        border.top
    );
}
