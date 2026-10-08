use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use gameplay::{
    controller::{
        Flashlight, FlashlightBeam, PlayerController, PlayerControllerPlugin,
        PlayerControlsEnabled, RESTART_CHARGE,
    },
    levels::{GameplaySound, GameplaySoundKind},
};

fn app(camera: bool) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
    if camera {
        app.add_plugins(PlayerControllerPlugin::default());
    } else {
        app.add_plugins(PlayerControllerPlugin::default().without_camera());
    }
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
fn beam_is_view_attached_only_in_rendered_mode() {
    let (rendered, player) = app(true);
    let children = rendered.world().get::<Children>(player).unwrap();
    let beam = children
        .iter()
        .find(|child| rendered.world().get::<FlashlightBeam>(*child).is_some())
        .unwrap();
    let pose = rendered.world().get::<Transform>(beam).unwrap();
    assert!(pose.translation.z < 0.0);
    assert_eq!(pose.rotation * -Vec3::Z, -Vec3::Z);
    assert_eq!(
        rendered.world().get::<Visibility>(beam),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        rendered.world().get::<Flashlight>(player),
        Some(&Flashlight {
            charge: 1.0,
            on: false
        })
    );

    let (headless, player) = app(false);
    assert_eq!(
        headless.world().get::<Flashlight>(player),
        Some(&Flashlight {
            charge: 1.0,
            on: false
        })
    );
    assert!(headless.world().get::<Children>(player).is_none());
}

#[test]
fn click_held_while_player_spawns_does_not_toggle_until_released_and_pressed_again() {
    let (mut app, _) = app(false);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    assert!(!app.world().get::<Flashlight>(player).unwrap().on);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(app.world().get::<Flashlight>(player).unwrap().on);
}

#[test]
fn click_toggles_and_charge_drains_then_recharges() {
    let (mut app, player) = app(false);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    let active = *app.world().get::<Flashlight>(player).unwrap();
    assert!(active.on);
    assert!(active.charge < 1.0);
    app.world_mut()
        .get_mut::<Flashlight>(player)
        .unwrap()
        .charge = 0.001;
    app.update();
    let empty = *app.world().get::<Flashlight>(player).unwrap();
    assert!(!empty.on);
    assert_eq!(empty.charge, 0.0);
    app.update();
    assert!(app.world().get::<Flashlight>(player).unwrap().charge > 0.0);
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<Flashlight>(player).unwrap().charge >= RESTART_CHARGE);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(app.world().get::<Flashlight>(player).unwrap().on);
}

#[test]
fn controls_gate_toggle_and_charge_time() {
    let (mut app, player) = app(false);
    app.world_mut()
        .get_mut::<Flashlight>(player)
        .unwrap()
        .charge = 0.5;
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(
        *app.world().get::<Flashlight>(player).unwrap(),
        Flashlight {
            on: false,
            charge: 0.5
        }
    );
}

#[test]
fn empty_charge_requires_recovery_before_restart() {
    let mut light = Flashlight {
        charge: 0.0,
        on: false,
    };
    light.toggle();
    assert!(!light.on);
    light.advance(0.5);
    assert!(light.charge < RESTART_CHARGE);
    light.toggle();
    assert!(!light.on);
    light.advance(1.0);
    light.toggle();
    assert!(light.on);
}

#[test]
fn manual_switches_emit_one_click_each_but_depletion_and_blocked_toggles_are_silent() {
    let (mut app, player) = app(false);
    let clicks = |app: &mut App| -> Vec<GameplaySound> {
        app.world_mut()
            .resource_mut::<Messages<GameplaySound>>()
            .drain()
            .collect()
    };
    assert!(clicks(&mut app).is_empty());

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    let played = clicks(&mut app);
    assert_eq!(played.len(), 1);
    assert_eq!(played[0].kind, GameplaySoundKind::FlashlightClick);
    assert_eq!(played[0].position, Vec3::new(0.0, 1.6, 0.0));
    app.update();
    assert!(clicks(&mut app).is_empty());

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(clicks(&mut app).len(), 1);

    app.world_mut()
        .get_mut::<Flashlight>(player)
        .unwrap()
        .charge = 0.01;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    assert!(clicks(&mut app).is_empty());
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(clicks(&mut app).is_empty());

    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert!(clicks(&mut app).is_empty());
}
