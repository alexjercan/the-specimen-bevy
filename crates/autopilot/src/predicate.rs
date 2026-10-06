use std::sync::Arc;

use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::AutopilotClock;

pub type Predicate = dyn Fn(&World) -> bool + Send + Sync;

pub fn elapsed(seconds: f32) -> Arc<Predicate> {
    assert!(
        seconds.is_finite() && seconds >= 0.0,
        "elapsed seconds must be finite and nonnegative"
    );
    Arc::new(move |world| {
        world
            .get_resource::<AutopilotClock>()
            .is_some_and(|clock| clock.step_elapsed >= seconds)
    })
}

pub fn frames(count: u64) -> Arc<Predicate> {
    Arc::new(move |world| {
        world
            .get_resource::<AutopilotClock>()
            .is_some_and(|clock| clock.step_frames >= count)
    })
}

pub fn state_is<S: States>(state: S) -> Arc<Predicate> {
    Arc::new(move |world| {
        world
            .get_resource::<State<S>>()
            .is_some_and(|current| *current.get() == state)
    })
}

pub fn resource_where<R: Resource>(
    predicate: impl Fn(&R) -> bool + Send + Sync + 'static,
) -> Arc<Predicate> {
    Arc::new(move |world| world.get_resource::<R>().is_some_and(&predicate))
}

pub fn entity_count<F: QueryFilter + 'static>(expected: usize) -> Arc<Predicate> {
    Arc::new(move |world| {
        world
            .try_query_filtered::<Entity, F>()
            .is_some_and(|mut query| query.iter(world).count() == expected)
    })
}

pub fn any_entity<F: QueryFilter + 'static>() -> Arc<Predicate> {
    Arc::new(|world| {
        world
            .try_query_filtered::<Entity, F>()
            .is_some_and(|mut query| query.iter(world).next().is_some())
    })
}

pub fn and(left: Arc<Predicate>, right: Arc<Predicate>) -> Arc<Predicate> {
    Arc::new(move |world| left(world) && right(world))
}

pub fn or(left: Arc<Predicate>, right: Arc<Predicate>) -> Arc<Predicate> {
    Arc::new(move |world| left(world) || right(world))
}

pub fn not(predicate: Arc<Predicate>) -> Arc<Predicate> {
    Arc::new(move |world| !predicate(world))
}
