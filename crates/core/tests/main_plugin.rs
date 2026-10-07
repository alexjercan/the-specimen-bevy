use bevy::prelude::*;
use game_core::{AppBuilder, CoreState};
use gameplay::levels::Room;

#[derive(Resource)]
struct Installed;

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Installed);
    }
}

#[test]
fn headless_builder_enters_ready_and_builds_first_floor() {
    let mut app = AppBuilder::headless().build();
    assert_eq!(
        *app.world().resource::<State<CoreState>>().get(),
        CoreState::Ready
    );
    app.finish();
    app.cleanup();
    app.update();
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 18);
}

#[test]
fn custom_plugin_replaces_default_game_plugin() {
    let mut app = AppBuilder::headless().with_main_plugin(MainPlugin).build();
    assert!(app.world().contains_resource::<Installed>());
    app.finish();
    app.cleanup();
    app.update();
    let mut rooms = app.world_mut().query::<&Room>();
    assert_eq!(rooms.iter(app.world()).count(), 0);
}
