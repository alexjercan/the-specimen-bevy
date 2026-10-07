use bevy::prelude::*;

use crate::theme;

pub const FUSE_SLOT_COUNT: usize = 3;

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseSlots {
    pub filled: usize,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseSlot(pub usize);

#[derive(Component, Clone, Copy, Debug)]
pub struct FuseIconPart {
    pub filled: Color,
}

pub fn fuse_part_paint(color: Color, filled: bool) -> (BackgroundColor, BorderColor) {
    if filled {
        (BackgroundColor(color), BorderColor::all(color))
    } else {
        (
            BackgroundColor(theme::FUSE_EMPTY),
            BorderColor::all(theme::FUSE_EMPTY_EDGE),
        )
    }
}

pub fn fuse_slots() -> impl Bundle {
    (
        FuseSlots::default(),
        Name::new("Fuse slots"),
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            bottom: px(24),
            column_gap: px(8),
            ..default()
        },
        children![fuse_slot(0), fuse_slot(1), fuse_slot(2)],
    )
}

fn fuse_slot(index: usize) -> impl Bundle {
    (
        FuseSlot(index),
        Name::new("Fuse slot"),
        Node {
            width: px(72),
            height: px(40),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(theme::RADIUS)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(theme::PANEL_BORDER),
        children![
            fuse_part(6.0, 6.0, theme::FUSE_METAL),
            fuse_part(7.0, 16.0, theme::FUSE_METAL),
            (
                fuse_part(26.0, 14.0, theme::FUSE_CERAMIC),
                children![fuse_part(6.0, 12.0, theme::FUSE_HAZARD)],
            ),
            fuse_part(7.0, 16.0, theme::FUSE_METAL),
            fuse_part(6.0, 6.0, theme::FUSE_METAL),
        ],
    )
}

fn fuse_part(width: f32, height: f32, filled: Color) -> impl Bundle {
    (
        FuseIconPart { filled },
        Node {
            width: px(width),
            height: px(height),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(1)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        fuse_part_paint(filled, false),
    )
}

pub(crate) fn paint_fuse_slots(
    huds: Query<(&FuseSlots, &Children), Changed<FuseSlots>>,
    slots: Query<&FuseSlot>,
    children: Query<&Children>,
    mut parts: Query<(&FuseIconPart, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (state, hud) in &huds {
        for slot_entity in hud.iter() {
            let Ok(slot) = slots.get(slot_entity) else {
                continue;
            };
            let filled = slot.0 < state.filled;
            for part in children.iter_descendants(slot_entity) {
                if let Ok((icon, mut background, mut border)) = parts.get_mut(part) {
                    (*background, *border) = fuse_part_paint(icon.filled, filled);
                }
            }
        }
    }
}
