use std::collections::HashSet;

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;

use crate::{
    ball::Ball,
    input_state::{GameAction, InputState},
    GameAssets, GameState, PEG_RADIUS,
};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HashSet::<Entity>::new())
            .add_system(spawn_peg_system)
            .add_system(peg_despawn_system.before(peg_hit_system))
            .add_system(peg_hit_system)
            .add_system(peg_hit_reaction_system.after(peg_hit_system));
    }
}

#[derive(Component)]
struct Peg;

#[derive(Component)]
pub struct PegToDespawn;

#[derive(Component)]
pub struct PegHit;

fn spawn_peg_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    game_assets: Res<GameAssets>,
) {
    if !input_state.just_active(GameAction::SpawnPegs) {
        return;
    }
    let image_handle = game_assets.peg_image.clone();
    commands.spawn_batch((0..=12).flat_map(move |i| {
        (1..=7).map({
            let image_handle = image_handle.clone();
            move |j| {
                (
                    Collider::ball(PEG_RADIUS),
                    Transform::from_xyz(
                        (12 / 2 - i) as f32 * PEG_RADIUS * 5.0,
                        -j as f32 * PEG_RADIUS * 5.0,
                        0.0,
                    ),
                    Sprite {
                        custom_size: Some(Vec2::new(PEG_RADIUS * 2.0, PEG_RADIUS * 2.0)),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                    image_handle.clone(),
                    Visibility::default(),
                    Name::new("Peg"),
                    Peg,
                )
            }
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
    pegs: Query<Entity, (With<Peg>, Without<PegToDespawn>, Without<PegHit>)>,
    mut currently_in_contact: Local<HashSet<Entity>>,
    audio: Res<Audio>,
    assets: Res<GameAssets>,
) {
    let mut contact_entities = HashSet::new();
    balls.for_each(|entity| {
        for contact_pair in rapier_context.contacts_with(entity) {
            let other_collider = if contact_pair.collider1() == entity {
                contact_pair.collider2()
            } else {
                contact_pair.collider1()
            };
            contact_entities.insert(other_collider);
            if currently_in_contact.contains(&other_collider) {
                continue;
            }
            if contact_pair
                .manifold(0)
                .map_or(true, |m| m.num_points() == 0)
            {
                continue;
            }
            if !pegs.contains(other_collider) {
                currently_in_contact.insert(other_collider);
                let idx = fastrand::usize(..assets.ball_hit_sound.len());
                audio.play(assets.ball_hit_sound[idx].clone());
                continue;
            }
            currently_in_contact.insert(other_collider);
            game_state.player_score += 1;
            // commands.entity(other_collider).insert(PegToDespawn);
            commands.entity(other_collider).insert(PegHit);
            let idx = fastrand::usize(..assets.peg_hit_sound.len());
            audio.play(assets.peg_hit_sound[idx].clone());
        }
    });
    currently_in_contact.retain(|e| contact_entities.contains(e));
}

fn peg_hit_reaction_system(
    game_assets: Res<GameAssets>,
    mut hit_pegs: Query<(&mut Handle<Image>, &mut Sprite), (With<Peg>, Added<PegHit>)>,
) {
    for (mut peg_image, mut peg_sprite) in hit_pegs.iter_mut() {
        *peg_image = game_assets.peg_hit_image.clone();
        peg_sprite.color = Color::GREEN;
    }
}
