#![cfg(feature = "halloween")]

use game_core::AppBuilder;
use game_halloween::HalloweenPlugin;

#[test]
fn headless_builds_skip_the_cosmetic_halloween_theme() {
    let app = AppBuilder::headless().build();
    assert!(!app.is_plugin_added::<HalloweenPlugin>());
}
