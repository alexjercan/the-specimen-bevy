use bevy::{picking::hover::Hovered, prelude::*, ui::Pressed, ui_widgets::Button};

use crate::theme;

#[derive(Component, Default)]
pub struct MenuButton;

pub fn button_paint(hovered: bool, pressed: bool) -> (BackgroundColor, BorderColor) {
    let (face, border) = if pressed {
        (theme::BUTTON_PRESSED, theme::ACCENT)
    } else if hovered {
        (theme::BUTTON_HOVER, theme::ACCENT)
    } else {
        (theme::BUTTON, theme::BUTTON_BORDER)
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
        Hovered::default(),
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
        Hovered::default(),
        MenuButton,
        Node {
            width: percent(100),
            padding: UiRect::axes(px(18), px(10)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            ..default()
        },
        button_paint(false, false),
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
    mut buttons: Query<
        (
            &Hovered,
            Has<Pressed>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<MenuButton>,
    >,
) {
    for (hovered, pressed, mut background, mut border) in &mut buttons {
        let (want_background, want_border) = button_paint(hovered.get(), pressed);
        if *background != want_background {
            *background = want_background;
        }
        if *border != want_border {
            *border = want_border;
        }
    }
}
