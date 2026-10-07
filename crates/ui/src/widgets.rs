use bevy::prelude::*;

use crate::theme;

#[derive(Component, Default)]
pub struct MenuButton;

type ChangedMenuButton = (With<MenuButton>, Changed<Interaction>);

pub fn button_paint(interaction: Interaction) -> (BackgroundColor, BorderColor) {
    let (face, border) = match interaction {
        Interaction::None => (theme::BUTTON, theme::BUTTON_BORDER),
        Interaction::Hovered => (theme::BUTTON_HOVER, theme::ACCENT),
        Interaction::Pressed => (theme::BUTTON_PRESSED, theme::ACCENT),
    };
    (BackgroundColor(face), BorderColor::all(border))
}

pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(18.0),
        TextColor(theme::TEXT),
    )
}

pub fn text(value: impl Into<String>, size: f32, color: Color, font: Handle<Font>) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size).with_font(font),
        TextColor(color),
    )
}

pub fn button() -> impl Bundle {
    (
        Button,
        Node {
            padding: UiRect::axes(px(16), px(9)),
            ..default()
        },
        BackgroundColor(theme::BUTTON),
    )
}

pub fn menu_button(value: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Button,
        MenuButton,
        Node {
            width: percent(100),
            padding: UiRect::axes(px(18), px(10)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            ..default()
        },
        button_paint(Interaction::None),
        children![text(value, 18.0, theme::TEXT, font)],
    )
}

pub fn panel() -> impl Bundle {
    (
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::PANEL_BORDER),
    )
}

pub(crate) fn paint_buttons(
    mut buttons: Query<(&Interaction, &mut BackgroundColor, &mut BorderColor), ChangedMenuButton>,
) {
    for (interaction, mut background, mut border) in &mut buttons {
        (*background, *border) = button_paint(*interaction);
    }
}
