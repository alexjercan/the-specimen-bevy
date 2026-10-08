use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderRange, SliderStep, SliderValue, TrackClick},
};

use crate::theme;

#[derive(Component)]
pub struct SliderFill;

fn fill_width(value: f32, range: &SliderRange) -> Val {
    percent(range.thumb_position(range.clamp(value)) * 100.0)
}

pub fn slider(value: f32, start: f32, end: f32, step: f32) -> impl Bundle {
    let range = SliderRange::new(start, end);
    (
        Slider {
            track_click: TrackClick::Snap,
            ..default()
        },
        SliderValue(value),
        range,
        SliderStep(step),
        Node {
            width: percent(100),
            height: px(16),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            ..default()
        },
        BackgroundColor(theme::BUTTON),
        BorderColor::all(theme::BUTTON_BORDER),
        children![(
            SliderFill,
            Pickable::IGNORE,
            Node {
                width: fill_width(value, &range),
                height: percent(100),
                border: UiRect::right(px(3)),
                ..default()
            },
            BackgroundColor(theme::ACCENT),
            BorderColor::all(theme::TEXT),
        )],
    )
}

pub(crate) fn sync_slider_fills(
    sliders: Query<
        (&SliderValue, &SliderRange, &Children),
        Or<(Changed<SliderValue>, Changed<SliderRange>)>,
    >,
    mut fills: Query<&mut Node, With<SliderFill>>,
) {
    for (value, range, children) in &sliders {
        let width = fill_width(value.0, range);
        for child in children {
            if let Ok(mut node) = fills.get_mut(*child) {
                if node.width != width {
                    node.width = width;
                }
            }
        }
    }
}
