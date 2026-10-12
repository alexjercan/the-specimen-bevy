use bevy::{animation::graph::AnimationNodeIndex, prelude::*, world_serialization::WorldAssetRoot};

use super::{
    assets::MonsterAssets,
    logic::{
        Caught, Monster, MonsterFigure, ATTACK_DURATION, MODEL_LIFT, MODEL_PIVOT, MODEL_SCALE,
    },
    sounds::MonsterSoundsPlugin,
};

#[derive(Component)]
struct MonsterScene;

#[derive(Resource)]
struct MonsterAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    chase: AnimationNodeIndex,
    attack: AnimationNodeIndex,
}

// NOTE: app glue inserts MonsterAssets once the loader is ready; this plugin never inits it.
// NOTE: build_animations runs in the Update frame where MonsterAssets appears; the chained
// NOTE: scene systems wait for MonsterAnimations, so nothing spawns before the assets exist.
pub struct MonsterViewPlugin;

impl Plugin for MonsterViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MonsterSoundsPlugin).add_systems(
            Update,
            (
                build_animations.run_if(resource_added::<MonsterAssets>),
                (attach_scene, pose_figure, animate_monster)
                    .chain()
                    .run_if(resource_exists::<MonsterAnimations>),
            )
                .chain(),
        );
    }
}

fn build_animations(
    assets: Res<MonsterAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
) {
    let (graph, clips) = AnimationGraph::from_clips([
        assets.idle.clone(),
        assets.walk.clone(),
        assets.chase.clone(),
        assets.attack.clone(),
    ]);
    commands.insert_resource(MonsterAnimations {
        graph: graphs.add(graph),
        idle: clips[0],
        walk: clips[1],
        chase: clips[2],
        attack: clips[3],
    });
}

fn attach_scene(
    mut commands: Commands,
    monsters: Query<
        Entity,
        (
            Or<(With<Monster>, With<MonsterFigure>)>,
            Without<MonsterScene>,
        ),
    >,
    assets: Res<MonsterAssets>,
) {
    for entity in &monsters {
        commands.entity(entity).with_children(|children| {
            children.spawn((
                MonsterScene,
                WorldAssetRoot(assets.scene.clone()),
                Transform::from_xyz(
                    -MODEL_PIVOT.x * MODEL_SCALE,
                    MODEL_LIFT,
                    -MODEL_PIVOT.y * MODEL_SCALE,
                )
                .with_scale(Vec3::splat(MODEL_SCALE)),
            ));
        });
        commands.entity(entity).insert(MonsterScene);
    }
}

fn animate_monster(
    mut commands: Commands,
    animations: Res<MonsterAnimations>,
    monsters: Query<(Entity, &Monster, &Children)>,
    caught: Query<&Caught>,
    scene_roots: Query<(), With<MonsterScene>>,
    children: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    for (entity, monster, roots) in &monsters {
        let attacking = caught.iter().any(|caught| caught.monster == entity);
        for root in roots.iter().filter(|root| scene_roots.contains(*root)) {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok((entity, mut player, graph)) = players.get_mut(entity) {
                    if graph.is_none() {
                        commands
                            .entity(entity)
                            .insert(AnimationGraphHandle(animations.graph.clone()));
                    }
                    let clip = if attacking {
                        animations.attack
                    } else if monster.is_chasing() {
                        animations.chase
                    } else if monster.is_idle() {
                        animations.idle
                    } else {
                        animations.walk
                    };
                    if player.is_added() || !player.is_playing_animation(clip) {
                        player.stop_all();
                        let playing = player.play(clip);
                        if !attacking {
                            playing.repeat();
                        }
                    }
                }
                if let Ok(descendants) = children.get(entity) {
                    stack.extend(descendants.iter());
                }
            }
        }
    }
}

fn pose_figure(
    mut commands: Commands,
    animations: Res<MonsterAnimations>,
    figures: Query<&Children, With<MonsterFigure>>,
    scene_roots: Query<(), With<MonsterScene>>,
    children: Query<&Children>,
    mut players: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) {
    for roots in &figures {
        for root in roots.iter().filter(|root| scene_roots.contains(*root)) {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok((entity, mut player, graph)) = players.get_mut(entity) {
                    if graph.is_none() {
                        commands
                            .entity(entity)
                            .insert(AnimationGraphHandle(animations.graph.clone()));
                    }
                    if !player.is_playing_animation(animations.attack) {
                        player
                            .play(animations.attack)
                            .seek_to(ATTACK_DURATION)
                            .pause();
                    }
                }
                if let Ok(descendants) = children.get(entity) {
                    stack.extend(descendants.iter());
                }
            }
        }
    }
}
