use bevy::prelude::*;
use game_settings::GameSettings;
use game_ui::{flashlight_meter, FlashlightMeter, FlashlightPrompt};
use gameplay::controller::{Flashlight, PlayerController};

pub struct FlashlightHudPlugin;

impl Plugin for FlashlightHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (sync_hud, sync_prompt));
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((flashlight_meter(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_hud(
    players: Query<&Flashlight, With<PlayerController>>,
    mut huds: Query<(&mut FlashlightMeter, &mut Visibility)>,
) {
    let player = players.iter().next();
    for (mut meter, mut visibility) in &mut huds {
        let state = player.map_or(FlashlightMeter::default(), |flashlight| FlashlightMeter {
            charge: flashlight.charge,
            on: flashlight.on,
        });
        visibility.set_if_neq(if player.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        meter.set_if_neq(state);
    }
}

fn sync_prompt(settings: Res<GameSettings>, mut prompts: Query<&mut Text, With<FlashlightPrompt>>) {
    let value = format!("{} TOGGLE", settings.keys.flashlight.label());
    for mut prompt in &mut prompts {
        if prompt.0 != value {
            prompt.0 = value.clone();
        }
    }
}
