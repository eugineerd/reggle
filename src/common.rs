use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

#[derive(Default)]
pub struct GameAssets {
    pub peg_hit_sound: Vec<Handle<AudioSource>>,
    pub peg_image: Handle<Image>,
    pub peg_hit_image: Handle<Image>,
    pub ball_hit_sound: Vec<Handle<AudioSource>>,
    pub ball_image: Handle<Image>,
    pub launcher_image: Handle<Image>,
    pub background_image: Handle<Image>,
}

#[derive(Default)]
pub struct GameState {
    pub player_score: usize,
}