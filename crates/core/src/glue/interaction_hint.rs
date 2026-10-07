use bevy::{prelude::*, transform::TransformSystems};
use game_assets::{GameAssetsState, UiAssets};
use gameplay::{
    controller::PlayerController,
    levels::{DoorState, FuseInventory, InteractTarget, InteractTargets},
};

const DOOR_HINT_WIDTH: f32 = 130.0;
const FUSE_HINT_WIDTH: f32 = 160.0;
const PANEL_HINT_WIDTH: f32 = 172.0;

#[derive(Component)]
struct InteractionHint;

#[derive(Component)]
struct InteractionHintLabel;

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
                ImageNode::new(assets.interact_key.clone()),
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
        (&Transform, &Camera, &GlobalTransform, Option<&FuseInventory>),
        With<PlayerController>,
    >,
    targets: InteractTargets,
    mut hint: Query<(&mut Node, &mut Visibility), With<InteractionHint>>,
    mut label: Query<&mut Text, With<InteractionHintLabel>>,
) {
    let Ok((mut node, mut visibility)) = hint.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let Some((player, camera, camera_transform, inventory)) = players.iter().next() else {
        return;
    };
    let Some(target) = targets.aimed(player, inventory) else {
        return;
    };
    let (action, width) = match target {
        InteractTarget::Door(entity) if targets.locked(entity) => ("LOCKED", DOOR_HINT_WIDTH),
        InteractTarget::Door(entity) => match targets.door(entity).map(|door| door.state) {
            Some(DoorState::Closed) => ("OPEN", DOOR_HINT_WIDTH),
            Some(DoorState::Open) => ("CLOSE", DOOR_HINT_WIDTH),
            None => return,
        },
        InteractTarget::Fuse(_) => ("PICK UP FUSE", FUSE_HINT_WIDTH),
        InteractTarget::Panel(_) => ("INSTALL FUSES", PANEL_HINT_WIDTH),
    };
    let Some(anchor) = targets.anchor(target) else {
        return;
    };
    let Ok(point) = camera.world_to_viewport(camera_transform, anchor) else {
        return;
    };
    let Some(viewport) = camera.logical_viewport_rect() else {
        return;
    };
    if point.x < viewport.min.x
        || point.x >= viewport.max.x
        || point.y < viewport.min.y + 20.0
        || point.y >= viewport.max.y
    {
        return;
    }
    if let Ok(mut text) = label.single_mut() {
        if text.0 != action {
            text.0 = action.to_owned();
        }
    }
    node.width = px(width);
    node.left = px(point.x - width / 2.0);
    node.top = px(point.y - 19.0);
    *visibility = Visibility::Visible;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_spawns_from_loaded_ui_assets_and_starts_hidden() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameAssetsState>()
            .insert_resource(UiAssets {
                interact_key: Handle::default(),
                font: Handle::default(),
            })
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
