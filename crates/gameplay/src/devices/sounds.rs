use bevy::prelude::*;
use game_audio::{Bus, PlaySound, LISTENER_GAIN};

use super::logic::DeviceCue;

#[derive(Resource)]
pub struct DeviceSounds {
    pub detector_nearby: Handle<AudioSource>,
    pub flashbang_throw: Handle<AudioSource>,
    pub flashbang_burst: Handle<AudioSource>,
}

impl DeviceSounds {
    fn cue(&self, cue: DeviceCue) -> Handle<AudioSource> {
        match cue {
            DeviceCue::DetectorNearby => &self.detector_nearby,
            DeviceCue::FlashbangThrow => &self.flashbang_throw,
            DeviceCue::FlashbangBurst => &self.flashbang_burst,
        }
        .clone()
    }
}

// NOTE: app glue inserts DeviceSounds once the loader is ready; this plugin never inits it.
pub struct DeviceSoundsPlugin;

impl Plugin for DeviceSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_device_cues.run_if(resource_exists::<DeviceSounds>),
        );
    }
}

fn play_device_cues(
    mut cues: MessageReader<DeviceCue>,
    sounds: Res<DeviceSounds>,
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
