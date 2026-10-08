use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, Sound};
use gameplay::{
    controller::{
        PlayerController, PlayerControllerPlugin, PlayerControlsEnabled, SprintExhausted, Stamina,
        RUN_SPEED, SPRINT_DRAIN_SECONDS, SPRINT_RECHARGE_SECONDS, SPRINT_RESTART_CHARGE,
        WALK_SPEED,
    },
    levels::{Hidden, HidingMotion, HidingPhase},
};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera());
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

#[test]
fn stamina_drains_only_while_moving_and_sprinting() {
    let mut stamina = Stamina::default();
    assert!(!stamina.advance(true, false, 1.0));
    assert_eq!(stamina.charge, 1.0);
    assert!(!stamina.advance(true, true, SPRINT_DRAIN_SECONDS / 2.0));
    assert_eq!(stamina.charge, 0.5);
    assert!(stamina.sprinting);
    assert!(!stamina.advance(false, true, SPRINT_RECHARGE_SECONDS / 2.0));
    assert_eq!(stamina.charge, 1.0);
    assert!(!stamina.sprinting);
}

#[test]
fn exhaustion_occurs_once_and_requires_release_and_recovery() {
    let mut stamina = Stamina::default();
    assert!(stamina.advance(true, true, SPRINT_DRAIN_SECONDS));
    assert_eq!(stamina.charge, 0.0);
    assert!(stamina.exhausted);
    assert!(!stamina.advance(true, true, SPRINT_RECHARGE_SECONDS));
    assert_eq!(stamina.charge, 1.0);
    assert!(!stamina.sprinting);
    assert!(!stamina.advance(false, true, 0.0));
    assert!(!stamina.exhausted);
    assert!(!stamina.advance(true, true, 0.1));
    assert!(stamina.sprinting);

    stamina.charge = 0.0;
    stamina.exhausted = true;
    assert!(!stamina.advance(
        false,
        false,
        SPRINT_RESTART_CHARGE * SPRINT_RECHARGE_SECONDS / 2.0
    ));
    assert!(stamina.exhausted);
    assert!(!stamina.advance(
        false,
        false,
        SPRINT_RESTART_CHARGE * SPRINT_RECHARGE_SECONDS / 2.0
    ));
    assert!(!stamina.advance(false, false, 0.0));
    assert!(!stamina.exhausted);
}

#[test]
fn hiding_recharges_stamina_even_when_movement_and_shift_are_held() {
    let (mut app, player) = app();
    let spot = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(player).insert(Hidden {
        spot,
        height: 1.6,
        phase: HidingPhase::Hidden,
    });
    app.world_mut().get_mut::<Stamina>(player).unwrap().charge = 0.25;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);

    let before = app.world().get::<Transform>(player).unwrap().translation;
    app.update();
    let stamina = app.world().get::<Stamina>(player).unwrap();
    assert!((stamina.charge - (0.25 + 0.1 / SPRINT_RECHARGE_SECONDS)).abs() < 0.001);
    assert!(!stamina.sprinting);
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        before
    );

    app.world_mut().get_mut::<Hidden>(player).unwrap().phase =
        HidingPhase::Entering(HidingMotion {
            from: before,
            rotation: Quat::IDENTITY,
            progress: 0.0,
        });
    app.update();
    let stamina = app.world().get::<Stamina>(player).unwrap();
    assert!((stamina.charge - (0.25 + 0.2 / SPRINT_RECHARGE_SECONDS)).abs() < 0.001);
    assert!(!stamina.sprinting);
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        before
    );
}

#[test]
fn shift_falls_back_to_walk_when_exhausted_and_emits_one_event() {
    let (mut app, player) = app();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();
    assert!(app.world().get::<Stamina>(player).unwrap().sprinting);
    assert!(
        (app.world().get::<Transform>(player).unwrap().translation.z + RUN_SPEED * 0.1).abs()
            < 0.01
    );

    app.world_mut().get_mut::<Stamina>(player).unwrap().charge = 0.01;
    app.update();
    let stamina = app.world().get::<Stamina>(player).unwrap();
    assert!(stamina.exhausted);
    assert!(!stamina.sprinting);
    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<SprintExhausted>>()
        .drain()
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].position.y, 1.6);
    let cues: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].sound, Sound::SprintExhausted);
    assert_eq!(cues[0].position, None);
    let z = app.world().get::<Transform>(player).unwrap().translation.z;
    app.update();
    assert!(
        (app.world().get::<Transform>(player).unwrap().translation.z - z + WALK_SPEED * 0.1).abs()
            < 0.01
    );
    assert!(app
        .world_mut()
        .resource_mut::<Messages<SprintExhausted>>()
        .drain()
        .next()
        .is_none());

    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    let charge = app.world().get::<Stamina>(player).unwrap().charge;
    app.update();
    assert_eq!(app.world().get::<Stamina>(player).unwrap().charge, charge);
}
