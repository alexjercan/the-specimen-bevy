mod animation;
mod assets;
mod builder;
mod doors;
mod exit_cinematic;
mod exterior;
mod first_floor;
mod lighting_view;
mod lights;
mod lights_view;
mod menu_background;
mod module;
mod objective;
mod power;
mod run;
mod sounds;
mod view;

use bevy::prelude::*;

pub use animation::DoorSwing;
pub use assets::FacilityAssets;
pub use builder::{
    Ceiling, Door, DoorOf, DoorRef, DoorState, Doors, Floor, LevelRoot, LightEffect,
    LightIntensity, Passage, Prop, PropCollider, Room, Walls, TILE,
};
pub(crate) use doors::{
    aimed_door, box_hit, panel_hinge, panel_rotation, INTERACT_RANGE, PANEL_HALF_THICKNESS,
    PANEL_OFFSET, PANEL_WIDTH,
};
pub use doors::{
    panel_center, panel_top, DoorCue, DoorCueKind, DoorLock, DoorPanel, DoorPlugin, ExitDoor,
    ExitOpened, ToggleDoor,
};
pub use exit_cinematic::{build_closed_exit_cinematic, build_exit_cinematic, ExitCinematic};
pub use first_floor::{
    build_first_floor, select_fuse_slots, spawn_first_floor_actors, FuseTable, FuseZone,
    FUSE_TABLES, FUSE_ZONES,
};
pub use lighting_view::FacilityLightingPlugin;
pub use lights::{
    AmbientNits, FacilityLight, LightConfig, LightScale, LightingSettings, MainMenuAccentLight,
};
pub use lights_view::PropLightsPlugin;
pub use menu_background::build_main_menu_background;
pub use module::{LightSpec, Module};
pub use objective::{Escaped, ObjectivePlugin, ObjectiveSystems};
pub use power::{
    Boiler, FacilityPower, FacilityPowerPlugin, MainsPowered, PowerCue, PowerState,
    MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS,
};
pub use run::{ActiveRun, RunPlugin, RunRoot, StartRun};
pub use sounds::{FacilitySounds, FacilitySoundsPlugin, BOILER_CUE_OFFSET, DOOR_CUE_OFFSET};
pub use view::{FacilityViewPlugin, RenderCeilings};

pub struct FacilityPlugin;

impl Plugin for FacilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DoorPlugin, FacilityPowerPlugin, ObjectivePlugin, RunPlugin));
    }
}
