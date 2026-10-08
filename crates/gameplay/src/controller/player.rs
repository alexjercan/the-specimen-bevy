use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::*;

use crate::levels::{
    Door, DoorOf, DoorRef, DoorSwing, Doors, GameplaySound, Hidden, Passage, PropCollider, Room,
};

use super::{
    collision, flashlight,
    stamina::{SprintExhausted, Stamina},
};

pub const WALK_SPEED: f32 = 3.0;
pub const RUN_SPEED: f32 = 6.0;
pub const LOOK_SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = 1.54;

#[derive(Component, Default)]
#[require(Transform, PlayerInput, Stamina, Visibility)]
pub struct PlayerController;

#[derive(Resource)]
pub struct PlayerControlsEnabled(pub bool);

impl Default for PlayerControlsEnabled {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub running: bool,
    pub look: Vec2,
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

#[derive(InputAction)]
#[action_output(bool)]
struct Run;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Look;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Interact;

pub struct PlayerControllerPlugin {
    camera: bool,
}

impl Default for PlayerControllerPlugin {
    fn default() -> Self {
        Self { camera: true }
    }
}

impl PlayerControllerPlugin {
    pub fn without_camera(mut self) -> Self {
        self.camera = false;
        self
    }
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            app.is_plugin_added::<EnhancedInputPlugin>(),
            "PlayerControllerPlugin requires EnhancedInputPlugin"
        );
        app.init_resource::<PlayerControlsEnabled>()
            .add_message::<GameplaySound>()
            .add_message::<SprintExhausted>()
            .add_input_context::<PlayerController>()
            .add_observer(attach_input)
            .add_observer(flashlight::attach)
            .add_observer(flashlight::toggle)
            .add_observer(collision::attach_prop_collider)
            .add_observer(on_move)
            .add_observer(on_move_complete)
            .add_observer(on_run)
            .add_observer(on_run_complete)
            .add_observer(on_look)
            .add_systems(
                Update,
                (
                    apply_input,
                    flashlight::arm_after_release,
                    flashlight::advance,
                    flashlight::sync_beam,
                )
                    .chain(),
            );
        if self.camera {
            app.add_observer(attach_camera)
                .add_observer(flashlight::attach_beam)
                .add_systems(Update, update_cursor);
        }
    }
}

fn attach_input(added: On<Add, PlayerController>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert(actions!(PlayerController[
            (
                Action::<Move>::new(),
                Bindings::spawn((Cardinal::wasd_keys(),)),
            ),
            (
                Action::<Run>::new(),
                bindings![KeyCode::ShiftLeft],
            ),
            (
                Action::<Look>::new(),
                bindings![Binding::mouse_motion()],
            ),
            (
                Action::<Interact>::new(),
                bindings![KeyCode::KeyF],
            ),
            (
                Action::<flashlight::ToggleFlashlight>::new(),
                bindings![MouseButton::Left],
            ),
        ]));
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

fn on_move(fire: On<Fire<Move>>, mut players: Query<&mut PlayerInput, With<PlayerController>>) {
    for mut input in &mut players {
        input.movement = fire.value;
    }
}

fn on_move_complete(
    _: On<Complete<Move>>,
    mut players: Query<&mut PlayerInput, With<PlayerController>>,
) {
    for mut input in &mut players {
        input.movement = Vec2::ZERO;
    }
}

fn on_run(fire: On<Fire<Run>>, mut players: Query<&mut PlayerInput, With<PlayerController>>) {
    for mut input in &mut players {
        input.running = fire.value;
    }
}

fn on_run_complete(
    _: On<Complete<Run>>,
    mut players: Query<&mut PlayerInput, With<PlayerController>>,
) {
    for mut input in &mut players {
        input.running = false;
    }
}

fn on_look(fire: On<Fire<Look>>, mut players: Query<&mut PlayerInput, With<PlayerController>>) {
    for mut input in &mut players {
        input.look += fire.value;
    }
}

fn update_cursor(
    enabled: Res<PlayerControlsEnabled>,
    players: Query<(), With<PlayerController>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let capture = enabled.0 && !players.is_empty();
    for mut cursor in &mut cursors {
        cursor.visible = !capture;
        cursor.grab_mode = if capture {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

pub(crate) fn apply_input(
    time: Res<Time>,
    enabled: Res<PlayerControlsEnabled>,
    rooms: Query<(&Room, Option<&Doors>)>,
    links: Query<(&DoorRef, &DoorOf)>,
    doors: Query<(&Door, &DoorSwing)>,
    passages: Query<&Passage>,
    props: Query<(&PropCollider, &Transform), Without<PlayerController>>,
    mut players: Query<
        (&mut Transform, &mut PlayerInput, &mut Stamina, Option<&Hidden>),
        With<PlayerController>,
    >,
    mut exhaustion: MessageWriter<SprintExhausted>,
) {
    if !enabled.0 {
        for (_, mut input, mut stamina, _) in &mut players {
            *input = PlayerInput::default();
            stamina.sprinting = false;
        }
        return;
    }
    for (mut transform, mut input, mut stamina, hidden) in &mut players {
        if hidden.is_some_and(|hidden| !hidden.settled()) {
            *input = PlayerInput::default();
            stamina.advance(false, false, time.delta_secs());
            continue;
        }
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw - input.look.x * LOOK_SENSITIVITY;
        let pitch = (pitch - input.look.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        input.look = Vec2::ZERO;
        if hidden.is_some() {
            input.movement = Vec2::ZERO;
            input.running = false;
            stamina.advance(false, false, time.delta_secs());
            continue;
        }

        let movement = input.movement.clamp_length_max(1.0);
        if stamina.advance(input.running, movement != Vec2::ZERO, time.delta_secs()) {
            exhaustion.write(SprintExhausted {
                position: transform.translation,
            });
        }
        let direction = Quat::from_rotation_y(yaw) * Vec3::new(movement.x, 0.0, -movement.y);
        let speed = if stamina.sprinting { RUN_SPEED } else { WALK_SPEED };
        let delta = direction * speed * time.delta_secs();
        if delta != Vec3::ZERO {
            let obstacles = collision::colliders(&rooms, &links, &doors, &passages, &props);
            let start = transform.translation.xz();
            let next = collision::move_player(start, delta.xz(), &obstacles);
            transform.translation.x = next.x;
            transform.translation.z = next.y;
        }
    }
}
