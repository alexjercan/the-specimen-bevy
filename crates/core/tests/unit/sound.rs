use super::*;

#[test]
fn ambience_layout_has_one_bed_and_local_facility_sources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(SoundGluePlugin);
    let sources = &app.world().resource::<AmbientEmitters>().0;
    assert_eq!(sources.len(), 10);
    assert_eq!(sources.iter().filter(|emitter| emitter.position.is_none()).count(), 2);
    assert_eq!(sources.iter().filter(|emitter| emitter.sound == AmbientSound::Vent).count(), 3);
    assert_eq!(sources.iter().filter(|emitter| emitter.sound == AmbientSound::CoolBuzz).count(), 3);
    assert!(sources.iter().all(|emitter| emitter.volume > 0.0 && emitter.volume <= 0.2));
}

#[test]
fn hiding_cues_keep_their_kind_and_world_position_in_audio_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<GameplaySound>()
        .add_message::<PlaySound>()
        .add_systems(Update, forward_gameplay_sounds);

    let cues = [
        (GameplaySoundKind::LockerOpen, Sound::LockerOpen),
        (GameplaySoundKind::LockerClose, Sound::LockerClose),
        (GameplaySoundKind::TableEnter, Sound::TableEnter),
        (GameplaySoundKind::TableLeave, Sound::TableLeave),
    ];
    for (index, &(kind, _)) in cues.iter().enumerate() {
        app.world_mut().write_message(GameplaySound {
            kind,
            position: Vec3::new(index as f32, 0.0, -2.0),
        });
    }
    app.update();

    let played: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect();
    assert_eq!(played.len(), cues.len());
    for (index, (play, &(_, sound))) in played.iter().zip(&cues).enumerate() {
        assert_eq!(play.sound, sound);
        assert_eq!(play.position, Some(Vec3::new(index as f32, 0.0, -2.0)));
    }
}
