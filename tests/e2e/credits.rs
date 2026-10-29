use std::sync::Arc;

use autopilot::{frames, state_is, AutopilotPlugin, Predicate};
use bevy::{prelude::*, ui::UiGlobalTransform};
use game_app::{AppBuilder, Frontend};
use game_flow::GameState;

pub(super) fn rendered_credits_layout() -> Result<(), String> {
    let fits_window: Arc<Predicate> = Arc::new(|world| {
        let Some(window) = world
            .try_query_filtered::<&Window, With<bevy::window::PrimaryWindow>>()
            .and_then(|mut query| query.iter(world).next())
        else {
            return false;
        };
        let height = window.resolution.physical_height() as f32;
        let Some(mut nodes) = world.try_query::<(&Name, &ComputedNode, &UiGlobalTransform)>()
        else {
            return false;
        };
        let Some((_, screen, _)) = nodes
            .iter(world)
            .find(|(name, _, _)| name.as_str() == "Credits screen")
        else {
            return false;
        };
        let Some((_, _, retry)) = nodes
            .iter(world)
            .find(|(name, _, _)| name.as_str() == "Retry button")
        else {
            return false;
        };
        let Some(mut titles) = world.try_query::<(&Text, &UiGlobalTransform)>() else {
            return false;
        };
        let Some((_, title)) = titles.iter(world).find(|(text, _)| text.0 == "ESCAPED") else {
            return false;
        };
        let width = window.resolution.physical_width() as f32;
        (screen.size().y - height).abs() < 2.0
            && (retry.translation.y - height * 0.5).abs() < height * 0.15
            && title.translation.x < width * 0.3
            && (height * 0.25..height * 0.5).contains(&title.translation.y)
    });
    let script = AutopilotPlugin::new()
        .step("load menu")
        .until(state_is(GameState::MainMenu), 60.0)
        .add()
        .step("win")
        .act(|world| {
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::Complete);
        })
        .until(state_is(GameState::Credits), 25.0)
        .add()
        .step("lay out credits")
        .until(frames(30), 10.0)
        .expect(
            fits_window,
            "victory menu must fit and be centered in the window",
        )
        .add();
    let mut app = AppBuilder::new(Frontend::Windowed { menu: true }).build();
    app.add_plugins(script);
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(format!("credits layout exited with code {code}")),
    }
}
