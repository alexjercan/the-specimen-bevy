use bevy::{prelude::*, world_serialization::WorldAsset};

#[derive(Resource)]
pub struct MonsterAssets {
    pub scene: Handle<WorldAsset>,
    pub idle: Handle<AnimationClip>,
    pub walk: Handle<AnimationClip>,
    pub chase: Handle<AnimationClip>,
    pub attack: Handle<AnimationClip>,
}
