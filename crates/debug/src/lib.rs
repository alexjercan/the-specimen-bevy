use bevy::camera::RenderTarget;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::gizmos::{prelude::GizmoConfig, AppGizmoBuilder};
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_egui::{
    egui, EguiContext, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, EguiSchedule,
    PrimaryEguiContext,
};
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};
mod monster_tracking;

use gameplay::{
    devices::{Detector, Flashbangs},
    facility::{Door, FacilityPower, Prop, Room},
    fuses::{FuseInventory, FusePanel},
    interaction::PickupKind,
    monster::Monster,
    player::{Flashlight, PlayerController, Stamina, StaminaState},
};

pub const INSPECTOR_TOGGLE_KEY: KeyCode = KeyCode::F12;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSettings {
    pub inspector: bool,
    pub wireframe: bool,
    pub monster_tracking: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WireframePlugin::default(),
            EguiPlugin::default(),
            DefaultInspectorConfigPlugin,
        ))
        .add_systems(Update, log_frame_time)
        .insert_gizmo_config(
            monster_tracking::MonsterRouteGizmos,
            GizmoConfig {
                depth_bias: -1.0,
                ..default()
            },
        )
        .init_resource::<DebugSettings>();
        app.world_mut()
            .resource_mut::<EguiGlobalSettings>()
            .auto_create_primary_context = false;
        app.add_systems(
            Update,
            (
                monster_tracking::draw_monster_routes,
                keep_inspector_on_window_camera,
                toggle_inspector.run_if(input_just_pressed(INSPECTOR_TOGGLE_KEY)),
                sync_wireframe.run_if(resource_changed::<DebugSettings>),
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                inspector_ui.run_if(|settings: Res<DebugSettings>| settings.inspector),
                monster_tracking::monster_tracking_ui,
            ),
        );
    }
}

fn toggle_inspector(mut settings: ResMut<DebugSettings>) {
    settings.inspector = !settings.inspector;
}

fn sync_wireframe(settings: Res<DebugSettings>, mut config: ResMut<WireframeConfig>) {
    if config.global != settings.wireframe {
        config.global = settings.wireframe;
    }
}

fn keep_inspector_on_window_camera(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera, &RenderTarget, Has<PrimaryEguiContext>)>,
) {
    let primary = cameras
        .iter()
        .filter(|(_, camera, target, _)| {
            camera.is_active && matches!(target, RenderTarget::Window(_))
        })
        .max_by_key(|(_, camera, _, _)| camera.order)
        .map(|(entity, _, _, _)| entity);
    for (entity, _, _, has_context) in &cameras {
        if has_context && Some(entity) != primary {
            commands
                .entity(entity)
                .remove::<(PrimaryEguiContext, EguiContext, EguiSchedule)>();
        }
    }
    if let Some(primary) = primary {
        if cameras
            .get(primary)
            .is_ok_and(|(_, _, _, has_context)| !has_context)
        {
            commands.entity(primary).insert(PrimaryEguiContext);
        }
    }
}

fn inspector_ui(world: &mut World) {
    let Ok(context) = world
        .query_filtered::<&EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut context = context.clone();
    let mut wireframe = world.resource::<DebugSettings>().wireframe;
    let mut monster_tracking = world.resource::<DebugSettings>().monster_tracking;
    egui::Window::new("Inspector").show(context.get_mut(), |ui| {
        ui.checkbox(&mut wireframe, "Wireframe");
        ui.checkbox(&mut monster_tracking, "Monster tracking (X-ray)")
            .on_hover_text("Show the monster through walls and draw its route, destination, and last sensed position on the floor. Remains visible when F12 closes.");
        if monster_tracking {
            ui.label("Gold: walk | Orange: run | Violet: long target | Blue: last sensed");
        }
        facility_power_ui(world, ui);
        ui.collapsing("Player cheats", |ui| {
            let mut players = world.query_filtered::<Entity, With<PlayerController>>();
            let entities: Vec<_> = players.iter(world).collect();
            if entities.is_empty() {
                ui.label("Start a run to edit player tools.");
            }
            for player in entities {
                player_tools_ui(world, player, ui);
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            grouped_entities_ui(world, ui);
            ui.collapsing("All entities (search)", |ui| {
                bevy_inspector::ui_for_entities(world, ui);
            });
            ui.collapsing("Resources", |ui| {
                bevy_inspector::ui_for_resources(world, ui);
            });
            ui.collapsing("Assets", |ui| {
                bevy_inspector::ui_for_all_assets(world, ui);
            });
        });
    });
    let mut settings = world.resource_mut::<DebugSettings>();
    let updated = DebugSettings {
        wireframe,
        monster_tracking,
        ..*settings
    };
    settings.set_if_neq(updated);
}

fn facility_power_ui(world: &mut World, ui: &mut egui::Ui) {
    let mut on = world.resource::<FacilityPower>().is_on();
    if ui.checkbox(&mut on, "Facility power").changed() {
        let mut power = world.resource_mut::<FacilityPower>();
        if on {
            power.restore();
        } else {
            power.outage();
        }
    }
}

const ENTITY_GROUPS: [&str; 8] = [
    "Players",
    "Monsters",
    "Fuse panels",
    "Pickups and devices",
    "Rooms",
    "Doors",
    "Props",
    "Other",
];

fn entity_group(world: &World, entity: Entity) -> usize {
    if world.get::<PlayerController>(entity).is_some() {
        0
    } else if world.get::<Monster>(entity).is_some() {
        1
    } else if world.get::<FusePanel>(entity).is_some() {
        2
    } else if world.get::<PickupKind>(entity).is_some()
        || world.get::<Detector>(entity).is_some()
        || world.get::<Flashbangs>(entity).is_some()
    {
        3
    } else if world.get::<Room>(entity).is_some() {
        4
    } else if world.get::<Door>(entity).is_some() {
        5
    } else if world.get::<Prop>(entity).is_some() {
        6
    } else {
        7
    }
}

fn player_tools_ui(world: &mut World, player: Entity, ui: &mut egui::Ui) {
    ui.label(format!("Player {player}"));
    if let Some(mut count) = world.get::<FuseInventory>(player).map(|items| items.0) {
        if ui
            .add(
                egui::DragValue::new(&mut count)
                    .range(0..=99)
                    .prefix("Fuses: "),
            )
            .changed()
        {
            world.entity_mut(player).insert(FuseInventory(count));
        }
    }
    let mut count = world.get::<Flashbangs>(player).map_or(0, |items| items.0);
    if ui
        .add(
            egui::DragValue::new(&mut count)
                .range(0..=99)
                .prefix("Flashbangs: "),
        )
        .changed()
    {
        world.entity_mut(player).insert(Flashbangs(count));
    }
    let mut has_detector = world.get::<Detector>(player).is_some();
    if ui.checkbox(&mut has_detector, "Detector").changed() {
        if has_detector {
            world.entity_mut(player).insert(Detector::default());
        } else {
            world.entity_mut(player).remove::<Detector>();
        }
    }
    if let Some(mut charge) = world.get::<Flashlight>(player).map(|light| light.charge) {
        if ui
            .add(egui::Slider::new(&mut charge, 0.0..=1.0).text("Flashlight charge"))
            .changed()
        {
            world
                .entity_mut(player)
                .get_mut::<Flashlight>()
                .unwrap()
                .charge = charge;
        }
    }
    if let Some(mut charge) = world.get::<Stamina>(player).map(|stamina| stamina.charge) {
        if ui
            .add(egui::Slider::new(&mut charge, 0.0..=1.0).text("Sprint stamina"))
            .changed()
        {
            let mut entity = world.entity_mut(player);
            let mut stamina = entity.get_mut::<Stamina>().unwrap();
            stamina.charge = charge;
            let low = charge < gameplay::player::SPRINT_RESTART_CHARGE;
            stamina.state = match stamina.state {
                StaminaState::Locked => StaminaState::Locked,
                _ if low => StaminaState::Exhausted,
                StaminaState::Exhausted => StaminaState::Rested,
                state => state,
            };
        }
    }
}

fn grouped_entities_ui(world: &mut World, ui: &mut egui::Ui) {
    let mut groups: [Vec<Entity>; ENTITY_GROUPS.len()] = std::array::from_fn(|_| Vec::new());
    let mut roots = world.query_filtered::<Entity, Without<ChildOf>>();
    let entities: Vec<_> = roots.iter(world).collect();
    for entity in entities {
        groups[entity_group(world, entity)].push(entity);
    }
    for (label, entities) in ENTITY_GROUPS.iter().zip(groups) {
        ui.collapsing(format!("{label} ({})", entities.len()), |ui| {
            for entity in entities {
                let name = world
                    .get::<Name>(entity)
                    .map(|name| name.as_str().to_owned())
                    .or_else(|| {
                        world
                            .get::<Prop>(entity)
                            .map(|prop| format!("{:?}", prop.0))
                    })
                    .unwrap_or_else(|| label.to_string());
                egui::CollapsingHeader::new(format!("{name} ({entity})"))
                    .id_salt(("grouped_entity", entity))
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_entity_with_children(world, entity, ui);
                    });
            }
        });
    }
}

// NOTE: debug level, so it shows only with RUST_LOG=debug; FrameTimeDiagnosticsPlugin comes from game_ui::FpsOverlayPlugin.
fn log_frame_time(
    time: Res<Time<Real>>,
    diagnostics: Res<DiagnosticsStore>,
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    if *since < 1.0 {
        return;
    }
    *since = 0.0;
    let smoothed = |path| {
        diagnostics
            .get(path)
            .and_then(|diagnostic| diagnostic.smoothed())
    };
    if let (Some(fps), Some(frame_time)) = (
        smoothed(&FrameTimeDiagnosticsPlugin::FPS),
        smoothed(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
    ) {
        debug!("fps {fps:.0}, frame time {frame_time:.2} ms");
    }
}
