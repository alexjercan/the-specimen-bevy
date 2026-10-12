mod common;

use std::time::Duration;

use bevy::{asset::uuid::Uuid, prelude::*};
use game_audio::{
    AmbienceActive, AmbientEmitters, Bus, Curve, Falloff, GameAudioPlugin, Placement, PlaySound,
    LISTENER_GAIN,
};
use gameplay::{
    ambience::{
        AmbiencePlugin, AmbienceSounds, AmbienceSoundsPlugin, AmbienceViewPlugin, AmbientCue,
        AmbientCueKind, AmbientSource, AmbientSourceKind,
    },
    devices::{DeviceCue, DevicePlugin, DeviceSounds, DeviceSoundsPlugin},
    facility::{
        DoorCue, DoorCueKind, DoorPlugin, FacilityPower, FacilityPowerPlugin, FacilitySounds,
        FacilitySoundsPlugin, MainsPowered, Module, PowerCue, Room, Walls, BOILER_CUE_OFFSET,
        DOOR_CUE_OFFSET,
    },
    fuses::{FuseCue, FusePlugin, FuseSounds, FuseSoundsPlugin},
    hiding::{
        HidingCue, HidingCueKind, HidingPlugin, HidingSounds, HidingSoundsPlugin,
        LOCKER_CUE_OFFSET, TABLE_CUE_OFFSET,
    },
    interaction::{InteractionSounds, InteractionSoundsPlugin, PickupCue, PickupPlugin},
    monster::{MonsterCue, MonsterCueKind, MonsterPlugin, MonsterSounds, MonsterSoundsPlugin},
    player::{PlayerController, PlayerCue, PlayerSounds, PlayerSoundsPlugin},
};

fn clip(id: u128) -> Handle<AudioSource> {
    Handle::<AudioSource>::Uuid(Uuid::from_u128(id), default())
}

fn app() -> App {
    let mut app = common::player_app(Duration::from_millis(16));
    app.add_plugins((
        TransformPlugin,
        GameAudioPlugin,
        DoorPlugin,
        FacilityPowerPlugin,
        MonsterPlugin,
        DevicePlugin,
        PickupPlugin,
        FusePlugin,
        HidingPlugin,
        AmbiencePlugin,
    ));
    app
}

fn play<T: Message>(app: &mut App, cue: T) -> Vec<PlaySound> {
    app.world_mut().write_message(cue);
    app.update();
    app.world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .drain()
        .collect()
}

fn global(sound: &PlaySound) -> Handle<AudioSource> {
    assert_eq!(sound.bus, Bus::World);
    assert_eq!(
        sound.placement,
        Placement::Global {
            gain: LISTENER_GAIN
        }
    );
    sound.handle.clone()
}

fn from(
    sound: &PlaySound,
    expected: Entity,
    expected_offset: Vec3,
) -> (Handle<AudioSource>, Falloff) {
    assert_eq!(sound.bus, Bus::World);
    let Placement::From {
        source,
        offset,
        falloff,
    } = sound.placement
    else {
        panic!("expected a spatial cue attached to a source");
    };
    assert_eq!(source, expected);
    assert_eq!(offset, expected_offset);
    (sound.handle.clone(), falloff)
}

#[test]
fn player_cues_play_flat_on_the_world_bus_and_cycle_three_steps() {
    let mut app = app();
    app.add_plugins(PlayerSoundsPlugin)
        .insert_resource(PlayerSounds {
            steps: [clip(1), clip(2), clip(3)],
            sprint_exhausted: clip(4),
            flashlight_click: clip(5),
        });
    for (index, expected) in [(0, 1), (1, 2), (2, 3), (3, 1), (7, 2)] {
        let played = play(&mut app, PlayerCue::Step(index));
        assert_eq!(played.len(), 1);
        assert_eq!(global(&played[0]), clip(expected));
    }
    assert_eq!(
        global(&play(&mut app, PlayerCue::SprintExhausted)[0]),
        clip(4)
    );
    assert_eq!(
        global(&play(&mut app, PlayerCue::FlashlightClick)[0]),
        clip(5)
    );
}

#[test]
fn player_cues_are_silent_until_the_sounds_arrive() {
    let mut app = app();
    app.add_plugins(PlayerSoundsPlugin);
    assert!(play(&mut app, PlayerCue::FlashlightClick).is_empty());
}

#[test]
fn monster_cues_follow_the_monster_with_their_own_falloffs() {
    let mut app = app();
    app.add_plugins(MonsterSoundsPlugin)
        .insert_resource(MonsterSounds {
            presence: clip(10),
            steps: [clip(11), clip(12), clip(13), clip(14), clip(15), clip(16)],
            detected: clip(17),
            attack: clip(18),
            heartbeat: clip(19),
        });
    let monster = app.world_mut().spawn(Transform::IDENTITY).id();
    let offset = Vec3::Y;

    let played = play(
        &mut app,
        MonsterCue {
            monster,
            kind: MonsterCueKind::Step(7),
        },
    );
    let (handle, falloff) = from(&played[0], monster, offset);
    assert_eq!(handle, clip(12));
    assert_eq!(falloff.range, 16.0);
    assert_eq!(falloff.scale, 0.12);
    assert_eq!(falloff.curve, Curve::SquaredFade { gain: 0.8 });

    let played = play(
        &mut app,
        MonsterCue {
            monster,
            kind: MonsterCueKind::Presence,
        },
    );
    let (handle, falloff) = from(&played[0], monster, offset);
    assert_eq!(handle, clip(10));
    assert_eq!(falloff.range, 32.0);

    let played = play(
        &mut app,
        MonsterCue {
            monster,
            kind: MonsterCueKind::Detected,
        },
    );
    let (handle, falloff) = from(&played[0], monster, offset);
    assert_eq!(handle, clip(17));
    assert_eq!(
        falloff.curve,
        Curve::InverseSquare {
            gain: 1.0,
            coefficient: 0.015
        }
    );

    for (kind, expected) in [
        (MonsterCueKind::Attack, 18),
        (MonsterCueKind::Heartbeat, 19),
    ] {
        let played = play(&mut app, MonsterCue { monster, kind });
        let (handle, falloff) = from(&played[0], monster, offset);
        assert_eq!(handle, clip(expected));
        assert_eq!(falloff, Falloff::DEFAULT);
    }
}

#[test]
fn door_cues_attach_to_the_door_and_power_cues_to_the_boiler_or_listener() {
    let mut app = app();
    app.add_plugins(FacilitySoundsPlugin)
        .insert_resource(FacilitySounds {
            door_unlatch: clip(20),
            door_swing: clip(21),
            door_shut: clip(22),
            door_locked: clip(23),
            power_down: clip(24),
            breaker_trip: clip(25),
            boiler_reset: clip(26),
            boiler_restart: clip(27),
        });
    let door = app.world_mut().spawn(Transform::IDENTITY).id();
    let boiler = app.world_mut().spawn(Transform::IDENTITY).id();

    for (kind, expected) in [
        (DoorCueKind::Unlatch, 20),
        (DoorCueKind::Swing, 21),
        (DoorCueKind::Shut, 22),
        (DoorCueKind::Locked, 23),
    ] {
        let played = play(&mut app, DoorCue { door, kind });
        assert_eq!(played.len(), 1);
        let (handle, falloff) = from(&played[0], door, DOOR_CUE_OFFSET);
        assert_eq!(handle, clip(expected));
        assert_eq!(falloff, Falloff::DEFAULT);
    }

    let played = play(&mut app, PowerCue::PowerDown);
    assert_eq!(played.len(), 1);
    assert_eq!(global(&played[0]), clip(24));

    for (cue, expected) in [
        (PowerCue::BreakerTrip { boiler }, 25),
        (PowerCue::BoilerReset { boiler }, 26),
        (PowerCue::BoilerRestart { boiler }, 27),
    ] {
        let played = play(&mut app, cue);
        assert_eq!(played.len(), 1);
        let (handle, falloff) = from(&played[0], boiler, BOILER_CUE_OFFSET);
        assert_eq!(handle, clip(expected));
        assert_eq!(falloff, Falloff::DEFAULT);
    }
}

#[test]
fn device_cues_play_flat_on_the_world_bus() {
    let mut app = app();
    app.add_plugins(DeviceSoundsPlugin)
        .insert_resource(DeviceSounds {
            detector_nearby: clip(30),
            flashbang_throw: clip(31),
            flashbang_burst: clip(32),
        });
    for (cue, expected) in [
        (DeviceCue::DetectorNearby, 30),
        (DeviceCue::FlashbangThrow, 31),
        (DeviceCue::FlashbangBurst, 32),
    ] {
        let played = play(&mut app, cue);
        assert_eq!(played.len(), 1);
        assert_eq!(global(&played[0]), clip(expected));
    }
}

#[test]
fn pickup_cues_pick_the_fuse_slot_by_count_and_skip_out_of_range_counts() {
    let mut app = app();
    app.add_plugins(InteractionSoundsPlugin)
        .insert_resource(InteractionSounds {
            fuse_slots: [clip(40), clip(41), clip(42)],
            flashbang_pickup: clip(43),
            detector_pickup: clip(44),
        });
    for (count, expected) in [(1, 40), (2, 41), (3, 42)] {
        let played = play(&mut app, PickupCue::Fuse { count });
        assert_eq!(played.len(), 1);
        assert_eq!(global(&played[0]), clip(expected));
    }
    for count in [0, 4] {
        assert!(play(&mut app, PickupCue::Fuse { count }).is_empty());
    }
    assert_eq!(global(&play(&mut app, PickupCue::Flashbang)[0]), clip(43));
    assert_eq!(global(&play(&mut app, PickupCue::Detector)[0]), clip(44));
}

#[test]
fn fuse_completion_plays_flat_on_the_world_bus() {
    let mut app = app();
    app.add_plugins(FuseSoundsPlugin)
        .insert_resource(FuseSounds { complete: clip(50) });
    let played = play(&mut app, FuseCue::Complete);
    assert_eq!(played.len(), 1);
    assert_eq!(global(&played[0]), clip(50));
}

#[test]
fn hiding_cues_attach_to_the_spot_with_the_locker_or_table_offset() {
    let mut app = app();
    app.add_plugins(HidingSoundsPlugin)
        .insert_resource(HidingSounds {
            locker_open: clip(60),
            locker_close: clip(61),
            table_enter: clip(62),
            table_leave: clip(63),
        });
    let spot = app.world_mut().spawn(Transform::IDENTITY).id();
    for (kind, expected, offset) in [
        (HidingCueKind::LockerOpen, 60, LOCKER_CUE_OFFSET),
        (HidingCueKind::LockerClose, 61, LOCKER_CUE_OFFSET),
        (HidingCueKind::TableEnter, 62, TABLE_CUE_OFFSET),
        (HidingCueKind::TableLeave, 63, TABLE_CUE_OFFSET),
    ] {
        let played = play(&mut app, HidingCue { spot, kind });
        assert_eq!(played.len(), 1);
        let (handle, falloff) = from(&played[0], spot, offset);
        assert_eq!(handle, clip(expected));
        assert_eq!(falloff, Falloff::DEFAULT);
    }
}

fn ambience_sounds() -> AmbienceSounds {
    AmbienceSounds {
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
    }
}

#[test]
fn ambient_cues_play_from_their_prop() {
    let mut app = app();
    app.add_plugins(AmbienceSoundsPlugin)
        .insert_resource(ambience_sounds());
    let prop = app
        .world_mut()
        .spawn(Transform::from_xyz(3.0, 1.0, -2.0))
        .id();
    let offset = Vec3::new(0.0, -0.1, -0.25);
    for (kind, expected) in [
        (AmbientCueKind::BoilerTick, 78),
        (AmbientCueKind::FaucetBurst, 79),
    ] {
        let played = play(&mut app, AmbientCue { kind, prop, offset });
        assert_eq!(played.len(), 1);
        assert_eq!(played[0].bus, Bus::World);
        assert_eq!(played[0].handle, clip(expected));
        assert_eq!(
            played[0].placement,
            Placement::From {
                source: prop,
                offset,
                falloff: Falloff::DEFAULT
            }
        );
    }
}

fn emitter_kinds(app: &App) -> Vec<(AmbientSourceKind, Handle<AudioSource>, bool)> {
    let emitters = app.world().resource::<AmbientEmitters>();
    emitters
        .0
        .iter()
        .map(|emitter| {
            let kind = app
                .world()
                .get::<AmbientSource>(emitter.source)
                .expect("emitter points at a source")
                .kind;
            (kind, emitter.handle.clone(), emitter.spatial)
        })
        .collect()
}

#[test]
fn ambient_emitters_follow_the_player_room_and_the_facility_power() {
    let mut app = app();
    app.add_plugins(AmbienceViewPlugin)
        .insert_resource(ambience_sounds());
    app.finish();
    app.cleanup();
    for kind in [
        AmbientSourceKind::Roomtone,
        AmbientSourceKind::Conduit,
        AmbientSourceKind::Boiler,
    ] {
        let source = app
            .world_mut()
            .spawn((AmbientSource { kind, volume: 0.5 }, Transform::IDENTITY))
            .id();
        if kind == AmbientSourceKind::Boiler {
            app.world_mut().entity_mut(source).insert(MainsPowered);
        }
    }
    app.world_mut().spawn((
        Room(Rect::new(10.0, 10.0, 20.0, 20.0)),
        Walls(Module::WallConduit),
    ));
    app.update();
    assert!(!app.world().resource::<AmbienceActive>().0);

    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    assert!(app.world().resource::<AmbienceActive>().0);
    let mut kinds = emitter_kinds(&app);
    kinds.sort_by_key(|(kind, _, _)| *kind as u8);
    assert_eq!(
        kinds,
        vec![
            (AmbientSourceKind::Roomtone, clip(70), false),
            (AmbientSourceKind::Boiler, clip(73), true),
        ]
    );

    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(15.0, 1.6, 15.0));
    app.update();
    let kinds: Vec<_> = emitter_kinds(&app)
        .into_iter()
        .map(|(kind, _, _)| kind)
        .collect();
    assert!(kinds.contains(&AmbientSourceKind::Conduit));
    assert_eq!(kinds.len(), 3);

    *app.world_mut().resource_mut::<FacilityPower>() = FacilityPower::blackout(common::SEED);
    app.update();
    let kinds: Vec<_> = emitter_kinds(&app)
        .into_iter()
        .map(|(kind, _, _)| kind)
        .collect();
    assert!(!kinds.contains(&AmbientSourceKind::Boiler));
    assert_eq!(kinds.len(), 2);
}
