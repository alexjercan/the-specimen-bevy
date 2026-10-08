use std::{
    io::{self, BufRead, Write},
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use capture::{
    loops::{loop_end, loop_start, loop_written},
    CaptureState,
};
use std::path::PathBuf;

use bevy::{
    app::{App, AppExit, Main, Plugin},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
    prelude::{Local, Resource, World},
    time::{TimeReceiver, TimeUpdateStrategy},
    winit::{EventLoopProxyWrapper, WinitSettings, WinitUserEvent},
};

use crate::{
    perception,
    transport::{snapshot, Command, Error, FRAME_TIME},
};

#[derive(Resource, Default)]
pub struct TransportTimeline {
    updates: u64,
    pub first_tick: Option<u64>,
}

impl TransportTimeline {
    pub fn ready(&mut self) {
        self.first_tick.get_or_insert(self.updates + 1);
    }

    pub fn tick(&self) -> Option<u64> {
        self.first_tick
            .and_then(|first| self.updates.checked_sub(first))
    }

    pub(crate) fn updating(&self) -> Option<u64> {
        self.first_tick.map(|first| self.updates + 1 - first)
    }

    pub fn advance(&mut self) {
        self.updates += 1;
    }
}

pub struct RenderedTransportPlugin;

#[derive(Resource)]
pub struct RecordTransport(pub PathBuf);

struct Request {
    tick: u64,
    next_frame: Instant,
}

#[derive(Resource)]
struct RenderedTransport {
    incoming: Mutex<Receiver<String>>,
    pending: Option<Request>,
    finalizing: Option<Instant>,
}

impl Plugin for RenderedTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME))
            .insert_resource(WinitSettings::desktop_app())
            .init_resource::<TransportTimeline>();
        perception::build(app);
    }

    fn finish(&self, app: &mut App) {
        let proxy = (**app.world().resource::<EventLoopProxyWrapper>()).clone();
        let (sender, incoming) = mpsc::channel();
        thread::spawn(move || {
            for line in io::stdin().lock().lines() {
                match line {
                    Ok(line) => {
                        if sender.send(line).is_err() {
                            return;
                        }
                        let _ = proxy.send_event(WinitUserEvent::WakeUp);
                    }
                    Err(error) => {
                        eprintln!("transport read failed: {error}");
                        break;
                    }
                }
            }
            let _ = proxy.send_event(WinitUserEvent::WakeUp);
        });
        app.world_mut().insert_resource(RenderedTransport {
            incoming: Mutex::new(incoming),
            pending: None,
            finalizing: None,
        });
        if let Some(receiver) = app.world_mut().remove_resource::<TimeReceiver>() {
            discard_render_times(receiver);
        }
        let mut main = app
            .world_mut()
            .resource_mut::<Schedules>()
            .remove(Main)
            .expect("Main schedule exists");
        main.remove_systems_in_set(
            Main::run_main,
            app.world_mut(),
            ScheduleCleanupPolicy::RemoveSetAndSystems,
        )
        .expect("Main runner system exists");
        main.add_systems(step);
        app.world_mut().resource_mut::<Schedules>().reinsert(main);
    }
}

fn reply(value: &impl serde::Serialize) -> bool {
    match serde_json::to_string(value) {
        Ok(json) => write_line(&json),
        Err(_) => false,
    }
}

fn write_line(line: &str) -> bool {
    let mut output = io::stdout().lock();
    writeln!(output, "{line}")
        .and_then(|()| output.flush())
        .is_ok()
}

fn step(world: &mut World, started: Local<bool>) {
    if !*started || world.resource::<TransportTimeline>().first_tick.is_none() {
        run_main(world, started);
        if let Some(tick) = world.resource::<TransportTimeline>().tick() {
            if tick == 0 {
                if let Some(path) = world
                    .get_resource::<RecordTransport>()
                    .map(|record| record.0.clone())
                {
                    loop_start(world, path.to_string_lossy().as_ref());
                }
            }
            if !write_line(&snapshot(world, tick)) {
                world.write_message(AppExit::error());
            }
        }
        wake(world);
        return;
    }

    let mut state = world.remove_resource::<RenderedTransport>().unwrap();
    if let Some(start) = state.finalizing {
        run_main(world, started);
        let capture = world.resource::<CaptureState>();
        let failed = capture.failed().is_some();
        let done = capture.pending() == 0;
        if failed || done || start.elapsed() > Duration::from_secs(120) {
            let written = world
                .get_resource::<RecordTransport>()
                .is_some_and(|record| loop_written(world, record.0.to_string_lossy().as_ref()));
            world.write_message(if written {
                AppExit::Success
            } else {
                AppExit::error()
            });
        } else {
            wake(world);
        }
        world.insert_resource(state);
        return;
    }
    if state.pending.is_none() {
        loop {
            let received = state.incoming.lock().unwrap().try_recv();
            match received {
                Ok(line) if line.trim().is_empty() => continue,
                Ok(line) => match serde_json::from_str::<Command>(&line) {
                    Ok(command)
                        if command.tick
                            <= world.resource::<TransportTimeline>().tick().unwrap() =>
                    {
                        if !reply(&Error {
                            error: "tick must be greater than the current tick",
                        }) {
                            world.write_message(AppExit::error());
                        }
                    }
                    Ok(command) if !command.input.valid() => {
                        if !reply(&Error {
                            error: "look values must be finite",
                        }) {
                            world.write_message(AppExit::error());
                        }
                    }
                    Ok(command) => {
                        command.input.apply(world);
                        state.pending = Some(Request {
                            tick: command.tick,
                            next_frame: Instant::now() + FRAME_TIME,
                        });
                        break;
                    }
                    Err(_) => {
                        if !reply(&Error {
                            error: "expected a JSON object with tick and optional input controls",
                        }) {
                            world.write_message(AppExit::error());
                        }
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if world.contains_resource::<RecordTransport>() {
                        loop_end(world);
                        state.finalizing = Some(Instant::now());
                        wake(world);
                    } else {
                        world.write_message(AppExit::Success);
                    }
                    break;
                }
            }
        }
    }
    if let Some(request) = &mut state.pending {
        pace(&mut request.next_frame);
        run_main(world, started);
        let tick = world.resource::<TransportTimeline>().tick().unwrap();
        if tick == request.tick {
            if !write_line(&snapshot(world, tick)) {
                world.write_message(AppExit::error());
            }
            state.pending = None;
        } else {
            wake(world);
        }
    }
    world.insert_resource(state);
}

fn pace(next_frame: &mut Instant) {
    thread::sleep(next_frame.saturating_duration_since(Instant::now()));
    *next_frame = Instant::now() + FRAME_TIME;
}

fn discard_render_times(receiver: TimeReceiver) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("transport-render-time".into())
        .spawn(move || while receiver.0.recv().is_ok() {})
        .expect("render time receiver thread starts")
}

fn run_main(world: &mut World, started: Local<bool>) {
    Main::run_main(world, started);
    world.resource_mut::<TransportTimeline>().advance();
}

fn wake(world: &World) {
    let _ = world
        .resource::<EventLoopProxyWrapper>()
        .send_event(WinitUserEvent::WakeUp);
}

#[cfg(test)]
#[path = "../tests/unit/rendered.rs"]
mod tests;
