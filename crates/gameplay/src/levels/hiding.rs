use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use game_audio::{PlaySourceSound, Sound, SourceSounds};

use crate::controller::player::{
    apply_input, Interact, PlayerController, PlayerControlsEnabled, PlayerInput,
};

use super::{
    builder::Prop,
    doors::box_hit,
    fuses::FuseInventory,
    interaction::{InteractTarget, InteractTargets},
    module_names::{CONCEPT_LOCKER, CONCEPT_TABLE},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Transform)]
pub enum HidingSpot {
    Locker,
    Table,
}

pub const HIDING_TRANSITION: f32 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HidingMotion {
    pub from: Vec3,
    pub rotation: Quat,
    pub progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HidingPhase {
    Entering(HidingMotion),
    Hidden,
    Leaving(HidingMotion),
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Hidden {
    pub spot: Entity,
    pub height: f32,
    pub phase: HidingPhase,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseHidingSpot {
    pub player: Entity,
    pub spot: Entity,
}

pub struct HidingPlugin;

impl Plugin for HidingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UseHidingSpot>()
            .add_message::<PlaySourceSound>()
            .add_observer(use_hiding_spots)
            .add_observer(attach_prop_hiding)
            .add_observer(attach_hiding_sounds)
            .add_systems(
                Update,
                (toggle_hiding, animate_hiding).chain().before(apply_input),
            );
    }
}

fn attach_prop_hiding(added: On<Add, Prop>, props: Query<&Prop>, mut commands: Commands) {
    let spot = match props.get(added.entity).map(|prop| prop.0.as_str()) {
        Ok(CONCEPT_LOCKER) => HidingSpot::Locker,
        Ok(CONCEPT_TABLE) => HidingSpot::Table,
        _ => return,
    };
    commands.entity(added.entity).insert(spot);
}

fn attach_hiding_sounds(
    added: On<Add, HidingSpot>,
    spots: Query<&HidingSpot>,
    mut commands: Commands,
) {
    let Ok(spot) = spots.get(added.entity) else {
        return;
    };
    let sounds = match spot {
        HidingSpot::Locker => SourceSounds(vec![
            (Sound::LockerOpen, Vec3::Y),
            (Sound::LockerClose, Vec3::Y),
        ]),
        HidingSpot::Table => SourceSounds(vec![
            (Sound::TableEnter, Vec3::Y * 0.4),
            (Sound::TableLeave, Vec3::Y * 0.4),
        ]),
    };
    commands.entity(added.entity).insert(sounds);
}

impl Hidden {
    pub fn settled(&self) -> bool {
        self.phase == HidingPhase::Hidden
    }
}

impl HidingMotion {
    fn from(transform: &Transform) -> Self {
        Self {
            from: transform.translation,
            rotation: transform.rotation,
            progress: 0.0,
        }
    }

    fn position(&self, waypoint: Vec3, target: Vec3) -> Vec3 {
        let first = self.from.distance(waypoint);
        let second = waypoint.distance(target);
        let distance = (first + second) * ease(self.progress);
        if distance < first {
            self.from.lerp(waypoint, distance / first)
        } else if second > f32::EPSILON {
            waypoint.lerp(target, ((distance - first) / second).min(1.0))
        } else {
            target
        }
    }
}

fn ease(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

impl HidingSpot {
    fn bounds(self) -> (Vec3, Vec3) {
        match self {
            Self::Locker => (Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 1.95, 0.3)),
            Self::Table => (Vec3::new(-0.9, 0.0, -0.45), Vec3::new(0.9, 0.8, 0.45)),
        }
    }

    pub fn eye(self, transform: &Transform) -> Vec3 {
        let local = match self {
            Self::Locker => Vec3::new(0.0, 1.45, 0.0),
            Self::Table => Vec3::new(0.0, 0.45, 0.0),
        };
        transform.transform_point(local)
    }

    pub fn exit(self, transform: &Transform) -> Vec2 {
        let local = match self {
            Self::Locker => Vec3::new(0.0, 0.0, -0.95),
            Self::Table => Vec3::new(0.0, 0.0, -1.1),
        };
        transform.transform_point(local).xz()
    }

    pub fn facing(transform: &Transform) -> Quat {
        let (yaw, ..) = transform.rotation.to_euler(EulerRot::YXZ);
        Quat::from_rotation_y(yaw)
    }

    fn sound(self, entering: bool) -> Sound {
        match (self, entering) {
            (Self::Locker, true) => Sound::LockerOpen,
            (Self::Locker, false) => Sound::LockerClose,
            (Self::Table, true) => Sound::TableEnter,
            (Self::Table, false) => Sound::TableLeave,
        }
    }

    pub(crate) fn anchor(self, transform: &Transform) -> Vec3 {
        let (min, max) = self.bounds();
        transform.transform_point((min + max) / 2.0)
    }

    pub(crate) fn hit(self, transform: &Transform, origin: Vec3, forward: Vec3) -> Option<f32> {
        let inverse = transform.compute_affine().inverse();
        let (min, max) = self.bounds();
        box_hit(
            inverse.transform_point3(origin),
            inverse.transform_vector3(forward),
            min,
            max,
        )
    }
}

fn use_hiding_spots(
    _: On<Start<Interact>>,
    enabled: Res<PlayerControlsEnabled>,
    targets: InteractTargets,
    players: Query<
        (Entity, &Transform, Option<&FuseInventory>, Option<&Hidden>),
        With<PlayerController>,
    >,
    mut uses: MessageWriter<UseHidingSpot>,
) {
    if !enabled.0 {
        return;
    }
    for (player, transform, inventory, hidden) in &players {
        if let Some(InteractTarget::Hide(spot) | InteractTarget::Leave(spot)) =
            targets.aimed(transform, inventory, hidden)
        {
            uses.write(UseHidingSpot { player, spot });
        }
    }
}

fn toggle_hiding(
    mut uses: MessageReader<UseHidingSpot>,
    spots: Query<&HidingSpot, Without<PlayerController>>,
    mut players: Query<(&Transform, &mut PlayerInput, Option<&mut Hidden>), With<PlayerController>>,
    mut sounds: MessageWriter<PlaySourceSound>,
    mut commands: Commands,
) {
    let mut occupied: Vec<Entity> = players
        .iter()
        .filter_map(|(_, _, hidden)| hidden.map(|hidden| hidden.spot))
        .collect();
    for &UseHidingSpot { player, spot } in uses.read() {
        let Ok(kind) = spots.get(spot) else {
            continue;
        };
        let Ok((transform, mut input, hidden)) = players.get_mut(player) else {
            continue;
        };
        let motion = HidingMotion::from(transform);
        match hidden {
            Some(mut hidden) if hidden.spot == spot => {
                let (phase, entering) = match hidden.phase {
                    HidingPhase::Leaving(_) => (HidingPhase::Entering(motion), true),
                    HidingPhase::Entering(_) | HidingPhase::Hidden => {
                        (HidingPhase::Leaving(motion), false)
                    }
                };
                hidden.phase = phase;
                sounds.write(PlaySourceSound {
                    source: spot,
                    sound: kind.sound(entering),
                });
            }
            None if !occupied.contains(&spot) => {
                commands.entity(player).insert(Hidden {
                    spot,
                    height: transform.translation.y,
                    phase: HidingPhase::Entering(motion),
                });
                *input = PlayerInput::default();
                occupied.push(spot);
                sounds.write(PlaySourceSound {
                    source: spot,
                    sound: kind.sound(true),
                });
            }
            _ => {}
        }
    }
}

fn animate_hiding(
    time: Res<Time>,
    spots: Query<(&HidingSpot, &Transform), Without<PlayerController>>,
    mut players: Query<(Entity, &mut Transform, &mut Hidden), With<PlayerController>>,
    mut commands: Commands,
) {
    let step = time.delta_secs() / HIDING_TRANSITION;
    for (player, mut transform, mut hidden) in &mut players {
        let Ok((kind, place)) = spots.get(hidden.spot) else {
            transform.translation.y = hidden.height;
            commands.entity(player).remove::<Hidden>();
            continue;
        };
        let eye = kind.eye(place);
        let exit = kind.exit(place);
        let waypoint = Vec3::new(exit.x, eye.y, exit.y);
        let height = hidden.height;
        let (motion, target, leaving) = match &mut hidden.phase {
            HidingPhase::Hidden => continue,
            HidingPhase::Entering(motion) => (motion, eye, false),
            HidingPhase::Leaving(motion) => (motion, Vec3::new(exit.x, height, exit.y), true),
        };
        motion.progress = (motion.progress + step).min(1.0);
        transform.translation = motion.position(waypoint, target);
        transform.rotation = motion
            .rotation
            .slerp(HidingSpot::facing(place), ease(motion.progress));
        if motion.progress < 1.0 {
            continue;
        }
        transform.translation = target;
        if leaving {
            commands.entity(player).remove::<Hidden>();
        } else {
            hidden.phase = HidingPhase::Hidden;
        }
    }
}
