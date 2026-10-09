mod device;
mod flashlight;
mod hud;
mod slider;
mod stamina;
pub mod theme;
mod widgets;

use bevy::prelude::*;

pub use device::{
    detector_readout, flash_alpha, flash_overlay, flashbang_status, tracker_blip, tracker_edge,
    tracker_range_text, tracker_status_text, DetectorReadout, DetectorSignal, FlashOverlay,
    FlashbangPrompt, FlashbangStatus, TrackerEdge,
};
pub use flashlight::{flashlight_meter, FlashlightMeter, FlashlightPrompt};
pub use hud::{fuse_part_paint, fuse_slots, FuseIconPart, FuseSlot, FuseSlots, FUSE_SLOT_COUNT};
pub use slider::{slider, SliderFill};
pub use stamina::{stamina_meter, StaminaMeter};
pub use widgets::{button, button_paint, label, menu_button, panel, text, MenuButton};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(bevy::ui_widgets::slider_self_update)
            .add_systems(
                Update,
                (
                    widgets::paint_buttons,
                    slider::sync_slider_fills,
                    hud::paint_fuse_slots,
                    flashlight::paint_flashlight,
                    stamina::paint_stamina,
                    device::paint_flashbang_status,
                    device::paint_detector_readout,
                    device::paint_flash_overlay,
                ),
            );
    }
}
