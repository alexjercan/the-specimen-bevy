use bevy::prelude::*;
use game_audio::{Bus, PlaySound, PlaySoundsSet};
use game_flow::GameState;

use super::MenuSounds;

const MENU_MUSIC_GAIN: f32 = 0.45;

#[derive(Resource, Default)]
struct MenuMusicPlaying(bool);

pub(super) fn plugin(app: &mut App) {
    // NOTE: ordered before PlaySoundsSet so a loop started on the exit frame is spawned before OnExit despawns it, or it would leak past the state change.
    app.init_resource::<MenuMusicPlaying>()
        .add_systems(
            Update,
            start_menu_music
                .before(PlaySoundsSet)
                .run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), stop_menu_music);
}

fn start_menu_music(
    menu_sounds: Res<MenuSounds>,
    mut playing: ResMut<MenuMusicPlaying>,
    mut sounds: MessageWriter<PlaySound>,
) {
    if playing.0 {
        return;
    }
    playing.0 = true;
    sounds.write(PlaySound::loop_global(
        menu_sounds.ambient.clone(),
        Bus::Music,
        MENU_MUSIC_GAIN,
    ));
}

fn stop_menu_music(
    mut playing: ResMut<MenuMusicPlaying>,
    voices: Query<(Entity, &Bus)>,
    mut commands: Commands,
) {
    playing.0 = false;
    for (entity, bus) in &voices {
        if *bus == Bus::Music {
            commands.entity(entity).despawn();
        }
    }
}
