use bevy::asset::uuid::Uuid;

use super::*;

fn handle(n: u128) -> Handle<WorldAsset> {
    Handle::Uuid(Uuid::from_u128(n), default())
}

#[test]
fn only_the_fuse_module_is_rewrapped_as_candy() {
    let fuse = handle(1);
    let candy = handle(2);
    let other = handle(3);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(CandyFuse {
            fuse: fuse.id(),
            candy: candy.clone(),
        })
        .add_observer(wrap_fuse);
    let world = app.world_mut();
    let wrapped = world.spawn(WorldAssetRoot(fuse.clone())).id();
    let untouched = world.spawn(WorldAssetRoot(other.clone())).id();
    world.flush();
    assert_eq!(world.get::<WorldAssetRoot>(wrapped).unwrap().0, candy);
    assert_eq!(world.get::<WorldAssetRoot>(untouched).unwrap().0, other);
}

#[test]
fn fuses_keep_their_model_without_halloween_assets() {
    let fuse = handle(1);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_observer(wrap_fuse);
    let world = app.world_mut();
    let entity = world.spawn(WorldAssetRoot(fuse.clone())).id();
    world.flush();
    assert_eq!(world.get::<WorldAssetRoot>(entity).unwrap().0, fuse);
}
