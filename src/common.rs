use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Reflect, Resource)]
pub struct GameStats {
    pub player_score: usize,
    pub target_pegs_left: usize,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            player_score: 0,
            target_pegs_left: 20,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, States, Default)]
pub enum GameState {
    #[default]
    LoadingAssets,
    Menu,
    InGame,
}

impl GameState {
    pub fn next(&self) -> Self {
        use GameState::*;
        match *self {
            LoadingAssets => InGame,
            Menu => InGame,
            InGame => InGame,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, States, Default)]
pub enum InGameState {
    #[default]
    Launcher,
    Ball,
    Cleanup,
}
