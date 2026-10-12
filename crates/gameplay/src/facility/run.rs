use bevy::{
    ecs::message::MessageCursor,
    prelude::*,
    state::state::{setup_state_transitions_in_world, StateTransitionSystems},
};

use super::{
    builder::{Door, DoorRef, LevelRoot, Passage, Prop, Room},
    first_floor::{build_first_floor, spawn_first_floor_actors},
    power::FacilityPower,
};
use crate::{
    fuses::FusePanel, interaction::PickupKind, monster::Monster, player::PlayerController,
    presets::GamePreset,
};

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct StartRun {
    pub preset: GamePreset,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRoot;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveRun;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: StatesPlugin makes the same call; whichever plugin comes first sets up the schedule.
        setup_state_transitions_in_world(app.world_mut());
        // NOTE: after the enter schedules, so a run started with a state change exists by Update.
        app.add_message::<StartRun>()
            .add_observer(end_run)
            .add_systems(
                StateTransition,
                start_run.after(StateTransitionSystems::EnterSchedules),
            );
    }
}

fn start_run(world: &mut World, mut cursor: Local<MessageCursor<StartRun>>) {
    let Some(run) = cursor
        .read(world.resource::<Messages<StartRun>>())
        .last()
        .copied()
    else {
        return;
    };
    *world.resource_mut::<GamePreset>() = run.preset;
    world.spawn((Name::new("Run"), ActiveRun, RunRoot));
    if let Err(error) = world.run_system_cached(build_first_floor) {
        error!("first floor build failed: {error}");
        return;
    }
    if let Err(error) = world.run_system_cached(spawn_first_floor_actors) {
        error!("first floor actor spawn failed: {error}");
        return;
    }
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, (
            Without<ChildOf>,
            Without<RunRoot>,
            Or<(
                With<Room>,
                With<LevelRoot>,
                With<Door>,
                With<DoorRef>,
                With<Passage>,
                With<Prop>,
                With<FusePanel>,
                With<PickupKind>,
                With<Monster>,
                With<PlayerController>,
            )>,
        )>()
        .iter(world)
        .collect();
    for entity in roots {
        world.entity_mut(entity).insert(RunRoot);
    }
}

// NOTE: menus and cinematics light the facility after a run, so the end of a run restores mains.
fn end_run(_: On<Remove, ActiveRun>, mut power: ResMut<FacilityPower>) {
    *power = FacilityPower::default();
}
