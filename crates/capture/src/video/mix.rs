use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    num::NonZero,
    path::Path,
};

use bevy::platform::collections::HashMap;
use rodio::{source::UniformSourceIterator, Decoder, Source};

use super::{AudioEvent, Spatial};

pub const SAMPLE_RATE: u32 = 48_000;

struct Clip {
    channels: u16,
    stereo: Vec<f32>,
}

impl Clip {
    fn frames(&self) -> usize {
        self.stereo.len() / 2
    }

    // NOTE: rodio's spatial source sums the input channels instead of averaging them.
    fn mono(&self, frame: usize) -> f32 {
        if self.channels == 1 {
            self.stereo[frame * 2]
        } else {
            self.stereo[frame * 2] + self.stereo[frame * 2 + 1]
        }
    }
}

#[derive(Clone, Copy)]
struct Tuning {
    volume: f32,
    speed: f32,
    paused: bool,
    spatial: Option<Spatial>,
}

impl Tuning {
    fn gains(&self) -> (f32, f32) {
        match self.spatial {
            None => (self.volume, self.volume),
            Some(spatial) => {
                let (left, right) = ear_gains(spatial);
                (self.volume * left, self.volume * right)
            }
        }
    }
}

struct Voice {
    path: String,
    looped: bool,
    timeline: Vec<(u32, Tuning)>,
    stop: Option<u32>,
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b.iter()).map(|(a, b)| (a - b) * (a - b)).sum()
}

// NOTE: mirrors rodio::source::Spatial::set_positions so the mix matches what the game plays.
fn ear_gains(spatial: Spatial) -> (f32, f32) {
    let left_sq = dist_sq(spatial.left, spatial.emitter);
    let right_sq = dist_sq(spatial.right, spatial.emitter);
    let max_diff = dist_sq(spatial.left, spatial.right).sqrt();
    let left = left_sq.sqrt();
    let right = right_sq.sqrt();
    let left_diff = (((left - right) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
    let right_diff = (((right - left) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
    (
        left_diff * (1.0 / left_sq).min(1.0),
        right_diff * (1.0 / right_sq).min(1.0),
    )
}

fn voices(events: &[AudioEvent]) -> Result<Vec<Voice>, String> {
    let mut voices: Vec<Voice> = Vec::new();
    let mut open: HashMap<u64, usize> = HashMap::default();
    for event in events {
        match event {
            AudioEvent::Start {
                frame,
                voice,
                path,
                volume,
                speed,
                looped,
                paused,
                spatial,
                ..
            } => {
                open.insert(*voice, voices.len());
                voices.push(Voice {
                    path: path.clone(),
                    looped: *looped,
                    timeline: vec![(
                        *frame,
                        Tuning {
                            volume: *volume,
                            speed: *speed,
                            paused: *paused,
                            spatial: *spatial,
                        },
                    )],
                    stop: None,
                });
            }
            AudioEvent::Retune {
                frame,
                voice,
                volume,
                speed,
                paused,
                spatial,
                ..
            } => {
                let index = *open
                    .get(voice)
                    .ok_or_else(|| format!("retune for unknown voice {voice}"))?;
                voices[index].timeline.push((
                    *frame,
                    Tuning {
                        volume: *volume,
                        speed: *speed,
                        paused: *paused,
                        spatial: *spatial,
                    },
                ));
            }
            AudioEvent::Stop { frame, voice, .. } => {
                let index = open
                    .remove(voice)
                    .ok_or_else(|| format!("stop for unknown voice {voice}"))?;
                voices[index].stop = Some(*frame);
            }
        }
    }
    Ok(voices)
}

fn decode(path: &Path) -> Result<Clip, String> {
    let file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let decoder = Decoder::new(BufReader::new(file))
        .map_err(|error| format!("decode {}: {error}", path.display()))?;
    let channels = decoder.channels().get();
    let stereo = UniformSourceIterator::new(
        decoder,
        NonZero::new(2).expect("stereo"),
        NonZero::new(SAMPLE_RATE).expect("sample rate"),
    )
    .collect();
    Ok(Clip { channels, stereo })
}

fn sample_at(frame: u32, fps: u32) -> usize {
    (u64::from(frame) * u64::from(SAMPLE_RATE) / u64::from(fps)) as usize
}

pub fn mix(
    events: &[AudioEvent],
    assets: &Path,
    fps: u32,
    frames: u32,
) -> Result<Vec<f32>, String> {
    let total = sample_at(frames, fps);
    let mut output = vec![0.0f32; total * 2];
    let mut clips: HashMap<String, Clip> = HashMap::default();
    for voice in voices(events)? {
        if !clips.contains_key(&voice.path) {
            clips.insert(voice.path.clone(), decode(&assets.join(&voice.path))?);
        }
        let clip = &clips[&voice.path];
        if clip.frames() == 0 {
            continue;
        }
        let end = voice
            .stop
            .map_or(total, |frame| sample_at(frame, fps).min(total));
        let mut playhead = 0.0f64;
        let mut finished = false;
        for (index, (frame, tuning)) in voice.timeline.iter().enumerate() {
            let from = sample_at(*frame, fps);
            let to = voice
                .timeline
                .get(index + 1)
                .map_or(end, |(next, _)| sample_at(*next, fps))
                .min(end);
            if tuning.paused || finished || from >= to {
                continue;
            }
            let (left, right) = tuning.gains();
            for n in from..to {
                let mut position = playhead as usize;
                if position >= clip.frames() {
                    if !voice.looped {
                        finished = true;
                        break;
                    }
                    playhead %= clip.frames() as f64;
                    position = playhead as usize;
                }
                let (l, r) = if tuning.spatial.is_some() {
                    let mono = clip.mono(position);
                    (mono, mono)
                } else {
                    (clip.stereo[position * 2], clip.stereo[position * 2 + 1])
                };
                output[n * 2] += l * left;
                output[n * 2 + 1] += r * right;
                playhead += f64::from(tuning.speed);
            }
        }
    }
    for sample in &mut output {
        *sample = sample.clamp(-1.0, 1.0);
    }
    Ok(output)
}

pub fn write_f32le(path: &Path, samples: &[f32]) -> Result<(), String> {
    let file = File::create(path).map_err(|error| format!("create {}: {error}", path.display()))?;
    let mut writer = BufWriter::new(file);
    for sample in samples {
        writer
            .write_all(&sample.to_le_bytes())
            .map_err(|error| format!("write {}: {error}", path.display()))?;
    }
    writer
        .flush()
        .map_err(|error| format!("write {}: {error}", path.display()))
}
