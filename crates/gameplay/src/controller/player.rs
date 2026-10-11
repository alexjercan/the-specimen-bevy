use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::*;

use crate::levels::{
    AmbientSource, AmbientSourceKind, Caught, Door, DoorOf, DoorRef, DoorSwing, Doors, Hidden,
    Passage, PropCollider, Room,
};
use game_audio::{PlaySound, Sound};

use super::{
    collision, flashlight,
    settings::{ControllerSettings, KeyBindings},
    stamina::{SprintExhausted, Stamina},
};

pub const WALK_SPEED: f32 = 3.0;
pub const RUN_SPEED: f32 = 6.0;
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

#[derive(InputAction)]
#[action_output(bool)]
pub struct UseFlashbang;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<flashlight::Flashlight>()
            .register_type::<Stamina>()
            .init_resource::<PlayerControlsEnabled>()
            .add_message::<PlaySound>()
            .add_message::<SprintExhausted>()
            .add_input_context::<PlayerController>()
            .add_observer(attach_input)
            .add_systems(Update, refresh_input_bindings)
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
                (apply_input, flashlight::advance, flashlight::sync_beam).chain(),
            );
    }
}

pub struct PlayerPresentationPlugin;

impl Plugin for PlayerPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(attach_audio)
            .add_observer(attach_camera)
            .add_observer(flashlight::attach_beam)
            .add_systems(Update, update_cursor);
    }
}

fn input_actions(keys: KeyBindings) -> impl Bundle {
    actions!(PlayerController[
        (
            Action::<Move>::new(),
            Bindings::spawn((Cardinal::new(
                keys.forward,
                keys.left,
                keys.backward,
                keys.right,
            ),)),
        ),
        (Action::<Run>::new(), bindings![keys.sprint]),
        (Action::<Look>::new(), bindings![Binding::mouse_motion()]),
        (Action::<Interact>::new(), bindings![keys.interact]),
        (
            Action::<flashlight::ToggleFlashlight>::new(),
            ActionSettings {
                require_reset: true,
                ..default()
            },
            bindings![keys.flashlight],
        ),
        (Action::<UseFlashbang>::new(), bindings![keys.flashbang]),
    ])
}

fn attach_input(
    added: On<Add, PlayerController>,
    settings: Res<ControllerSettings>,
    mut commands: Commands,
) {
    commands
        .entity(added.entity)
        .insert(input_actions(settings.keys));
}

fn refresh_input_bindings(
    settings: Res<ControllerSettings>,
    players: Query<Entity, With<PlayerController>>,
    mut bound: Local<Option<KeyBindings>>,
    mut commands: Commands,
) {
    let previous = bound.replace(settings.keys);
    if previous.is_none_or(|previous| previous == settings.keys) {
        return;
    }
    for player in &players {
        commands
            .entity(player)
            .despawn_related::<Actions<PlayerController>>();
        commands.entity(player).insert(input_actions(settings.keys));
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
    settings: Res<ControllerSettings>,
    rooms: Query<(&Room, Option<&Doors>)>,
    links: Query<(&DoorRef, &DoorOf)>,
    doors: Query<(&Door, &DoorSwing)>,
    passages: Query<&Passage>,
    props: Query<(&PropCollider, &Transform), Without<PlayerController>>,
    mut players: Query<
        (
            &mut Transform,
            &mut PlayerInput,
            &mut Stamina,
            Option<&Hidden>,
            Option<&crate::levels::InstallingFuses>,
        ),
        (With<PlayerController>, Without<Caught>),
    >,
    mut exhaustion: MessageWriter<SprintExhausted>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if !enabled.0 {
        for (_, mut input, mut stamina, _, _) in &mut players {
            *input = PlayerInput::default();
            stamina.sprinting = false;
        }
        return;
    }
    for (mut transform, mut input, mut stamina, hidden, installing) in &mut players {
        if installing.is_some() {
            *input = PlayerInput::default();
            stamina.advance(false, false, time.delta_secs());
            continue;
        }
        if hidden.is_some_and(|hidden| !hidden.settled()) {
            *input = PlayerInput::default();
            stamina.advance(false, false, time.delta_secs());
            continue;
        }
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let sensitivity = settings.mouse_sensitivity;
        let yaw = yaw - input.look.x * sensitivity;
        let pitch = (pitch - input.look.y * sensitivity).clamp(-PITCH_LIMIT, PITCH_LIMIT);
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
            sounds.write(PlaySound::global(Sound::SprintExhausted));
        }
        let direction = Quat::from_rotation_y(yaw) * Vec3::new(movement.x, 0.0, -movement.y);
        let speed = if stamina.sprinting {
            RUN_SPEED
        } else {
            WALK_SPEED
        };
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
