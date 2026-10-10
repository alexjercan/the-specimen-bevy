use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    controller::PlayerController,
    levels::{Caught, Escaped, FacilityPower},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Achievement {
    EscapeWithoutFlashbang,
    EscapeWithoutDetector,
    FlashbangHitMonster,
    EscapeWithoutBoiler,
    RestoreBoiler,
    CaughtAfterExitOpen,
    EscapeUndetected,
}

impl Achievement {
    pub const ALL: [Self; 7] = [
        Self::EscapeWithoutFlashbang,
        Self::EscapeWithoutDetector,
        Self::FlashbangHitMonster,
        Self::EscapeWithoutBoiler,
        Self::RestoreBoiler,
        Self::CaughtAfterExitOpen,
        Self::EscapeUndetected,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::EscapeWithoutFlashbang => "ESCAPE_WITHOUT_FLASHBANG",
            Self::EscapeWithoutDetector => "ESCAPE_WITHOUT_DETECTOR",
            Self::FlashbangHitMonster => "FLASHBANG_HIT_MONSTER",
            Self::EscapeWithoutBoiler => "ESCAPE_WITHOUT_BOILER",
            Self::RestoreBoiler => "RESTORE_BOILER",
            Self::CaughtAfterExitOpen => "CAUGHT_AFTER_EXIT_OPEN",
            Self::EscapeUndetected => "ESCAPE_UNDETECTED",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|achievement| achievement.id() == id)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::EscapeWithoutFlashbang => "Escape empty-handed",
            Self::EscapeWithoutDetector => "Trust your ears",
            Self::FlashbangHitMonster => "Buy some time",
            Self::EscapeWithoutBoiler => "In the dark",
            Self::RestoreBoiler => "Let there be light",
            Self::CaughtAfterExitOpen => "So close",
            Self::EscapeUndetected => "Unseen",
        }
    }

    pub const fn criteria(self) -> &'static str {
        match self {
            Self::EscapeWithoutFlashbang => "Escape without picking up a flashbang during the run.",
            Self::EscapeWithoutDetector => "Escape without picking up the detector during the run.",
            Self::FlashbangHitMonster => "Land a flashbang burst on the monster.",
            Self::EscapeWithoutBoiler => {
                "Escape while the power is out, without restoring the boiler."
            }
            Self::RestoreBoiler => "Restore power at the boiler.",
            Self::CaughtAfterExitOpen => {
                "Open the unlocked exit door, then get caught before you leave the facility."
            }
            Self::EscapeUndetected => {
                "Escape without the monster ever detecting you during the run."
            }
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct AchievementProgress {
    pub unlocked: HashSet<Achievement>,
}

#[derive(Event, Clone, Copy, Debug, Eq, PartialEq)]
pub struct AchievementUnlocked(pub Achievement);

#[derive(Component, Default, Debug)]
pub struct RunAchievements {
    pub picked_flashbang: bool,
    pub picked_detector: bool,
    pub restored_boiler: bool,
    pub exit_opened: bool,
    pub detected: bool,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct AchievementSignal {
    pub player: Entity,
    pub kind: AchievementSignalKind,
}

#[derive(Clone, Copy, Debug)]
pub enum AchievementSignalKind {
    PickedFlashbang,
    PickedDetector,
    RestoredBoiler,
    ExitOpened,
    Detected,
    FlashbangHitMonster,
}

pub struct AchievementPlugin;

impl Plugin for AchievementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AchievementProgress>()
            .add_observer(attach_run)
            .add_observer(record_signal)
            .add_observer(record_escape)
            .add_observer(record_caught);
    }
}

fn attach_run(added: On<Add, PlayerController>, mut commands: Commands) {
    commands
        .entity(added.entity)
        .insert(RunAchievements::default());
}

fn record_signal(
    signal: On<AchievementSignal>,
    mut runs: Query<&mut RunAchievements, With<PlayerController>>,
    mut progress: ResMut<AchievementProgress>,
    mut commands: Commands,
) {
    let Ok(mut run) = runs.get_mut(signal.player) else {
        return;
    };
    match signal.kind {
        AchievementSignalKind::PickedFlashbang => run.picked_flashbang = true,
        AchievementSignalKind::PickedDetector => run.picked_detector = true,
        AchievementSignalKind::RestoredBoiler => {
            run.restored_boiler = true;
            unlock(&mut progress, &mut commands, Achievement::RestoreBoiler);
        }
        AchievementSignalKind::ExitOpened => run.exit_opened = true,
        AchievementSignalKind::Detected => run.detected = true,
        AchievementSignalKind::FlashbangHitMonster => {
            unlock(
                &mut progress,
                &mut commands,
                Achievement::FlashbangHitMonster,
            );
        }
    }
}

fn record_escape(
    added: On<Add, Escaped>,
    runs: Query<&RunAchievements, With<PlayerController>>,
    power: Option<Res<FacilityPower>>,
    mut progress: ResMut<AchievementProgress>,
    mut commands: Commands,
) {
    let Ok(run) = runs.get(added.entity) else {
        return;
    };
    for (earned, achievement) in [
        (!run.picked_flashbang, Achievement::EscapeWithoutFlashbang),
        (!run.picked_detector, Achievement::EscapeWithoutDetector),
        (
            !run.restored_boiler && power.is_some_and(|power| !power.on),
            Achievement::EscapeWithoutBoiler,
        ),
        (!run.detected, Achievement::EscapeUndetected),
    ] {
        if earned {
            unlock(&mut progress, &mut commands, achievement);
        }
    }
}

fn record_caught(
    added: On<Add, Caught>,
    runs: Query<&RunAchievements, With<PlayerController>>,
    mut progress: ResMut<AchievementProgress>,
    mut commands: Commands,
) {
    if runs.get(added.entity).is_ok_and(|run| run.exit_opened) {
        unlock(
            &mut progress,
            &mut commands,
            Achievement::CaughtAfterExitOpen,
        );
    }
}

fn unlock(progress: &mut AchievementProgress, commands: &mut Commands, achievement: Achievement) {
    if progress.unlocked.insert(achievement) {
        commands.trigger(AchievementUnlocked(achievement));
    }
}

#[cfg(test)]
#[path = "../tests/unit/achievements.rs"]
mod tests;
