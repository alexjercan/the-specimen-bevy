use bevy::prelude::*;
use game_audio::{Bus, PlaySound, LISTENER_GAIN};

use super::pickups::PickupCue;

#[derive(Resource)]
pub struct InteractionSounds {
    pub fuse_slots: [Handle<AudioSource>; 3],
    pub flashbang_pickup: Handle<AudioSource>,
    pub detector_pickup: Handle<AudioSource>,
}

impl InteractionSounds {
    fn cue(&self, cue: PickupCue) -> Option<Handle<AudioSource>> {
        match cue {
            PickupCue::Fuse { count } => self.fuse_slots.get(count.checked_sub(1)?),
            PickupCue::Flashbang => Some(&self.flashbang_pickup),
            PickupCue::Detector => Some(&self.detector_pickup),
        }
        .cloned()
    }
}

// NOTE: app glue inserts InteractionSounds once the loader is ready; this plugin never inits it.
pub struct InteractionSoundsPlugin;

impl Plugin for InteractionSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_pickup_cues.run_if(resource_exists::<InteractionSounds>),
        );
    }
}

fn play_pickup_cues(
    mut cues: MessageReader<PickupCue>,
    sounds: Res<InteractionSounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        if let Some(handle) = sounds.cue(*cue) {
            play.write(PlaySound::global(handle, Bus::World, LISTENER_GAIN));
        }
    }
}
