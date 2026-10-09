use super::*;

#[test]
fn picked_up_item_lifts_into_the_camera_hand() {
    let from = Vec3::new(2.0, 0.9, -3.0);
    let hand = Vec3::new(2.0, 1.4, -2.5);
    assert_eq!(pickup_position(from, hand, 0.0), from);
    assert!(pickup_position(from, hand, PICKUP_MOTION_SECS / 2.0).y > 1.15);
    assert!((pickup_position(from, hand, PICKUP_MOTION_SECS) - hand).length() < 1e-5);
}

#[test]
fn thrown_flashbang_follows_a_spinning_arc_before_burst() {
    let start = throw_position(0.0, -1.52);
    let middle = throw_position(THROW_SECS / 2.0, -1.52);
    let landing = throw_position(THROW_SECS, -1.52);
    assert!(middle.y > start.y);
    assert!(landing.z < middle.z);
    assert_eq!(landing.y, -1.52);
    assert_eq!(throw_position(THROW_SECS + 0.2, -1.52), landing);
}
