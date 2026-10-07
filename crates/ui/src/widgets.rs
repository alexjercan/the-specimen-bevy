use bevy::prelude::*;

pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(18.0),
        TextColor(Color::srgb(0.9, 0.93, 0.88)),
    )
}

pub fn button() -> impl Bundle {
    (
        Button,
        Node {
            padding: UiRect::axes(px(16), px(9)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.11, 0.13, 0.9)),
    )
}
