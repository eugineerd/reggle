use std::collections::HashSet;

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;

use crate::{
    ball::Ball,
    input_state::{GameAction, InputState},
    GameAssets, GameState,
};

#[derive(Component)]
struct Peg;

#[derive(Component)]
pub struct PegToDespawn;

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HashSet::<Entity>::new())
            .add_system(spawn_peg_system)
            .add_system(peg_despawn_system.before(peg_hit_system))
            .add_system(peg_hit_system);
    }
}

fn spawn_peg_system(mut commands: Commands, input_state: Res<InputState>) {
    if !input_state.just_active(GameAction::SpawnPegs) {
        return;
    }
    commands.spawn_batch((0..=20).flat_map(|i| {
        (1..=7).map(move |j| {
            (
                Collider::ball(5.0),
                Transform::from_xyz((10 - i) as f32 * 30.0, -j as f32 * 30.0, 0.0),
                Name::new("Peg"),
                Peg,
            )
        })
    }))
}

// Note: sometimes rapier reports collision events but doesn't perfom physics calculation.
// this results in situations where the ball `grazes` the peg causing it to despawn,
// but doesn't change its own trajectory. Not sure how to fix this.
fn peg_despawn_system(
    mut commands: Commands,
    mut pegs: Query<Entity, (With<Peg>, With<PegToDespawn>)>,
) {
    for entity in pegs.iter_mut() {
        commands.entity(entity).despawn();
    }
}

fn peg_hit_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    balls: Query<Entity, With<Ball>>,
    pegs: Query<Entity, (With<Peg>, Without<PegToDespawn>)>,
    removed_pegs: Query<Entity, With<PegToDespawn>>,
    audio: Res<Audio>,
    assets: Res<GameAssets>,
) {
    balls.for_each(|entity| {
        for contact_pair in rapier_context.contacts_with(entity) {
            let other_collider = if contact_pair.collider1() == entity {
                contact_pair.collider2()
            } else {
                contact_pair.collider1()
            };

            if !pegs.contains(other_collider) {
                if !removed_pegs.contains(other_collider) {
                    let idx = fastrand::usize(..assets.ball_hit_sound.len());
                    audio.play(assets.ball_hit_sound[idx].clone());
                }
                continue;
            }
            if let Some(norm) = contact_pair.manifold(0).map(|x| x.normal()) {
                // Reduces number of `ghost` collisions a bit. See peg_despawn_system note
                if norm.length() > 0.01 {
                    game_state.player_score += 1;
                    commands.entity(other_collider).insert(PegToDespawn);
                    let idx = fastrand::usize(..assets.peg_hit_sound.len());
                    audio.play(assets.peg_hit_sound[idx].clone());
                }
            }
        }
    });
}
