use bevy::prelude::*;
use game_audio::PlaySound;

use super::logic::{AmbientCue, AmbientCueKind, AmbientSourceKind};
use crate::audible::Audible;

#[derive(Resource)]
pub struct AmbienceSounds {
    pub roomtone: Handle<AudioSource>,
    pub low_pressure: Handle<AudioSource>,
    pub conduit: Handle<AudioSource>,
    pub boiler: Handle<AudioSource>,
    pub tank: Handle<AudioSource>,
    pub vent: Handle<AudioSource>,
    pub vent_wind: Handle<AudioSource>,
    pub cool_buzz: Handle<AudioSource>,
    pub boiler_tick: Handle<AudioSource>,
    pub faucet_burst: Handle<AudioSource>,
}

impl AmbienceSounds {
    pub fn source(&self, kind: AmbientSourceKind) -> Handle<AudioSource> {
        match kind {
            AmbientSourceKind::Roomtone => &self.roomtone,
            AmbientSourceKind::LowPressure => &self.low_pressure,
            AmbientSourceKind::Conduit => &self.conduit,
            AmbientSourceKind::Boiler => &self.boiler,
            AmbientSourceKind::Tank => &self.tank,
            AmbientSourceKind::Vent => &self.vent,
            AmbientSourceKind::VentWind => &self.vent_wind,
            AmbientSourceKind::CoolBuzz => &self.cool_buzz,
        }
        .clone()
    }

    fn cue(&self, kind: AmbientCueKind) -> Handle<AudioSource> {
        match kind {
            AmbientCueKind::BoilerTick => &self.boiler_tick,
            AmbientCueKind::FaucetBurst => &self.faucet_burst,
        }
        .clone()
    }
}

// NOTE: app glue inserts AmbienceSounds once the loader is ready; this plugin never inits it.
pub struct AmbienceSoundsPlugin;

impl Plugin for AmbienceSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_ambient_cues.run_if(resource_exists::<AmbienceSounds>),
        );
    }
}

fn play_ambient_cues(
    mut cues: MessageReader<AmbientCue>,
    sounds: Res<AmbienceSounds>,
    mut play: MessageWriter<PlaySound>,
) {
    for cue in cues.read() {
        play.write(PlaySound::from(
            sounds.cue(cue.kind),
            cue.prop,
            cue.offset,
            cue.falloff(),
        ));
    }
}
