use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_kira_audio::AudioSource;

#[derive(Default)]
pub struct GameAssets {
    pub peg_hit_sound: Vec<Handle<AudioSource>>,
    pub peg_image: Handle<Image>,
    pub peg_hit_image: Handle<Image>,
    pub peg_pop_sound: Handle<AudioSource>,
    pub ball_hit_sound: Vec<Handle<AudioSource>>,
    pub ball_image: Handle<Image>,
    pub launcher_image: Handle<Image>,
    pub background_image: Handle<Image>,
    pub normal_font: Handle<Font>,
}

#[derive(Inspectable)]
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum GameState {
    _Menu,
    InGame,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InGameState {
    Launcher,
    Ball,
    Cleanup,
}
