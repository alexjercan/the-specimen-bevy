pub mod theme;
mod widgets;

use bevy::prelude::*;

pub use widgets::{button, button_paint, label, menu_button, panel, text, MenuButton};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, widgets::paint_buttons);
    }
}
