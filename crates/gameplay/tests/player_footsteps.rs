use std::time::Duration;

use bevy::prelude::*;
use bevy_rand::prelude::{ChaCha8Rng, EntropyPlugin};
use gameplay::player::{PlayerController, PlayerControlsEnabled, PlayerCue};

mod common;

fn app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed([0; 32]));
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)));
    app.update();
    app
}

fn step_frames(app: &mut App, frames: usize) -> Vec<(usize, usize)> {
    let mut steps = Vec::new();
    for frame in 1..=frames {
        app.update();
        let cues: Vec<PlayerCue> = app
            .world_mut()
            .resource_mut::<Messages<PlayerCue>>()
            .drain()
            .collect();
        for cue in cues {
            if let PlayerCue::Step(index) = cue {
                steps.push((frame, index));
            }
        }
    }
    steps
}

#[test]
fn walking_steps_every_stride_with_a_cycling_index() {
    let mut app = app();
    assert!(step_frames(&mut app, 5).is_empty());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    let steps = step_frames(&mut app, 20);
    let frames: Vec<usize> = steps.iter().map(|(frame, _)| *frame).collect();
    assert_eq!(frames, vec![5, 9, 13, 17]);
    assert!(steps.iter().all(|(_, index)| *index < 3));
}

#[test]
fn sprinting_lengthens_the_stride() {
    let mut app = app();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::ShiftLeft);
    let frames: Vec<usize> = step_frames(&mut app, 16)
        .iter()
        .map(|(frame, _)| *frame)
        .collect();
    assert_eq!(frames, vec![4, 8, 11, 15]);
}

#[test]
fn disabled_controls_make_no_steps() {
    let mut app = app();
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    assert!(step_frames(&mut app, 20).is_empty());
}
