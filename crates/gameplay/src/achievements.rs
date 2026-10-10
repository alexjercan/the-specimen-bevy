use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    controller::PlayerController,
    levels::{Caught, Escaped},
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
}

#[derive(Resource, Default, Debug)]
pub struct AchievementProgress {
    pub unlocked: HashSet<Achievement>,
}

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
) {
    let Ok(mut run) = runs.get_mut(signal.player) else {
        return;
    };
    match signal.kind {
        AchievementSignalKind::PickedFlashbang => run.picked_flashbang = true,
        AchievementSignalKind::PickedDetector => run.picked_detector = true,
        AchievementSignalKind::RestoredBoiler => {
            run.restored_boiler = true;
            progress.unlocked.insert(Achievement::RestoreBoiler);
        }
        AchievementSignalKind::ExitOpened => run.exit_opened = true,
        AchievementSignalKind::Detected => run.detected = true,
        AchievementSignalKind::FlashbangHitMonster => {
            progress.unlocked.insert(Achievement::FlashbangHitMonster);
        }
    }
}

fn record_escape(
    added: On<Add, Escaped>,
    runs: Query<&RunAchievements, With<PlayerController>>,
    mut progress: ResMut<AchievementProgress>,
) {
    let Ok(run) = runs.get(added.entity) else {
        return;
    };
    if !run.picked_flashbang {
        progress
            .unlocked
            .insert(Achievement::EscapeWithoutFlashbang);
    }
    if !run.picked_detector {
        progress.unlocked.insert(Achievement::EscapeWithoutDetector);
    }
    if !run.restored_boiler {
        progress.unlocked.insert(Achievement::EscapeWithoutBoiler);
    }
    if !run.detected {
        progress.unlocked.insert(Achievement::EscapeUndetected);
    }
}

fn record_caught(
    added: On<Add, Caught>,
    runs: Query<&RunAchievements, With<PlayerController>>,
    mut progress: ResMut<AchievementProgress>,
) {
    if runs.get(added.entity).is_ok_and(|run| run.exit_opened) {
        progress.unlocked.insert(Achievement::CaughtAfterExitOpen);
    }
}

#[cfg(test)]
#[path = "../tests/unit/achievements.rs"]
mod tests;
