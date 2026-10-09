use bevy::prelude::*;

use crate::theme;

const FLASH_PEAK_ALPHA: f32 = 0.55;
const FLASH_DURATION: f32 = 0.9;

const TRACKER_WIDTH: f32 = 236.0;
const TRACKER_HEIGHT: f32 = 168.0;
const TRACKER_BOTTOM: f32 = 18.0;
const TRACKER_RADIUS: f32 = 10.0;
const TRACKER_PADDING: f32 = 10.0;
const GRIP_WIDTH: f32 = 48.0;
const GRIP_HEIGHT: f32 = 8.0;
const LED_COUNT: usize = 6;
const LED_WIDTH: f32 = 4.0;
const LED_HEIGHT: f32 = 8.0;
const SCREEN_WIDTH: f32 = 200.0;
const SCREEN_HEIGHT: f32 = 132.0;
const SCREEN_RADIUS: f32 = 6.0;
const APEX: Vec2 = Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT);
const FAN_HALF_ANGLE_DEG: f32 = 50.0;
const FAN_HALF_ANGLE_RAD: f32 = FAN_HALF_ANGLE_DEG * std::f32::consts::PI / 180.0;
const FAN_RADIUS: f32 = 112.0;
const SPOKE_ANGLES_DEG: [f32; 5] = [-50.0, -25.0, 0.0, 25.0, 50.0];
const SPOKE_THICKNESS: f32 = 1.5;
const ARC_ANGLES_DEG: [f32; 11] = [
    -50.0, -40.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 40.0, 50.0,
];
const ARC_SEGMENT_LENGTH: f32 = 10.0;
const ARC_THICKNESS: f32 = 1.5;
const BLIP_SIZE: f32 = 8.0;

const CASING_COLOR: Color = Color::srgb(0.06, 0.062, 0.058);
const CASING_BORDER: Color = Color::srgb(0.16, 0.16, 0.16);
const GRIP_COLOR: Color = Color::srgb(0.03, 0.03, 0.03);
const SCREEN_COLOR: Color = Color::srgb(0.015, 0.07, 0.035);
const SCREEN_BORDER: Color = Color::srgba(0.2, 0.5, 0.3, 0.5);
const LED_DIM: Color = Color::srgba(0.75, 0.4, 0.05, 0.22);
const LED_LIT: Color = Color::srgb(1.0, 0.52, 0.06);
const SPOKE_COLOR: Color = Color::srgba(0.3, 0.68, 0.4, 0.4);
const ARC_OUTER_COLOR: Color = Color::srgba(0.35, 0.78, 0.45, 0.55);
const ARC_INNER_COLOR: Color = Color::srgba(0.25, 0.6, 0.35, 0.35);
const BLIP_COLOR: Color = Color::srgb(0.4, 1.0, 0.5);
const SCREEN_TEXT_COLOR: Color = Color::srgb(0.45, 0.95, 0.55);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct FlashbangStatus {
    pub count: usize,
}

#[derive(Component)]
pub struct FlashbangLabel;

#[derive(Component)]
pub struct FlashbangPrompt;

#[derive(Component)]
pub struct FlashbangIcon;

pub fn flashbang_status() -> impl Bundle {
    (
        FlashbangStatus { count: 0 },
        Name::new("Flashbang status"),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            bottom: px(136),
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
                FlashbangIcon,
                Name::new("Flashbang icon"),
                Node {
                    width: px(20),
                    height: px(22),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(theme::FUSE_METAL),
                BorderColor::all(theme::FUSE_EMPTY_EDGE),
                children![(
                    Name::new("Flashbang stripe"),
                    Node {
                        width: px(12),
                        height: px(5),
                        ..default()
                    },
                    BackgroundColor(theme::FUSE_HAZARD),
                )],
            ),
            (
                Name::new("Flashbang text"),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    (
                        FlashbangLabel,
                        Text::new("FLASHBANG x0"),
                        TextFont::from_font_size(12.0),
                        TextColor(theme::TEXT),
                    ),
                    (
                        FlashbangPrompt,
                        Text::new("RMB THROW"),
                        TextFont::from_font_size(10.0),
                        TextColor(theme::TEXT),
                    ),
                ],
            ),
        ],
    )
}

pub(crate) fn paint_flashbang_status(
    statuses: Query<(Entity, &FlashbangStatus), Changed<FlashbangStatus>>,
    children: Query<&Children>,
    mut labels: Query<&mut Text, With<FlashbangLabel>>,
    mut icons: Query<&mut BackgroundColor, With<FlashbangIcon>>,
) {
    for (entity, status) in &statuses {
        for descendant in children.iter_descendants(entity) {
            if let Ok(mut icon) = icons.get_mut(descendant) {
                icon.0 = if status.count > 0 {
                    theme::FUSE_METAL
                } else {
                    theme::FUSE_EMPTY
                };
            }
            if let Ok(mut text) = labels.get_mut(descendant) {
                let message = format!("FLASHBANG x{}", status.count);
                if text.0 != message {
                    text.0 = message;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectorSignal {
    pub distance: f32,
    pub bearing: f32,
    pub range: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackerEdge {
    Left,
    Right,
}

pub fn tracker_blip(signal: Option<DetectorSignal>) -> Option<Vec2> {
    let signal = signal?;
    if signal.bearing.abs() > FAN_HALF_ANGLE_RAD {
        return None;
    }
    let radius = (signal.distance / signal.range).clamp(0.0, 1.0) * FAN_RADIUS;
    Some(Vec2::new(
        radius * signal.bearing.sin(),
        -radius * signal.bearing.cos(),
    ))
}

pub fn tracker_edge(signal: Option<DetectorSignal>) -> Option<TrackerEdge> {
    let signal = signal?;
    if signal.bearing.abs() <= FAN_HALF_ANGLE_RAD {
        return None;
    }
    Some(if signal.bearing < 0.0 {
        TrackerEdge::Left
    } else {
        TrackerEdge::Right
    })
}

pub fn tracker_range_text(signal: Option<DetectorSignal>) -> String {
    match signal {
        Some(signal) => format!("{:.0} M", signal.distance),
        None => "-- M".to_owned(),
    }
}

pub fn tracker_status_text(signal: Option<DetectorSignal>) -> &'static str {
    match signal {
        None => "NO SIG",
        Some(signal) if signal.bearing.abs() <= FAN_HALF_ANGLE_RAD => "TRACK",
        Some(_) => "EDGE",
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct DetectorReadout {
    pub signal: Option<DetectorSignal>,
}

#[derive(Component)]
pub(crate) struct TrackerBlip;

#[derive(Component)]
pub(crate) struct TrackerRangeLabel;

#[derive(Component)]
pub(crate) struct TrackerStatusLabel;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrackerLed(TrackerEdge);

fn polar_offset(radius: f32, angle_deg: f32) -> Vec2 {
    let angle = angle_deg * std::f32::consts::PI / 180.0;
    Vec2::new(radius * angle.sin(), -radius * angle.cos())
}

fn tracker_grip() -> impl Bundle {
    (
        Name::new("Tracker grip"),
        Node {
            width: px(GRIP_WIDTH),
            height: px(GRIP_HEIGHT),
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
        BackgroundColor(GRIP_COLOR),
    )
}

fn tracker_led(side: TrackerEdge, side_name: &'static str) -> impl Bundle {
    (
        TrackerLed(side),
        Name::new(side_name),
        Node {
            width: px(LED_WIDTH),
            height: px(LED_HEIGHT),
            border_radius: BorderRadius::all(px(1)),
            ..default()
        },
        BackgroundColor(LED_DIM),
    )
}

fn tracker_led_column(side: TrackerEdge, side_name: &'static str) -> impl Bundle {
    (
        Name::new("Tracker LED column"),
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            height: percent(100),
            ..default()
        },
        Children::spawn(SpawnIter(
            (0..LED_COUNT).map(move |_| tracker_led(side, side_name)),
        )),
    )
}

fn radial_line(angle_deg: f32) -> impl Bundle {
    let center = APEX + polar_offset(FAN_RADIUS / 2.0, angle_deg);
    (
        Name::new("Tracker spoke"),
        Node {
            position_type: PositionType::Absolute,
            left: px(center.x - SPOKE_THICKNESS / 2.0),
            top: px(center.y - FAN_RADIUS / 2.0),
            width: px(SPOKE_THICKNESS),
            height: px(FAN_RADIUS),
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(angle_deg)),
        BackgroundColor(SPOKE_COLOR),
    )
}

fn arc_segment(radius: f32, angle_deg: f32, color: Color) -> impl Bundle {
    let center = APEX + polar_offset(radius, angle_deg);
    (
        Name::new("Tracker arc segment"),
        Node {
            position_type: PositionType::Absolute,
            left: px(center.x - ARC_SEGMENT_LENGTH / 2.0),
            top: px(center.y - ARC_THICKNESS / 2.0),
            width: px(ARC_SEGMENT_LENGTH),
            height: px(ARC_THICKNESS),
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(angle_deg)),
        BackgroundColor(color),
    )
}

fn tracker_blip_node() -> impl Bundle {
    (
        TrackerBlip,
        Name::new("Tracker blip"),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: px(BLIP_SIZE),
            height: px(BLIP_SIZE),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        BackgroundColor(BLIP_COLOR),
        BoxShadow::new(BLIP_COLOR.with_alpha(0.6), px(0), px(0), px(0), px(6)),
    )
}

fn tracker_range_label() -> impl Bundle {
    (
        TrackerRangeLabel,
        Name::new("Tracker range"),
        Node {
            position_type: PositionType::Absolute,
            left: px(6),
            bottom: px(4),
            ..default()
        },
        Text::new(tracker_range_text(None)),
        TextFont::from_font_size(10.0),
        TextColor(SCREEN_TEXT_COLOR),
    )
}

fn tracker_status_label() -> impl Bundle {
    (
        TrackerStatusLabel,
        Name::new("Tracker status"),
        Node {
            position_type: PositionType::Absolute,
            right: px(6),
            bottom: px(4),
            ..default()
        },
        Text::new(tracker_status_text(None)),
        TextFont::from_font_size(10.0),
        TextColor(SCREEN_TEXT_COLOR),
    )
}

fn tracker_screen() -> impl Bundle {
    (
        Name::new("Tracker screen"),
        Node {
            width: px(SCREEN_WIDTH),
            height: px(SCREEN_HEIGHT),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(SCREEN_RADIUS)),
            ..default()
        },
        BackgroundColor(SCREEN_COLOR),
        BorderColor::all(SCREEN_BORDER),
        Children::spawn((
            SpawnIter(SPOKE_ANGLES_DEG.into_iter().map(radial_line)),
            SpawnIter(
                ARC_ANGLES_DEG
                    .into_iter()
                    .map(|angle| arc_segment(FAN_RADIUS, angle, ARC_OUTER_COLOR)),
            ),
            SpawnIter(
                ARC_ANGLES_DEG
                    .into_iter()
                    .step_by(2)
                    .map(|angle| arc_segment(FAN_RADIUS / 3.0, angle, ARC_INNER_COLOR)),
            ),
            SpawnIter(
                ARC_ANGLES_DEG
                    .into_iter()
                    .step_by(2)
                    .map(|angle| arc_segment(2.0 * FAN_RADIUS / 3.0, angle, ARC_INNER_COLOR)),
            ),
            Spawn(tracker_blip_node()),
            Spawn(tracker_range_label()),
            Spawn(tracker_status_label()),
        )),
    )
}

pub fn detector_readout() -> impl Bundle {
    (
        DetectorReadout::default(),
        Name::new("Detector readout"),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(TRACKER_BOTTOM),
            width: px(TRACKER_WIDTH),
            height: px(TRACKER_HEIGHT),
            margin: UiRect::left(px(-TRACKER_WIDTH / 2.0)),
            padding: UiRect::all(px(TRACKER_PADDING)),
            border: UiRect::all(px(theme::BORDER)),
            border_radius: BorderRadius::all(px(TRACKER_RADIUS)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            ..default()
        },
        BackgroundColor(CASING_COLOR),
        BorderColor::all(CASING_BORDER),
        children![
            tracker_grip(),
            (
                Name::new("Tracker body"),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    flex_grow: 1.0,
                    width: percent(100),
                    ..default()
                },
                children![
                    tracker_led_column(TrackerEdge::Left, "Tracker LED left"),
                    tracker_screen(),
                    tracker_led_column(TrackerEdge::Right, "Tracker LED right"),
                ],
            ),
        ],
    )
}

pub(crate) fn paint_detector_readout(
    readouts: Query<(Entity, &DetectorReadout), Changed<DetectorReadout>>,
    children: Query<&Children>,
    mut blips: Query<(&mut Node, &mut Visibility), With<TrackerBlip>>,
    mut ranges: Query<&mut Text, (With<TrackerRangeLabel>, Without<TrackerStatusLabel>)>,
    mut statuses: Query<&mut Text, (With<TrackerStatusLabel>, Without<TrackerRangeLabel>)>,
    mut leds: Query<(&TrackerLed, &mut BackgroundColor)>,
) {
    for (entity, readout) in &readouts {
        let blip = tracker_blip(readout.signal);
        let edge = tracker_edge(readout.signal);
        let range_text = tracker_range_text(readout.signal);
        let status_text = tracker_status_text(readout.signal);
        for descendant in children.iter_descendants(entity) {
            if let Ok((mut node, mut visibility)) = blips.get_mut(descendant) {
                match blip {
                    Some(offset) => {
                        node.left = px(APEX.x + offset.x - BLIP_SIZE / 2.0);
                        node.top = px(APEX.y + offset.y - BLIP_SIZE / 2.0);
                        *visibility = Visibility::Inherited;
                    }
                    None => *visibility = Visibility::Hidden,
                }
            }
            if let Ok(mut text) = ranges.get_mut(descendant) {
                if text.0 != range_text {
                    text.0 = range_text.clone();
                }
            }
            if let Ok(mut text) = statuses.get_mut(descendant) {
                if text.0 != status_text {
                    text.0 = status_text.to_owned();
                }
            }
            if let Ok((led, mut background)) = leds.get_mut(descendant) {
                let target = if Some(led.0) == edge {
                    LED_LIT
                } else {
                    LED_DIM
                };
                if background.0 != target {
                    background.0 = target;
                }
            }
        }
    }
}

pub fn flash_alpha(elapsed: f32) -> f32 {
    (FLASH_PEAK_ALPHA * (1.0 - elapsed / FLASH_DURATION)).max(0.0)
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct FlashOverlay {
    pub elapsed: f32,
}

pub fn flash_overlay() -> impl Bundle {
    (
        FlashOverlay::default(),
        Name::new("Flash overlay"),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::WHITE.with_alpha(flash_alpha(0.0))),
    )
}

pub(crate) fn paint_flash_overlay(
    mut overlays: Query<(&FlashOverlay, &mut BackgroundColor), Changed<FlashOverlay>>,
) {
    for (overlay, mut background) in &mut overlays {
        background.0 = Color::WHITE.with_alpha(flash_alpha(overlay.elapsed));
    }
}
