use std::time::Duration;

use bevy::prelude::*;
use game_settings::GameSettingsPlugin;
use gameplay::{
    facility::FacilityPowerPlugin,
    interaction::{InteractionAssets, InteractionViewPlugin},
};

mod common;

fn view_app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((
        FacilityPowerPlugin,
        GameSettingsPlugin { persist: false },
        InteractionViewPlugin,
    ));
    app.update();
    app
}

fn interaction_assets() -> InteractionAssets {
    InteractionAssets {
        font: default(),
        interact_glyph: default(),
    }
}

fn hint_nodes(app: &mut App) -> Vec<(Visibility, usize)> {
    let world = app.world_mut();
    let mut hints =
        world.query_filtered::<(&Visibility, &Children), (With<Node>, With<GlobalZIndex>)>();
    hints
        .iter(world)
        .map(|(visibility, children)| (*visibility, children.len()))
        .collect()
}

#[test]
fn hint_waits_for_interaction_assets() {
    let mut app = view_app();
    app.update();
    app.update();
    assert!(hint_nodes(&mut app).is_empty());
    app.insert_resource(interaction_assets());
    app.update();
    assert_eq!(hint_nodes(&mut app), vec![(Visibility::Hidden, 2)]);
    app.update();
    assert_eq!(hint_nodes(&mut app).len(), 1, "the hint spawns once");
}

#[test]
fn hint_label_starts_with_the_default_action() {
    let mut app = view_app();
    app.insert_resource(interaction_assets());
    app.update();
    let world = app.world_mut();
    let labels: Vec<String> = world
        .query::<&Text>()
        .iter(world)
        .map(|text| text.0.clone())
        .collect();
    assert_eq!(labels, vec!["OPEN".to_owned()]);
    assert_eq!(world.query::<&ImageNode>().iter(world).count(), 1);
}
