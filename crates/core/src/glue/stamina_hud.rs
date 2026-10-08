use bevy::prelude::*;
use game_ui::{stamina_meter, GameUiPlugin, StaminaMeter};
use gameplay::controller::{PlayerController, Stamina};

pub struct StaminaHudPlugin;

impl Plugin for StaminaHudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<GameUiPlugin>() {
            app.add_plugins(GameUiPlugin);
        }
        app.add_systems(Startup, spawn_hud).add_systems(Update, sync_hud);
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((stamina_meter(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_hud(
    players: Query<&Stamina, With<PlayerController>>,
    mut huds: Query<(&mut StaminaMeter, &mut Visibility)>,
) {
    let player = players.iter().next();
    for (mut meter, mut visibility) in &mut huds {
        let state = player.map_or(StaminaMeter::default(), |stamina| StaminaMeter {
            charge: stamina.charge,
            sprinting: stamina.sprinting,
        });
        visibility.set_if_neq(if player.is_some() && state.visible() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        meter.set_if_neq(state);
    }
}
