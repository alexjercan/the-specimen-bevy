use super::*;

#[test]
fn new_animation_player_and_clip_switch_initialize_without_query_conflicts() {
    let mut app = App::new();
    let (_graph, nodes) = AnimationGraph::from_clips(
        (0..CLIP_NAMES.len()).map(|_| Handle::<AnimationClip>::default()),
    );
    app.insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(DeerScene {
            scene: Handle::default(),
            graph: Handle::default(),
            nodes,
            selected: 0,
        })
        .add_systems(Update, play_animation);
    let player = app.world_mut().spawn(AnimationPlayer::default()).id();
    let label = app
        .world_mut()
        .spawn((Text::new("IDLE"), AnimationLabel))
        .id();

    app.update();
    assert!(app
        .world()
        .entity(player)
        .contains::<AnimationGraphHandle>());
    assert_eq!(app.world().resource::<DeerScene>().selected, 0);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowRight);
    app.update();
    assert_eq!(app.world().resource::<DeerScene>().selected, 1);
    assert_eq!(app.world().entity(label).get::<Text>().unwrap().0, "WALK");
}
