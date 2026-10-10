use bevy::{
    asset::AssetPlugin,
    gizmos::{prelude::GizmoConfigStore, GizmoPlugin},
    prelude::*,
};

use gameplay::levels::MonsterNavigation;

use super::{
    floor_segments, point, route_color, screen_point, target_color, MonsterRouteGizmos,
    FLOOR_LINE_HEIGHT, LONG_TARGET_COLOR_3D, ROUTE_COLOR_3D, RUN_COLOR_3D,
};

#[test]
fn navigation_draws_world_space_floor_segments_from_monster_to_destination() {
    let navigation = MonsterNavigation {
        waypoints: vec![Vec2::new(5.0, -16.25), Vec2::new(3.75, -18.75)],
        destination: Some(Vec2::new(3.75, -18.75)),
        last_sensed: None,
        long_range: false,
        running: false,
    };
    let origin = Vec3::new(5.0, 4.0, -12.5);
    assert_eq!(
        floor_segments(origin, &navigation),
        vec![
            (point(origin.xz()), point(navigation.waypoints[0])),
            (
                point(navigation.waypoints[0]),
                point(navigation.waypoints[1])
            ),
        ]
    );
    assert_eq!(point(Vec2::new(5.0, -16.25)).y, FLOOR_LINE_HEIGHT);
    assert!(floor_segments(
        origin,
        &MonsterNavigation {
            waypoints: vec![],
            destination: None,
            last_sensed: None,
            long_range: false,
            running: false,
        }
    )
    .is_empty());
}

#[test]
fn long_range_target_and_running_route_have_distinct_colors() {
    let mut navigation = MonsterNavigation {
        waypoints: vec![Vec2::X],
        destination: Some(Vec2::X),
        last_sensed: None,
        long_range: false,
        running: false,
    };
    assert_eq!(route_color(&navigation), ROUTE_COLOR_3D);
    assert_eq!(target_color(&navigation), ROUTE_COLOR_3D);
    navigation.long_range = true;
    assert_eq!(route_color(&navigation), ROUTE_COLOR_3D);
    assert_eq!(target_color(&navigation), LONG_TARGET_COLOR_3D);
    navigation.running = true;
    assert_eq!(route_color(&navigation), RUN_COLOR_3D);
    assert_eq!(target_color(&navigation), LONG_TARGET_COLOR_3D);
    navigation.long_range = false;
    assert_eq!(target_color(&navigation), RUN_COLOR_3D);
}

#[test]
fn debug_plugin_registers_xray_floor_gizmos_when_rendering_is_available() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        GizmoPlugin,
        crate::DebugPlugin,
    ));
    assert_eq!(
        app.world()
            .resource::<GizmoConfigStore>()
            .config::<MonsterRouteGizmos>()
            .0
            .depth_bias,
        -1.0
    );
}

#[test]
fn projection_skips_cameras_without_a_viewport() {
    assert!(screen_point(&Camera::default(), &GlobalTransform::IDENTITY, Vec3::ZERO).is_none());
}
