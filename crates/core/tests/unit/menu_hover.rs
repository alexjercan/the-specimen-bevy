use bevy::prelude::*;
use game_audio::{PlaySound, Sound};
use game_ui::menu_button;

use super::{hover_buttons, settings::SettingsAction, MenuAction};

#[test]
fn all_main_menu_buttons_play_hover_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<PlaySound>()
        .add_systems(Update, hover_buttons);

    let font = Handle::<Font>::default();
    let buttons = [
        app.world_mut()
            .spawn((MenuAction::Play, menu_button("Play", font.clone())))
            .id(),
        app.world_mut()
            .spawn((SettingsAction::Open, menu_button("Settings", font.clone())))
            .id(),
        app.world_mut()
            .spawn((MenuAction::Quit, menu_button("Quit", font)))
            .id(),
    ];
    app.update();
    app.world_mut()
        .resource_mut::<Messages<PlaySound>>()
        .clear();

    for button in buttons {
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Hovered);
        app.update();
        let sounds: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<PlaySound>>()
            .drain()
            .collect();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].sound, Sound::UiHover);
        assert_eq!(sounds[0].position, None);

        app.update();
        assert!(app
            .world_mut()
            .resource_mut::<Messages<PlaySound>>()
            .drain()
            .next()
            .is_none());
    }
}
