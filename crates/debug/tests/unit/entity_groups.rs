use super::*;
use gameplay::controller::{Flashlight, Stamina};

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

#[test]
fn player_cheats_render_with_and_without_optional_devices() {
    let mut world = World::new();
    let player = world
        .spawn((
            PlayerController,
            FuseInventory(2),
            Flashlight::default(),
            Stamina::default(),
        ))
        .id();
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
        player_tools_ui(&mut world, player, ui);
    });
    world
        .entity_mut(player)
        .insert((Flashbangs(3), Detector::default()));
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
        player_tools_ui(&mut world, player, ui);
    });
    assert_eq!(world.get::<Flashbangs>(player).unwrap().0, 3);
    assert!(world.get::<Detector>(player).is_some());
}
