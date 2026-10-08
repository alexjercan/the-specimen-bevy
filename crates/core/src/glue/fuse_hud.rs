use bevy::prelude::*;
use game_ui::{fuse_slots, FuseSlots, GameUiPlugin, FUSE_SLOT_COUNT};
use gameplay::{
    controller::PlayerController,
    levels::{FuseInventory, FUSE_COUNT},
};

const _: () = assert!(FUSE_SLOT_COUNT == FUSE_COUNT);

pub struct FuseHudPlugin;

impl Plugin for FuseHudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<GameUiPlugin>() {
            app.add_plugins(GameUiPlugin);
        }
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, sync_hud);
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((fuse_slots(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_hud(
    players: Query<&FuseInventory, With<PlayerController>>,
    mut huds: Query<(&mut FuseSlots, &mut Visibility)>,
) {
    let inventory = players.iter().next();
    for (mut slots, mut visibility) in &mut huds {
        visibility.set_if_neq(if inventory.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        slots.set_if_neq(FuseSlots {
            filled: inventory.map_or(0, |inventory| inventory.0),
        });
    }
}
