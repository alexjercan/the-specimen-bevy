use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub const MOVE_SPEED: f32 = 3.0;
pub const LOOK_SENSITIVITY: f32 = 0.002;
pub const PITCH_LIMIT: f32 = 1.54;
pub const LOCK_BUTTON: MouseButton = MouseButton::Left;
pub const RELEASE_KEY: KeyCode = KeyCode::Escape;
pub const HOLD_KEY: KeyCode = KeyCode::F12;

#[derive(Component, Debug, Default, Clone, Copy)]
#[require(Transform)]
pub struct PlayerController;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CursorHold(pub bool);

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorHold>().add_systems(
            Update,
            (
                (lock_cursor, release_cursor, toggle_hold).chain(),
                (look, walk).chain().run_if(cursor_locked),
            )
                .chain(),
        );
    }
}

pub fn cursor_locked(cursors: Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursors
        .single()
        .is_ok_and(|cursor| cursor.grab_mode != CursorGrabMode::None)
}

fn set_locked(mut cursor: Mut<CursorOptions>, locked: bool) {
    let grab_mode = if locked {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != grab_mode || cursor.visible == locked {
        cursor.grab_mode = grab_mode;
        cursor.visible = !locked;
    }
}

fn lock_cursor(
    buttons: Res<ButtonInput<MouseButton>>,
    hold: Res<CursorHold>,
    controllers: Query<(), With<PlayerController>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !buttons.just_pressed(LOCK_BUTTON) || hold.0 || controllers.is_empty() {
        return;
    }
    if let Ok(cursor) = cursors.single_mut() {
        set_locked(cursor, true);
    }
}

fn release_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(RELEASE_KEY) {
        return;
    }
    if let Ok(cursor) = cursors.single_mut() {
        set_locked(cursor, false);
    }
}

fn toggle_hold(
    keys: Res<ButtonInput<KeyCode>>,
    mut hold: ResMut<CursorHold>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(HOLD_KEY) {
        return;
    }
    hold.0 = !hold.0;
    if !hold.0 {
        return;
    }
    if let Ok(cursor) = cursors.single_mut() {
        set_locked(cursor, false);
    }
}

fn look(
    motion: Res<AccumulatedMouseMotion>,
    mut controllers: Query<&mut Transform, With<PlayerController>>,
) {
    if motion.delta == Vec2::ZERO {
        return;
    }
    for mut transform in &mut controllers {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw - motion.delta.x * LOOK_SENSITIVITY;
        let pitch = (pitch - motion.delta.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

fn walk(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut controllers: Query<&mut Transform, With<PlayerController>>,
) {
    let axis = |positive, negative| {
        f32::from(u8::from(keys.pressed(positive))) - f32::from(u8::from(keys.pressed(negative)))
    };
    let input = Vec2::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(KeyCode::KeyW, KeyCode::KeyS),
    );
    if input == Vec2::ZERO {
        return;
    }
    for mut transform in &mut controllers {
        let (yaw, ..) = transform.rotation.to_euler(EulerRot::YXZ);
        let heading = Quat::from_rotation_y(yaw);
        let direction = heading * Vec3::new(input.x, 0.0, -input.y);
        transform.translation += direction.normalize_or_zero() * MOVE_SPEED * time.delta_secs();
    }
}
