use bevy::prelude::World;
use serde::{Deserialize, Serialize};

use crate::Playable;

pub const VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(deserialize = "I: Deserialize<'de> + Default")
)]
pub struct Command<I> {
    pub version: u32,
    pub tick: u64,
    #[serde(default)]
    pub input: I,
}

#[derive(Serialize)]
pub struct Reply<V> {
    pub version: u32,
    pub tick: u64,
    #[serde(flatten)]
    pub view: V,
}

#[derive(Serialize)]
pub struct Failure {
    pub version: u32,
    pub error: String,
}

impl Failure {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            version: VERSION,
            error: error.into(),
        }
    }

    pub fn line(&self) -> String {
        serde_json::to_string(self).expect("failure serializes")
    }
}

pub fn reply<P: Playable>(world: &mut World, tick: u64) -> String {
    serde_json::to_string(&Reply {
        version: VERSION,
        tick,
        view: P::observe(world, tick),
    })
    .expect("reply serializes")
}

pub(crate) fn parse<P: Playable>(line: &str, current: u64) -> Result<Command<P::Input>, Failure> {
    let command = serde_json::from_str::<Command<P::Input>>(line).map_err(|_| {
        Failure::new("expected a JSON object with version, tick and optional input")
    })?;
    if command.version != VERSION {
        return Err(Failure::new(format!(
            "unsupported protocol version {}, expected {VERSION}",
            command.version
        )));
    }
    if command.tick <= current {
        return Err(Failure::new("tick must be greater than the current tick"));
    }
    Ok(command)
}
