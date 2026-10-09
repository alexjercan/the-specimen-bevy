use std::{collections::HashSet, time::Duration};

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::EnhancedInputPlugin;
use game_audio::{PlaySound, Sound};
use gameplay::{
    controller::{PlayerController, PlayerControllerPlugin, PlayerControlsEnabled},
    levels::{
        build_first_floor, select_fuse_slots, Door, DoorPlugin, DoorState, FuseInventory,
        FusePickup, FusePlugin, FuseSeed, Prop, Room, FUSE_COUNT, FUSE_TABLES,
    },
};

fn first_floor(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FuseSeed(seed))
        .add_systems(Startup, build_first_floor);
    app.update();
    app
}

fn fuse_placements(app: &mut App) -> Vec<(usize, Vec3)> {
    let mut placements: Vec<_> = app
        .world_mut()
        .query::<(&FusePickup, &Transform)>()
        .iter(app.world())
        .map(|(fuse, transform)| (fuse.slot, transform.translation))
        .collect();
    placements.sort_by_key(|(slot, _)| *slot);
    placements
}

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(PlayerControllerPlugin::default().without_camera())
        .add_plugins((DoorPlugin, FusePlugin));
    app.finish();
    app.cleanup();
    app.update();
    let player = app
        .world_mut()
        .spawn((PlayerController, Transform::from_xyz(0.0, 1.6, 0.0)))
        .id();
    app.update();
    (app, player)
}

fn fuse(app: &mut App, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            FusePickup { slot: 0 },
            Transform::from_translation(position),
        ))
        .id()
}

fn aim(app: &mut App, player: Entity, target: Vec3) {
    let eye = app.world().get::<Transform>(player).unwrap().translation;
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_translation(eye).looking_at(target, Vec3::Y));
}

fn press_f(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
}

fn inventory(app: &App, player: Entity) -> usize {
    app.world().get::<FuseInventory>(player).unwrap().0
}

fn exists(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

#[test]
fn eligible_tables_are_five_or_six_distinct_rooms_on_table_tops() {
    assert!((5..=6).contains(&FUSE_TABLES.len()));
    let rooms: HashSet<_> = FUSE_TABLES.iter().map(|table| table.room).collect();
    assert_eq!(rooms.len(), FUSE_TABLES.len());

    let mut app = first_floor(0);
    let world = app.world_mut();
    let bounds: Vec<(String, Rect)> = world
        .query::<(&Name, &Room)>()
        .iter(world)
        .map(|(name, room)| (name.as_str().to_owned(), room.0))
        .collect();
    let tables: Vec<(String, Transform)> = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .filter(|(prop, _)| prop.0 == "workbench")
        .map(|(prop, transform)| (prop.0.clone(), *transform))
        .collect();
    let hiding_props: Vec<Vec2> = world
        .query::<(&Prop, &Transform)>()
        .iter(world)
        .filter(|(prop, _)| prop.0 == "concept_table" || prop.0 == "concept_locker")
        .map(|(_, transform)| transform.translation.xz())
        .collect();
    let doors: Vec<Vec2> = world
        .query::<&Door>()
        .iter(world)
        .map(|door| door.position)
        .collect();
    for table in &FUSE_TABLES {
        assert!(
            hiding_props
                .iter()
                .all(|&position| table.position.xz().distance(position) > 1.8),
            "{} fuse is too close to a hiding spot",
            table.room
        );
        assert!(
            doors
                .iter()
                .all(|&position| table.position.xz().distance(position) > 1.5),
            "{} fuse blocks a doorway",
            table.room
        );
        let (_, rect) = bounds
            .iter()
            .find(|(name, _)| name == table.room)
            .unwrap_or_else(|| panic!("missing room {}", table.room));
        assert!(rect.contains(table.position.xz()), "{}", table.room);
        let support = tables.iter().find(|(module, transform)| {
            if module != "workbench" {
                return false;
            }
            let local = transform.rotation.inverse() * (table.position - transform.translation);
            local.x.abs() + 0.1 <= 0.8
                && local.z.abs() + 0.03 <= 0.35
                && (table.position.y - 0.9).abs() < 1e-4
        });
        assert!(
            support.is_some(),
            "{} fuse is not on a table top",
            table.room
        );
    }
}

#[test]
fn seed_selects_three_distinct_reproducible_slots_covering_every_combination() {
    let mut combinations = HashSet::new();
    for seed in 0..500 {
        let slots = select_fuse_slots(seed, FUSE_TABLES.len());
        assert_eq!(slots, select_fuse_slots(seed, FUSE_TABLES.len()));
        assert!(slots.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(slots.iter().all(|&slot| slot < FUSE_TABLES.len()));
        combinations.insert(slots);
    }
    assert_eq!(combinations.len(), 10);
}

#[test]
fn first_floor_spawns_three_logical_fuses_at_the_seeded_tables() {
    let mut app = first_floor(42);
    let placements = fuse_placements(&mut app);
    assert_eq!(placements.len(), FUSE_COUNT);
    let expected = select_fuse_slots(42, FUSE_TABLES.len());
    for ((slot, position), expected) in placements.iter().zip(expected) {
        assert_eq!(*slot, expected);
        assert_eq!(*position, FUSE_TABLES[expected].position);
    }
    assert_eq!(placements, fuse_placements(&mut first_floor(42)));
    assert!(app
        .world_mut()
        .query_filtered::<(), (With<FusePickup>, With<Children>)>()
        .iter(app.world())
        .next()
        .is_none());
}

#[test]
fn spawned_player_starts_with_an_empty_inventory() {
    let (app, player) = app();
    assert_eq!(inventory(&app, player), 0);
}

#[test]
fn f_picks_up_an_aimed_fuse_once() {
    let (mut app, player) = app();
    let target = fuse(&mut app, Vec3::new(0.0, 0.8, -1.0));
    let other = fuse(&mut app, Vec3::new(3.0, 0.8, -1.0));
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.0));
    press_f(&mut app);
    assert!(!exists(&app, target));
    assert!(exists(&app, other));
    assert_eq!(inventory(&app, player), 1);
    press_f(&mut app);
    assert!(exists(&app, other));
    assert_eq!(inventory(&app, player), 1);
}

#[test]
fn successive_pickups_emit_distinct_slot_cues() {
    let (mut app, player) = app();
    let position = Vec3::new(0.0, 0.8, -1.0);
    aim(&mut app, player, position + Vec3::Y * 0.03);
    for slot in 1..=FUSE_COUNT {
        fuse(&mut app, position);
        press_f(&mut app);
        let cues: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<PlaySound>>()
            .drain()
            .collect();
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].sound, Sound::FuseSlot(slot));
        assert_eq!(cues[0].position, None);
        assert_eq!(inventory(&app, player), slot);
    }
}

#[test]
fn disabled_controls_do_not_pick_up_fuses() {
    let (mut app, player) = app();
    let target = fuse(&mut app, Vec3::new(0.0, 0.8, -1.0));
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.0));
    app.world_mut().resource_mut::<PlayerControlsEnabled>().0 = false;
    press_f(&mut app);
    assert!(exists(&app, target));
    assert_eq!(inventory(&app, player), 0);
}

#[test]
fn full_inventory_leaves_the_fuse_in_place() {
    let (mut app, player) = app();
    app.world_mut()
        .entity_mut(player)
        .insert(FuseInventory(FUSE_COUNT));
    let target = fuse(&mut app, Vec3::new(0.0, 0.8, -1.0));
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.0));
    press_f(&mut app);
    assert!(exists(&app, target));
    assert_eq!(inventory(&app, player), FUSE_COUNT);
}

#[test]
fn nearer_door_wins_over_a_fuse_behind_it() {
    let (mut app, player) = app();
    let door = app
        .world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, -0.8),
            rotation: Quat::IDENTITY,
            frame: String::new(),
            panel: String::new(),
            state: DoorState::Closed,
        })
        .id();
    let target = fuse(&mut app, Vec3::new(0.0, 0.8, -1.5));
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.5));
    press_f(&mut app);
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Open
    );
    assert!(exists(&app, target));
    assert_eq!(inventory(&app, player), 0);
}

#[test]
fn nearer_fuse_wins_over_a_door_behind_it() {
    let (mut app, player) = app();
    let door = app
        .world_mut()
        .spawn(Door {
            position: Vec2::new(0.0, -1.6),
            rotation: Quat::IDENTITY,
            frame: String::new(),
            panel: String::new(),
            state: DoorState::Closed,
        })
        .id();
    let target = fuse(&mut app, Vec3::new(0.0, 1.2, -0.8));
    aim(&mut app, player, Vec3::new(0.0, 1.23, -0.8));
    press_f(&mut app);
    assert!(!exists(&app, target));
    assert_eq!(inventory(&app, player), 1);
    assert_eq!(
        app.world().get::<Door>(door).unwrap().state,
        DoorState::Closed
    );
}

#[test]
fn walls_block_fuse_pickup() {
    let (mut app, player) = app();
    app.world_mut()
        .spawn(Room(Rect::new(-1.25, -3.75, 1.25, -1.25)));
    app.world_mut()
        .entity_mut(player)
        .insert(Transform::from_xyz(0.0, 1.6, -0.8));
    let target = fuse(&mut app, Vec3::new(0.0, 0.8, -1.6));
    aim(&mut app, player, Vec3::new(0.0, 0.83, -1.6));
    press_f(&mut app);
    assert!(exists(&app, target));
    assert_eq!(inventory(&app, player), 0);
}
