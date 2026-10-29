use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, render::render_resource::Face, world_serialization::WorldAssetRoot};

use super::{
    hud::{fuse_slots, FuseHudPlugin, FuseSlots, FUSE_SLOT_COUNT},
    logic::{
        FuseInstallation, FuseInventory, FusePanel, FUSE_COUNT, FUSE_LENGTH, FUSE_RADIUS,
        INSERT_SECS_PER_FUSE,
    },
    sounds::FuseSoundsPlugin,
};
use crate::{
    facility::{FacilityAssets, Module},
    interaction::PickupKind,
    player::PlayerController,
};

const _: () = assert!(FUSE_SLOT_COUNT == FUSE_COUNT);

const FUSE_OUTLINE_WIDTH: f32 = 0.008;
const FUSE_OUTLINE: Color = Color::srgb(1.0, 0.72, 0.22);
const PANEL_FUSE_SPACING: f32 = 0.16;
const PANEL_FUSE_DEPTH: f32 = -0.13;
const PANEL_FUSE_START_HEIGHT: f32 = -0.15;

#[derive(Component)]
struct PendingFuseRender;

#[derive(Component)]
struct PanelFuseVisual {
    slot: usize,
    from: Vec3,
}

// NOTE: app glue fills FacilityAssets once the loader is ready; this plugin never inits it.
pub struct FuseViewPlugin;

impl Plugin for FuseViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FuseSoundsPlugin, FuseHudPlugin))
            .add_observer(mark_fuse_for_render)
            .add_observer(attach_panel_fuses)
            .add_systems(Startup, spawn_hud)
            .add_systems(
                PostUpdate,
                render_fuses.run_if(resource_exists::<FacilityAssets>),
            )
            .add_systems(Update, (animate_panel_fuses, sync_hud));
    }
}

fn mark_fuse_for_render(
    added: On<Add<PickupKind>>,
    pickups: Query<&PickupKind>,
    mut commands: Commands,
) {
    if let Ok(PickupKind::Fuse { .. }) = pickups.get(added.entity) {
        commands.entity(added.entity).insert(PendingFuseRender);
    }
}

fn panel_fuse_destination(slot: usize) -> Vec3 {
    Vec3::new(
        (slot as f32 - 1.0) * PANEL_FUSE_SPACING,
        0.0,
        PANEL_FUSE_DEPTH,
    )
}

fn panel_fuse_position(from: Vec3, slot: usize, elapsed: f32) -> Vec3 {
    let progress =
        ((elapsed - slot as f32 * INSERT_SECS_PER_FUSE) / INSERT_SECS_PER_FUSE).clamp(0.0, 1.0);
    from.lerp(
        panel_fuse_destination(slot),
        progress * progress * (3.0 - 2.0 * progress),
    )
}

fn attach_panel_fuses(
    added: On<Add<FuseInstallation>>,
    panels: Query<(&FusePanel, &FuseInstallation, &Transform)>,
    players: Query<&Transform, With<PlayerController>>,
    assets: Res<FacilityAssets>,
    mut commands: Commands,
) {
    let Ok((panel, installation, transform)) = panels.get(added.entity) else {
        return;
    };
    if panel.installed != 0 {
        return;
    }
    let Ok(player) = players.get(installation.player) else {
        return;
    };
    let hand = player.translation + player.forward() * 0.3 - player.up() * 0.2;
    let from = transform.rotation.inverse() * (hand - transform.translation)
        + Vec3::Y * PANEL_FUSE_START_HEIGHT;
    let module = fuse_module(&assets);
    commands.entity(added.entity).with_children(|children| {
        for slot in 0..FUSE_COUNT {
            children
                .spawn((
                    Name::new(format!("panel fuse {}", slot + 1)),
                    PanelFuseVisual { slot, from },
                    Transform::from_translation(from),
                    Visibility::Hidden,
                ))
                .with_children(|children| {
                    children.spawn((WorldAssetRoot(module.clone()), Transform::IDENTITY));
                });
        }
    });
}

fn animate_panel_fuses(
    panels: Query<(Option<&FuseInstallation>, &FusePanel)>,
    mut visuals: Query<(&PanelFuseVisual, &ChildOf, &mut Transform, &mut Visibility)>,
) {
    for (visual, parent, mut transform, mut visibility) in &mut visuals {
        let Ok((installation, panel)) = panels.get(parent.parent()) else {
            continue;
        };
        let elapsed = installation.map_or_else(
            || {
                if panel.installed == FUSE_COUNT {
                    INSERT_SECS_PER_FUSE * FUSE_COUNT as f32
                } else {
                    0.0
                }
            },
            |installation| installation.elapsed,
        );
        if elapsed < visual.slot as f32 * INSERT_SECS_PER_FUSE {
            continue;
        }
        *visibility = Visibility::Visible;
        transform.translation = panel_fuse_position(visual.from, visual.slot, elapsed);
    }
}

fn fuse_module(assets: &FacilityAssets) -> &Handle<bevy::world_serialization::WorldAsset> {
    assets
        .module(Module::FusePickup)
        .unwrap_or_else(|| panic!("missing fuse module: {:?}", Module::FusePickup))
}

fn render_fuses(
    pickups: Query<Entity, With<PendingFuseRender>>,
    assets: Res<FacilityAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if pickups.is_empty() {
        return;
    }
    let module = fuse_module(&assets);
    let shape =
        Transform::from_xyz(0.0, FUSE_RADIUS, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2));
    let outline_mesh = meshes.add(Cylinder::new(
        FUSE_RADIUS + FUSE_OUTLINE_WIDTH,
        FUSE_LENGTH + 2.0 * FUSE_OUTLINE_WIDTH,
    ));
    let outline_material = materials.add(StandardMaterial {
        base_color: FUSE_OUTLINE,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    for entity in &pickups {
        commands.entity(entity).with_children(|children| {
            children.spawn((WorldAssetRoot(module.clone()), Transform::IDENTITY));
            children.spawn((
                Name::new("fuse outline"),
                Mesh3d(outline_mesh.clone()),
                MeshMaterial3d(outline_material.clone()),
                shape,
            ));
        });
        commands.entity(entity).remove::<PendingFuseRender>();
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((fuse_slots(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_hud(
    player: Option<Single<&FuseInventory, With<PlayerController>>>,
    hud: Single<(&mut FuseSlots, &mut Visibility)>,
) {
    let inventory = player.map(|player| *player.into_inner());
    let (mut slots, mut visibility) = hud.into_inner();
    visibility.set_if_neq(if inventory.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    slots.set_if_neq(FuseSlots {
        filled: inventory.map_or(0, |inventory| inventory.0),
    });
}
