use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::prelude::*;
use gameplay::{
    devices::{DevicePlugin, Flashbangs, Flashed, FLASHBANG_BURST_DELAY},
    facility::LevelRoot,
    monster::Monster,
    player::PlayerController,
};

mod common;

const FRAME: f32 = 0.1;

fn app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins(DevicePlugin);
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn throw(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    app.update();
}

fn projectiles(app: &mut App) -> Vec<(Entity, Vec3)> {
    let world = app.world_mut();
    let mut throws = world.query_filtered::<(Entity, &Transform), (
        With<LevelRoot>,
        Without<PlayerController>,
        Without<Monster>,
    )>();
    throws
        .iter(world)
        .map(|(entity, transform)| (entity, transform.translation))
        .collect()
}

#[test]
fn rapid_throws_are_independent_world_space_projectiles() {
    let mut app = app();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(0.0, 1.6, 0.0),
            Flashbangs(3),
        ))
        .id();
    app.update();
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(0.0, 0.0, -3.8)));
    app.world_mut()
        .spawn((Monster::default(), Transform::from_xyz(3.8, 0.0, 0.0)));
    throw(&mut app);
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .rotation = Quat::from_rotation_y(-FRAC_PI_2);
    throw(&mut app);
    assert_eq!(app.world().get::<Flashbangs>(player).unwrap().0, 1);
    let before = projectiles(&mut app);
    assert_eq!(before.len(), 2);
    assert!(before
        .iter()
        .all(|(entity, _)| app.world().get::<ChildOf>(*entity).is_none()));
    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(50.0, 1.6, 50.0);
    app.update();
    for (entity, position) in before {
        let after = app.world().get::<Transform>(entity).unwrap().translation;
        assert_ne!(position, after);
        assert!(after.x.abs() < 5.0 && after.z.abs() < 5.0);
    }
    for _ in 0..6 {
        app.update();
    }
    assert!(app.world().get::<Flashed>(player).is_some());
    assert_eq!(projectiles(&mut app).len(), 2);
    app.world_mut().entity_mut(player).despawn();
    app.update();
    assert!(projectiles(&mut app).is_empty());
}

#[test]
fn thrown_flashbang_follows_a_world_space_ballistic_arc_then_rests() {
    let mut app = app();
    let player = app
        .world_mut()
        .spawn((
            PlayerController,
            Transform::from_xyz(2.0, 1.6, 3.0),
            Flashbangs(1),
        ))
        .id();
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Right);
    let start = projectiles(&mut app);
    assert_eq!(start.len(), 1);
    let (projectile, start) = start[0];
    let eye = app.world().get::<Transform>(player).unwrap().translation;
    assert!(start.distance(eye) < 1.0);
    let flight = (FLASHBANG_BURST_DELAY / FRAME).round() as usize;
    let mut track = vec![start];
    for _ in 1..flight {
        app.update();
        track.push(
            app.world()
                .get::<Transform>(projectile)
                .unwrap()
                .translation,
        );
    }
    app.update();
    let landing = app
        .world()
        .get::<Transform>(projectile)
        .unwrap()
        .translation;
    assert!(track.iter().any(|position| position.y > start.y));
    assert!(landing.y < start.y);
    assert!(landing.y < 0.2);
    track.push(landing);
    assert!(track.windows(2).all(|pair| pair[1].z <= pair[0].z));
    assert!(landing.z < start.z);
    assert!((landing.xz() - eye.xz()).length() > 3.0);
    app.update();
    assert_eq!(
        app.world()
            .get::<Transform>(projectile)
            .unwrap()
            .translation,
        landing
    );
}
