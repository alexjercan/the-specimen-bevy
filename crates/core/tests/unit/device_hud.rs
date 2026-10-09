use game_ui::{DetectorReadout, FlashOverlay, FlashbangStatus};
use gameplay::{
    controller::PlayerController,
    levels::{Detector, DetectorReading, Flashbangs, Flashed, DETECTOR_RANGE},
};

use super::*;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(DeviceHudPlugin);
    app
}

#[test]
fn flashbang_hud_hidden_without_player() {
    let mut app = test_app();
    app.update();
    let mut huds = app.world_mut().query::<(&FlashbangStatus, &Visibility)>();
    let (status, visibility) = huds.single(app.world()).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
    assert_eq!(status.count, 0);
}

#[test]
fn flashbang_hud_shows_count_while_flashed() {
    let mut app = test_app();
    app.update();
    app.world_mut()
        .spawn((PlayerController, Flashbangs(2), Flashed { remaining: 2.5 }));
    app.update();
    let mut huds = app.world_mut().query::<(&FlashbangStatus, &Visibility)>();
    let (status, visibility) = huds.single(app.world()).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(status.count, 2);
}

#[test]
fn hud_prompts_follow_rebound_controls() {
    use game_settings::GameSettings;
    use game_ui::{FlashbangPrompt, FlashlightMeter, FlashlightPrompt};

    let mut app = test_app();
    app.add_plugins(super::super::FlashlightHudPlugin);
    let mut settings = GameSettings::default();
    settings.keys.flashbang = "KeyQ".into();
    settings.keys.flashlight = "MouseMiddle".into();
    app.insert_resource(settings);
    app.update();
    app.world_mut().spawn((
        PlayerController,
        Flashbangs(1),
        gameplay::controller::Flashlight::default(),
    ));
    app.update();
    let world = app.world_mut();
    let mut flashbang = world.query_filtered::<&Text, With<FlashbangPrompt>>();
    assert_eq!(flashbang.single(world).unwrap().0, "Q THROW");
    let mut flashlight = world.query_filtered::<&Text, With<FlashlightPrompt>>();
    assert_eq!(flashlight.single(world).unwrap().0, "MMB TOGGLE");
    let mut meters = world.query::<(&FlashlightMeter, &Visibility)>();
    assert_eq!(meters.single(world).unwrap().1, &Visibility::Inherited);
}

#[test]
fn detector_hud_tracks_reading() {
    let mut app = test_app();
    app.update();
    app.world_mut().spawn((
        PlayerController,
        Detector {
            reading: Some(DetectorReading {
                distance: 9.0,
                bearing: 1.0,
            }),
        },
    ));
    app.update();
    let mut huds = app.world_mut().query::<(&DetectorReadout, &Visibility)>();
    let (readout, visibility) = huds.single(app.world()).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(readout.signal.unwrap().distance, 9.0);
    assert_eq!(readout.signal.unwrap().range, DETECTOR_RANGE);
}

#[test]
fn flash_overlay_tracks_elapsed_and_hides_when_absent() {
    let mut app = test_app();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Flashed { remaining: 4.75 }))
        .id();
    app.update();
    {
        let mut huds = app.world_mut().query::<(&FlashOverlay, &Visibility)>();
        let (overlay, visibility) = huds.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!(overlay.elapsed, 0.0);
    }

    app.world_mut()
        .entity_mut(player)
        .insert(Flashed { remaining: 4.25 });
    app.update();
    {
        let mut huds = app.world_mut().query::<(&FlashOverlay, &Visibility)>();
        let (overlay, visibility) = huds.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Inherited);
        assert!((overlay.elapsed - 0.15).abs() < 1e-5);
    }

    app.world_mut()
        .entity_mut(player)
        .insert(Flashed { remaining: 3.0 });
    app.update();
    {
        let mut huds = app.world_mut().query::<(&FlashOverlay, &Visibility)>();
        let (overlay, visibility) = huds.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
        assert!((overlay.elapsed - 1.4).abs() < 1e-5);
    }

    app.world_mut().entity_mut(player).remove::<Flashed>();
    app.update();
    let mut huds = app.world_mut().query::<(&FlashOverlay, &Visibility)>();
    let (_, visibility) = huds.single(app.world()).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
}

#[test]
fn flashbang_hud_only_appears_with_inventory() {
    let mut app = test_app();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Flashbangs(0)))
        .id();
    let mut huds = app.world_mut().query::<(&FlashbangStatus, &Visibility)>();
    app.update();
    assert_eq!(
        huds.single(app.world()).unwrap(),
        (&FlashbangStatus { count: 0 }, &Visibility::Hidden)
    );

    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    app.update();
    assert_eq!(
        huds.single(app.world()).unwrap(),
        (&FlashbangStatus { count: 1 }, &Visibility::Inherited)
    );

    app.world_mut()
        .entity_mut(player)
        .insert((Flashbangs(0), Flashed { remaining: 1.0 }));
    app.update();
    assert_eq!(
        huds.single(app.world()).unwrap(),
        (&FlashbangStatus { count: 0 }, &Visibility::Hidden)
    );

    app.world_mut().entity_mut(player).remove::<Flashed>();
    app.update();
    assert_eq!(
        huds.single(app.world()).unwrap(),
        (&FlashbangStatus { count: 0 }, &Visibility::Hidden)
    );

    app.world_mut().entity_mut(player).insert(Flashbangs(1));
    app.update();
    assert_eq!(
        huds.single(app.world()).unwrap(),
        (&FlashbangStatus { count: 1 }, &Visibility::Inherited)
    );
}

#[test]
fn flash_overlay_does_not_block_ui_picking() {
    let mut app = test_app();
    app.update();
    let mut overlays = app
        .world_mut()
        .query_filtered::<&Pickable, With<FlashOverlay>>();
    assert_eq!(*overlays.single(app.world()).unwrap(), Pickable::IGNORE);
}
