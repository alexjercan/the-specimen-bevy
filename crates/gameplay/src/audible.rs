use bevy::prelude::*;
use game_audio::Falloff;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Origin {
    Listener,
    At(Vec3),
    From(Entity),
}

pub trait Audible: Message + Copy + Send + Sync + 'static {
    fn name(&self) -> Option<&'static str>;

    fn origin(&self) -> Origin;

    fn falloff(&self) -> Falloff;
}
