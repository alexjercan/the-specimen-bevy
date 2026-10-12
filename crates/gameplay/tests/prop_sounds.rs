mod common;

use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{asset::uuid::Uuid, audio::SpatialListener, prelude::*};
use bevy_rand::prelude::{ChaCha8Rng, EntropyPlugin};
use game_audio::GameAudioPlugin;
use gameplay::facility::Module;
use gameplay::{
    ambience::{
        AmbiencePlugin, AmbienceSounds, AmbienceSoundsPlugin, AmbientCue, AmbientCueKind,
        AmbientSource, AmbientSourceKind, IntermittentSound, IntermittentTimer,
    },
    facility::{FacilityPowerPlugin, Prop},
    player::PlayerController,
};

fn app() -> App {
    let mut app = common::player_app(Duration::from_millis(16));
    app.add_plugins((FacilityPowerPlugin, AmbiencePlugin));
    app
}

struct Intermittent {
    kind: AmbientCueKind,
    interval: f32,
    variation: f32,
    remaining: f32,
}

fn intermittent_child(app: &App, entity: Entity) -> Option<Entity> {
    let world = app.world();
    world
        .get::<Children>(entity)
        .into_iter()
        .flat_map(|children| children.iter())
        .find(|child| world.get::<IntermittentSound>(*child).is_some())
}

fn intermittent(app: &App, entity: Entity) -> Option<Intermittent> {
    let child = intermittent_child(app, entity)?;
    let world = app.world();
    let sound = world.get::<IntermittentSound>(child)?;
    let timer = world.get::<IntermittentTimer>(child)?;
    Some(Intermittent {
        kind: sound.kind,
        interval: sound.interval,
        variation: sound.variation,
        remaining: timer.remaining,
    })
}

fn source_children(app: &App, entity: Entity) -> Vec<AmbientSourceKind> {
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
    let vent = app.world_mut().spawn(Prop(Module::WallVent)).id();
    let boiler = app.world_mut().spawn(Prop(Module::BoilerUnit)).id();
    let pipe = app.world_mut().spawn(Prop(Module::PipeManifold)).id();
    let tank = app
        .world_mut()
        .spawn(Prop(Module::ConceptContainmentTank))
        .id();
    let cool = app.world_mut().spawn(Prop(Module::CeilingLightCool)).id();
    let unrelated = app.world_mut().spawn(Prop(Module::StorageCrate)).id();
    app.update();

    assert_eq!(
        source_children(&app, vent),
        vec![AmbientSourceKind::Vent, AmbientSourceKind::VentWind]
    );
    assert_eq!(
        source_children(&app, boiler),
        vec![AmbientSourceKind::Boiler]
    );
    assert_eq!(source_children(&app, tank), vec![AmbientSourceKind::Tank]);
    assert_eq!(
        source_children(&app, cool),
        vec![AmbientSourceKind::CoolBuzz]
    );
    let tick = intermittent(&app, boiler).unwrap();
    assert!(matches!(tick.kind, AmbientCueKind::BoilerTick));
    assert!(tick.remaining > 0.0 && tick.remaining <= tick.interval + tick.variation);
    let faucet = intermittent(&app, pipe).unwrap();
    assert!(matches!(faucet.kind, AmbientCueKind::FaucetBurst));
    assert!(faucet.remaining > 0.0 && faucet.remaining <= faucet.interval + faucet.variation);
    assert!(source_children(&app, pipe).is_empty());
    assert!(source_children(&app, unrelated).is_empty());
    assert!(intermittent(&app, unrelated).is_none());
}

#[test]
fn adding_a_prop_to_an_existing_entity_attaches_sources_once() {
    let mut app = app();
    let entity = app.world_mut().spawn(Transform::IDENTITY).id();
    app.world_mut()
        .entity_mut(entity)
        .insert(Prop(Module::WallVent));
    app.update();
    assert_eq!(source_children(&app, entity).len(), 2);
    app.update();
    assert_eq!(source_children(&app, entity).len(), 2);
    app.world_mut().entity_mut(entity).despawn();
    assert!(app.world().get_entity(entity).is_err());
}

fn clip(id: u128) -> Handle<AudioSource> {
    Handle::<AudioSource>::Uuid(Uuid::from_u128(id), default())
}

fn voice_app() -> App {
    let mut app = common::player_app(Duration::from_millis(100));
    app.add_plugins((
        TransformPlugin,
        EntropyPlugin::<ChaCha8Rng>::with_seed([0; 32]),
        GameAudioPlugin,
        FacilityPowerPlugin,
        AmbiencePlugin,
        AmbienceSoundsPlugin,
    ))
    .insert_resource(AmbienceSounds {
        roomtone: clip(70),
        low_pressure: clip(71),
        conduit: clip(72),
        boiler: clip(73),
        tank: clip(74),
        vent: clip(75),
        vent_wind: clip(76),
        cool_buzz: clip(77),
        boiler_tick: clip(78),
        faucet_burst: clip(79),
    });
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut().spawn((
        PlayerController,
        SpatialListener::default(),
        Transform::from_xyz(2.0, 1.6, -3.0),
    ));
    app
}

fn voices(app: &mut App) -> Vec<(Entity, Entity, Vec3)> {
    let world = app.world_mut();
    world
        .query_filtered::<(Entity, &ChildOf, &GlobalTransform), With<AudioPlayer>>()
        .iter(world)
        .map(|(voice, parent, transform)| (voice, parent.parent(), transform.translation()))
        .collect()
}

fn burst(app: &mut App, module: Module, expected: Handle<AudioSource>) -> (Entity, Entity, Vec3) {
    let pose = Transform::from_xyz(4.0, 0.5, -3.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2));
    let prop = app.world_mut().spawn((Prop(module), pose)).id();
    app.update();
    let child = intermittent_child(app, prop).expect("prop has an intermittent sound");
    let offset = app.world().get::<IntermittentSound>(child).unwrap().offset;
    app.world_mut()
        .get_mut::<IntermittentTimer>(child)
        .unwrap()
        .remaining = 0.0;
    app.update();
    let cues: Vec<AmbientCue> = app
        .world()
        .resource::<Messages<AmbientCue>>()
        .iter_current_update_messages()
        .copied()
        .collect();
    assert_eq!(cues.len(), 1);
    assert_eq!((cues[0].prop, cues[0].offset), (prop, offset));
    for _ in 0..3 {
        app.update();
    }
    let found = voices(app);
    assert_eq!(found.len(), 1);
    let (voice, parent, position) = found[0];
    assert_eq!(parent, prop);
    assert_eq!(app.world().get::<AudioPlayer>(voice).unwrap().0, expected);
    let expected_position = pose.transform_point(offset);
    assert!(position.distance(expected_position) < 1e-4);
    (prop, voice, expected_position)
}

#[test]
fn intermittent_voices_play_from_the_prop_world_pose() {
    let mut app = voice_app();
    let (_, _, faucet) = burst(&mut app, Module::PipeManifold, clip(79));
    assert!(faucet.distance(Vec3::new(4.0, 0.5, -3.0)) > 0.1);
}

#[test]
fn intermittent_voices_end_when_their_prop_despawns() {
    let mut app = voice_app();
    for (module, handle) in [
        (Module::PipeManifold, clip(79)),
        (Module::BoilerUnit, clip(78)),
    ] {
        let (prop, voice, _) = burst(&mut app, module, handle);
        app.world_mut().entity_mut(prop).despawn();
        app.update();
        assert!(app.world().get_entity(voice).is_err());
        assert!(voices(&mut app).is_empty());
    }
}
