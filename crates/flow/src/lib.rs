use bevy::prelude::*;

#[derive(States, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CoreState {
    #[default]
    Loading,
    Ready,
}

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(CoreState = CoreState::Ready)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Complete,
    GameOver,
    Credits,
    Achievements,
}

#[derive(SubStates, Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}
