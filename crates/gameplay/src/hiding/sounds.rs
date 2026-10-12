use bevy::prelude::*;
use game_audio::PlaySound;

use super::logic::{HidingCue, HidingCueKind};
use crate::audible::Audible;

pub const LOCKER_CUE_OFFSET: Vec3 = Vec3::Y;
pub const TABLE_CUE_OFFSET: Vec3 = Vec3::new(0.0, 0.4, 0.0);

#[derive(Resource)]
pub struct HidingSounds {
    pub locker_open: Handle<AudioSource>,
    pub locker_close: Handle<AudioSource>,
    pub table_enter: Handle<AudioSource>,
    pub table_leave: Handle<AudioSource>,
}

impl HidingSounds {
    fn cue(&self, kind: HidingCueKind) -> Handle<AudioSource> {
        match kind {
            HidingCueKind::LockerOpen => &self.locker_open,
            HidingCueKind::LockerClose => &self.locker_close,
            HidingCueKind::TableEnter => &self.table_enter,
            HidingCueKind::TableLeave => &self.table_leave,
        }
        .clone()
    }
}

pub fn cue_offset(kind: HidingCueKind) -> Vec3 {
    match kind {
        HidingCueKind::LockerOpen | HidingCueKind::LockerClose => LOCKER_CUE_OFFSET,
        HidingCueKind::TableEnter | HidingCueKind::TableLeave => TABLE_CUE_OFFSET,
    }
}

// NOTE: app glue inserts HidingSounds once the loader is ready; this plugin never inits it.
pub struct HidingSoundsPlugin;

impl Plugin for HidingSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_hiding_cues.run_if(resource_exists::<HidingSounds>),
        );
    }
}

fn play_hiding_cues(
    mut cues: MessageReader<HidingCue>,
    sounds: Res<HidingSounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        play.write(PlaySound::from(
            sounds.cue(cue.kind),
            cue.spot,
            cue_offset(cue.kind),
            cue.falloff(),
        ));
    }
}
