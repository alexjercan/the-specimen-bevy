use bevy::prelude::*;
use game_audio::{Bus, PlaySound, LISTENER_GAIN};

use super::{
    doors::{DoorCue, DoorCueKind},
    power::PowerCue,
};
use crate::audible::{Audible, Origin};

pub const DOOR_CUE_OFFSET: Vec3 = Vec3::Y;
pub const BOILER_CUE_OFFSET: Vec3 = Vec3::new(0.0, 1.2, 0.0);

#[derive(Resource)]
pub struct FacilitySounds {
    pub door_unlatch: Handle<AudioSource>,
    pub door_swing: Handle<AudioSource>,
    pub door_shut: Handle<AudioSource>,
    pub door_locked: Handle<AudioSource>,
    pub power_down: Handle<AudioSource>,
    pub breaker_trip: Handle<AudioSource>,
    pub boiler_reset: Handle<AudioSource>,
    pub boiler_restart: Handle<AudioSource>,
}

impl FacilitySounds {
    fn door(&self, kind: DoorCueKind) -> Handle<AudioSource> {
        match kind {
            DoorCueKind::Unlatch => &self.door_unlatch,
            DoorCueKind::Swing => &self.door_swing,
            DoorCueKind::Shut => &self.door_shut,
            DoorCueKind::Locked => &self.door_locked,
        }
        .clone()
    }

    fn power(&self, cue: PowerCue) -> Handle<AudioSource> {
        match cue {
            PowerCue::PowerDown => &self.power_down,
            PowerCue::BreakerTrip { .. } => &self.breaker_trip,
            PowerCue::BoilerReset { .. } => &self.boiler_reset,
            PowerCue::BoilerRestart { .. } => &self.boiler_restart,
        }
        .clone()
    }
}

// NOTE: app glue inserts FacilitySounds once the loader is ready; this plugin never inits it.
pub struct FacilitySoundsPlugin;

impl Plugin for FacilitySoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (play_door_cues, play_power_cues).run_if(resource_exists::<FacilitySounds>),
        );
    }
}

fn play_door_cues(
    mut cues: MessageReader<DoorCue>,
    sounds: Res<FacilitySounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        play.write(PlaySound::from(
            sounds.door(cue.kind),
            cue.door,
            DOOR_CUE_OFFSET,
            cue.falloff(),
        ));
    }
}

fn play_power_cues(
    mut cues: MessageReader<PowerCue>,
    sounds: Res<FacilitySounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        let handle = sounds.power(*cue);
        play.write(match cue.origin() {
            Origin::Listener => PlaySound::global(handle, Bus::World, LISTENER_GAIN),
            Origin::At(position) => PlaySound::at(handle, position, cue.falloff()),
            Origin::From(boiler) => {
                PlaySound::from(handle, boiler, BOILER_CUE_OFFSET, cue.falloff())
            }
        });
    }
}
