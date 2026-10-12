mod log;
mod mix;

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy::{
    audio::{DefaultSpatialScale, GlobalVolume},
    ecs::message::MessageCursor,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
};

use crate::CaptureState;

pub use log::{AudioEvent, LoggedBus, Spatial, AUDIO_LOG};
pub use mix::{mix, SAMPLE_RATE};

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct VideoProfile {
    pub fps: u32,
    pub crf: u8,
    pub output_size: Option<UVec2>,
    pub deadline: Duration,
    pub staging: Option<PathBuf>,
}

impl VideoProfile {
    pub const PREVIEW: Self = Self {
        fps: 30,
        crf: 40,
        output_size: Some(UVec2::new(640, 360)),
        deadline: Duration::from_secs(120),
        staging: None,
    };

    pub const FINAL: Self = Self {
        fps: 30,
        crf: 24,
        output_size: None,
        deadline: Duration::from_secs(600),
        staging: None,
    };

    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    pub fn with_staging(mut self, staging: impl Into<PathBuf>) -> Self {
        self.staging = Some(staging.into());
        self
    }

    fn stride(&self, step: Duration) -> u32 {
        let stride = 1.0 / (step.as_secs_f64() * f64::from(self.fps));
        let rounded = stride.round();
        assert!(
            rounded >= 1.0 && (stride - rounded).abs() < 1e-3,
            "video fps {} must divide the {:?} frame step evenly",
            self.fps,
            step
        );
        rounded as u32
    }
}

#[derive(Clone)]
pub struct VideoCapturePlugin {
    pub profile: VideoProfile,
    pub step: Duration,
}

impl VideoCapturePlugin {
    pub fn new(profile: VideoProfile, step: Duration) -> Self {
        assert!(profile.fps > 0, "video fps must be positive");
        assert!(!step.is_zero(), "frame step must be positive");
        profile.stride(step);
        Self { profile, step }
    }
}

#[derive(Resource)]
struct Stride(u32);

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
    skipped: u32,
    closed: Option<Instant>,
    audio: log::AudioLog,
}

impl Drop for Recording {
    fn drop(&mut self) {
        self.audio.release();
        match std::fs::remove_dir_all(&self.staging) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => warn!("remove video staging {}: {error}", self.staging.display()),
        }
    }
}

impl Plugin for VideoCapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.profile.clone())
            .insert_resource(Stride(self.profile.stride(self.step)))
            .init_resource::<Recorder>()
            .init_resource::<GlobalVolume>()
            .init_resource::<DefaultSpatialScale>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(self.step))
            .add_systems(
                Last,
                (
                    log::log_audio.run_if(resource_exists::<AssetServer>),
                    record_frame,
                )
                    .chain(),
            );
    }
}

pub fn video_start(world: &mut World, name: &str) {
    let output = video_path(name);
    assert!(
        world.resource::<Recorder>().active.is_none(),
        "video already open"
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base = world
        .resource::<VideoProfile>()
        .staging
        .clone()
        .unwrap_or_else(std::env::temp_dir);
    let staging = base.join(format!("bevy-capture-video-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&staging).expect("create unique video staging directory");
    let audio = log::AudioLog::open(&staging.join(AUDIO_LOG));
    world.resource_mut::<CaptureState>().register();
    let mut recorder = world.resource_mut::<Recorder>();
    recorder.written = None;
    recorder.active = Some(Recording {
        output,
        staging,
        requested: 0,
        received: 0,
        skipped: 0,
        closed: None,
        audio,
    });
}

pub fn video_end(world: &mut World) {
    let mut recorder = world.resource_mut::<Recorder>();
    let recording = recorder.active.as_mut().expect("no open video");
    assert!(recording.closed.is_none(), "video already closed");
    recording.closed = Some(Instant::now());
    recording.audio.close();
}

pub fn video_staging(world: &World) -> Option<PathBuf> {
    world
        .resource::<Recorder>()
        .active
        .as_ref()
        .map(|recording| recording.staging.clone())
}

pub fn video_written(world: &World, name: &str) -> bool {
    world.resource::<Recorder>().written.as_ref() == Some(&video_path(name))
}

pub fn video_written_at(name: impl Into<String>) -> Arc<dyn Fn(&World) -> bool + Send + Sync> {
    let name = name.into();
    Arc::new(move |world| video_written(world, &name))
}

fn video_path(name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.extension().is_some() {
        path.to_path_buf()
    } else {
        PathBuf::from(format!("{name}.webm"))
    }
}

// NOTE: the runner stops after the update that wrote AppExit, so an open recording is finished here or reported lost.
pub fn video_close_at_exit(world: &mut World) {
    let exits: Vec<AppExit> = {
        let messages = world.resource::<Messages<AppExit>>();
        MessageCursor::default().read(messages).cloned().collect()
    };
    if exits.is_empty() || world.resource::<Recorder>().active.is_none() {
        return;
    }
    let failure = world.resource::<CaptureState>().failed().map(str::to_owned);
    let mut recording = world
        .resource_mut::<Recorder>()
        .active
        .take()
        .expect("recording present");
    if let Some(failure) = failure {
        lost(world, &recording.output, failure);
        return;
    }
    if exits.iter().any(AppExit::is_error) {
        lost(
            world,
            &recording.output,
            "the app exited with an error before the video ended".into(),
        );
        return;
    }
    let closed = *recording.closed.get_or_insert_with(Instant::now);
    recording.audio.close();
    recording.requested = recording.received;
    finish(world, recording, closed);
}

fn finish(world: &mut World, recording: Recording, closed: Instant) {
    if recording.requested == 0 {
        lost(world, &recording.output, "video has no frames".into());
        return;
    }
    let profile = world.resource::<VideoProfile>().clone();
    match encode(&recording, &profile, closed + profile.deadline) {
        Ok(()) => {
            info!("video saved to {}", recording.output.display());
            world.resource_mut::<Recorder>().written = Some(recording.output.clone());
            world.resource_mut::<CaptureState>().finish();
        }
        Err(error) => lost(world, &recording.output, error),
    }
}

fn record_frame(world: &mut World) {
    if let Some(failure) = world.resource::<CaptureState>().failed().map(str::to_owned) {
        let recording = world.resource_mut::<Recorder>().active.take();
        if let Some(recording) = recording {
            lost(world, &recording.output, failure);
        }
        return;
    }
    let profile = world.resource::<VideoProfile>().clone();
    let stride = world.resource::<Stride>().0;
    let Some(recording) = world.resource::<Recorder>().active.as_ref() else {
        return;
    };
    if let Some(closed) = recording.closed {
        if recording.received == recording.requested {
            let recording = world
                .resource_mut::<Recorder>()
                .active
                .take()
                .expect("recording present");
            finish(world, recording, closed);
        } else if closed.elapsed() > profile.deadline {
            let reason = format!(
                "{} of {} video frames arrived within {:?}",
                recording.received, recording.requested, profile.deadline
            );
            let recording = world
                .resource_mut::<Recorder>()
                .active
                .take()
                .expect("recording present");
            lost(world, &recording.output, reason);
        }
        return;
    }
    let mut recorder = world.resource_mut::<Recorder>();
    let recording = recorder.active.as_mut().expect("recording present");
    if recording.skipped + 1 < stride {
        recording.skipped += 1;
        return;
    }
    recording.skipped = 0;
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
                    if let Some(recording) = recorder.active.as_mut() {
                        recording.received += 1;
                    }
                }
                Err(reason) => {
                    error!("video frame {} failed: {reason}", path.display());
                    state.fail(format!("video frame {}: {reason}", path.display()));
                    exit.write(AppExit::error());
                }
            }
        },
    );
}

fn encode(recording: &Recording, profile: &VideoProfile, deadline: Instant) -> Result<(), String> {
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
    let events = log::read(&recording.staging.join(AUDIO_LOG))?;
    if events
        .iter()
        .any(|event| matches!(event, AudioEvent::Start { .. }))
    {
        let samples = mix(
            &events,
            &log::assets_root(),
            profile.fps,
            recording.requested,
        )?;
        mix::write_f32le(&audio, &samples)?;
    }
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
        .arg(profile.fps.to_string())
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
            .args([
                "-f",
                "f32le",
                "-ar",
                &SAMPLE_RATE.to_string(),
                "-ac",
                "2",
                "-i",
            ])
            .arg(&audio)
            .args(["-c:a", "libopus", "-shortest"]);
    }
    if let Some(size) = profile.output_size {
        command.args(["-vf", &format!("scale={}:{}", size.x, size.y)]);
    }
    command
        .args(["-frames:v"])
        .arg(recording.requested.to_string())
        .args(["-c:v", "libvpx-vp9", "-crf"])
        .arg(profile.crf.to_string())
        .args(["-b:v", "0", "-pix_fmt", "yuv420p"])
        .arg(&recording.output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("ffmpeg unavailable: {error}"))?;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "ffmpeg exceeded the {:?} deadline",
                    profile.deadline
                ));
            }
            Err(error) => return Err(format!("ffmpeg wait failed: {error}")),
        }
    };
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut pipe) = child.stderr.take() {
            let _ = std::io::Read::read_to_string(&mut pipe, &mut stderr);
        }
        return Err(format!("ffmpeg failed: {stderr}"));
    }
    if !recording.output.is_file() {
        return Err(format!(
            "ffmpeg did not write {}",
            recording.output.display()
        ));
    }
    Ok(())
}

fn lost(world: &mut World, output: &Path, reason: String) {
    error!("video {} was not written: {reason}", output.display());
    world.resource_mut::<CaptureState>().fail(reason);
    world.write_message(AppExit::error());
}
