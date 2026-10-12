mod headless;
mod protocol;
mod rendered;

use bevy::prelude::{App, Resource, World};
use serde::{de::DeserializeOwned, Serialize};

pub use headless::{run, HeadlessStdio};
pub use protocol::{reply, Command, Failure, Reply, VERSION};
pub use rendered::{Recording, RenderedStdio};

pub trait Playable: Send + Sync + 'static {
    type Input: DeserializeOwned + Default + Send + Sync + 'static;
    type View: Serialize;

    fn build(_app: &mut App) {}
    fn ready(world: &World) -> bool;
    fn apply(input: &Self::Input, world: &mut World) -> Result<(), String>;
    fn observe(world: &mut World, tick: u64) -> Self::View;
}

#[derive(Resource, Default)]
pub struct StdioTimeline {
    updates: u64,
    pub first_tick: Option<u64>,
}

impl StdioTimeline {
    pub fn ready(&mut self) {
        self.first_tick.get_or_insert(self.updates + 1);
    }

    pub fn tick(&self) -> Option<u64> {
        self.first_tick
            .and_then(|first| self.updates.checked_sub(first))
    }

    pub fn updating(&self) -> Option<u64> {
        self.first_tick.map(|first| self.updates + 1 - first)
    }

    pub fn advance(&mut self) {
        self.updates += 1;
    }
}

// NOTE: readiness is marked before Last too, so cue listeners in Last count the ready frame as tick 0.
fn mark_ready<P: Playable>(world: &mut World) {
    if world.resource::<StdioTimeline>().first_tick.is_none() && P::ready(world) {
        world.resource_mut::<StdioTimeline>().ready();
    }
}

fn settle<P: Playable>(world: &mut World) {
    let ready = P::ready(world);
    let mut timeline = world.resource_mut::<StdioTimeline>();
    if ready {
        timeline.ready();
    }
    timeline.advance();
}
