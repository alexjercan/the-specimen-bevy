use bevy::prelude::*;
use game_settings::GameSettings;
use game_ui::{
    detector_readout, flash_alpha, flash_overlay, flashbang_status, DetectorReadout,
    DetectorSignal, FlashOverlay, FlashbangPrompt, FlashbangStatus, GameUiPlugin,
};
use gameplay::{
    controller::PlayerController,
    levels::{Detector, Flashbangs, Flashed, DETECTOR_RANGE, FLASHBANG_BURST_DELAY},
};

pub struct DeviceHudPlugin;

impl Plugin for DeviceHudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<GameUiPlugin>() {
            app.add_plugins(GameUiPlugin);
        }
        app.add_systems(Startup, spawn_hud).add_systems(
            Update,
            (
                sync_flashbang_status,
                sync_flashbang_prompt,
                sync_detector_readout,
                sync_flash_overlay,
            ),
        );
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((flashbang_status(), Visibility::Hidden, GlobalZIndex(5)));
    commands.spawn((detector_readout(), Visibility::Hidden, GlobalZIndex(5)));
    commands.spawn((flash_overlay(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_flashbang_status(
    players: Query<Option<&Flashbangs>, With<PlayerController>>,
    mut huds: Query<(&mut FlashbangStatus, &mut Visibility)>,
) {
    let player = players.iter().next();
    let flashbangs = player.flatten();
    let state = FlashbangStatus {
        count: flashbangs.map_or(0, |flashbangs| flashbangs.0),
    };
    for (mut status, mut visibility) in &mut huds {
        visibility.set_if_neq(if state.count > 0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        status.set_if_neq(state);
    }
}

fn sync_flashbang_prompt(
    settings: Option<Res<GameSettings>>,
    mut prompts: Query<&mut Text, With<FlashbangPrompt>>,
) {
    let binding = settings
        .as_ref()
        .map_or("MouseRight", |settings| settings.keys.flashbang.as_str());
    let value = format!("{} THROW", super::hud_binding_label(binding));
    for mut prompt in &mut prompts {
        if prompt.0 != value {
            prompt.0 = value.clone();
        }
    }
}

fn sync_detector_readout(
    players: Query<Option<&Detector>, With<PlayerController>>,
    mut huds: Query<(&mut DetectorReadout, &mut Visibility)>,
) {
    let detector = players.iter().next().flatten();
    let signal = detector
        .and_then(|detector| detector.reading)
        .map(|reading| DetectorSignal {
            distance: reading.distance,
            bearing: reading.bearing,
            range: DETECTOR_RANGE,
        });
    for (mut readout, mut visibility) in &mut huds {
        visibility.set_if_neq(if detector.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        readout.set_if_neq(DetectorReadout { signal });
    }
}

fn sync_flash_overlay(
    players: Query<Option<&Flashed>, With<PlayerController>>,
    mut huds: Query<(&mut FlashOverlay, &mut Visibility)>,
) {
    let flashed = players.iter().next().flatten();
    let elapsed = flashed.map_or(0.0, |flashed| flashed.elapsed());
    let burst = elapsed - FLASHBANG_BURST_DELAY;
    for (mut overlay, mut visibility) in &mut huds {
        visibility.set_if_neq(
            if flashed.is_some() && burst >= 0.0 && flash_alpha(burst) > 0.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        );
        overlay.set_if_neq(FlashOverlay {
            elapsed: burst.max(0.0),
        });
    }
}

#[cfg(test)]
#[path = "../../tests/unit/device_hud.rs"]
mod tests;
