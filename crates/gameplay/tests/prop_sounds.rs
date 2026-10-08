use bevy::prelude::*;
use game_audio::{AmbientSound, Sound, SourceSounds};
use gameplay::levels::{AmbientSource, IntermittentSound, Prop, PropSoundsPlugin};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PropSoundsPlugin));
    app
}

fn source_children(app: &App, entity: Entity) -> Vec<(AmbientSound, f32, Vec3)> {
    let world = app.world();
    world
        .get::<Children>(entity)
        .into_iter()
        .flat_map(|children| children.iter())
        .filter_map(|child| {
            Some((
                world.get::<AmbientSource>(child)?.kind,
                world.get::<AmbientSource>(child)?.volume,
                world.get::<Transform>(child)?.translation,
            ))
        })
        .collect()
}

#[test]
fn sound_sources_follow_prop_type_without_level_builder_setup() {
    let mut app = app();
    let vent = app.world_mut().spawn(Prop("wall_vent".into())).id();
    let boiler = app.world_mut().spawn(Prop("boiler_unit".into())).id();
    let pipe = app.world_mut().spawn(Prop("pipe_manifold".into())).id();
    let tank = app
        .world_mut()
        .spawn(Prop("concept_containment_tank".into()))
        .id();
    let cool = app
        .world_mut()
        .spawn(Prop("ceiling_light_cool".into()))
        .id();
    let unrelated = app.world_mut().spawn(Prop("storage_crate".into())).id();
    app.update();

    assert_eq!(
        source_children(&app, vent),
        vec![
            (AmbientSound::Vent, 0.09, Vec3::ZERO),
            (AmbientSound::VentWind, 0.035, Vec3::ZERO),
        ]
    );
    assert_eq!(
        source_children(&app, boiler),
        vec![(AmbientSound::Boiler, 0.17, Vec3::Y)]
    );
    assert_eq!(
        source_children(&app, tank),
        vec![(AmbientSound::Tank, 0.15, Vec3::Y * 1.3)]
    );
    assert_eq!(
        source_children(&app, cool),
        vec![(AmbientSound::CoolBuzz, 0.045, Vec3::Y * 2.7)]
    );
    assert_eq!(
        app.world().get::<SourceSounds>(boiler).unwrap().0,
        vec![
            (Sound::BreakerTrip, Vec3::Y * 1.2),
            (Sound::BoilerReset, Vec3::Y * 1.2),
            (Sound::BoilerRestart, Vec3::Y * 1.2),
        ]
    );
    let tick = app.world().get::<IntermittentSound>(boiler).unwrap();
    assert!(matches!(tick.kind, Sound::BoilerTick));
    assert_eq!(tick.offset, Vec3::Y * 1.2);
    assert_eq!(
        (tick.range, tick.interval, tick.variation, tick.remaining),
        (18.0, 6.0, 14.0, 8.0)
    );
    let faucet = app.world().get::<IntermittentSound>(pipe).unwrap();
    assert!(matches!(faucet.kind, Sound::FaucetBurst));
    assert_eq!(faucet.offset, Vec3::new(0.0, -0.1, -0.25));
    assert_eq!(
        (
            faucet.range,
            faucet.interval,
            faucet.variation,
            faucet.remaining
        ),
        (12.0, 10.0, 8.0, 10.0)
    );
    assert!(source_children(&app, pipe).is_empty());
    assert!(source_children(&app, unrelated).is_empty());
    assert!(app.world().get::<IntermittentSound>(unrelated).is_none());
    assert!(app.world().get::<SourceSounds>(unrelated).is_none());
}

#[test]
fn adding_a_prop_to_an_existing_entity_attaches_sources_once() {
    let mut app = app();
    let entity = app.world_mut().spawn(Transform::IDENTITY).id();
    app.world_mut()
        .entity_mut(entity)
        .insert(Prop("wall_vent".into()));
    app.update();
    assert_eq!(source_children(&app, entity).len(), 2);
    app.update();
    assert_eq!(source_children(&app, entity).len(), 2);
    app.world_mut().entity_mut(entity).despawn();
    assert!(app.world().get_entity(entity).is_err());
}
