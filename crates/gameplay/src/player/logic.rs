use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::audible::{Audible, Origin};
use crate::{
    facility::{Door, DoorOf, DoorRef, DoorSwing, Doors, Passage, PropCollider, Room},
    hiding::Hidden,
    monster::Caught,
};
use game_audio::Falloff;

use super::{
    collision, flashlight, footsteps,
    settings::{ControllerSettings, Input, KeyBindings},
    stamina::{SprintExhausted, Stamina},
};

pub const WALK_SPEED: f32 = 3.0;
pub const RUN_SPEED: f32 = 6.0;
pub const PITCH_LIMIT: f32 = 1.54;

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct PlayerSet;

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerCue {
    Step(usize),
    SprintExhausted,
    FlashlightClick,
}

impl Audible for PlayerCue {
    fn name(&self) -> Option<&'static str> {
        match self {
            Self::Step(_) => None,
            Self::SprintExhausted => Some("sprint_exhausted"),
            Self::FlashlightClick => Some("flashlight_click"),
        }
    }

    fn origin(&self) -> Origin {
        Origin::Listener
    }

    fn falloff(&self) -> Falloff {
        Falloff::DEFAULT
    }
}

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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<flashlight::Flashlight>()
            .register_type::<Stamina>()
            .init_resource::<PlayerControlsEnabled>()
            .add_message::<SprintExhausted>()
            .add_message::<PlayerCue>()
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
                (apply_input, flashlight::advance, footsteps::footsteps)
                    .chain()
                    .in_set(PlayerSet),
            );
    }
}

fn binding(input: Input) -> Binding {
    match input {
        Input::Key(key) => key.into(),
        Input::Mouse(button) => button.into(),
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
            bindings![binding(keys.flashlight)],
        ),
        (Action::<UseFlashbang>::new(), bindings![binding(keys.flashbang)]),
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
            Option<&crate::fuses::InstallingFuses>,
        ),
        (With<PlayerController>, Without<Caught>),
    >,
    mut exhaustion: MessageWriter<SprintExhausted>,
    mut cues: MessageWriter<PlayerCue>,
) {
    if !enabled.0 {
        for (_, mut input, mut stamina, _, _) in &mut players {
            *input = PlayerInput::default();
            stamina.stop_sprinting();
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
        let vertical_sensitivity = if settings.invert_y {
            -sensitivity
        } else {
            sensitivity
        };
        let yaw = yaw - input.look.x * sensitivity;
        let pitch = (pitch - input.look.y * vertical_sensitivity).clamp(-PITCH_LIMIT, PITCH_LIMIT);
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
            cues.write(PlayerCue::SprintExhausted);
        }
        let direction = Quat::from_rotation_y(yaw) * Vec3::new(movement.x, 0.0, -movement.y);
        let speed = if stamina.sprinting() {
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
