use std::{sync::Arc, time::Duration};

use bevy::{
    ecs::{message::MessageCursor, query::QuerySingleError},
    prelude::*,
    render::renderer::RenderAdapterInfo,
    window::{ExitSystems, PresentMode, PrimaryWindow},
    winit::{UpdateMode, WinitSettings},
    world_serialization::{WorldAssetRoot, WorldInstance, WorldInstanceSpawner},
};

pub mod args;
pub mod error;
pub mod output;
pub mod stats;

pub use args::ProbeArgs;
pub use error::ProbeError;
pub use output::{Output, ProbeReport, CSV_HEADER};
pub use stats::FrameStats;

pub const DEFAULT_WARMUP: u32 = 180;
pub const DEFAULT_FRAMES: u32 = 900;
pub const DEFAULT_RESOLUTION: &str = "1280x720";
pub const DEFAULT_PRESENT_MODE: &str = "autonovsync";
pub const DEFAULT_LABEL: &str = "facility";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
pub const PRESENT_MODES: [&str; 6] = [
    "immediate",
    "mailbox",
    "fifo",
    "fiforelaxed",
    "autovsync",
    "autonovsync",
];

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct ProbeConfig {
    pub label: String,
    pub warmup: u32,
    pub frames: u32,
    pub resolution: UVec2,
    pub present_mode: PresentMode,
    pub out: Option<Output>,
    pub timeout: Duration,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            label: DEFAULT_LABEL.to_string(),
            warmup: DEFAULT_WARMUP,
            frames: DEFAULT_FRAMES,
            resolution: UVec2::new(1280, 720),
            present_mode: PresentMode::AutoNoVsync,
            out: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ProbeConfig {
    pub fn validate(&self) -> Result<(), ProbeError> {
        parse_label(&self.label)?;
        if self.frames == 0 {
            return Err(ProbeError::ZeroFrames);
        }
        if self.resolution.cmpeq(UVec2::ZERO).any() {
            return Err(ProbeError::InvalidResolution(format!(
                "{}x{}",
                self.resolution.x, self.resolution.y
            )));
        }
        if self.timeout.is_zero() {
            return Err(ProbeError::ZeroTimeout);
        }
        if let Some(out) = &self.out {
            Output::from_path(out.path())?;
        }
        Ok(())
    }
}

pub fn parse_resolution(value: &str) -> Result<UVec2, ProbeError> {
    let invalid = || ProbeError::InvalidResolution(value.to_string());
    let (width, height) = value.split_once('x').ok_or_else(invalid)?;
    let width = width.parse::<u32>().map_err(|_| invalid())?;
    let height = height.parse::<u32>().map_err(|_| invalid())?;
    if width == 0 || height == 0 {
        return Err(invalid());
    }
    Ok(UVec2::new(width, height))
}

pub fn parse_present_mode(value: &str) -> Result<PresentMode, ProbeError> {
    match value {
        "immediate" => Ok(PresentMode::Immediate),
        "mailbox" => Ok(PresentMode::Mailbox),
        "fifo" => Ok(PresentMode::Fifo),
        "fiforelaxed" => Ok(PresentMode::FifoRelaxed),
        "autovsync" => Ok(PresentMode::AutoVsync),
        "autonovsync" => Ok(PresentMode::AutoNoVsync),
        _ => Err(ProbeError::InvalidPresentMode(value.to_string())),
    }
}

pub fn present_mode_name(mode: PresentMode) -> &'static str {
    match mode {
        PresentMode::Immediate => "immediate",
        PresentMode::Mailbox => "mailbox",
        PresentMode::Fifo => "fifo",
        PresentMode::FifoRelaxed => "fiforelaxed",
        PresentMode::AutoVsync => "autovsync",
        PresentMode::AutoNoVsync => "autonovsync",
    }
}

pub fn parse_label(value: &str) -> Result<String, ProbeError> {
    if value.is_empty()
        || value
            .chars()
            .any(|char| matches!(char, ',' | '"') || char.is_control())
    {
        return Err(ProbeError::InvalidLabel(value.to_string()));
    }
    Ok(value.to_string())
}

pub fn world_instances_ready(world: &World) -> bool {
    let Some(mut roots) = world.try_query_filtered::<Entity, With<WorldAssetRoot>>() else {
        return true;
    };
    let spawner = world.get_resource::<WorldInstanceSpawner>();
    roots.iter(world).all(|root| {
        world
            .get::<WorldInstance>(root)
            .zip(spawner)
            .is_some_and(|(instance, spawner)| spawner.instance_is_ready(**instance))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Readiness {
    Waiting,
    Ready,
    Failed(String),
}

type ReadyFn = Arc<dyn Fn(&World) -> Readiness + Send + Sync>;

pub struct ProbePlugin {
    config: ProbeConfig,
    ready: ReadyFn,
}

impl ProbePlugin {
    pub fn new(
        config: ProbeConfig,
        ready: impl Fn(&World) -> Readiness + Send + Sync + 'static,
    ) -> Result<Self, ProbeError> {
        config.validate()?;
        Ok(Self {
            config,
            ready: Arc::new(ready),
        })
    }
}

impl Plugin for ProbePlugin {
    fn build(&self, app: &mut App) {
        let config = &self.config;
        configure_window(app, config);
        info!(
            "probe: armed label={} warmup={} frames={} res={}x{} present={} out={} timeout={}s",
            config.label,
            config.warmup,
            config.frames,
            config.resolution.x,
            config.resolution.y,
            present_mode_name(config.present_mode),
            config.out.as_ref().map_or_else(
                || "none".to_string(),
                |out| out.path().display().to_string()
            ),
            config.timeout.as_secs_f32(),
        );
        app.insert_resource(WinitSettings::continuous())
            .insert_resource(config.clone())
            .insert_resource(ProbeReady(self.ready.clone()))
            .insert_resource(ProbeState {
                samples: Vec::with_capacity(config.frames as usize),
                ..default()
            })
            .add_systems(Update, (measure, wait_ready.run_if(probe_waiting)).chain())
            .add_systems(Last, refuse_early_exit.after(ExitSystems));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    WaitReady,
    Warmup,
    Capture,
    Done,
    Failed,
}

#[derive(Resource, Debug, Default)]
pub struct ProbeState {
    phase: Phase,
    started: Option<Duration>,
    warmed: u32,
    samples: Vec<f64>,
    report: Option<ProbeReport>,
    failure: Option<String>,
}

impl ProbeState {
    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn samples(&self) -> &[f64] {
        &self.samples
    }

    pub fn report(&self) -> Option<&ProbeReport> {
        self.report.as_ref()
    }

    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    fn fail(&mut self, reason: String) -> AppExit {
        error!(
            "probe: FAILED phase={:?} warmed={} captured={} - {reason}. No stats were written.",
            self.phase,
            self.warmed,
            self.samples.len()
        );
        self.phase = Phase::Failed;
        self.failure = Some(reason);
        AppExit::error()
    }
}

#[derive(Resource)]
struct ProbeReady(ReadyFn);

fn configure_window(app: &mut App, config: &ProbeConfig) {
    let world = app.world_mut();
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let Ok(mut window) = windows.single_mut(world) else {
        panic!("ProbePlugin needs exactly one primary window; add it after DefaultPlugins");
    };
    window
        .resolution
        .set(config.resolution.x as f32, config.resolution.y as f32);
    window.present_mode = config.present_mode;
    window.resizable = false;
}

fn probe_waiting(state: Res<ProbeState>) -> bool {
    state.phase == Phase::WaitReady
}

fn wait_ready(world: &mut World) {
    let now = world.resource::<Time<Real>>().elapsed();
    let timeout = world.resource::<ProbeConfig>().timeout;
    let ready = world.resource::<ProbeReady>().0.clone();
    let readiness = ready(world);
    let mut state = world.resource_mut::<ProbeState>();
    let started = *state.started.get_or_insert(now);
    let exit = match readiness {
        Readiness::Ready => {
            info!(
                "probe: scene ready after {:.2}s, discarding warm-up frames",
                (now - started).as_secs_f32()
            );
            state.phase = Phase::Warmup;
            None
        }
        Readiness::Failed(reason) => Some(state.fail(format!("scene failed to load: {reason}"))),
        Readiness::Waiting if now - started >= timeout => Some(state.fail(format!(
            "scene was not ready after {:.0}s",
            timeout.as_secs_f32()
        ))),
        Readiness::Waiting => None,
    };
    if let Some(exit) = exit {
        world.write_message(exit);
    }
}

fn measure(
    time: Res<Time<Real>>,
    config: Res<ProbeConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    winit: Option<Res<WinitSettings>>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut state: ResMut<ProbeState>,
    mut exit: MessageWriter<AppExit>,
) {
    if !matches!(state.phase, Phase::Warmup | Phase::Capture) {
        return;
    }
    let window = match check_environment(&config, windows.single(), winit.as_deref()) {
        Ok(window) => window,
        Err(reason) => {
            exit.write(state.fail(reason));
            return;
        }
    };
    if state.phase == Phase::Warmup {
        state.warmed += 1;
        if state.warmed >= config.warmup {
            state.phase = Phase::Capture;
            info!("probe: warm-up done, capturing {} frames", config.frames);
        }
        return;
    }
    state.samples.push(time.delta_secs_f64() * 1000.0);
    if state.samples.len() < config.frames as usize {
        return;
    }
    let stats = match FrameStats::from_samples(&state.samples) {
        Ok(stats) => stats,
        Err(error) => {
            exit.write(state.fail(error.to_string()));
            return;
        }
    };
    let report = ProbeReport {
        label: config.label.clone(),
        stats,
        warmup: config.warmup,
        width: config.resolution.x,
        height: config.resolution.y,
        physical_width: window.resolution.physical_width(),
        physical_height: window.resolution.physical_height(),
        scale_factor: window.resolution.scale_factor(),
        present_mode: present_mode_name(config.present_mode).to_string(),
        adapter: adapter
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |info| info.0.name.clone()),
        backend: adapter
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |info| info.0.backend.to_string()),
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_string(),
    };
    if let Some(out) = &config.out {
        if let Err(error) = out.write(&report) {
            exit.write(state.fail(format!("could not write stats: {error}")));
            return;
        }
        info!("probe: wrote {}", out.path().display());
    }
    info!("{}", report.summary_line());
    state.phase = Phase::Done;
    state.report = Some(report);
    exit.write(AppExit::Success);
}

fn check_environment<'w>(
    config: &ProbeConfig,
    window: Result<&'w Window, QuerySingleError>,
    winit: Option<&WinitSettings>,
) -> Result<&'w Window, String> {
    if let Some(winit) = winit {
        let focused = winit.update_mode(true);
        let unfocused = winit.update_mode(false);
        if !matches!(focused, UpdateMode::Continuous)
            || !matches!(unfocused, UpdateMode::Continuous)
        {
            return Err(format!(
                "WinitSettings paces the app at focused={focused:?} unfocused={unfocused:?}; the probe needs Continuous for both"
            ));
        }
    }
    let window = match window {
        Ok(window) => window,
        Err(QuerySingleError::NoEntities(_)) => {
            return Err("the primary window is missing".to_string())
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            return Err("there is more than one primary window".to_string())
        }
    };
    let (width, height) = (window.resolution.width(), window.resolution.height());
    let (want_width, want_height) = (config.resolution.x as f32, config.resolution.y as f32);
    if (width - want_width).abs() >= 0.5 || (height - want_height).abs() >= 0.5 {
        return Err(format!(
            "the primary window is {width}x{height} but the probe needs {want_width}x{want_height}; float the window or stop the window manager from resizing it"
        ));
    }
    if window.present_mode != config.present_mode {
        return Err(format!(
            "the primary window present mode is {} but the probe needs {}",
            present_mode_name(window.present_mode),
            present_mode_name(config.present_mode)
        ));
    }
    Ok(window)
}

fn refuse_early_exit(
    mut cursor: Local<MessageCursor<AppExit>>,
    mut state: ResMut<ProbeState>,
    mut exits: ResMut<Messages<AppExit>>,
) {
    let exiting = cursor.read(&exits).next().is_some();
    if !exiting
        || !matches!(
            state.phase,
            Phase::WaitReady | Phase::Warmup | Phase::Capture
        )
    {
        return;
    }
    let exit = state.fail("the app exited before the capture finished".to_string());
    exits.write(exit);
}
