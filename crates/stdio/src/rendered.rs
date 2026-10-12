use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, TryRecvError},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use bevy::{
    app::{App, AppExit, Main, Plugin, PostUpdate},
    ecs::schedule::{ScheduleCleanupPolicy, Schedules},
    prelude::{Local, Resource, World},
    time::{TimeReceiver, TimeUpdateStrategy},
    winit::{EventLoopProxyWrapper, WinitSettings, WinitUserEvent},
};
use capture::{
    video::{video_close_at_exit, video_end, video_start, video_written},
    CaptureState,
};

use crate::{
    mark_ready,
    protocol::{parse, reply, Failure},
    settle, Playable, StdioTimeline,
};

pub struct RenderedStdio<P> {
    step: Duration,
    playable: PhantomData<P>,
}

impl<P> RenderedStdio<P> {
    pub fn new(step: Duration) -> Self {
        Self {
            step,
            playable: PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct Recording(pub PathBuf);

struct Request {
    tick: u64,
    next_frame: Instant,
}

#[derive(Resource)]
struct Incoming {
    step: Duration,
    lines: Mutex<Receiver<String>>,
    pending: Option<Request>,
    finalizing: bool,
}

impl<P: Playable> Plugin for RenderedStdio<P> {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(self.step))
            .insert_resource(WinitSettings::desktop_app())
            .init_resource::<StdioTimeline>()
            .add_systems(PostUpdate, mark_ready::<P>);
        P::build(app);
    }

    fn finish(&self, app: &mut App) {
        let proxy = (**app.world().resource::<EventLoopProxyWrapper>()).clone();
        let (sender, lines) = mpsc::channel();
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
                        eprintln!("stdio read failed: {error}");
                        break;
                    }
                }
            }
            let _ = proxy.send_event(WinitUserEvent::WakeUp);
        });
        app.world_mut().insert_resource(Incoming {
            step: self.step,
            lines: Mutex::new(lines),
            pending: None,
            finalizing: false,
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
        main.add_systems(step::<P>);
        app.world_mut().resource_mut::<Schedules>().reinsert(main);
    }
}

fn write_line(line: &str) -> bool {
    let mut output = io::stdout().lock();
    writeln!(output, "{line}")
        .and_then(|()| output.flush())
        .is_ok()
}

fn fail(world: &mut World, failure: Failure) {
    if !write_line(&failure.line()) {
        world.write_message(AppExit::error());
    }
}

fn step<P: Playable>(world: &mut World, started: Local<bool>) {
    advance::<P>(world, started);
    if world.contains_resource::<Recording>() {
        video_close_at_exit(world);
    }
}

fn advance<P: Playable>(world: &mut World, started: Local<bool>) {
    if !*started || world.resource::<StdioTimeline>().first_tick.is_none() {
        run_main::<P>(world, started);
        if let Some(tick) = world.resource::<StdioTimeline>().tick() {
            if tick == 0 {
                if let Some(path) = world
                    .get_resource::<Recording>()
                    .map(|record| record.0.clone())
                {
                    video_start(world, path.to_string_lossy().as_ref());
                }
            }
            if !write_line(&reply::<P>(world, tick)) {
                world.write_message(AppExit::error());
            }
        }
        wake(world);
        return;
    }

    let mut state = world.remove_resource::<Incoming>().unwrap();
    if state.finalizing {
        run_main::<P>(world, started);
        let capture = world.resource::<CaptureState>();
        if capture.failed().is_some() || capture.pending() == 0 {
            let written = world
                .get_resource::<Recording>()
                .is_some_and(|record| video_written(world, record.0.to_string_lossy().as_ref()));
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
            let received = state.lines.lock().unwrap().try_recv();
            match received {
                Ok(line) if line.trim().is_empty() => continue,
                Ok(line) => {
                    let current = world.resource::<StdioTimeline>().tick().unwrap();
                    match parse::<P>(&line, current) {
                        Ok(command) => match P::apply(&command.input, world) {
                            Ok(()) => {
                                state.pending = Some(Request {
                                    tick: command.tick,
                                    next_frame: Instant::now() + state.step,
                                });
                                break;
                            }
                            Err(error) => fail(world, Failure::new(error)),
                        },
                        Err(failure) => fail(world, failure),
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if world.contains_resource::<Recording>() {
                        video_end(world);
                        state.finalizing = true;
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
        pace(&mut request.next_frame, state.step);
        run_main::<P>(world, started);
        let tick = world.resource::<StdioTimeline>().tick().unwrap();
        if tick == request.tick {
            if !write_line(&reply::<P>(world, tick)) {
                world.write_message(AppExit::error());
            }
            state.pending = None;
        }
        // NOTE: wake after a reply too; lines sent in one burst already spent their wake-ups.
        wake(world);
    }
    world.insert_resource(state);
}

fn pace(next_frame: &mut Instant, step: Duration) {
    thread::sleep(next_frame.saturating_duration_since(Instant::now()));
    *next_frame = Instant::now() + step;
}

fn discard_render_times(receiver: TimeReceiver) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("stdio-render-time".into())
        .spawn(move || while receiver.0.recv().is_ok() {})
        .expect("render time receiver thread starts")
}

fn run_main<P: Playable>(world: &mut World, started: Local<bool>) {
    Main::run_main(world, started);
    settle::<P>(world);
}

fn wake(world: &World) {
    let _ = world
        .resource::<EventLoopProxyWrapper>()
        .send_event(WinitUserEvent::WakeUp);
}
