use bevy::{prelude::*, transform::TransformSystems};
use game_assets::{GameAssetsState, UiAssets};
use game_settings::GameSettings;
use gameplay::{
    controller::PlayerController,
    levels::{DoorState, FuseInventory, Hidden, InteractTarget, InteractTargets, PickupKind},
};

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

pub struct InteractionHintPlugin;

impl Plugin for InteractionHintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), spawn_hint)
            .add_systems(PostUpdate, update_hint.after(TransformSystems::Propagate));
    }
}

fn spawn_hint(mut commands: Commands, assets: Res<UiAssets>) {
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
                ImageNode::new(assets.key_glyph("KeyF").unwrap_or_default()),
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
    players: Query<
        (
            &Transform,
            &Camera,
            &GlobalTransform,
            Option<&FuseInventory>,
            Option<&Hidden>,
        ),
        With<PlayerController>,
    >,
    targets: InteractTargets,
    settings: Option<Res<GameSettings>>,
    mut hint: Query<(&mut Node, &mut Visibility), (With<InteractionHint>, Without<HintKeyImage>)>,
    mut key_image: Query<&mut Node, (With<HintKeyImage>, Without<InteractionHint>)>,
    mut label: Query<&mut Text, With<InteractionHintLabel>>,
) {
    let Ok((mut node, mut visibility)) = hint.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let Some((player, camera, camera_transform, inventory, hidden)) = players.iter().next() else {
        return;
    };
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
    let key = settings
        .as_ref()
        .map_or("KeyF", |settings| settings.keys.interact.as_str());
    let custom = key != "KeyF";
    if let Ok(mut image) = key_image.single_mut() {
        image.display = if custom { Display::None } else { Display::Flex };
    }
    if let Ok(mut text) = label.single_mut() {
        let message = if custom {
            format!("{}: {action}", key.strip_prefix("Key").unwrap_or(key))
        } else {
            action.to_owned()
        };
        if text.0 != message {
            text.0 = message;
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn leave_hint_stays_above_bottom_center_detector() {
        let viewport = Rect::from_corners(Vec2::ZERO, Vec2::new(1280.0, 720.0));
        let position = leave_hint_position(viewport);
        assert_eq!(position.x, viewport.center().x);
        assert_eq!(position.y, viewport.min.y + LEAVE_HINT_TOP);
        let mut app = App::new();
        app.world_mut().spawn(game_ui::detector_readout());
        let mut tracker = app
            .world_mut()
            .query_filtered::<&Node, With<game_ui::DetectorReadout>>();
        let node = tracker.single(app.world()).unwrap();
        let (Val::Px(bottom), Val::Px(height)) = (node.bottom, node.height) else {
            panic!("detector dimensions must be in pixels");
        };
        let hint_bottom = position.y + 19.0;
        let tracker_top = viewport.max.y - bottom - height;
        assert!(hint_bottom < tracker_top);
    }

    use super::*;

    #[test]
    fn hint_spawns_from_loaded_ui_assets_and_starts_hidden() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameAssetsState>()
            .insert_resource(UiAssets::default())
            .add_plugins(InteractionHintPlugin);
        app.world_mut()
            .resource_mut::<NextState<GameAssetsState>>()
            .set(GameAssetsState::Ready);
        app.update();
        let mut hints = app
            .world_mut()
            .query_filtered::<(&Visibility, &Children), With<InteractionHint>>();
        let (visibility, children) = hints.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!(children.len(), 2);
        assert!(app.world().get::<ImageNode>(children[0]).is_some());
        assert_eq!(app.world().get::<Text>(children[1]).unwrap().0, "OPEN");
        assert!(app.world().get::<TextFont>(children[1]).is_some());
    }
}
