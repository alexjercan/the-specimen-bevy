use bevy::prelude::*;
use game_audio::{AmbientSound, Sound, SourceSounds};
use gameplay::levels::{AmbientSource, IntermittentSound, Prop, PropSoundsPlugin};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PropSoundsPlugin));
    app
}

fn source_children(app: &App, entity: Entity) -> Vec<AmbientSound> {
    let world = app.world();
    world
        .get::<Children>(entity)
        .into_iter()
        .flat_map(|children| children.iter())
        .filter_map(|child| Some(world.get::<AmbientSource>(child)?.kind))
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
        vec![AmbientSound::Vent, AmbientSound::VentWind]
    );
    assert_eq!(source_children(&app, boiler), vec![AmbientSound::Boiler]);
    assert_eq!(source_children(&app, tank), vec![AmbientSound::Tank]);
    assert_eq!(source_children(&app, cool), vec![AmbientSound::CoolBuzz]);
    let boiler_cues: Vec<Sound> = app
        .world()
        .get::<SourceSounds>(boiler)
        .unwrap()
        .0
        .iter()
        .map(|(sound, _)| *sound)
        .collect();
    assert_eq!(
        boiler_cues,
        vec![Sound::BreakerTrip, Sound::BoilerReset, Sound::BoilerRestart]
    );
    let tick = app.world().get::<IntermittentSound>(boiler).unwrap();
    assert!(matches!(tick.kind, Sound::BoilerTick));
    assert!(tick.remaining > 0.0 && tick.remaining <= tick.interval + tick.variation);
    let faucet = app.world().get::<IntermittentSound>(pipe).unwrap();
    assert!(matches!(faucet.kind, Sound::FaucetBurst));
    assert!(faucet.remaining > 0.0 && faucet.remaining <= faucet.interval + faucet.variation);
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
