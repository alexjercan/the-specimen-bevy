use super::*;

#[test]
fn death_camera_falls_without_pushing_toward_the_monster() {
    let view = Transform::from_xyz(4.0, 1.6, -8.0).with_rotation(Quat::from_rotation_y(0.6));
    let fallen = fallen_view(view);
    assert_eq!(fallen.translation.x, view.translation.x);
    assert_eq!(fallen.translation.z, view.translation.z);
    assert_eq!(fallen.translation.y, FALL_HEIGHT);
    assert!((fallen.rotation.angle_between(view.rotation) - FALL_ROLL).abs() < 1e-5);
}
