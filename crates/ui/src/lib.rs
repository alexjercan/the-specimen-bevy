mod flashlight;
mod hud;
mod stamina;
pub mod theme;
mod widgets;

use bevy::prelude::*;

pub use flashlight::{flashlight_meter, FlashlightMeter};
pub use hud::{fuse_part_paint, fuse_slots, FuseIconPart, FuseSlot, FuseSlots, FUSE_SLOT_COUNT};
pub use stamina::{stamina_meter, StaminaMeter};
pub use widgets::{button, button_paint, label, menu_button, panel, text, MenuButton};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                widgets::paint_buttons,
                hud::paint_fuse_slots,
                flashlight::paint_flashlight,
                stamina::paint_stamina,
            ),
        );
    }
}
