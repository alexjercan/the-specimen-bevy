use bevy::prelude::*;
use game_audio::PlaySound;

use super::logic::{MonsterCue, MonsterCueKind};
use crate::audible::Audible;

pub const MONSTER_CUE_OFFSET: Vec3 = Vec3::Y;

#[derive(Resource)]
pub struct MonsterSounds {
    pub presence: Handle<AudioSource>,
    pub steps: [Handle<AudioSource>; 6],
    pub detected: Handle<AudioSource>,
    pub attack: Handle<AudioSource>,
    pub heartbeat: Handle<AudioSource>,
}

impl MonsterSounds {
    fn cue(&self, kind: MonsterCueKind) -> Handle<AudioSource> {
        match kind {
            MonsterCueKind::Presence => &self.presence,
            MonsterCueKind::Step(index) => &self.steps[index % self.steps.len()],
            MonsterCueKind::Detected => &self.detected,
            MonsterCueKind::Attack => &self.attack,
            MonsterCueKind::Heartbeat => &self.heartbeat,
        }
        .clone()
    }
}

// NOTE: app glue inserts MonsterSounds once the loader is ready; this plugin never inits it.
pub struct MonsterSoundsPlugin;

impl Plugin for MonsterSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_monster_cues.run_if(resource_exists::<MonsterSounds>),
        );
    }
}

fn play_monster_cues(
    mut cues: MessageReader<MonsterCue>,
    sounds: Res<MonsterSounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        play.write(PlaySound::from(
            sounds.cue(cue.kind),
            cue.monster,
            MONSTER_CUE_OFFSET,
            cue.falloff(),
        ));
    }
}
