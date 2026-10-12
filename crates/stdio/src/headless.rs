use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    time::Duration,
};

use bevy::{
    app::{App, AppExit, Plugin, PluginsState, PostUpdate},
    time::TimeUpdateStrategy,
};

use crate::{
    mark_ready,
    protocol::{parse, reply, Failure},
    settle, Playable, StdioTimeline,
};

pub struct HeadlessStdio<P> {
    step: Duration,
    playable: PhantomData<P>,
}

impl<P> HeadlessStdio<P> {
    pub fn new(step: Duration) -> Self {
        Self {
            step,
            playable: PhantomData,
        }
    }
}

impl<P: Playable> Plugin for HeadlessStdio<P> {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(self.step))
            .init_resource::<StdioTimeline>()
            .add_systems(PostUpdate, mark_ready::<P>);
        P::build(app);
        app.set_runner(|app| run::<P>(app, io::stdin().lock(), io::stdout().lock()));
    }
}

fn step<P: Playable>(app: &mut App) -> Option<AppExit> {
    app.update();
    settle::<P>(app.world_mut());
    app.should_exit()
}

pub fn run<P: Playable>(mut app: App, input: impl BufRead, mut output: impl Write) -> AppExit {
    if app.plugins_state() != PluginsState::Cleaned {
        while app.plugins_state() == PluginsState::Adding {
            #[cfg(not(target_arch = "wasm32"))]
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }
    while app.world().resource::<StdioTimeline>().first_tick.is_none() {
        if let Some(exit) = step::<P>(&mut app) {
            return exit;
        }
    }

    let mut tick = app.world().resource::<StdioTimeline>().tick().unwrap();
    if writeln!(output, "{}", reply::<P>(app.world_mut(), tick))
        .and_then(|()| output.flush())
        .is_err()
    {
        return AppExit::error();
    }
    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("stdio read failed: {error}");
                return AppExit::error();
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let result = match parse::<P>(&line, tick) {
            Ok(command) => match P::apply(&command.input, app.world_mut()) {
                Ok(()) => {
                    while tick < command.tick {
                        if let Some(exit) = step::<P>(&mut app) {
                            return exit;
                        }
                        tick = app.world().resource::<StdioTimeline>().tick().unwrap();
                    }
                    reply::<P>(app.world_mut(), tick)
                }
                Err(error) => Failure::new(error).line(),
            },
            Err(failure) => failure.line(),
        };
        if writeln!(output, "{result}")
            .and_then(|()| output.flush())
            .is_err()
        {
            return AppExit::error();
        }
    }
    AppExit::Success
}
