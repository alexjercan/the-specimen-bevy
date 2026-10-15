#![allow(dead_code)]

use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    ecs::query::QueryFilter,
    prelude::*,
};
use game_flow::GameState;
use gameplay::{
    ambience::AmbiencePlugin,
    devices::DevicePlugin,
    facility::{DoorPlugin, FacilityPowerPlugin},
    fuses::FusePlugin,
    hiding::HidingPlugin,
    interaction::PickupPlugin,
    monster::MonsterPlugin,
    player::{ControllerSettings, Input, KeyBindings, PlayerController, PlayerPlugin},
};

// NOTE: the gameplay plugins that own the cue messages the Specimen snapshot listens to.
pub struct CueOwners;

impl PluginGroup for CueOwners {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PlayerPlugin)
            .add(DoorPlugin)
            .add(FacilityPowerPlugin)
            .add(MonsterPlugin)
            .add(DevicePlugin)
            .add(PickupPlugin)
            .add(FusePlugin)
            .add(HidingPlugin)
            .add(AmbiencePlugin)
    }
}

pub fn controller_settings() -> ControllerSettings {
    ControllerSettings {
        keys: KeyBindings {
            forward: KeyCode::KeyW,
            left: KeyCode::KeyA,
            backward: KeyCode::KeyS,
            right: KeyCode::KeyD,
            sprint: KeyCode::ShiftLeft,
            interact: KeyCode::KeyF,
            flashlight: Input::Mouse(MouseButton::Left),
            flashbang: Input::Mouse(MouseButton::Right),
        },
        mouse_sensitivity: 0.0015,
        invert_y: false,
    }
}

pub fn count<F: QueryFilter>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), F>()
        .iter(app.world())
        .count()
}

pub fn named(app: &mut App, name: &str) -> Option<Entity> {
    app.world_mut()
        .query::<(Entity, &Name)>()
        .iter(app.world())
        .find_map(|(entity, candidate)| (candidate.as_str() == name).then_some(entity))
}

pub fn game_state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

pub fn player(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<PlayerController>>()
        .single(app.world())
        .unwrap()
}
