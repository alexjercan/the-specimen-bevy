use bevy::prelude::*;
use game_ui::{
    fuse_part_paint, fuse_slots, theme, FuseIconPart, FuseSlot, FuseSlots, GameUiPlugin,
    FUSE_SLOT_COUNT,
};

fn slot_parts(app: &mut App, index: usize) -> Vec<(FuseIconPart, BackgroundColor)> {
    let slot = app
        .world_mut()
        .query::<(Entity, &FuseSlot)>()
        .iter(app.world())
        .find_map(|(entity, slot)| (slot.0 == index).then_some(entity))
        .unwrap();
    let mut descendants = Vec::new();
    let mut pending = vec![slot];
    while let Some(entity) = pending.pop() {
        if let Some(children) = app.world().get::<Children>(entity) {
            pending.extend(children.iter());
            descendants.extend(children.iter());
        }
    }
    descendants
        .into_iter()
        .filter_map(|entity| {
            let part = *app.world().get::<FuseIconPart>(entity)?;
            let background = *app.world().get::<BackgroundColor>(entity)?;
            Some((part, background))
        })
        .collect()
}

fn assert_slot(app: &mut App, index: usize, filled: bool) {
    let parts = slot_parts(app, index);
    assert_eq!(parts.len(), 6);
    for (part, background) in parts {
        assert_eq!(background.0, fuse_part_paint(part.filled, filled).0 .0);
    }
}

#[test]
fn fuse_slots_start_as_dark_silhouettes_and_fill_in_order() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(fuse_slots()).id();
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&FuseSlot>()
            .iter(app.world())
            .count(),
        FUSE_SLOT_COUNT
    );
    for index in 0..FUSE_SLOT_COUNT {
        assert_slot(&mut app, index, false);
    }
    assert_eq!(
        fuse_part_paint(theme::FUSE_CERAMIC, false).0 .0,
        theme::FUSE_EMPTY
    );

    app.world_mut().get_mut::<FuseSlots>(hud).unwrap().filled = 2;
    app.update();
    assert_slot(&mut app, 0, true);
    assert_slot(&mut app, 1, true);
    assert_slot(&mut app, 2, false);

    app.world_mut().get_mut::<FuseSlots>(hud).unwrap().filled = FUSE_SLOT_COUNT;
    app.update();
    for index in 0..FUSE_SLOT_COUNT {
        assert_slot(&mut app, index, true);
    }
}
