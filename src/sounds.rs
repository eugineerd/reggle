use std::sync::Arc;

use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::common::GameState;

pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            play_collision_sound.run_if(in_state(GameState::InGame)),
        );
    }
}
#[derive(Default, Clone)]
pub enum SoundType {
    #[default]
    None,
    Single(Handle<AudioSource>),
    Random(Arc<Vec<Handle<AudioSource>>>),
}

#[derive(Component, Clone)]
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

impl CollisionSound {
    pub fn play(&self, audio: &Audio) {
        match &self.sound {
            SoundType::Single(h) => {
                audio.play(h.clone());
            }
            SoundType::Random(hs) => {
                if let Some(h) = fastrand::choice(hs.as_ref()) {
                    audio.play(h.clone()).with_volume(self.volume);
                }
            }
            SoundType::None => (),
        };
    }
}

pub fn play_collision_sound(
    mut collision_events: EventReader<CollisionEvent>,
    ents: Query<&CollisionSound>,
    audio: Res<Audio>,
) {
    for e in collision_events.iter() {
        let CollisionEvent::Started(e1, e2, _) = e else {continue};
        let css = match (ents.get(*e1), ents.get(*e2)) {
            (Ok(cs_a), Ok(cs_b)) => {
                if cs_a.priority == cs_b.priority {
                    [Some(cs_a), Some(cs_b)]
                } else if cs_a.priority > cs_b.priority {
                    [Some(cs_a), None]
                } else {
                    [Some(cs_b), None]
                }
            }
            (Ok(cs), Err(_)) => [Some(cs), None],
            (Err(_), Ok(cs)) => [Some(cs), None],
            (Err(_), Err(_)) => [None, None],
        };
        for cs in css.iter().filter_map(|x| *x) {
            cs.play(&audio)
        }
    }
}
