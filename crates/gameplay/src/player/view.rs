use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use game_settings::GameSettings;

use super::{
    flashlight::Flashlight,
    flashlight_hud::{flashlight_meter, FlashlightHudPlugin, FlashlightMeter, FlashlightPrompt},
    logic::{PlayerController, PlayerControlsEnabled, PlayerSet},
    sounds::PlayerSoundsPlugin,
    stamina::Stamina,
    stamina_hud::{stamina_meter, StaminaHudPlugin, StaminaMeter},
};
use crate::ambience::{AmbientSource, AmbientSourceKind};

#[derive(Component)]
pub struct FlashlightBeam;

pub struct PlayerViewPlugin;

impl Plugin for PlayerViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerSoundsPlugin, StaminaHudPlugin, FlashlightHudPlugin))
            .add_observer(attach_audio)
            .add_observer(attach_camera)
            .add_observer(attach_beam)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    update_cursor,
                    sync_beam,
                    sync_stamina_hud,
                    sync_flashlight_hud,
                    sync_flashlight_prompt,
                )
                    .after(PlayerSet),
            );
    }
}

fn attach_audio(added: On<Add, PlayerController>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert(game_audio::spatial_listener(0.18))
        .with_children(|children| {
            children.spawn((
                AmbientSource {
                    kind: AmbientSourceKind::Roomtone,
                    volume: 0.12,
                },
                Transform::IDENTITY,
            ));
            children.spawn((
                AmbientSource {
                    kind: AmbientSourceKind::LowPressure,
                    volume: 0.04,
                },
                Transform::IDENTITY,
            ));
            children.spawn((
                AmbientSource {
                    kind: AmbientSourceKind::Conduit,
                    volume: 0.055,
                },
                Transform::IDENTITY,
            ));
        });
}

fn attach_camera(added: On<Add, PlayerController>, mut commands: Commands) {
    commands.entity(added.entity).insert((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 55.0_f32.to_radians(),
            ..default()
        }),
    ));
}

fn attach_beam(added: On<Add, PlayerController>, mut commands: Commands) {
    commands.entity(added.entity).with_children(|children| {
        children.spawn((
            FlashlightBeam,
            Name::new("Flashlight beam"),
            SpotLight {
                color: Color::srgb(0.95, 0.94, 0.83),
                intensity: 45_000.0,
                range: 19.0,
                radius: 0.13,
                inner_angle: 0.18,
                outer_angle: 0.47,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.08, -0.12, -0.15),
            Visibility::Hidden,
        ));
    });
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((stamina_meter(), Visibility::Hidden, GlobalZIndex(5)));
    commands.spawn((flashlight_meter(), Visibility::Hidden, GlobalZIndex(5)));
}

fn update_cursor(
    enabled: Res<PlayerControlsEnabled>,
    player: Option<Single<(), With<PlayerController>>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let capture = enabled.0 && player.is_some();
    cursor.visible = !capture;
    cursor.grab_mode = if capture {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

fn sync_beam(
    flashlight: Single<&Flashlight, (With<PlayerController>, Changed<Flashlight>)>,
    mut beam: Single<&mut Visibility, With<FlashlightBeam>>,
) {
    **beam = if flashlight.on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn sync_stamina_hud(
    player: Option<Single<&Stamina, With<PlayerController>>>,
    hud: Single<(&mut StaminaMeter, &mut Visibility)>,
) {
    let (mut meter, mut visibility) = hud.into_inner();
    let state = player
        .as_deref()
        .map_or(StaminaMeter::default(), |stamina| StaminaMeter {
            charge: stamina.charge,
            sprinting: stamina.sprinting(),
        });
    visibility.set_if_neq(if player.is_some() && state.visible() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    meter.set_if_neq(state);
}

fn sync_flashlight_hud(
    player: Option<Single<&Flashlight, With<PlayerController>>>,
    hud: Single<(&mut FlashlightMeter, &mut Visibility)>,
) {
    let (mut meter, mut visibility) = hud.into_inner();
    let state = player
        .as_deref()
        .map_or(FlashlightMeter::default(), |flashlight| FlashlightMeter {
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

fn sync_flashlight_prompt(
    settings: Res<GameSettings>,
    mut prompt: Single<&mut Text, With<FlashlightPrompt>>,
) {
    let value = format!("{} TOGGLE", settings.keys.flashlight.label());
    if prompt.0 != value {
        prompt.0 = value;
    }
}
