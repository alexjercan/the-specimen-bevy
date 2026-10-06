use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
};

use crate::CaptureState;

pub const AUDIO_SAMPLE_RATE: u32 = 44_100;
pub const DEFAULT_FPS: u32 = 30;

#[derive(Clone, Copy)]
pub struct LoopCapturePlugin {
    pub fps: u32,
}

impl LoopCapturePlugin {
    pub fn new(fps: u32) -> Self {
        assert!(fps > 0, "loop fps must be positive");
        Self { fps }
    }
}

impl Default for LoopCapturePlugin {
    fn default() -> Self {
        Self::new(DEFAULT_FPS)
    }
}

#[derive(Resource)]
struct Profile(LoopCapturePlugin);

#[derive(Resource, Default)]
struct Recorder {
    active: Option<Recording>,
    written: Option<PathBuf>,
}

struct Recording {
    output: PathBuf,
    staging: PathBuf,
    requested: u32,
    received: u32,
    closed: bool,
}

impl Plugin for LoopCapturePlugin {
    fn build(&self, app: &mut App) {
        assert!(self.fps > 0, "loop fps must be positive");
        app.insert_resource(Profile(*self))
            .init_resource::<Recorder>()
            .init_resource::<CaptureState>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / f64::from(self.fps),
            )))
            .add_systems(Last, record_frame);
    }
}

pub fn loop_start(world: &mut World, name: &str) {
    let Some(profile) = world.get_resource::<Profile>() else {
        panic!("loop_start requires an armed LoopCapturePlugin");
    };
    assert!(profile.0.fps > 0);
    let output = loop_path(name);
    assert!(
        world.resource::<Recorder>().active.is_none(),
        "loop already open"
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let staging =
        std::env::temp_dir().join(format!("bevy-capture-loop-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&staging).expect("create unique loop staging directory");
    world.resource_mut::<CaptureState>().register();
    let mut recorder = world.resource_mut::<Recorder>();
    recorder.written = None;
    recorder.active = Some(Recording {
        output,
        staging,
        requested: 0,
        received: 0,
        closed: false,
    });
}

pub fn loop_end(world: &mut World) {
    let mut recorder = world.resource_mut::<Recorder>();
    let recording = recorder.active.as_mut().expect("no open loop");
    assert!(!recording.closed, "loop already closed");
    recording.closed = true;
}

pub fn loop_audio_path(world: &World) -> Option<PathBuf> {
    world
        .get_resource::<Recorder>()?
        .active
        .as_ref()
        .map(|recording| recording.staging.join("audio.f32le"))
}

pub fn loop_written(world: &World, name: &str) -> bool {
    world
        .get_resource::<Recorder>()
        .is_some_and(|recorder| recorder.written.as_ref() == Some(&loop_path(name)))
}

pub fn loop_written_at(name: impl Into<String>) -> Arc<dyn Fn(&World) -> bool + Send + Sync> {
    let name = name.into();
    Arc::new(move |world| loop_written(world, &name))
}

fn loop_path(name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.extension().is_some() {
        path.to_path_buf()
    } else {
        PathBuf::from(format!("{name}.webm"))
    }
}

fn record_frame(world: &mut World) {
    if world.resource::<CaptureState>().failed().is_some() {
        return;
    }
    let profile = world.resource::<Profile>().0;
    let Some(recording) = world.resource::<Recorder>().active.as_ref() else {
        return;
    };
    if recording.closed {
        if recording.received == recording.requested {
            let recording = world
                .resource_mut::<Recorder>()
                .active
                .take()
                .expect("recording present");
            if recording.requested == 0 {
                abort(world, "loop has no frames".into());
                return;
            }
            match encode(&recording, profile.fps) {
                Ok(()) => {
                    info!("loop video saved to {}", recording.output.display());
                    world.resource_mut::<Recorder>().written = Some(recording.output);
                    world.resource_mut::<CaptureState>().finish();
                }
                Err(error) => abort(world, error),
            }
        }
        return;
    }
    let mut recorder = world.resource_mut::<Recorder>();
    let recording = recorder.active.as_mut().expect("recording present");
    let index = recording.requested;
    recording.requested += 1;
    let path = recording.staging.join(format!("frame-{index:06}.png"));
    world.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>,
              mut recorder: ResMut<Recorder>,
              mut state: ResMut<CaptureState>,
              mut exit: MessageWriter<AppExit>| {
            let result = event
                .image
                .clone()
                .try_into_dynamic()
                .map_err(|error| error.to_string())
                .and_then(|image| {
                    image
                        .to_rgb8()
                        .save(&path)
                        .map_err(|error| error.to_string())
                });
            match result {
                Ok(()) => {
                    recorder
                        .active
                        .as_mut()
                        .expect("recording present")
                        .received += 1
                }
                Err(reason) => {
                    error!("loop frame {} failed: {reason}", path.display());
                    state.fail(format!("loop frame {}: {reason}", path.display()));
                    exit.write(AppExit::error());
                }
            }
        },
    );
}

fn encode(recording: &Recording, fps: u32) -> Result<(), String> {
    let parent = recording
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty());
    if let Some(parent) = parent {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create video directory: {error}"))?;
    }
    let input = recording.staging.join("frame-%06d.png");
    let audio = recording.staging.join("audio.f32le");
    let mut command = Command::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-y",
            "-framerate",
        ])
        .arg(fps.to_string())
        .arg("-i")
        .arg(&input);
    if audio.is_file() {
        let bytes = std::fs::metadata(&audio)
            .map_err(|error| format!("read staged audio: {error}"))?
            .len();
        if bytes == 0 || bytes % 8 != 0 {
            return Err("staged stereo f32le audio must contain complete nonempty frames".into());
        }
        command
            .args(["-f", "f32le", "-ar", "44100", "-ac", "2", "-i"])
            .arg(&audio)
            .args(["-c:a", "libopus", "-shortest"]);
    }
    let result = command
        .args(["-frames:v"])
        .arg(recording.requested.to_string())
        .args([
            "-c:v",
            "libvpx-vp9",
            "-crf",
            "34",
            "-b:v",
            "0",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&recording.output)
        .output()
        .map_err(|error| format!("ffmpeg unavailable: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    if !recording.output.is_file() {
        return Err(format!(
            "ffmpeg did not write {}",
            recording.output.display()
        ));
    }
    std::fs::remove_dir_all(&recording.staging)
        .map_err(|error| format!("remove owned staging: {error}"))?;
    Ok(())
}

fn abort(world: &mut World, reason: String) {
    error!("capture loop: {reason}");
    world.resource_mut::<CaptureState>().fail(reason);
    world.write_message(AppExit::error());
    world.resource_mut::<Recorder>().active = None;
}

#[cfg(test)]
#[path = "../tests/unit/loops.rs"]
mod tests;
