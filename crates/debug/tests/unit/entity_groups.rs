use super::*;

#[test]
fn debug_groups_identify_players_and_panels_without_changing_hierarchy() {
    let mut world = World::new();
    let player = world.spawn(PlayerController).id();
    let panel = world.spawn(FusePanel::default()).id();
    let child = world
        .spawn((ChildOf(panel), Name::new("Panel visual")))
        .id();

    assert_eq!(ENTITY_GROUPS[entity_group(&world, player)], "Players");
    assert_eq!(ENTITY_GROUPS[entity_group(&world, panel)], "Fuse panels");
    assert_eq!(world.get::<ChildOf>(child).unwrap().parent(), panel);
}
