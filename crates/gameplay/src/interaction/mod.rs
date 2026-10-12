mod assets;
mod pickups;
mod sounds;
mod targets;
mod view;

use bevy::prelude::*;

pub use assets::InteractionAssets;
pub use pickups::{PickupCue, PickupKind, PickupPlugin};
pub use sounds::{InteractionSounds, InteractionSoundsPlugin};
pub(crate) use targets::StructuralSight;
pub use targets::{InteractTarget, InteractTargets};
pub use view::InteractionViewPlugin;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PickupPlugin);
    }
}
