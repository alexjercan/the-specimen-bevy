use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use bevy::{
    audio::{
        AudioSink, AudioSinkPlayback, DefaultSpatialScale, GlobalVolume, PlaybackMode,
        SpatialAudioSink,
    },
    platform::collections::HashMap,
    prelude::*,
};
use game_audio::Bus;
use serde::{Deserialize, Serialize};

use super::{Recorder, VideoProfile};

pub const AUDIO_LOG: &str = "audio.jsonl";

// NOTE: bevy's AssetPlugin default file path; the game keeps it.
const ASSET_DIR: &str = "assets";

// NOTE: the log records the bus for each voice; volumes are already effective, so the mixer does not scale by bus.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoggedBus {
    World,
    Ui,
    Unrouted,
}

impl From<Option<&Bus>> for LoggedBus {
    fn from(bus: Option<&Bus>) -> Self {
        match bus {
            Some(Bus::World) => Self::World,
            Some(Bus::Ui) => Self::Ui,
            None => Self::Unrouted,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AudioEvent {
    Start {
        frame: u32,
        voice: u64,
        bus: LoggedBus,
        path: String,
        volume: f32,
        speed: f32,
        looped: bool,
        paused: bool,
        spatial: Option<Spatial>,
    },
    Retune {
        frame: u32,
        voice: u64,
        bus: LoggedBus,
        volume: f32,
        speed: f32,
        paused: bool,
        spatial: Option<Spatial>,
    },
    Stop {
        frame: u32,
        voice: u64,
        bus: LoggedBus,
    },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Spatial {
    pub emitter: [f32; 3],
    pub left: [f32; 3],
    pub right: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
struct Tuning {
    volume: f32,
    speed: f32,
    paused: bool,
    spatial: Option<Spatial>,
}

impl Tuning {
    fn close_to(&self, other: &Self) -> bool {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
        let point = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(a, b)| near(*a, b));
        near(self.volume, other.volume)
            && near(self.speed, other.speed)
            && self.paused == other.paused
            && match (self.spatial, other.spatial) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    point(a.emitter, b.emitter) && point(a.left, b.left) && point(a.right, b.right)
                }
                _ => false,
            }
    }
}

pub(super) struct AudioLog {
    file: Option<BufWriter<File>>,
    voices: HashMap<Entity, (LoggedBus, Tuning)>,
    pass: Option<u32>,
}

impl AudioLog {
    pub(super) fn open(path: &Path) -> Self {
        let file = File::create(path).expect("create audio event log");
        Self {
            file: Some(BufWriter::new(file)),
            voices: HashMap::default(),
            pass: None,
        }
    }

    pub(super) fn release(&mut self) {
        self.file = None;
    }

    pub(super) fn close(&mut self) {
        if let Some(mut file) = self.file.take() {
            file.flush().expect("flush audio event log");
        }
    }

    fn write(&mut self, event: &AudioEvent) {
        if let Some(file) = self.file.as_mut() {
            serde_json::to_writer(&mut *file, event).expect("serialize audio event");
            file.write_all(b"\n")
                .and_then(|()| file.flush())
                .expect("write audio event log");
        }
    }
}

pub(super) fn read(path: &Path) -> Result<Vec<AudioEvent>, String> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let file = File::open(path).map_err(|error| format!("open audio event log: {error}"))?;
    BufReader::new(file)
        .lines()
        .map(|line| {
            let line = line.map_err(|error| format!("read audio event log: {error}"))?;
            serde_json::from_str(&line).map_err(|error| format!("parse audio event log: {error}"))
        })
        .collect()
}

pub(super) fn assets_root() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from(ASSET_DIR)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        bevy::asset::io::file::FileAssetReader::get_base_path().join(ASSET_DIR)
    }
}

type Voice<'a> = (
    Entity,
    &'a AudioPlayer<AudioSource>,
    &'a PlaybackSettings,
    Option<&'a GlobalTransform>,
    Option<&'a AudioSink>,
    Option<&'a SpatialAudioSink>,
    Option<&'a Bus>,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn log_audio(
    mut recorder: ResMut<Recorder>,
    profile: Res<VideoProfile>,
    assets: Res<AssetServer>,
    global: Res<GlobalVolume>,
    default_scale: Res<DefaultSpatialScale>,
    listeners: Query<(&GlobalTransform, &SpatialListener)>,
    voices: Query<Voice>,
    mut stopped: RemovedComponents<AudioPlayer<AudioSource>>,
) {
    let Some(recording) = recorder.active.as_mut() else {
        stopped.clear();
        return;
    };
    if recording.closed.is_some() {
        stopped.clear();
        return;
    }
    let frame = recording.requested;
    let log = &mut recording.audio;
    for entity in stopped.read() {
        if let Some((bus, _)) = log.voices.remove(&entity) {
            log.write(&AudioEvent::Stop {
                frame,
                voice: entity.to_bits(),
                bus,
            });
        }
    }
    let ears = listeners.iter().next().map(|(transform, listener)| {
        (
            transform.transform_point(listener.left_ear_offset),
            transform.transform_point(listener.right_ear_offset),
        )
    });
    let cadence = (profile.fps / 4).max(1);
    let pass = frame / cadence;
    let retune = log.pass != Some(pass);
    log.pass = Some(pass);
    for (entity, player, settings, transform, sink, spatial_sink, bus) in &voices {
        let bus = LoggedBus::from(bus);
        let tuning = tuning(
            settings,
            transform,
            sink,
            spatial_sink,
            ears,
            &global,
            &default_scale,
        );
        match log.voices.get(&entity) {
            None => {
                let Some(path) = assets.get_path(player.0.id()) else {
                    continue;
                };
                log.voices.insert(entity, (bus, tuning));
                log.write(&AudioEvent::Start {
                    frame,
                    voice: entity.to_bits(),
                    bus,
                    path: path.path().to_string_lossy().into_owned(),
                    volume: tuning.volume,
                    speed: tuning.speed,
                    looped: matches!(settings.mode, PlaybackMode::Loop),
                    paused: tuning.paused,
                    spatial: tuning.spatial,
                });
            }
            Some((_, previous)) if retune && !previous.close_to(&tuning) => {
                log.voices.insert(entity, (bus, tuning));
                log.write(&AudioEvent::Retune {
                    frame,
                    voice: entity.to_bits(),
                    bus,
                    volume: tuning.volume,
                    speed: tuning.speed,
                    paused: tuning.paused,
                    spatial: tuning.spatial,
                });
            }
            Some(_) => {}
        }
    }
}

fn tuning(
    settings: &PlaybackSettings,
    transform: Option<&GlobalTransform>,
    sink: Option<&AudioSink>,
    spatial_sink: Option<&SpatialAudioSink>,
    ears: Option<(Vec3, Vec3)>,
    global: &GlobalVolume,
    default_scale: &DefaultSpatialScale,
) -> Tuning {
    let (volume, speed, paused) = match (sink, spatial_sink) {
        (Some(sink), _) => (sink.volume().to_linear(), sink.speed(), sink.is_paused()),
        (_, Some(sink)) => (sink.volume().to_linear(), sink.speed(), sink.is_paused()),
        (None, None) => (
            global.volume.to_linear() * settings.volume.to_linear(),
            settings.speed,
            settings.paused,
        ),
    };
    let volume = if settings.muted { 0.0 } else { volume };
    let spatial = match (settings.spatial, transform, ears) {
        (true, Some(transform), Some((left, right))) => {
            let scale = settings.spatial_scale.unwrap_or(default_scale.0).0;
            Some(Spatial {
                emitter: (transform.translation() * scale).to_array(),
                left: (left * scale).to_array(),
                right: (right * scale).to_array(),
            })
        }
        _ => None,
    };
    Tuning {
        volume,
        speed,
        paused,
        spatial,
    }
}
