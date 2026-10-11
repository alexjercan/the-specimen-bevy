mod common;

use bevy::world_serialization::WorldAssetRoot;
use game_halloween::CandyFuse;

use common::{fuse_module, halloween_app, halloween_assets, world_asset};

#[test]
fn only_the_fuse_module_is_rewrapped_as_candy_once_assets_are_ready() {
    let fuse = fuse_module();
    let other = world_asset(3);
    let candy = halloween_assets().candy_fuse;
    let mut app = halloween_app();
    app.update();

    let wrapper = app.world().resource::<CandyFuse>();
    assert_eq!(wrapper.fuse, fuse.id());
    assert_eq!(wrapper.candy, candy);

    let world = app.world_mut();
    let wrapped = world.spawn(WorldAssetRoot(fuse.clone())).id();
    let untouched = world.spawn(WorldAssetRoot(other.clone())).id();
    world.flush();
    assert_eq!(world.get::<WorldAssetRoot>(wrapped).unwrap().0, candy);
    assert_eq!(world.get::<WorldAssetRoot>(untouched).unwrap().0, other);
}
