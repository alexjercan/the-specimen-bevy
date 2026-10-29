use bevy::{prelude::*, render::render_resource::Face};
use game_settings::GameSettings;

use super::{
    hud::{
        detector_readout, flash_alpha, flash_overlay, flashbang_status, DetectorReadout,
        DetectorSignal, DeviceHudPlugin, FlashOverlay, FlashbangPrompt, FlashbangStatus,
    },
    logic::{
        tick_burst, Detector, FlashExposure, Flashbangs, ThrownFlashbang, DETECTOR_RANGE,
        FLASHBANG_BURST_DELAY,
    },
    sounds::DeviceSoundsPlugin,
};
use crate::{interaction::PickupKind, player::PlayerController};

const DEVICE_OUTLINE_WIDTH: f32 = 0.01;
const DEVICE_OUTLINE: Color = Color::srgb(0.55, 0.85, 1.0);
const FLASHBANG_RADIUS: f32 = 0.035;
const FLASHBANG_LENGTH: f32 = 0.12;
const FLASHBANG_BODY: Color = Color::srgb(0.16, 0.24, 0.14);
const DETECTOR_SIZE: Vec3 = Vec3::new(0.16, 0.07, 0.1);
const DETECTOR_BODY: Color = Color::srgb(0.32, 0.33, 0.34);
const DETECTOR_FACE: Color = Color::srgb(1.0, 0.62, 0.18);
const DETECTOR_FACE_SIZE: Vec3 = Vec3::new(0.1, 0.004, 0.06);
const BURST_INTENSITY: f32 = 9_000_000.0;
const BURST_DECAY: f32 = 8.0;

#[derive(Component)]
struct PendingDeviceRender;

#[derive(Component)]
struct FlashbangBurstLight;

pub struct DeviceViewPlugin;

impl Plugin for DeviceViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DeviceSoundsPlugin, DeviceHudPlugin))
            .add_observer(mark_device_for_render)
            .add_observer(attach_throw_visual)
            .add_systems(Startup, spawn_hud)
            .add_systems(PostUpdate, render_devices)
            .add_systems(
                Update,
                (
                    animate_throws.after(tick_burst),
                    sync_flashbang_status,
                    sync_flashbang_prompt,
                    sync_detector_readout,
                    sync_flash_overlay,
                ),
            );
    }
}

fn mark_device_for_render(
    added: On<Add<PickupKind>>,
    pickups: Query<&PickupKind>,
    mut commands: Commands,
) {
    if let Ok(PickupKind::Flashbang | PickupKind::Detector) = pickups.get(added.entity) {
        commands.entity(added.entity).insert(PendingDeviceRender);
    }
}

fn attach_throw_visual(
    added: On<Add<ThrownFlashbang>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mesh = meshes.add(Cylinder::new(FLASHBANG_RADIUS, FLASHBANG_LENGTH));
    let material = materials.add(StandardMaterial {
        base_color: FLASHBANG_BODY,
        perceptual_roughness: 0.6,
        ..default()
    });
    commands.entity(added.entity).with_children(|children| {
        children.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, FLASHBANG_LENGTH / 2.0, 0.0),
        ));
        children.spawn((
            FlashbangBurstLight,
            PointLight {
                intensity: 0.0,
                range: 8.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::IDENTITY,
        ));
    });
}

fn animate_throws(
    throws: Query<(&ThrownFlashbang, &Children)>,
    mut lights: Query<&mut PointLight, With<FlashbangBurstLight>>,
) {
    for (thrown, children) in &throws {
        for child in children.iter() {
            if let Ok(mut light) = lights.get_mut(child) {
                let since = thrown.elapsed - FLASHBANG_BURST_DELAY;
                light.intensity = if since >= 0.0 {
                    BURST_INTENSITY * (-BURST_DECAY * since).exp()
                } else {
                    0.0
                };
            }
        }
    }
}

fn render_devices(
    pickups: Query<(Entity, &PickupKind), With<PendingDeviceRender>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if pickups.is_empty() {
        return;
    }
    let outline = materials.add(StandardMaterial {
        base_color: DEVICE_OUTLINE,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    });
    for (entity, device) in &pickups {
        let parts = match device {
            PickupKind::Fuse { .. } => continue,
            PickupKind::Flashbang => {
                let shape = Transform::from_xyz(0.0, FLASHBANG_LENGTH / 2.0, 0.0);
                vec![
                    (
                        meshes.add(Cylinder::new(FLASHBANG_RADIUS, FLASHBANG_LENGTH)),
                        materials.add(StandardMaterial {
                            base_color: FLASHBANG_BODY,
                            perceptual_roughness: 0.6,
                            ..default()
                        }),
                        shape,
                    ),
                    (
                        meshes.add(Cylinder::new(
                            FLASHBANG_RADIUS + DEVICE_OUTLINE_WIDTH,
                            FLASHBANG_LENGTH + 2.0 * DEVICE_OUTLINE_WIDTH,
                        )),
                        outline.clone(),
                        shape,
                    ),
                ]
            }
            PickupKind::Detector => {
                let shape = Transform::from_xyz(0.0, DETECTOR_SIZE.y / 2.0, 0.0);
                vec![
                    (
                        meshes.add(Cuboid::from_size(DETECTOR_SIZE)),
                        materials.add(StandardMaterial {
                            base_color: DETECTOR_BODY,
                            perceptual_roughness: 0.5,
                            ..default()
                        }),
                        shape,
                    ),
                    (
                        meshes.add(Cuboid::from_size(DETECTOR_FACE_SIZE)),
                        materials.add(StandardMaterial {
                            base_color: DETECTOR_FACE,
                            emissive: LinearRgba::from(DETECTOR_FACE) * 2.0,
                            ..default()
                        }),
                        Transform::from_xyz(0.0, DETECTOR_SIZE.y, 0.0),
                    ),
                    (
                        meshes.add(Cuboid::from_size(
                            DETECTOR_SIZE + Vec3::splat(2.0 * DEVICE_OUTLINE_WIDTH),
                        )),
                        outline.clone(),
                        shape,
                    ),
                ]
            }
        };
        commands.entity(entity).with_children(|children| {
            for (mesh, material, transform) in parts {
                children.spawn((Mesh3d(mesh), MeshMaterial3d(material), transform));
            }
        });
        commands.entity(entity).remove::<PendingDeviceRender>();
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((flashbang_status(), Visibility::Hidden, GlobalZIndex(5)));
    commands.spawn((detector_readout(), Visibility::Hidden, GlobalZIndex(5)));
    commands.spawn((flash_overlay(), Visibility::Hidden, GlobalZIndex(5)));
}

fn sync_flashbang_status(
    player: Option<Single<Option<&Flashbangs>, With<PlayerController>>>,
    hud: Single<(&mut FlashbangStatus, &mut Visibility)>,
) {
    let flashbangs = player.and_then(|player| *player);
    let state = FlashbangStatus {
        count: flashbangs.map_or(0, |flashbangs| flashbangs.0),
    };
    let (mut status, mut visibility) = hud.into_inner();
    visibility.set_if_neq(if state.count > 0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    status.set_if_neq(state);
}

fn sync_flashbang_prompt(
    settings: Res<GameSettings>,
    mut prompt: Single<&mut Text, With<FlashbangPrompt>>,
) {
    let value = format!("{} THROW", settings.keys.flashbang.label());
    if prompt.0 != value {
        prompt.0 = value;
    }
}

fn sync_detector_readout(
    player: Option<Single<Option<&Detector>, With<PlayerController>>>,
    hud: Single<(&mut DetectorReadout, &mut Visibility)>,
) {
    let detector = player.and_then(|player| *player);
    let signal = detector
        .and_then(|detector| detector.reading)
        .map(|reading| DetectorSignal {
            distance: reading.distance,
            bearing: reading.bearing,
            range: DETECTOR_RANGE,
        });
    let (mut readout, mut visibility) = hud.into_inner();
    visibility.set_if_neq(if detector.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    readout.set_if_neq(DetectorReadout { signal });
}

fn sync_flash_overlay(
    player: Option<Single<Option<&FlashExposure>, With<PlayerController>>>,
    hud: Single<(&mut FlashOverlay, &mut Visibility)>,
) {
    let exposure = player.and_then(|player| *player);
    let elapsed = exposure.map_or(0.0, |exposure| exposure.elapsed());
    let (mut overlay, mut visibility) = hud.into_inner();
    visibility.set_if_neq(if exposure.is_some() && flash_alpha(elapsed) > 0.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    overlay.set_if_neq(FlashOverlay { elapsed });
}
