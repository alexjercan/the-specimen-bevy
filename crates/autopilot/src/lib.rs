use std::sync::Arc;

use bevy::{input::InputSystems, prelude::*};

pub mod predicate;

pub use predicate::{
    and, any_entity, elapsed, entity_count, frames, not, or, resource_where, state_is, Predicate,
};

pub const DEFAULT_DEADLINE_SECS: f32 = 120.0;

type Action = dyn Fn(&mut World) + Send + Sync;
type Diagnosis = dyn Fn(&World) -> String + Send + Sync;

#[derive(Clone)]
struct Wait {
    predicate: Arc<Predicate>,
    deadline_secs: f32,
}

#[derive(Clone)]
struct Step {
    name: String,
    actions: Vec<Arc<Action>>,
    wait: Option<Wait>,
    expectation: Arc<Predicate>,
    diagnosis: Option<Arc<Diagnosis>>,
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
                wait: None,
                expectation: Arc::new(|_| true),
                diagnosis: None,
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
        self.step.wait = Some(Wait {
            predicate,
            deadline_secs,
        });
        self
    }

    pub fn expect(mut self, predicate: Arc<Predicate>) -> Self {
        self.step.expectation = predicate;
        self
    }

    pub fn diagnose(
        mut self,
        diagnosis: impl Fn(&World) -> String + Send + Sync + 'static,
    ) -> Self {
        self.step.diagnosis = Some(Arc::new(diagnosis));
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

#[derive(Resource, Default)]
pub struct CompletionGates {
    pending: usize,
}

impl CompletionGates {
    pub fn register(&mut self) {
        self.pending += 1;
    }

    pub fn finish(&mut self) {
        assert!(self.pending > 0, "no autopilot collector is pending");
        self.pending -= 1;
    }

    pub fn pending(&self) -> usize {
        self.pending
    }
}

#[derive(Resource, Debug, PartialEq, Eq)]
pub enum AutopilotStatus {
    Running,
    WaitingForCollectors,
    Success,
    Failed(String),
}

#[derive(Resource)]
struct Script {
    steps: Arc<[Step]>,
    index: usize,
    acted: bool,
    completed: bool,
    done: bool,
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
            if let Some(wait) = &step.wait {
                assert!(
                    valid_deadline(wait.deadline_secs),
                    "autopilot step deadline must be finite and positive"
                );
            }
        }
        app.insert_resource(Script {
            steps: self.steps.clone().into(),
            index: 0,
            acted: false,
            completed: false,
            done: false,
            deadline_secs: self.deadline_secs,
        })
        .init_resource::<AutopilotClock>()
        .init_resource::<CompletionGates>()
        .insert_resource(AutopilotStatus::Running)
        .add_systems(PreUpdate, drive.after(InputSystems));
    }
}

fn drive(world: &mut World) {
    let mut script = world
        .remove_resource::<Script>()
        .expect("armed script is present");
    if script.done {
        world.insert_resource(script);
        return;
    }

    let real_delta = world.resource::<Time<Real>>().delta_secs();
    world.resource_mut::<AutopilotClock>().run_real += real_delta;
    if world.resource::<AutopilotClock>().run_real >= script.deadline_secs {
        let name = script
            .steps
            .get(script.index)
            .map_or("collectors", |step| &step.name);
        fail(
            world,
            format!("autopilot: run deadline expired at `{name}`"),
        );
        script.done = true;
    } else if script.completed {
        if world.resource::<CompletionGates>().pending() == 0 {
            complete(world);
            script.done = true;
        }
    } else {
        let step = &script.steps[script.index];
        if !script.acted {
            info!("autopilot: step `{}` begins", step.name);
            for action in &step.actions {
                action(world);
            }
            script.acted = true;
            let mut clock = world.resource_mut::<AutopilotClock>();
            clock.step_elapsed = 0.0;
            clock.step_frames = 0;
            clock.step_real = 0.0;
            if step.wait.is_none() {
                if !(step.expectation)(world) {
                    fail(
                        world,
                        format!(
                            "autopilot: step `{}` failed expectation{}",
                            step.name,
                            diagnosis(step, world)
                        ),
                    );
                    script.done = true;
                } else {
                    advance_step(world, &mut script);
                }
            }
        } else {
            let delta = world.resource::<Time>().delta_secs();
            let step_real = {
                let mut clock = world.resource_mut::<AutopilotClock>();
                clock.step_elapsed += delta;
                clock.step_frames += 1;
                clock.step_real += real_delta;
                clock.step_real
            };
            if step
                .wait
                .as_ref()
                .is_some_and(|wait| !(wait.predicate)(world))
            {
                let wait = step.wait.as_ref().expect("wait exists");
                if step_real >= wait.deadline_secs {
                    fail(
                        world,
                        format!(
                            "autopilot: step `{}` timed out{}",
                            step.name,
                            diagnosis(step, world)
                        ),
                    );
                    script.done = true;
                }
            } else if !(step.expectation)(world) {
                fail(
                    world,
                    format!(
                        "autopilot: step `{}` failed expectation{}",
                        step.name,
                        diagnosis(step, world)
                    ),
                );
                script.done = true;
            } else {
                advance_step(world, &mut script);
            }
        }
    }
    world.insert_resource(script);
}

fn advance_step(world: &mut World, script: &mut Script) {
    script.index += 1;
    script.acted = false;
    if script.index == script.steps.len() {
        script.completed = true;
        if world.resource::<CompletionGates>().pending() == 0 {
            complete(world);
            script.done = true;
        } else {
            *world.resource_mut::<AutopilotStatus>() = AutopilotStatus::WaitingForCollectors;
        }
    }
}

fn diagnosis(step: &Step, world: &World) -> String {
    step.diagnosis
        .as_ref()
        .map(|f| format!(" - {}", f(world)))
        .unwrap_or_default()
}

fn complete(world: &mut World) {
    info!("autopilot: complete");
    *world.resource_mut::<AutopilotStatus>() = AutopilotStatus::Success;
    world.write_message(AppExit::Success);
}

fn fail(world: &mut World, message: String) {
    error!("{message}");
    *world.resource_mut::<AutopilotStatus>() = AutopilotStatus::Failed(message);
    world.write_message(AppExit::error());
}
