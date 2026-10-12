use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    audible::{Audible, Origin},
    player::{apply_input, Interact, PlayerController, PlayerControlsEnabled, PlayerInput},
};
use game_audio::Falloff;

use crate::facility::{box_hit, Module, Prop};

use crate::interaction::{InteractTarget, InteractTargets, StructuralSight};
use crate::monster::{catch_player, sees_player, Caught, Monster, WitnessedHiding};
use crate::{
    devices::{unseen, FlashTargets, Flashed},
    fuses::FuseInventory,
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Transform)]
pub enum HidingSpot {
    Locker,
    Table,
}

pub const HIDING_TRANSITION: f32 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidingCueKind {
    LockerOpen,
    LockerClose,
    TableEnter,
    TableLeave,
}

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct HidingCue {
    pub spot: Entity,
    pub kind: HidingCueKind,
}

impl Audible for HidingCue {
    fn name(&self) -> Option<&'static str> {
        Some(match self.kind {
            HidingCueKind::LockerOpen => "locker_open",
            HidingCueKind::LockerClose => "locker_close",
            HidingCueKind::TableEnter => "table_enter",
            HidingCueKind::TableLeave => "table_leave",
        })
    }

    fn origin(&self) -> Origin {
        Origin::From(self.spot)
    }

    fn falloff(&self) -> Falloff {
        Falloff::DEFAULT
    }
}

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
            .add_message::<HidingCue>()
            .add_observer(use_hiding_spots)
            .add_observer(attach_prop_hiding)
            .add_systems(
                Update,
                (toggle_hiding, animate_hiding)
                    .chain()
                    .after(catch_player)
                    .before(apply_input),
            );
    }
}

fn attach_prop_hiding(added: On<Add, Prop>, props: Query<&Prop>, mut commands: Commands) {
    let spot = match props.get(added.entity).map(|prop| prop.0) {
        Ok(Module::ConceptLocker) => HidingSpot::Locker,
        Ok(Module::ConceptTable) => HidingSpot::Table,
        _ => return,
    };
    commands.entity(added.entity).insert(spot);
}

impl Hidden {
    pub fn settled(&self) -> bool {
        self.phase == HidingPhase::Hidden
    }
}

impl HidingMotion {
    pub(crate) fn from(transform: &Transform) -> Self {
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

    fn cue(self, spot: Entity, entering: bool) -> HidingCue {
        let kind = match (self, entering) {
            (Self::Locker, true) => HidingCueKind::LockerOpen,
            (Self::Locker, false) => HidingCueKind::LockerClose,
            (Self::Table, true) => HidingCueKind::TableEnter,
            (Self::Table, false) => HidingCueKind::TableLeave,
        };
        HidingCue { spot, kind }
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
        (With<PlayerController>, Without<Caught>),
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
    mut players: Query<
        (
            &Transform,
            &mut PlayerInput,
            Option<&mut Hidden>,
            Option<&WitnessedHiding>,
            Option<&Flashed>,
            Option<&FlashTargets>,
        ),
        (With<PlayerController>, Without<Caught>),
    >,
    monsters: Query<(Entity, &Transform), With<Monster>>,
    sight: StructuralSight,
    mut cues: MessageWriter<HidingCue>,
    mut commands: Commands,
) {
    let mut occupied: Vec<Entity> = players
        .iter()
        .filter_map(|(_, _, hidden, _, _, _)| hidden.map(|hidden| hidden.spot))
        .collect();
    for &UseHidingSpot { player, spot } in uses.read() {
        let Ok(kind) = spots.get(spot) else {
            continue;
        };
        let Ok((transform, mut input, hidden, witnessed, flashed, targets)) =
            players.get_mut(player)
        else {
            continue;
        };
        let motion = HidingMotion::from(transform);
        match hidden {
            Some(mut hidden) if hidden.spot == spot => {
                if witnessed.is_some() {
                    continue;
                }
                let (phase, entering) = match hidden.phase {
                    HidingPhase::Leaving(_) => (HidingPhase::Entering(motion), true),
                    HidingPhase::Entering(_) | HidingPhase::Hidden => {
                        (HidingPhase::Leaving(motion), false)
                    }
                };
                hidden.phase = phase;
                cues.write(kind.cue(spot, entering));
            }
            None if !occupied.contains(&spot) => {
                let start = transform.translation.xz();
                let witness = monsters.iter().find_map(|(monster, pose)| {
                    (!unseen(flashed, targets, monster) && sees_player(pose, start, &sight))
                        .then_some(WitnessedHiding { monster })
                });
                commands.entity(player).insert(Hidden {
                    spot,
                    height: transform.translation.y,
                    phase: HidingPhase::Entering(motion),
                });
                if let Some(witness) = witness {
                    commands.entity(player).insert(witness);
                }
                *input = PlayerInput::default();
                occupied.push(spot);
                cues.write(kind.cue(spot, true));
            }
            _ => {}
        }
    }
}

fn animate_hiding(
    time: Res<Time>,
    spots: Query<(&HidingSpot, &Transform), Without<PlayerController>>,
    mut players: Query<
        (Entity, &mut Transform, &mut Hidden),
        (With<PlayerController>, Without<Caught>),
    >,
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
