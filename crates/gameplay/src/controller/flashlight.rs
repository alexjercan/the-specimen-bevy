use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::player::{PlayerController, PlayerControlsEnabled};
use game_audio::{PlaySound, Sound};

pub const DRAIN_SECONDS: f32 = 15.0;
pub const RECHARGE_SECONDS: f32 = 10.0;
pub const RESTART_CHARGE: f32 = 0.1;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Flashlight {
    pub charge: f32,
    pub on: bool,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            charge: 1.0,
            on: false,
        }
    }
}

impl Flashlight {
    pub fn toggle(&mut self) {
        if self.on {
            self.on = false;
        } else if self.charge >= RESTART_CHARGE {
            self.on = true;
        }
    }

    pub fn advance(&mut self, seconds: f32) {
        if self.on {
            self.charge = (self.charge - seconds / DRAIN_SECONDS).max(0.0);
            if self.charge == 0.0 {
                self.on = false;
            }
        } else {
            self.charge = (self.charge + seconds / RECHARGE_SECONDS).min(1.0);
        }
    }
}

#[derive(Component)]
pub struct FlashlightBeam;

#[derive(Component)]
pub(super) struct WaitForFlashlightRelease;

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct ToggleFlashlight;

pub(super) fn attach(
    added: On<Add, PlayerController>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    let mut player = commands.entity(added.entity);
    player.insert(Flashlight::default());
    if buttons.pressed(MouseButton::Left) {
        player.insert(WaitForFlashlightRelease);
    }
}

pub(super) fn arm_after_release(
    buttons: Res<ButtonInput<MouseButton>>,
    waiting: Query<Entity, With<WaitForFlashlightRelease>>,
    mut commands: Commands,
) {
    if !buttons.pressed(MouseButton::Left) {
        for entity in &waiting {
            commands.entity(entity).remove::<WaitForFlashlightRelease>();
        }
    }
}

pub(super) fn attach_beam(added: On<Add, PlayerController>, mut commands: Commands) {
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

pub(super) fn toggle(
    _: On<Start<ToggleFlashlight>>,
    enabled: Res<PlayerControlsEnabled>,
    mut players: Query<
        &mut Flashlight,
        (With<PlayerController>, Without<WaitForFlashlightRelease>),
    >,
    mut sounds: MessageWriter<PlaySound>,
) {
    if !enabled.0 {
        return;
    }
    for mut flashlight in &mut players {
        let was_on = flashlight.on;
        flashlight.toggle();
        if flashlight.on != was_on {
            sounds.write(PlaySound {
                sound: Sound::FlashlightClick,
                position: None,
            });
        }
    }
}

pub(super) fn advance(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    mut players: Query<&mut Flashlight, With<PlayerController>>,
) {
    if !enabled.0 {
        return;
    }
    for mut flashlight in &mut players {
        flashlight.advance(time.delta_secs());
    }
}

pub(super) fn sync_beam(
    players: Query<(&Flashlight, &Children), (With<PlayerController>, Changed<Flashlight>)>,
    mut beams: Query<&mut Visibility, With<FlashlightBeam>>,
) {
    for (flashlight, children) in &players {
        for child in children.iter() {
            if let Ok(mut visibility) = beams.get_mut(child) {
                *visibility = if flashlight.on {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}
