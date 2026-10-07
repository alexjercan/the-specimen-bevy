use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub const WALK_SPEED: f32 = 3.0;
pub const RUN_SPEED: f32 = 6.0;
pub const LOOK_SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = 1.54;

#[derive(Component, Default)]
#[require(Transform, PlayerInput)]
pub struct PlayerController;

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
        app.add_input_context::<PlayerController>()
            .add_observer(attach_input)
            .add_observer(on_move)
            .add_observer(on_move_complete)
            .add_observer(on_run)
            .add_observer(on_run_complete)
            .add_observer(on_look)
            .add_systems(Update, apply_input);
        if self.camera {
            app.add_observer(attach_camera);
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

fn apply_input(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &mut PlayerInput), With<PlayerController>>,
) {
    for (mut transform, mut input) in &mut players {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw - input.look.x * LOOK_SENSITIVITY;
        let pitch = (pitch - input.look.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        input.look = Vec2::ZERO;

        let movement = input.movement.clamp_length_max(1.0);
        let direction = Quat::from_rotation_y(yaw) * Vec3::new(movement.x, 0.0, -movement.y);
        let speed = if input.running { RUN_SPEED } else { WALK_SPEED };
        transform.translation += direction * speed * time.delta_secs();
    }
}
