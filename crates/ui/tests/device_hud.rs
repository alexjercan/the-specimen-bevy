use bevy::prelude::*;
use game_ui::{
    detector_readout, flash_alpha, flash_overlay, flashbang_status, tracker_blip, tracker_edge,
    tracker_range_text, tracker_status_text, DetectorReadout, DetectorSignal, FlashOverlay,
    FlashbangStatus, GameUiPlugin, TrackerEdge,
};

fn signal(distance: f32, bearing_deg: f32, range: f32) -> DetectorSignal {
    DetectorSignal {
        distance,
        bearing: bearing_deg.to_radians(),
        range,
    }
}

#[test]
fn tracker_blip_is_none_without_signal() {
    assert_eq!(tracker_blip(None), None);
}

#[test]
fn tracker_blip_ahead_points_straight_up() {
    let offset = tracker_blip(Some(signal(12.5, 0.0, 25.0))).unwrap();
    assert!(offset.x.abs() < 0.001);
    assert!(offset.y < 0.0);
}

#[test]
fn tracker_blip_shows_at_the_fan_edge() {
    let offset = tracker_blip(Some(signal(25.0, 50.0, 25.0)));
    assert!(offset.is_some());
}

#[test]
fn tracker_blip_hides_just_outside_the_fan() {
    assert_eq!(tracker_blip(Some(signal(10.0, 51.0, 25.0))), None);
    assert_eq!(tracker_blip(Some(signal(10.0, -51.0, 25.0))), None);
}

#[test]
fn tracker_edge_is_none_without_signal_or_in_fan() {
    assert_eq!(tracker_edge(None), None);
    assert_eq!(tracker_edge(Some(signal(10.0, 0.0, 25.0))), None);
    assert_eq!(tracker_edge(Some(signal(10.0, 50.0, 25.0))), None);
}

#[test]
fn tracker_edge_reports_the_side_outside_the_fan() {
    assert_eq!(
        tracker_edge(Some(signal(10.0, 51.0, 25.0))),
        Some(TrackerEdge::Right)
    );
    assert_eq!(
        tracker_edge(Some(signal(10.0, -51.0, 25.0))),
        Some(TrackerEdge::Left)
    );
}

#[test]
fn tracker_range_text_formats_distance_or_dashes() {
    assert_eq!(tracker_range_text(None), "-- M");
    assert_eq!(tracker_range_text(Some(signal(12.4, 0.0, 25.0))), "12 M");
}

#[test]
fn tracker_status_text_reports_track_edge_or_no_signal() {
    assert_eq!(tracker_status_text(None), "NO SIG");
    assert_eq!(tracker_status_text(Some(signal(10.0, 0.0, 25.0))), "TRACK");
    assert_eq!(tracker_status_text(Some(signal(10.0, 51.0, 25.0))), "EDGE");
}

#[test]
fn flash_alpha_decays_to_zero() {
    assert_eq!(flash_alpha(0.0), 0.55);
    assert_eq!(flash_alpha(0.9), 0.0);
    assert_eq!(flash_alpha(5.0), 0.0);
}

#[test]
fn flashbang_panel_sits_above_flashlight_and_shows_item_count() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(flashbang_status()).id();
    app.update();
    let node = app.world().get::<Node>(hud).unwrap();
    assert_eq!(node.bottom, px(136));
    assert_eq!(node.left, px(24));
    assert_eq!(node.width, px(184));

    app.world_mut()
        .entity_mut(hud)
        .insert(FlashbangStatus { count: 2 });
    app.update();
    let world = app.world_mut();
    let mut texts = world.query::<&Text>();
    assert!(texts.iter(world).any(|text| text.0 == "FLASHBANG x2"));
    assert!(texts.iter(world).any(|text| text.0 == "RMB THROW"));
    let mut names = world.query::<&Name>();
    assert!(names
        .iter(world)
        .any(|name| name.as_str() == "Flashbang icon"));
    assert!(!names
        .iter(world)
        .any(|name| name.as_str() == "Flashbang fill"));
    let mut icons = world.query::<(&Name, &BackgroundColor)>();
    let (_, color) = icons
        .iter(world)
        .find(|(name, _)| name.as_str() == "Flashbang icon")
        .unwrap();
    assert_eq!(color.0, game_ui::theme::FUSE_METAL);
    world.entity_mut(hud).insert(FlashbangStatus { count: 0 });
    app.update();
    let world = app.world_mut();
    let (_, color) = icons
        .iter(world)
        .find(|(name, _)| name.as_str() == "Flashbang icon")
        .unwrap();
    assert_eq!(color.0, game_ui::theme::FUSE_EMPTY);
}

#[test]
fn tracker_root_sits_at_bottom_center() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(detector_readout()).id();
    app.update();
    let node = app.world().get::<Node>(hud).unwrap();
    assert_eq!(node.bottom, px(18));
    assert_eq!(node.left, percent(50));
    assert_eq!(node.width, px(236));
    assert_eq!(node.height, px(168));
    assert_eq!(node.margin.left, px(-118));
}

#[test]
fn tracker_blip_shows_and_tracks_position_while_in_fan() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(detector_readout()).id();
    app.update();

    let ahead = signal(10.0, 0.0, 25.0);
    app.world_mut().entity_mut(hud).insert(DetectorReadout {
        signal: Some(ahead),
    });
    app.update();
    let offset_ahead = tracker_blip(Some(ahead)).unwrap();
    let (left_ahead, top_ahead) = {
        let world = app.world_mut();
        let mut blips = world.query::<(&Node, &Visibility)>();
        let (node, visibility) = blips
            .iter(world)
            .find(|(node, _)| node.width == px(8))
            .unwrap();
        assert_eq!(*visibility, Visibility::Inherited);
        let (Val::Px(left), Val::Px(top)) = (node.left, node.top) else {
            panic!("expected pixel positions");
        };
        (left, top)
    };

    let right_edge = signal(20.0, 25.0, 25.0);
    app.world_mut().entity_mut(hud).insert(DetectorReadout {
        signal: Some(right_edge),
    });
    app.update();
    let offset_right = tracker_blip(Some(right_edge)).unwrap();
    let world = app.world_mut();
    let mut blips = world.query::<(&Node, &Visibility)>();
    let (node, visibility) = blips
        .iter(world)
        .find(|(node, _)| node.width == px(8))
        .unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    let (Val::Px(left), Val::Px(top)) = (node.left, node.top) else {
        panic!("expected pixel positions");
    };
    assert!((left - left_ahead - (offset_right.x - offset_ahead.x)).abs() < 0.001);
    assert!((top - top_ahead - (offset_right.y - offset_ahead.y)).abs() < 0.001);
}

#[test]
fn tracker_blip_hides_and_status_reads_no_sig_without_signal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(detector_readout()).id();
    app.update();
    app.world_mut().entity_mut(hud).insert(DetectorReadout {
        signal: Some(signal(10.0, 0.0, 25.0)),
    });
    app.update();
    app.world_mut()
        .entity_mut(hud)
        .insert(DetectorReadout { signal: None });
    app.update();
    let world = app.world_mut();
    let mut blips = world.query::<(&Node, &Visibility)>();
    let (_, visibility) = blips
        .iter(world)
        .find(|(node, _)| node.width == px(8))
        .unwrap();
    assert_eq!(*visibility, Visibility::Hidden);

    let mut texts = world.query::<&Text>();
    assert!(texts.iter(world).any(|text| text.0 == "NO SIG"));
    assert!(texts.iter(world).any(|text| text.0 == "-- M"));
}

#[test]
fn tracker_leds_light_on_the_side_outside_the_fan() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let hud = app.world_mut().spawn(detector_readout()).id();
    app.update();
    app.world_mut().entity_mut(hud).insert(DetectorReadout {
        signal: Some(signal(10.0, 51.0, 25.0)),
    });
    app.update();
    let world = app.world_mut();
    let mut leds = world.query::<(&Name, &BackgroundColor)>();
    let lit = Color::srgb(1.0, 0.52, 0.06);
    let dim = Color::srgba(0.75, 0.4, 0.05, 0.22);
    for (name, background) in leds.iter(world) {
        if name.as_str() == "Tracker LED right" {
            assert_eq!(background.0, lit);
        } else if name.as_str() == "Tracker LED left" {
            assert_eq!(background.0, dim);
        }
    }
}

#[test]
fn flash_overlay_fades_as_elapsed_grows() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameUiPlugin));
    let overlay = app.world_mut().spawn(flash_overlay()).id();
    app.update();
    app.world_mut()
        .entity_mut(overlay)
        .insert(FlashOverlay { elapsed: 0.45 });
    app.update();
    let background = app.world().get::<BackgroundColor>(overlay).unwrap();
    assert_eq!(background.0.alpha(), flash_alpha(0.45));
}
