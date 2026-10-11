use bevy::{camera::RenderTarget, gizmos::prelude::GizmoConfigGroup, prelude::*};
use bevy_egui::{egui, EguiContext, PrimaryEguiContext};
use gameplay::levels::{Monster, MonsterNavigation};

use crate::DebugSettings;

const MONSTER_COLOR: Color = Color::srgb(1.0, 0.34, 0.34);
const ROUTE_COLOR: Color = Color::srgb(1.0, 0.82, 0.4);
const RUN_COLOR: Color = Color::srgb(1.0, 0.35, 0.18);
const LONG_TARGET_COLOR: Color = Color::srgb(0.75, 0.47, 1.0);
const SENSED_COLOR: Color = Color::srgb(0.49, 0.82, 1.0);
const MARKER_HEIGHT: f32 = 1.4;
const FLOOR_LINE_HEIGHT: f32 = 0.08;

fn egui_color(color: Color) -> egui::Color32 {
    let [r, g, b, _] = color.to_srgba().to_u8_array();
    egui::Color32::from_rgb(r, g, b)
}

fn route_color(navigation: &MonsterNavigation) -> Color {
    if navigation.running {
        RUN_COLOR
    } else {
        ROUTE_COLOR
    }
}

fn target_color(navigation: &MonsterNavigation) -> Color {
    if navigation.long_range {
        LONG_TARGET_COLOR
    } else {
        route_color(navigation)
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(super) struct MonsterRouteGizmos;

fn point(position: Vec2) -> Vec3 {
    Vec3::new(position.x, FLOOR_LINE_HEIGHT, position.y)
}

fn floor_segments(position: Vec3, navigation: &MonsterNavigation) -> Vec<(Vec3, Vec3)> {
    let mut previous = point(position.xz());
    navigation
        .waypoints
        .iter()
        .map(|&waypoint| {
            let next = point(waypoint);
            let segment = (previous, next);
            previous = next;
            segment
        })
        .collect()
}

pub(super) fn draw_monster_routes(
    settings: Res<DebugSettings>,
    monsters: Query<(&Monster, &GlobalTransform)>,
    mut gizmos: Gizmos<MonsterRouteGizmos>,
) {
    if !settings.monster_tracking {
        return;
    }
    for (monster, transform) in &monsters {
        let navigation = monster.debug_navigation();
        for (start, end) in floor_segments(transform.translation(), &navigation) {
            gizmos.line(start, end, route_color(&navigation));
        }
        let floor_circle = |position| {
            Isometry3d::new(
                point(position),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            )
        };
        if let Some(destination) = navigation.destination {
            gizmos.circle(floor_circle(destination), 0.35, target_color(&navigation));
        }
        if let Some(last_sensed) = navigation.last_sensed {
            gizmos.circle(floor_circle(last_sensed), 0.2, SENSED_COLOR);
        }
    }
}

fn screen_point(
    camera: &Camera,
    transform: &GlobalTransform,
    position: Vec3,
) -> Option<egui::Pos2> {
    let viewport = camera.logical_viewport_rect()?;
    let screen = camera.world_to_viewport(transform, position).ok()?;
    (screen.x >= viewport.min.x
        && screen.y >= viewport.min.y
        && screen.x <= viewport.max.x
        && screen.y <= viewport.max.y)
        .then_some(egui::pos2(screen.x, screen.y))
}

fn paint_monster(
    painter: &egui::Painter,
    camera: &Camera,
    transform: &GlobalTransform,
    position: Vec3,
    navigation: &MonsterNavigation,
) {
    let project = |world| screen_point(camera, transform, world);
    if let Some(destination) = navigation
        .destination
        .and_then(|target| project(point(target)))
    {
        painter.text(
            destination + egui::vec2(13.0, 0.0),
            egui::Align2::LEFT_CENTER,
            if navigation.long_range {
                "LONG TARGET"
            } else {
                "TARGET"
            },
            egui::FontId::proportional(12.0),
            if navigation.long_range {
                egui_color(LONG_TARGET_COLOR)
            } else if navigation.running {
                egui_color(RUN_COLOR)
            } else {
                egui_color(ROUTE_COLOR)
            },
        );
    }
    if let Some(sensed) = navigation
        .last_sensed
        .and_then(|target| project(point(target)))
    {
        painter.circle_stroke(
            sensed,
            5.0,
            egui::Stroke::new(1.5_f32, egui_color(SENSED_COLOR)),
        );
    }
    if let Some(center) = project(position + Vec3::Y * MARKER_HEIGHT) {
        painter.circle_filled(center, 8.0, egui_color(MONSTER_COLOR));
        painter.circle_stroke(
            center,
            12.0,
            egui::Stroke::new(2.0_f32, egui_color(MONSTER_COLOR)),
        );
        painter.text(
            center + egui::vec2(17.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "MONSTER",
            egui::FontId::proportional(13.0),
            egui_color(MONSTER_COLOR),
        );
    }
}

pub(super) fn monster_tracking_ui(world: &mut World) {
    if !world.resource::<DebugSettings>().monster_tracking {
        return;
    }
    let Some(context) = world
        .query_filtered::<&EguiContext, With<PrimaryEguiContext>>()
        .iter(world)
        .next()
        .cloned()
    else {
        return;
    };
    let Some((camera, transform)) = world
        .query_filtered::<(&Camera, &GlobalTransform, &RenderTarget), With<Camera3d>>()
        .iter(world)
        .find(|(camera, _, target)| camera.is_active && matches!(target, RenderTarget::Window(_)))
        .map(|(camera, transform, _)| (camera.clone(), *transform))
    else {
        return;
    };
    let monsters: Vec<_> = world
        .query::<(&Monster, &GlobalTransform)>()
        .iter(world)
        .map(|(monster, transform)| (transform.translation(), monster.debug_navigation()))
        .collect();
    let mut context = context;
    let painter = context.get_mut().layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("monster_tracking"),
    ));
    for (position, navigation) in monsters {
        paint_monster(&painter, &camera, &transform, position, &navigation);
    }
}
