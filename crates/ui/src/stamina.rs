use bevy::prelude::*;

use crate::theme;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct StaminaMeter {
    pub charge: f32,
    pub sprinting: bool,
}

impl Default for StaminaMeter {
    fn default() -> Self {
        Self {
            charge: 1.0,
            sprinting: false,
        }
    }
}

impl StaminaMeter {
    pub fn visible(self) -> bool {
        self.sprinting || self.charge < 1.0
    }
}

#[derive(Component)]
pub struct StaminaFill;

pub fn stamina_meter() -> impl Bundle {
    (
        StaminaMeter::default(),
        Name::new("Sprint charge"),
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
                Text::new("SPRINT"),
                TextFont::from_font_size(12.0),
                TextColor(theme::TEXT),
            ),
            (
                Name::new("Sprint charge track"),
                Node {
                    flex_grow: 1.0,
                    height: px(9),
                    padding: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(theme::FUSE_EMPTY),
                children![(
                    StaminaFill,
                    Name::new("Sprint fill"),
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

pub(crate) fn paint_stamina(
    meters: Query<(&StaminaMeter, &Children), Changed<StaminaMeter>>,
    children: Query<&Children>,
    mut fills: Query<&mut Node, With<StaminaFill>>,
) {
    for (meter, roots) in &meters {
        for descendant in roots.iter().flat_map(|root| children.iter_descendants(root)) {
            if let Ok(mut fill) = fills.get_mut(descendant) {
                fill.width = percent(meter.charge.clamp(0.0, 1.0) * 100.0);
            }
        }
    }
}
