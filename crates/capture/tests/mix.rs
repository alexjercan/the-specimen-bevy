use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use capture::video::{mix, AudioEvent, LoggedBus, Spatial, SAMPLE_RATE};

const FPS: u32 = 30;

fn unique_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "capture-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).expect("create test directory");
    dir
}

fn write_wav(path: &std::path::Path, channels: u16, samples: &[i16]) {
    let data_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * u32::from(channels) * 2).to_le_bytes());
    bytes.extend_from_slice(&(channels * 2).to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    std::fs::write(path, bytes).expect("write wav");
}

fn constant_clip(assets: &std::path::Path, name: &str, frames: usize, value: i16) {
    write_wav(&assets.join(name), 1, &vec![value; frames]);
}

fn samples_per_frame() -> usize {
    (SAMPLE_RATE / FPS) as usize
}

fn start(frame: u32, voice: u64, path: &str, volume: f32, looped: bool) -> AudioEvent {
    AudioEvent::Start {
        frame,
        voice,
        bus: LoggedBus::World,
        path: path.into(),
        volume,
        speed: 1.0,
        looped,
        paused: false,
        spatial: None,
    }
}

#[test]
fn one_shot_starts_at_its_frame_and_ends_with_the_clip() {
    let assets = unique_dir("mix-one-shot");
    constant_clip(&assets, "beep.wav", samples_per_frame(), 16_384);
    let events = [start(2, 1, "beep.wav", 0.5, false)];
    let output = mix(&events, &assets, FPS, 6).unwrap();
    assert_eq!(output.len(), 6 * samples_per_frame() * 2);
    let at = |frame: usize| output[frame * samples_per_frame() * 2];
    assert_eq!(at(0), 0.0);
    assert_eq!(at(1), 0.0);
    assert!((at(2) - 0.25).abs() < 1e-3);
    assert_eq!(at(3), 0.0);
    std::fs::remove_dir_all(assets).unwrap();
}

#[test]
fn loops_repeat_until_their_stop_event() {
    let assets = unique_dir("mix-loop");
    constant_clip(&assets, "hum.wav", samples_per_frame() / 2, 16_384);
    let events = [
        start(0, 7, "hum.wav", 1.0, true),
        AudioEvent::Stop {
            frame: 3,
            voice: 7,
            bus: LoggedBus::World,
        },
    ];
    let output = mix(&events, &assets, FPS, 5).unwrap();
    let at = |frame: usize| output[frame * samples_per_frame() * 2 + 1];
    assert!((at(0) - 0.5).abs() < 1e-3);
    assert!((at(2) - 0.5).abs() < 1e-3);
    assert_eq!(at(3), 0.0);
    assert_eq!(at(4), 0.0);
    std::fs::remove_dir_all(assets).unwrap();
}

#[test]
fn retunes_change_gain_and_paused_segments_hold_the_playhead() {
    let assets = unique_dir("mix-retune");
    let frames = samples_per_frame() * 2;
    let mut ramp = Vec::with_capacity(frames);
    for index in 0..frames {
        ramp.push(if index < samples_per_frame() {
            8_192
        } else {
            16_384
        });
    }
    write_wav(&assets.join("ramp.wav"), 1, &ramp);
    let events = [
        start(0, 3, "ramp.wav", 1.0, false),
        AudioEvent::Retune {
            frame: 1,
            voice: 3,
            bus: LoggedBus::World,
            volume: 1.0,
            speed: 1.0,
            paused: true,
            spatial: None,
        },
        AudioEvent::Retune {
            frame: 2,
            voice: 3,
            bus: LoggedBus::World,
            volume: 0.5,
            speed: 1.0,
            paused: false,
            spatial: None,
        },
    ];
    let output = mix(&events, &assets, FPS, 4).unwrap();
    let at = |frame: usize| output[frame * samples_per_frame() * 2];
    assert!((at(0) - 0.25).abs() < 1e-3);
    assert_eq!(at(1), 0.0);
    assert!((at(2) - 0.25).abs() < 1e-3);
    assert_eq!(at(3), 0.0);
    std::fs::remove_dir_all(assets).unwrap();
}

// NOTE: rodio gives the farther ear the larger pan modifier; the game swaps its ear offsets for that.
#[test]
fn spatial_voices_follow_rodio_ear_gains_and_fade_with_distance() {
    let assets = unique_dir("mix-spatial");
    constant_clip(&assets, "step.wav", samples_per_frame(), 16_384);
    let spatial = |x: f32| {
        Some(Spatial {
            emitter: [x, 0.0, 0.0],
            left: [-0.1, 0.0, 0.0],
            right: [0.1, 0.0, 0.0],
        })
    };
    let voice = |frame, id, position| AudioEvent::Start {
        frame,
        voice: id,
        bus: LoggedBus::World,
        path: "step.wav".into(),
        volume: 1.0,
        speed: 1.0,
        looped: false,
        paused: false,
        spatial: spatial(position),
    };
    let events = [voice(0, 1, 2.0), voice(1, 2, 8.0)];
    let output = mix(&events, &assets, FPS, 2).unwrap();
    let near = (output[0], output[1]);
    let far_index = samples_per_frame() * 2;
    let far = (output[far_index], output[far_index + 1]);
    assert!(
        near.0 > near.1,
        "rodio pans toward the farther ear: {near:?}"
    );
    assert!(
        near.0 > far.0 && near.1 > far.1,
        "near {near:?} far {far:?}"
    );
    assert!(far.1 > 0.0);
    std::fs::remove_dir_all(assets).unwrap();
}

#[test]
fn unknown_voices_and_missing_files_are_errors() {
    let assets = unique_dir("mix-errors");
    let stop = [AudioEvent::Stop {
        frame: 0,
        voice: 9,
        bus: LoggedBus::World,
    }];
    assert!(mix(&stop, &assets, FPS, 1).is_err());
    let missing = [start(0, 1, "missing.wav", 1.0, false)];
    assert!(mix(&missing, &assets, FPS, 1).is_err());
    std::fs::remove_dir_all(assets).unwrap();
}

#[test]
fn the_bus_is_recorded_but_the_mix_uses_the_logged_volume() {
    let assets = unique_dir("mix-bus");
    constant_clip(&assets, "beep.wav", samples_per_frame(), 16_384);
    let on = |bus| {
        let mut event = start(0, 1, "beep.wav", 0.5, false);
        if let AudioEvent::Start { bus: logged, .. } = &mut event {
            *logged = bus;
        }
        mix(&[event], &assets, FPS, 1).unwrap()
    };
    let world = on(LoggedBus::World);
    assert_eq!(world, on(LoggedBus::Ui));
    assert_eq!(world, on(LoggedBus::Unrouted));
    assert!((world[0] - 0.25).abs() < 1e-3);
    std::fs::remove_dir_all(assets).unwrap();
}
