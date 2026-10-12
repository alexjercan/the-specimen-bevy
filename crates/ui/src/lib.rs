mod fps;
mod slider;
pub mod theme;
mod widgets;

use bevy::prelude::*;

pub use fps::{fps_label, FpsOverlayPlugin, FpsText};
pub use slider::{slider, SliderFill};
pub use widgets::{button, button_paint, label, menu_button, panel, text, MenuButton};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(bevy::ui_widgets::slider_self_update)
            .add_systems(Update, (widgets::paint_buttons, slider::sync_slider_fills));
    }
}
