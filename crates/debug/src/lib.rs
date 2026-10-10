use bevy::camera::RenderTarget;
use bevy::input::common_conditions::input_just_pressed;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::PbrPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_egui::{
    egui, EguiContext, EguiGlobalSettings, EguiMultipassSchedule, EguiPlugin,
    EguiPrimaryContextPass, PrimaryEguiContext,
};
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};
pub use game_ui::{fps_label, FpsText};
use gameplay::{
    controller::PlayerController,
    levels::{Detector, Door, Flashbangs, FusePanel, Monster, PickupKind, Prop, Room},
};

#[cfg(test)]
#[path = "../tests/unit/context.rs"]
mod context_tests;
#[cfg(test)]
#[path = "../tests/unit/entity_groups.rs"]
mod entity_group_tests;

pub const INSPECTOR_TOGGLE_KEY: KeyCode = KeyCode::F12;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSettings {
    pub inspector: bool,
    pub wireframe: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<game_ui::GameUiPlugin>() {
            app.add_plugins(game_ui::GameUiPlugin);
        }
        if app.is_plugin_added::<PbrPlugin>() && !app.is_plugin_added::<WireframePlugin>() {
            app.add_plugins(WireframePlugin::default());
        }
        if app.is_plugin_added::<WinitPlugin>() {
            if !app.is_plugin_added::<EguiPlugin>() {
                app.add_plugins(EguiPlugin::default());
            }
            app.world_mut()
                .resource_mut::<EguiGlobalSettings>()
                .auto_create_primary_context = false;
            app.add_plugins(DefaultInspectorConfigPlugin)
                .add_systems(Update, keep_inspector_on_window_camera)
                .add_systems(
                    EguiPrimaryContextPass,
                    inspector_ui.run_if(|settings: Res<DebugSettings>| settings.inspector),
                );
        }
        app.init_resource::<DebugSettings>().add_systems(
            Update,
            (
                toggle_inspector.run_if(input_just_pressed(INSPECTOR_TOGGLE_KEY)),
                sync_wireframe.run_if(
                    resource_exists::<WireframeConfig>.and_then(resource_changed::<DebugSettings>),
                ),
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
                .remove::<(PrimaryEguiContext, EguiContext, EguiMultipassSchedule)>();
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
    egui::Window::new("Inspector").show(context.get_mut(), |ui| {
        ui.checkbox(&mut wireframe, "Wireframe");
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
    if settings.wireframe != wireframe {
        settings.wireframe = wireframe;
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
                    .or_else(|| world.get::<Prop>(entity).map(|prop| prop.0.clone()))
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
