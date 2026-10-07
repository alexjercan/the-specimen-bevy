use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

pub const MOVE_SPEED: f32 = 3.0;
pub const LOOK_SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = 1.54;
pub const LOOK_BUTTON: MouseButton = MouseButton::Right;
pub const RESUME_BUTTON: MouseButton = MouseButton::Left;
pub const RELEASE_KEY: KeyCode = KeyCode::Escape;
pub const HOLD_KEY: KeyCode = KeyCode::F12;
pub const UP_KEY: KeyCode = KeyCode::Space;
pub const DOWN_KEY: KeyCode = KeyCode::ShiftLeft;

#[derive(Component, Debug, Default, Clone, Copy)]
#[require(Transform)]
pub struct PlayerController;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ControllerState {
    #[default]
    Active,
    Released,
    Held,
}

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControllerState>().add_systems(
            Update,
            (
                (release, resume, toggle_hold).chain(),
                (look, fly).run_if(controller_active),
            )
                .chain(),
        );
    }
}

pub fn controller_active(state: Res<ControllerState>) -> bool {
    *state == ControllerState::Active
}

fn release(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<ControllerState>) {
    if keys.just_pressed(RELEASE_KEY) && *state == ControllerState::Active {
        *state = ControllerState::Released;
    }
}

fn resume(buttons: Res<ButtonInput<MouseButton>>, mut state: ResMut<ControllerState>) {
    if buttons.just_pressed(RESUME_BUTTON) && *state == ControllerState::Released {
        *state = ControllerState::Active;
    }
}

fn toggle_hold(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<ControllerState>) {
    if !keys.just_pressed(HOLD_KEY) {
        return;
    }
    *state = match *state {
        ControllerState::Held => ControllerState::Released,
        _ => ControllerState::Held,
    };
}

fn look(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut controllers: Query<&mut Transform, With<PlayerController>>,
) {
    if !buttons.pressed(LOOK_BUTTON) || motion.delta == Vec2::ZERO {
        return;
    }
    for mut transform in &mut controllers {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw - motion.delta.x * LOOK_SENSITIVITY;
        let pitch = (pitch - motion.delta.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

fn fly(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut controllers: Query<&mut Transform, With<PlayerController>>,
) {
    let axis = |positive, negative| {
        f32::from(u8::from(keys.pressed(positive))) - f32::from(u8::from(keys.pressed(negative)))
    };
    let input = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(UP_KEY, DOWN_KEY),
        axis(KeyCode::KeyW, KeyCode::KeyS),
    )
    .clamp_length_max(1.0);
    if input == Vec3::ZERO {
        return;
    }
    let step = MOVE_SPEED * time.delta_secs();
    for mut transform in &mut controllers {
        let (yaw, ..) = transform.rotation.to_euler(EulerRot::YXZ);
        let forward = transform.forward();
        let right = Quat::from_rotation_y(yaw) * Vec3::X;
        transform.translation += (forward * input.z + right * input.x + Vec3::Y * input.y) * step;
    }
}
