use bevy::{prelude::*, transform::TransformSystems};
use game_assets::{GameAssetsState, UiAssets};
use gameplay::{
    controller::PlayerController,
    levels::{aimed_door, panel_center, Door, DoorState, DoorSwing},
};

const HINT_WIDTH: f32 = 130.0;

#[derive(Component)]
struct DoorHint;

#[derive(Component)]
struct DoorHintLabel;

pub struct DoorHintPlugin;

impl Plugin for DoorHintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), spawn_hint)
            .add_systems(PostUpdate, update_hint.after(TransformSystems::Propagate));
    }
}

fn spawn_hint(mut commands: Commands, assets: Res<UiAssets>) {
    commands
        .spawn((
            DoorHint,
            Name::new("Door interaction hint"),
            Node {
                position_type: PositionType::Absolute,
                width: px(HINT_WIDTH),
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
                DoorHintLabel,
                Text::new("OPEN"),
                TextFont::from_font_size(16.0).with_font(assets.font.clone()),
                TextColor(Color::srgb(0.91, 0.94, 0.86)),
            ));
        });
}

fn update_hint(
    players: Query<(&Transform, &Camera, &GlobalTransform), With<PlayerController>>,
    doors: Query<(Entity, &Door, &DoorSwing)>,
    mut hint: Query<(&mut Node, &mut Visibility), With<DoorHint>>,
    mut label: Query<&mut Text, With<DoorHintLabel>>,
) {
    let Ok((mut node, mut visibility)) = hint.single_mut() else {
        return;
    };
    *visibility = Visibility::Hidden;
    let Some((player, camera, camera_transform)) = players.iter().next() else {
        return;
    };
    let Some(entity) = aimed_door(player, &doors) else {
        return;
    };
    let Ok((_, door, swing)) = doors.get(entity) else {
        return;
    };
    let Ok(point) = camera.world_to_viewport(camera_transform, panel_center(door, swing)) else {
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
        text.0 = match door.state {
            DoorState::Closed => "OPEN",
            DoorState::Open => "CLOSE",
        }
        .to_owned();
    }
    node.left = px(point.x - HINT_WIDTH / 2.0);
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
            .add_plugins(DoorHintPlugin);
        app.world_mut()
            .resource_mut::<NextState<GameAssetsState>>()
            .set(GameAssetsState::Ready);
        app.update();
        let mut hints = app
            .world_mut()
            .query_filtered::<(&Visibility, &Children), With<DoorHint>>();
        let (visibility, children) = hints.single(app.world()).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!(children.len(), 2);
        assert!(app.world().get::<ImageNode>(children[0]).is_some());
        assert_eq!(app.world().get::<Text>(children[1]).unwrap().0, "OPEN");
        assert!(app.world().get::<TextFont>(children[1]).is_some());
    }
}
