use std::{
    io::{self, BufRead, Write},
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Mutex,
    },
    thread,
};

use bevy::{
    app::{App, AppExit, Main, Plugin},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
    prelude::{Local, Resource, World},
    time::TimeUpdateStrategy,
    winit::{EventLoopProxyWrapper, WinitSettings, WinitUserEvent},
};

use crate::transport::{snapshot, Command, Error, FRAME_TIME};

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
        self.first_tick.map(|first| self.updates - first)
    }

    pub fn advance(&mut self) {
        self.updates += 1;
    }
}

pub struct RenderedTransportPlugin;

struct Request {
    tick: u64,
}

#[derive(Resource)]
struct RenderedTransport {
    incoming: Mutex<Receiver<String>>,
    pending: Option<Request>,
}

impl Plugin for RenderedTransportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_TIME))
            .insert_resource(WinitSettings::desktop_app())
            .init_resource::<TransportTimeline>();
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
        });
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
    let mut output = io::stdout().lock();
    match serde_json::to_string(value) {
        Ok(json) => writeln!(output, "{json}")
            .and_then(|()| output.flush())
            .is_ok(),
        Err(_) => false,
    }
}

fn step(world: &mut World, started: Local<bool>) {
    if !*started || world.resource::<TransportTimeline>().first_tick.is_none() {
        run_main(world, started);
        wake(world);
        return;
    }

    let mut state = world.remove_resource::<RenderedTransport>().unwrap();
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
                        state.pending = Some(Request { tick: command.tick });
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
                    world.write_message(AppExit::Success);
                    break;
                }
            }
        }
    }
    if let Some(request) = &state.pending {
        run_main(world, started);
        let tick = world.resource::<TransportTimeline>().tick().unwrap();
        if tick == request.tick {
            if !reply(&snapshot(world, tick)) {
                world.write_message(AppExit::error());
            }
            state.pending = None;
        } else {
            wake(world);
        }
    }
    world.insert_resource(state);
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
