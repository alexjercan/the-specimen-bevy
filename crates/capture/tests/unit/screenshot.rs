use super::*;

#[test]
fn screenshot_written_tracks_exact_paths() {
    let mut world = World::new();
    let path = PathBuf::from("art/visuals/screenshots/a.png");
    assert!(!screenshot_written(&world, &path));
    world.init_resource::<CaptureLog>();
    assert!(!screenshot_written(&world, &path));
    world
        .resource_mut::<CaptureLog>()
        .written
        .insert(path.clone());
    assert!(screenshot_written(&world, &path));
    assert!(screenshot_written_at(path.clone())(&world));
    assert!(!screenshot_written(&world, "a.png"));
    assert!(!screenshot_written_at("a.png")(&world));
    world.init_resource::<CaptureState>();
    screenshot_start(&mut world, &path);
    assert!(!screenshot_written(&world, &path));
}
