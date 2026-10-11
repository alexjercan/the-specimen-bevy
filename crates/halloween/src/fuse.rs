use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use game_assets::{FacilityAssets, GameAssetsState};
use gameplay::levels::FUSE_MODULE;

use crate::HalloweenAssets;

pub const CANDY_FUSE: &str = "candy_fuse";

#[derive(Resource, Clone, Debug)]
pub struct CandyFuse {
    pub fuse: AssetId<WorldAsset>,
    pub candy: Handle<WorldAsset>,
}

pub(crate) struct CandyFusePlugin;

impl Plugin for CandyFusePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameAssetsState::Ready), prepare_candy_fuse)
            .add_observer(wrap_fuse);
    }
}

fn prepare_candy_fuse(
    facility: Res<FacilityAssets>,
    halloween: Res<HalloweenAssets>,
    mut commands: Commands,
) {
    let Some(fuse) = facility.module(FUSE_MODULE) else {
        return;
    };
    commands.insert_resource(CandyFuse {
        fuse: fuse.id(),
        candy: halloween.candy_fuse.clone(),
    });
}

fn wrap_fuse(
    inserted: On<Insert, WorldAssetRoot>,
    candy: Option<Res<CandyFuse>>,
    mut roots: Query<&mut WorldAssetRoot>,
) {
    let Some(candy) = candy else {
        return;
    };
    let Ok(mut root) = roots.get_mut(inserted.entity) else {
        return;
    };
    if root.0.id() == candy.fuse {
        root.0 = candy.candy.clone();
    }
}

#[cfg(test)]
#[path = "../tests/unit/fuse.rs"]
mod tests;
