use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};

use crate::{
    ball::{Ball, BallCollisionEvent},
    common::GameState,
};

pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_collision_sound.run_if(in_state(GameState::InGame)),
        );
    }
}

pub enum SoundType {
    None,
    Single(Handle<AudioSource>),
    Random(Vec<Handle<AudioSource>>),
}

#[derive(Component)]
pub struct CollisionSound {
    pub sound: SoundType,
    pub volume: f64,
    pub priority: i32,
}

impl Default for CollisionSound {
    fn default() -> Self {
        Self {
            sound: SoundType::None,
            volume: 1.0,
            priority: 0,
        }
    }
}

pub fn play_collision_sound(
    mut collision_events: EventReader<BallCollisionEvent>,
    ents: Query<&CollisionSound>,
    ball: Query<&CollisionSound, With<Ball>>,
    audio: Res<Audio>,
) {
    let ball_sound = ball.get_single();
    for e in collision_events.iter() {
        let css = match (&ball_sound, ents.get(e.0)) {
            (Ok(cs_a), Ok(cs_b)) => {
                if cs_a.priority == cs_b.priority {
                    [Some(*cs_a), Some(cs_b)]
                } else if cs_a.priority > cs_b.priority {
                    [Some(*cs_a), None]
                } else {
                    [Some(cs_b), None]
                }
            }
            (Ok(cs), Err(_)) => [Some(*cs), None],
            (Err(_), Ok(cs)) => [Some(cs), None],
            (Err(_), Err(_)) => [None, None],
        };
        for cs in css.iter().filter_map(|x| *x) {
            match &cs.sound {
                SoundType::Single(h) => {
                    audio.play(h.clone());
                }
                SoundType::Random(hs) => {
                    if let Some(h) = fastrand::choice(hs) {
                        audio.play(h.clone()).with_volume(cs.volume);
                    }
                }
                _ => (),
            };
        }
    }
}
