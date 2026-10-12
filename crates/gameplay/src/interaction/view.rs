use bevy::{prelude::*, transform::TransformSystems};
use game_settings::{key_label, GameSettings};

use super::{
    assets::InteractionAssets,
    pickups::PickupKind,
    sounds::InteractionSoundsPlugin,
    targets::{InteractTarget, InteractTargets},
};
use crate::{facility::DoorState, fuses::FuseInventory, hiding::Hidden, player::PlayerController};

const DOOR_HINT_WIDTH: f32 = 130.0;
const FUSE_HINT_WIDTH: f32 = 160.0;
const PANEL_HINT_WIDTH: f32 = 172.0;
const BOILER_HINT_WIDTH: f32 = 184.0;
const HIDING_HINT_WIDTH: f32 = 130.0;
const FLASHBANG_HINT_WIDTH: f32 = 208.0;
const DETECTOR_HINT_WIDTH: f32 = 196.0;
const LEAVE_HINT_TOP: f32 = 80.0;

#[derive(Component)]
struct InteractionHint;

#[derive(Component)]
struct InteractionHintLabel;

#[derive(Component)]
struct HintKeyImage;

// NOTE: app glue inserts InteractionAssets once the loader is ready; this plugin never inits it.
pub struct InteractionViewPlugin;

impl Plugin for InteractionViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InteractionSoundsPlugin)
            .add_systems(
                Update,
                spawn_hint.run_if(resource_added::<InteractionAssets>),
            )
            .add_systems(
                PostUpdate,
                update_hint
                    .after(TransformSystems::Propagate)
                    .run_if(resource_exists::<InteractionAssets>),
            );
    }
}

fn spawn_hint(mut commands: Commands, assets: Res<InteractionAssets>) {
    commands
        .spawn((
            InteractionHint,
            Name::new("Interaction hint"),
            Node {
                position_type: PositionType::Absolute,
                width: px(DOOR_HINT_WIDTH),
                height: px(38),
                padding: UiRect::axes(px(7), px(4)),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                column_gap: px(7),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.09, 0.92)),
            BorderColor::all(Color::srgba(0.56, 0.65, 0.65, 0.8)),
            Visibility::Hidden,
            GlobalZIndex(10),
        ))
        .with_children(|hint| {
            hint.spawn((
                HintKeyImage,
                ImageNode::new(assets.interact_glyph.clone()),
                Node {
                    width: px(28),
                    height: px(28),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            hint.spawn((
                InteractionHintLabel,
                Text::new("OPEN"),
                TextFont::from_font_size(16.0).with_font(assets.font.clone()),
                TextColor(Color::srgb(0.91, 0.94, 0.86)),
            ));
        });
}

fn update_hint(
    player: Option<
        Single<
            (
                &Transform,
                &Camera,
                &GlobalTransform,
                Option<&FuseInventory>,
                Option<&Hidden>,
            ),
            With<PlayerController>,
        >,
    >,
    targets: InteractTargets,
    settings: Res<GameSettings>,
    hint: Single<(&mut Node, &mut Visibility), (With<InteractionHint>, Without<HintKeyImage>)>,
    mut key_image: Single<&mut Node, (With<HintKeyImage>, Without<InteractionHint>)>,
    mut label: Single<&mut Text, With<InteractionHintLabel>>,
) {
    let (mut node, mut visibility) = hint.into_inner();
    *visibility = Visibility::Hidden;
    let Some(player) = player else {
        return;
    };
    let (player, camera, camera_transform, inventory, hidden) = player.into_inner();
    let Some(target) = targets.aimed(player, inventory, hidden) else {
        return;
    };
    let (action, width) = match target {
        InteractTarget::Door(entity) if targets.locked(entity) => ("LOCKED", DOOR_HINT_WIDTH),
        InteractTarget::Door(entity) => match targets.door(entity).map(|door| door.state) {
            Some(DoorState::Closed) => ("OPEN", DOOR_HINT_WIDTH),
            Some(DoorState::Open) => ("CLOSE", DOOR_HINT_WIDTH),
            None => return,
        },
        InteractTarget::Panel(_) => ("INSTALL FUSES", PANEL_HINT_WIDTH),
        InteractTarget::Boiler(_) => ("RESTORE POWER", BOILER_HINT_WIDTH),
        InteractTarget::Pickup(entity) => match targets.pickup(entity) {
            Some(PickupKind::Fuse { .. }) => ("PICK UP FUSE", FUSE_HINT_WIDTH),
            Some(PickupKind::Flashbang) => ("PICK UP FLASHBANG", FLASHBANG_HINT_WIDTH),
            Some(PickupKind::Detector) => ("PICK UP DETECTOR", DETECTOR_HINT_WIDTH),
            None => return,
        },
        InteractTarget::Hide(_) => ("HIDE", HIDING_HINT_WIDTH),
        InteractTarget::Leave(_) => ("LEAVE", HIDING_HINT_WIDTH),
    };
    let Some(viewport) = camera.logical_viewport_rect() else {
        return;
    };
    let point = if let InteractTarget::Leave(_) = target {
        leave_hint_position(viewport)
    } else {
        let Some(anchor) = targets.anchor(target) else {
            return;
        };
        let Ok(point) = camera.world_to_viewport(camera_transform, anchor) else {
            return;
        };
        point
    };
    if point.x < viewport.min.x
        || point.x >= viewport.max.x
        || point.y < viewport.min.y + 20.0
        || point.y >= viewport.max.y
    {
        return;
    }
    let key = settings.keys.interact;
    let custom = key != KeyCode::KeyF;
    key_image.display = if custom { Display::None } else { Display::Flex };
    let message = if custom {
        format!("{}: {action}", key_label(key))
    } else {
        action.to_owned()
    };
    if label.0 != message {
        label.0 = message;
    }
    let width = if custom { width + 32.0 } else { width };
    node.width = px(width);
    node.left = px(point.x - width / 2.0);
    node.top = px(point.y - 19.0);
    *visibility = Visibility::Visible;
}

fn leave_hint_position(viewport: Rect) -> Vec2 {
    Vec2::new(viewport.center().x, viewport.min.y + LEAVE_HINT_TOP)
}
