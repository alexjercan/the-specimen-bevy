use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::levels::{
    builder::{door, passage, room, DoorOf, DoorRef},
    doors::{DoorLock, ExitDoor},
};

pub(super) fn spawn(commands: &mut Commands) {
    const FRAME: &str = "wall_doorway";
    const PANEL: &str = "door_panel";
    const FLOOR_TILE: &str = "floor_tile";
    const FLOOR_TILE_MARKED: &str = "floor_tile_marked";
    const WALL: &str = "wall";
    const WALL_CONDUIT: &str = "wall_conduit";
    const CEILING_TILE: &str = "ceiling_tile";

    let outside_exit = commands
        .spawn((
            door("exit / outside", Vec2::new(0.0, -31.25), PI, FRAME, PANEL),
            ExitDoor,
            DoorLock,
        ))
        .id();
    let exit_service = commands
        .spawn(door(
            "exit / service",
            Vec2::new(0.0, -21.25),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let exit_office = commands
        .spawn(door(
            "exit / office",
            Vec2::new(5.0, -21.25),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let exit_security = commands
        .spawn(door(
            "exit / security",
            Vec2::new(8.75, -27.5),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let service_maintenance = commands
        .spawn(door(
            "service / maintenance",
            Vec2::new(-1.25, -20.0),
            -FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let service_office = commands
        .spawn(door(
            "service / office",
            Vec2::new(1.25, -17.5),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let office_security_link = commands
        .spawn(door(
            "office / security link",
            Vec2::new(8.75, -20.0),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let security_link_security = commands
        .spawn(door(
            "security link / security",
            Vec2::new(12.5, -23.75),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let maintenance_utility = commands
        .spawn(passage("maintenance / utility", Vec2::new(-12.5, -16.25)))
        .id();
    let maintenance_west = commands
        .spawn(door(
            "maintenance / west hall",
            Vec2::new(-5.0, -16.25),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let service_reception = commands
        .spawn(passage("service / reception", Vec2::new(0.0, -16.25)))
        .id();
    let office_east = commands
        .spawn(door(
            "office / east hall",
            Vec2::new(5.0, -16.25),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let west_reception = commands
        .spawn(passage("west hall / reception", Vec2::new(-3.75, -12.5)))
        .id();
    let reception_east = commands
        .spawn(passage("reception / east hall", Vec2::new(3.75, -12.5)))
        .id();
    let east_storage = commands
        .spawn(door(
            "east hall / storage",
            Vec2::new(6.25, -12.5),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let reception_intake = commands
        .spawn(passage("reception / intake", Vec2::new(0.0, -8.75)))
        .id();
    let west_utility = commands
        .spawn(door(
            "west hall / utility",
            Vec2::new(-6.25, -10.0),
            -FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let west_west_link = commands
        .spawn(passage("west hall / west link", Vec2::new(-3.75, -5.0)))
        .id();
    let west_link_intake = commands
        .spawn(passage("west link / intake", Vec2::new(-1.25, -5.0)))
        .id();
    let intake_east_link = commands
        .spawn(passage("intake / east link", Vec2::new(1.25, -5.0)))
        .id();
    let east_link_east = commands
        .spawn(passage("east link / east hall", Vec2::new(3.75, -5.0)))
        .id();
    let intake_lab = commands
        .spawn(door(
            "intake / lab",
            Vec2::new(0.0, -3.75),
            0.0,
            FRAME,
            PANEL,
        ))
        .id();
    let west_boiler = commands
        .spawn(door(
            "west hall / boiler",
            Vec2::new(-6.25, 0.0),
            -FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let west_lab = commands
        .spawn(door(
            "west hall / lab",
            Vec2::new(-3.75, 0.0),
            -FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let lab_east = commands
        .spawn(door(
            "lab / east hall",
            Vec2::new(3.75, 0.0),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let east_prep = commands
        .spawn(door(
            "east hall / prep",
            Vec2::new(6.25, -5.0),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();
    let prep_hiding = commands
        .spawn(passage("prep / hiding", Vec2::new(10.0, -3.75)))
        .id();
    let east_hiding = commands
        .spawn(door(
            "east hall / hiding",
            Vec2::new(6.25, 0.0),
            FRAC_PI_2,
            FRAME,
            PANEL,
        ))
        .id();

    commands
        .spawn(room(
            "maintenance",
            Rect::new(-13.75, -23.75, -1.25, -16.25),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(service_maintenance))
        .with_related::<DoorOf>(DoorRef(maintenance_west))
        .with_related::<DoorOf>(DoorRef(maintenance_utility));
    commands
        .spawn(room(
            "exit",
            Rect::new(-1.25, -31.25, 8.75, -21.25),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(outside_exit))
        .with_related::<DoorOf>(DoorRef(exit_service))
        .with_related::<DoorOf>(DoorRef(exit_office))
        .with_related::<DoorOf>(DoorRef(exit_security));
    commands
        .spawn(room(
            "security",
            Rect::new(8.75, -31.25, 13.75, -23.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(exit_security))
        .with_related::<DoorOf>(DoorRef(security_link_security));
    commands
        .spawn(room(
            "service",
            Rect::new(-1.25, -21.25, 1.25, -16.25),
            FLOOR_TILE_MARKED,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(exit_service))
        .with_related::<DoorOf>(DoorRef(service_maintenance))
        .with_related::<DoorOf>(DoorRef(service_office))
        .with_related::<DoorOf>(DoorRef(service_reception));
    commands
        .spawn(room(
            "office",
            Rect::new(1.25, -21.25, 8.75, -16.25),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(exit_office))
        .with_related::<DoorOf>(DoorRef(service_office))
        .with_related::<DoorOf>(DoorRef(office_east))
        .with_related::<DoorOf>(DoorRef(office_security_link));
    commands
        .spawn(room(
            "security_link",
            Rect::new(8.75, -23.75, 13.75, -16.25),
            FLOOR_TILE_MARKED,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(office_security_link))
        .with_related::<DoorOf>(DoorRef(security_link_security));
    commands
        .spawn(room(
            "utility",
            Rect::new(-13.75, -16.25, -6.25, -6.25),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(west_utility))
        .with_related::<DoorOf>(DoorRef(maintenance_utility));
    commands
        .spawn(room(
            "prep",
            Rect::new(6.25, -8.75, 13.75, -3.75),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(east_prep))
        .with_related::<DoorOf>(DoorRef(prep_hiding));
    commands
        .spawn(room(
            "hiding",
            Rect::new(6.25, -3.75, 13.75, 3.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(east_hiding))
        .with_related::<DoorOf>(DoorRef(prep_hiding));
    commands
        .spawn(room(
            "west_hall",
            Rect::new(-6.25, -16.25, -3.75, 3.75),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(maintenance_west))
        .with_related::<DoorOf>(DoorRef(west_reception))
        .with_related::<DoorOf>(DoorRef(west_utility))
        .with_related::<DoorOf>(DoorRef(west_west_link))
        .with_related::<DoorOf>(DoorRef(west_boiler))
        .with_related::<DoorOf>(DoorRef(west_lab));
    commands
        .spawn(room(
            "reception",
            Rect::new(-3.75, -16.25, 3.75, -8.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(service_reception))
        .with_related::<DoorOf>(DoorRef(west_reception))
        .with_related::<DoorOf>(DoorRef(reception_east))
        .with_related::<DoorOf>(DoorRef(reception_intake));
    commands
        .spawn(room(
            "east_hall",
            Rect::new(3.75, -16.25, 6.25, 3.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(office_east))
        .with_related::<DoorOf>(DoorRef(reception_east))
        .with_related::<DoorOf>(DoorRef(east_storage))
        .with_related::<DoorOf>(DoorRef(east_link_east))
        .with_related::<DoorOf>(DoorRef(lab_east))
        .with_related::<DoorOf>(DoorRef(east_hiding))
        .with_related::<DoorOf>(DoorRef(east_prep));
    commands
        .spawn(room(
            "storage",
            Rect::new(6.25, -16.25, 13.75, -8.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(east_storage));
    commands
        .spawn(room(
            "west_link",
            Rect::new(-3.75, -8.75, -1.25, -3.75),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(west_west_link))
        .with_related::<DoorOf>(DoorRef(west_link_intake));
    commands
        .spawn(room(
            "intake",
            Rect::new(-1.25, -8.75, 1.25, -3.75),
            FLOOR_TILE_MARKED,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(reception_intake))
        .with_related::<DoorOf>(DoorRef(west_link_intake))
        .with_related::<DoorOf>(DoorRef(intake_east_link))
        .with_related::<DoorOf>(DoorRef(intake_lab));
    commands
        .spawn(room(
            "east_link",
            Rect::new(1.25, -8.75, 3.75, -3.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(intake_east_link))
        .with_related::<DoorOf>(DoorRef(east_link_east));
    commands
        .spawn(room(
            "lab",
            Rect::new(-3.75, -3.75, 3.75, 3.75),
            FLOOR_TILE,
            WALL,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(intake_lab))
        .with_related::<DoorOf>(DoorRef(west_lab))
        .with_related::<DoorOf>(DoorRef(lab_east));
    commands
        .spawn(room(
            "boiler",
            Rect::new(-13.75, -3.75, -6.25, 3.75),
            FLOOR_TILE,
            WALL_CONDUIT,
            CEILING_TILE,
        ))
        .with_related::<DoorOf>(DoorRef(west_boiler));
}
