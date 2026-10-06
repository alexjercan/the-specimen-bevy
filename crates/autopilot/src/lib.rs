use std::sync::Arc;

use bevy::{input::InputSystems, prelude::*};

pub mod predicate;

pub use predicate::{
    and, any_entity, elapsed, entity_count, frames, not, or, resource_where, state_is, Predicate,
};

pub const DEFAULT_DEADLINE_SECS: f32 = 120.0;

type Action = dyn Fn(&mut World) + Send + Sync;

#[derive(Clone)]
struct Expectation {
    predicate: Arc<Predicate>,
    message: String,
}

#[derive(Clone)]
struct ConditionTimeout {
    predicate: Arc<Predicate>,
    deadline_secs: f32,
}

#[derive(Clone)]
struct Step {
    name: String,
    actions: Vec<Arc<Action>>,
    condition_timeout: Option<ConditionTimeout>,
    expectation: Option<Expectation>,
    acted: bool,
}

pub struct AutopilotPlugin {
    deadline_secs: f32,
    steps: Vec<Step>,
}

impl Default for AutopilotPlugin {
    fn default() -> Self {
        Self {
            deadline_secs: DEFAULT_DEADLINE_SECS,
            steps: Vec::new(),
        }
    }
}

impl AutopilotPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_deadline_secs(mut self, deadline_secs: f32) -> Self {
        assert!(
            valid_deadline(deadline_secs),
            "autopilot run deadline must be finite and positive"
        );
        self.deadline_secs = deadline_secs;
        self
    }

    pub fn step(self, name: impl Into<String>) -> StepBuilder {
        StepBuilder {
            plugin: self,
            step: Step {
                name: name.into(),
                actions: Vec::new(),
                condition_timeout: None,
                expectation: None,
                acted: false,
            },
        }
    }
}

pub struct StepBuilder {
    plugin: AutopilotPlugin,
    step: Step,
}

impl StepBuilder {
    pub fn act(mut self, action: impl Fn(&mut World) + Send + Sync + 'static) -> Self {
        self.step.actions.push(Arc::new(action));
        self
    }

    pub fn until(mut self, predicate: Arc<Predicate>, deadline_secs: f32) -> Self {
        self.step.condition_timeout = Some(ConditionTimeout {
            predicate,
            deadline_secs,
        });
        self
    }

    pub fn expect(mut self, predicate: Arc<Predicate>, message: impl Into<String>) -> Self {
        self.step.expectation = Some(Expectation {
            predicate,
            message: message.into(),
        });
        self
    }

    pub fn add(mut self) -> AutopilotPlugin {
        self.plugin.steps.push(self.step);
        self.plugin
    }
}

#[derive(Resource, Default)]
pub struct AutopilotClock {
    pub step_elapsed: f32,
    pub step_frames: u64,
    step_real: f32,
    run_real: f32,
}

#[derive(Resource, Debug, PartialEq, Eq)]
pub enum AutopilotStatus {
    Running,
    Success,
    Failed(String),
}

#[derive(Resource)]
struct Script {
    steps: Vec<Step>,
    index: usize,
    deadline_secs: f32,
}

fn valid_deadline(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

impl Plugin for AutopilotPlugin {
    fn build(&self, app: &mut App) {
        assert!(!self.steps.is_empty(), "armed autopilot has no steps");
        for step in &self.steps {
            assert!(
                !step.name.trim().is_empty(),
                "autopilot step name must not be empty"
            );
            if let Some(condition_timeout) = &step.condition_timeout {
                assert!(
                    valid_deadline(condition_timeout.deadline_secs),
                    "autopilot step deadline must be finite and positive"
                );
            }
        }
        app.insert_resource(Script {
            steps: self.steps.clone(),
            index: 0,
            deadline_secs: self.deadline_secs,
        })
        .init_resource::<AutopilotClock>()
        .insert_resource(AutopilotStatus::Running)
        .add_systems(
            PreUpdate,
            drive.run_if(autopilot_running).after(InputSystems),
        )
        .add_systems(
            Update,
            exit_on_status_change.run_if(resource_changed::<AutopilotStatus>),
        )
        .add_systems(PostUpdate, check_run_timeout.run_if(autopilot_running));
    }
}

fn autopilot_running(status: Res<AutopilotStatus>) -> bool {
    matches!(*status, AutopilotStatus::Running)
}

fn drive(world: &mut World) {
    let mut script = world
        .remove_resource::<Script>()
        .expect("armed script is present");

    if script.index < script.steps.len() {
        let real_delta = world.resource::<Time<Real>>().delta_secs();
        match run_step(world, &mut script.steps[script.index], real_delta) {
            StepOutcome::Ok => advance_step(&mut script),
            StepOutcome::Err(reason) => fail(world, reason),
            StepOutcome::Pending => {}
        }
    }
    if script.index >= script.steps.len() {
        complete(world);
    }
    world.insert_resource(script);
}

fn check_run_timeout(world: &mut World) {
    let real_delta = world.resource::<Time<Real>>().delta_secs();
    let mut clock = world.resource_mut::<AutopilotClock>();
    clock.run_real += real_delta;
    let elapsed = clock.run_real;
    let script = world.resource::<Script>();
    if elapsed >= script.deadline_secs {
        let name = &script.steps[script.index].name;
        fail(
            world,
            format!("autopilot: run deadline expired at `{name}`"),
        );
    }
}

fn run_step(world: &mut World, step: &mut Step, real_delta: f32) -> StepOutcome {
    if !step.acted {
        start_step(world, step);
        step.acted = true;
        match step.condition_timeout {
            Some(_) => StepOutcome::Pending,
            None => step_outcome(world, step),
        }
    } else {
        let delta = world.resource::<Time>().delta_secs();
        let mut clock = world.resource_mut::<AutopilotClock>();
        clock.step_elapsed += delta;
        clock.step_frames += 1;
        clock.step_real += real_delta;
        step_outcome(world, step)
    }
}

fn start_step(world: &mut World, step: &Step) {
    let mut clock = world.resource_mut::<AutopilotClock>();
    clock.step_elapsed = 0.0;
    clock.step_frames = 0;
    clock.step_real = 0.0;
    info!("autopilot: step `{}` begins", step.name);
    for action in &step.actions {
        action(world);
    }
}

enum StepOutcome {
    Ok,
    Err(String),
    Pending,
}

fn step_outcome(world: &World, step: &Step) -> StepOutcome {
    if let Some(condition_timeout) = &step.condition_timeout {
        if !(condition_timeout.predicate)(world) {
            if world.resource::<AutopilotClock>().step_real >= condition_timeout.deadline_secs {
                return StepOutcome::Err(format!("autopilot: step `{}` timed out", step.name));
            }
            return StepOutcome::Pending;
        }
    }
    if let Some(expectation) = &step.expectation {
        if !(expectation.predicate)(world) {
            return StepOutcome::Err(format!(
                "autopilot: step `{}` failed expectation: {}",
                step.name, expectation.message
            ));
        }
    }
    StepOutcome::Ok
}

fn advance_step(script: &mut Script) {
    script.index += 1;
}

fn complete(world: &mut World) {
    info!("autopilot: complete");
    *world.resource_mut::<AutopilotStatus>() = AutopilotStatus::Success;
}

fn fail(world: &mut World, message: String) {
    error!("{message}");
    *world.resource_mut::<AutopilotStatus>() = AutopilotStatus::Failed(message);
}

fn exit_on_status_change(status: Res<AutopilotStatus>, mut exit: MessageWriter<AppExit>) {
    match &*status {
        AutopilotStatus::Running => {}
        AutopilotStatus::Success => {
            exit.write(AppExit::Success);
        }
        AutopilotStatus::Failed(_) => {
            exit.write(AppExit::error());
        }
    }
}
