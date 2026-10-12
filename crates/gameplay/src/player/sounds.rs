use bevy::prelude::*;
use game_audio::{Bus, PlaySound, LISTENER_GAIN};

use super::logic::PlayerCue;

#[derive(Resource)]
pub struct PlayerSounds {
    pub steps: [Handle<AudioSource>; 3],
    pub sprint_exhausted: Handle<AudioSource>,
    pub flashlight_click: Handle<AudioSource>,
}

impl PlayerSounds {
    fn cue(&self, cue: PlayerCue) -> Handle<AudioSource> {
        match cue {
            PlayerCue::Step(index) => &self.steps[index % self.steps.len()],
            PlayerCue::SprintExhausted => &self.sprint_exhausted,
            PlayerCue::FlashlightClick => &self.flashlight_click,
        }
        .clone()
    }
}

// NOTE: app glue inserts PlayerSounds once the loader is ready; this plugin never inits it.
pub struct PlayerSoundsPlugin;

impl Plugin for PlayerSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_player_cues.run_if(resource_exists::<PlayerSounds>),
        );
    }
}

fn play_player_cues(
    mut cues: MessageReader<PlayerCue>,
    sounds: Res<PlayerSounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        play.write(PlaySound::global(
            sounds.cue(*cue),
            Bus::World,
            LISTENER_GAIN,
        ));
    }
}
