use std::f32::consts::FRAC_PI_2;

use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
    world_serialization::WorldAssetRoot,
};

pub const DEFAULT_PART_DIR: &str = "facility/modules";
const DEFAULT_YAW: f32 = 2.55;
const DEFAULT_PITCH: f32 = 0.52;
const CAPTURE_SETTLE_FRAMES: u32 = 120;

#[derive(Clone, Copy)]
pub struct Part {
    pub module: &'static str,
    pub dir: &'static str,
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub scale: f32,
}

pub const fn part(module: &'static str, x: f32, y: f32, z: f32, yaw: f32, scale: f32) -> Part {
    Part {
        module,
        dir: DEFAULT_PART_DIR,
        position: Vec3::new(x, y, z),
        yaw,
        pitch: 0.0,
        scale,
    }
}

#[allow(dead_code)]
pub const fn wall_part(module: &'static str, x: f32, y: f32, z: f32) -> Part {
    Part {
        module,
        dir: DEFAULT_PART_DIR,
        position: Vec3::new(x, y, z),
        yaw: 0.0,
        pitch: -FRAC_PI_2,
        scale: 1.0,
    }
}

impl Part {
    #[allow(dead_code)]
    pub const fn in_dir(mut self, dir: &'static str) -> Part {
        self.dir = dir;
        self
    }

    fn asset_path(&self) -> String {
        format!("{}/{}.glb#Scene0", self.dir, self.module)
    }
}

pub struct Exhibit {
    pub group: &'static str,
    pub title: &'static str,
    pub parts: &'static [Part],
    pub radius: f32,
}

pub type SpawnExtraFn = fn(
    &mut Commands,
    Entity,
    &Exhibit,
    &AssetServer,
    &mut Assets<Mesh>,
    &mut Assets<StandardMaterial>,
);

#[derive(Resource, Clone, Copy)]
pub struct Gallery {
    pub exhibits: &'static [Exhibit],
    pub env_var: &'static str,
    pub focus_height: fn(&Exhibit) -> f32,
    pub spawn_extra: Option<SpawnExtraFn>,
    pub min_radius: f32,
}

impl Gallery {
    fn initial_index(&self) -> usize {
        std::env::var(self.env_var)
            .ok()
            .and_then(|name| {
                self.exhibits.iter().position(|entry| {
                    entry.title.eq_ignore_ascii_case(&name)
                        || entry.parts.iter().any(|part| part.module == name)
                })
            })
            .unwrap_or(0)
    }
}

#[derive(Resource)]
struct Selection {
    index: usize,
    root: Option<Entity>,
}

#[derive(Component)]
struct GalleryCamera;

#[derive(Component)]
struct GalleryLabel;

#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

#[derive(Resource)]
struct CaptureConfig {
    path: String,
}

#[derive(Resource, Default)]
struct CaptureState {
    settled: u32,
    taken: bool,
}

pub struct GalleryPlugin(pub Gallery);

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        let gallery = self.0;
        let initial = gallery.initial_index();
        let yaw = env_f32("GALLERY_YAW", DEFAULT_YAW);
        let pitch = env_f32("GALLERY_PITCH", DEFAULT_PITCH);
        app.add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/assets").into(),
            ..default()
        }))
        .insert_resource(gallery)
        .insert_resource(Selection {
            index: initial,
            root: None,
        })
        .insert_resource(Orbit {
            yaw,
            pitch,
            radius: gallery.exhibits[initial].radius,
        })
        .insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.09)))
        .add_systems(Startup, setup)
        .add_systems(Update, (select_exhibit, show_exhibit, orbit_camera).chain());

        if let Ok(path) = std::env::var("GALLERY_CAPTURE") {
            app.insert_resource(CaptureConfig { path })
                .init_resource::<CaptureState>()
                .add_systems(Update, capture_gallery.after(orbit_camera));
        }
    }
}

fn env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.24, 0.25, 0.26))),
        Transform::IDENTITY,
    ));
    commands.spawn((
        Camera3d::default(),
        GalleryCamera,
        Transform::from_xyz(2.0, 2.0, 3.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 220_000.0,
            range: 12.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(2.0, 4.0, 3.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 100_000.0,
            range: 12.0,
            ..default()
        },
        Transform::from_xyz(-3.0, 2.5, -2.0),
    ));
    commands.spawn((
        Text::new(""),
        GalleryLabel,
        TextFont {
            font_size: bevy::text::FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(18),
            left: px(22),
            ..default()
        },
    ));
}

fn select_exhibit(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheels: MessageReader<MouseWheel>,
    gallery: Res<Gallery>,
    mut selection: ResMut<Selection>,
    mut orbit: ResMut<Orbit>,
) {
    let step = i32::from(keys.just_pressed(KeyCode::ArrowRight))
        - i32::from(keys.just_pressed(KeyCode::ArrowLeft));
    if step != 0 {
        if let Some(root) = selection.root.take() {
            commands.entity(root).despawn();
        }
        selection.index = game_settings::cycle(
            &(0..gallery.exhibits.len()).collect::<Vec<_>>(),
            selection.index,
            step as isize,
        );
        orbit.radius = gallery.exhibits[selection.index].radius;
    }
    for wheel in wheels.read() {
        orbit.radius = (orbit.radius - wheel.y * 0.1).clamp(gallery.min_radius, 10.0);
    }
}

fn show_exhibit(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gallery: Res<Gallery>,
    mut selection: ResMut<Selection>,
    mut label: Single<&mut Text, With<GalleryLabel>>,
) {
    if selection.root.is_some() {
        return;
    }
    let entry = &gallery.exhibits[selection.index];
    let root = commands
        .spawn((
            Name::new(entry.title),
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .id();
    for part in entry.parts {
        commands.spawn((
            Name::new(part.module),
            WorldAssetRoot(assets.load(part.asset_path())),
            Transform::from_translation(part.position)
                .with_rotation(Quat::from_rotation_y(part.yaw) * Quat::from_rotation_x(part.pitch))
                .with_scale(Vec3::splat(part.scale)),
            ChildOf(root),
        ));
    }
    if let Some(spawn_extra) = gallery.spawn_extra {
        spawn_extra(
            &mut commands,
            root,
            entry,
            &assets,
            &mut meshes,
            &mut materials,
        );
    }
    selection.root = Some(root);
    **label = format!(
        "{}/{}   {} / {}\nLeft/Right: browse   Drag: orbit   Mouse Wheel: zoom",
        selection.index + 1,
        gallery.exhibits.len(),
        entry.group,
        entry.title,
    )
    .into();
}

fn orbit_camera(
    gallery: Res<Gallery>,
    selection: Res<Selection>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut orbit: ResMut<Orbit>,
    mut camera: Single<&mut Transform, With<GalleryCamera>>,
) {
    for event in motion.read() {
        if buttons.pressed(MouseButton::Left) {
            orbit.yaw -= event.delta.x * 0.008;
            orbit.pitch = (orbit.pitch + event.delta.y * 0.008).clamp(-1.2, 1.2);
        }
    }
    let height = (gallery.focus_height)(&gallery.exhibits[selection.index]);
    let target = Vec3::new(0.0, height, 0.0);
    camera.translation = target
        + orbit.radius
            * Vec3::new(
                orbit.yaw.sin() * orbit.pitch.cos(),
                orbit.pitch.sin(),
                orbit.yaw.cos() * orbit.pitch.cos(),
            );
    camera.look_at(target, Vec3::Y);
}

fn capture_gallery(
    mut commands: Commands,
    config: Res<CaptureConfig>,
    selection: Res<Selection>,
    mut state: ResMut<CaptureState>,
) {
    if state.taken || selection.root.is_none() {
        return;
    }
    state.settled += 1;
    if state.settled < CAPTURE_SETTLE_FRAMES {
        return;
    }
    state.taken = true;
    let path = config.path.clone();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}
