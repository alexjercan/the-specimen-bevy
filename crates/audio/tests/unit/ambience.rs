use super::*;

#[test]
fn ambience_without_layout_or_assets_is_inert() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<AmbienceActive>()
        .init_resource::<AudioPaused>()
        .init_resource::<ConduitAmbience>()
        .add_systems(Update, update_ambience);
    app.world_mut().resource_mut::<AmbienceActive>().0 = true;
    app.update();
    let world = app.world_mut();
    let mut query = world.query::<&AmbientVoice>();
    assert_eq!(query.iter(world).count(), 0);
}

#[test]
fn sound_layout_is_handle_independent() {
    let sources = [
        AmbientEmitter { sound: AmbientSound::Roomtone, position: None, volume: 0.12 },
        AmbientEmitter { sound: AmbientSound::Vent, position: Some(Vec3::new(1.0, 2.0, 3.0)), volume: 0.09 },
    ];
    assert!(sources[0].position.is_none());
    assert!(sources[1].position.is_some());
    assert!(sources.iter().all(|source| (0.0..=0.2).contains(&source.volume)));
}
