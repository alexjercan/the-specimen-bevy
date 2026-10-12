#![allow(dead_code)]

use bevy::{asset::uuid::Uuid, prelude::*};
use game_audio::GameAudioPlugin;

pub fn clip(id: u128) -> Handle<AudioSource> {
    Handle::<AudioSource>::Uuid(Uuid::from_u128(id), default())
}

pub fn audio_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, GameAudioPlugin));
    app
}
