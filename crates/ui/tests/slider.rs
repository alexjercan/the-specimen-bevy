use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderRange, SliderValue, TrackClick, ValueChange},
};
use game_ui::{slider, GameUiPlugin, SliderFill};

fn fill(app: &App, slider: Entity) -> Val {
    let child = app.world().get::<Children>(slider).unwrap()[0];
    assert!(app.world().get::<SliderFill>(child).is_some());
    app.world().get::<Node>(child).unwrap().width
}

#[test]
fn slider_commits_value_changes_and_moves_its_fill() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let entity = app.world_mut().spawn(slider(5.0, 0.0, 20.0, 1.0)).id();
    app.update();
    assert_eq!(
        app.world().get::<Slider>(entity).unwrap().track_click,
        TrackClick::Snap
    );
    assert_eq!(
        *app.world().get::<SliderRange>(entity).unwrap(),
        SliderRange::new(0.0, 20.0)
    );
    assert_eq!(fill(&app, entity), percent(25));

    app.world_mut().trigger(ValueChange::<f32> {
        source: entity,
        value: 15.0,
        is_final: false,
    });
    app.update();
    assert_eq!(app.world().get::<SliderValue>(entity).unwrap().0, 15.0);
    assert_eq!(fill(&app, entity), percent(75));

    app.world_mut().entity_mut(entity).insert(SliderValue(30.0));
    app.update();
    assert_eq!(fill(&app, entity), percent(100));
}
