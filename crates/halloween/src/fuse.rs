use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use gameplay::facility::{FacilityAssets, Module};

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
        app.add_systems(
            Update,
            prepare_candy_fuse.run_if(resource_added::<FacilityAssets>),
        );
    }
}

fn prepare_candy_fuse(
    facility: Res<FacilityAssets>,
    halloween: Res<HalloweenAssets>,
    mut commands: Commands,
) {
    let fuse = facility
        .module(Module::FusePickup)
        .unwrap_or_else(|| panic!("missing fuse module: {:?}", Module::FusePickup));
    commands.insert_resource(CandyFuse {
        fuse: fuse.id(),
        candy: halloween.candy_fuse.clone(),
    });
    commands.add_observer(wrap_fuse);
}

fn wrap_fuse(
    inserted: On<Insert, WorldAssetRoot>,
    candy: Res<CandyFuse>,
    mut roots: Query<&mut WorldAssetRoot>,
) {
    let Ok(mut root) = roots.get_mut(inserted.entity) else {
        return;
    };
    if root.0.id() == candy.fuse {
        root.0 = candy.candy.clone();
    }
}
