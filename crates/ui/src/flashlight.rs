use bevy::prelude::*;

use crate::theme;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct FlashlightMeter {
    pub charge: f32,
    pub on: bool,
}

impl Default for FlashlightMeter {
    fn default() -> Self {
        Self {
            charge: 1.0,
            on: false,
        }
    }
}

impl FlashlightMeter {
    pub fn visible(self) -> bool {
        self.on || self.charge < 1.0
    }
}

#[derive(Component)]
pub struct FlashlightFill;

pub fn flashlight_meter() -> impl Bundle {
    (
        FlashlightMeter::default(),
        Name::new("Flashlight charge"),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(24),
            width: px(184),
            padding: UiRect::all(px(9)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::PANEL_BORDER),
        children![
            (
                Text::new("LIGHT"),
                TextFont::from_font_size(12.0),
                TextColor(theme::TEXT),
            ),
            (
                Name::new("Flashlight charge track"),
                Node {
                    flex_grow: 1.0,
                    height: px(9),
                    padding: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(theme::FUSE_EMPTY),
                children![(
                    FlashlightFill,
                    Name::new("Flashlight fill"),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(theme::FUSE_HAZARD),
                )],
            ),
        ],
    )
}

pub(crate) fn paint_flashlight(
    meters: Query<(&FlashlightMeter, &Children), Changed<FlashlightMeter>>,
    children: Query<&Children>,
    mut fills: Query<&mut Node, With<FlashlightFill>>,
) {
    for (meter, roots) in &meters {
        for descendant in roots.iter().flat_map(|root| children.iter_descendants(root)) {
            if let Ok(mut fill) = fills.get_mut(descendant) {
                fill.width = percent(meter.charge.clamp(0.0, 1.0) * 100.0);
            }
        }
    }
}
