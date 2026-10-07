mod lights;
mod props;
mod rooms;
mod signs;

use bevy::prelude::*;

const COOL: Color = Color::linear_rgb(0.78, 0.88, 1.0);
const AMBER: Color = Color::linear_rgb(1.0, 0.58, 0.2);
const EXIT_GREEN: Color = Color::linear_rgb(0.12, 1.0, 0.35);
const FAULT_RED: Color = Color::linear_rgb(1.0, 0.06, 0.03);
const FIRE: Color = Color::linear_rgb(1.0, 0.35, 0.08);
const SPECIMEN: Color = Color::linear_rgb(0.3, 0.95, 0.85);

pub fn build_first_floor(mut commands: Commands) {
    rooms::spawn(&mut commands);
    lights::spawn(&mut commands);
    props::spawn_fixtures(&mut commands);
    signs::spawn(&mut commands);
    props::spawn_furniture(&mut commands);
}
