use bevy::prelude::*;
use game_audio::{Bus, PlaySound, LISTENER_GAIN};

use super::logic::FuseCue;

#[derive(Resource)]
pub struct FuseSounds {
    pub complete: Handle<AudioSource>,
}

impl FuseSounds {
    fn cue(&self, cue: FuseCue) -> Handle<AudioSource> {
        match cue {
            FuseCue::Complete => &self.complete,
        }
        .clone()
    }
}

// NOTE: app glue inserts FuseSounds once the loader is ready; this plugin never inits it.
pub struct FuseSoundsPlugin;

impl Plugin for FuseSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, play_fuse_cues.run_if(resource_exists::<FuseSounds>));
    }
}

fn play_fuse_cues(
    mut cues: MessageReader<FuseCue>,
    sounds: Res<FuseSounds>,
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
