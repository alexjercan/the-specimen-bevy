use bevy::prelude::*;

#[derive(Resource)]
pub struct InteractionAssets {
    pub font: Handle<Font>,
    pub interact_glyph: Handle<Image>,
}
