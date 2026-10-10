use super::*;
use std::time::Duration;

use bevy::{input::InputPlugin, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;

use crate::controller::PlayerControllerPlugin;

#[test]
fn rapid_throws_are_independent_world_space_projectiles() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins(DevicePlugin);
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, 0.0),
            Flashbangs(3),
        ))
        .id();
    app.update();
    let monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.0, 0.0, -3.8)))
        .id();
    let second_monster = app
        .world_mut()
        .spawn((Monster::default(), Transform::from_xyz(3.8, 0.0, 0.0)))
        .id();
    for index in 0..2 {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Right);
        app.update();
        if index == 0 {
            app.world_mut()
                .entity_mut(player)
                .get_mut::<Transform>()
                .unwrap()
                .rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
        }
    }
    assert_eq!(app.world().get::<Flashbangs>(player).unwrap().0, 1);
    let mut throws = app
        .world_mut()
        .query::<(Entity, &ThrownFlashbang, &Transform)>();
    let projectiles: Vec<_> = throws
        .iter(app.world())
        .map(|(entity, thrown, pose)| (entity, *thrown, pose.translation))
        .collect();
    assert_eq!(projectiles.len(), 2);
    assert!(projectiles.iter().all(|(entity, thrown, _)| {
        thrown.owner == player && app.world().get::<ChildOf>(*entity).is_none()
    }));
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(50.0, 1.6, 50.0);
    app.update();
    for (entity, thrown, before) in projectiles {
        let after = app.world().get::<Transform>(entity).unwrap().translation;
        assert_ne!(before, after);
        assert!(after.distance(thrown.landing) < before.distance(thrown.landing));
        assert!(after.x.abs() < 5.0 && after.z.abs() < 5.0);
    }
    for _ in 0..6 {
        app.update();
    }
    let targets = &app.world().get::<FlashTargets>(player).unwrap().0;
    assert!(targets.contains(&monster));
    assert!(targets.contains(&second_monster));
    app.world_mut().entity_mut(player).despawn();
    app.update();
    assert_eq!(throws.iter(app.world()).count(), 0);
}
