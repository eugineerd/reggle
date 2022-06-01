use std::collections::HashSet;

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;

use crate::{
    ball::{ball_collision_system, ball_hit_reaction_system, Ball, HitByBall},
    input_state::{GameAction, InputState},
    GameAssets, PEG_RADIUS,
};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HashSet::<Entity>::new())
            .add_system(spawn_peg_system)
            .add_system(peg_despawn_system.before(ball_collision_system))
            .add_system(
                peg_hit_system
                    .before(ball_hit_reaction_system)
                    .after(ball_collision_system),
            );
    }
}

#[derive(Component)]
pub struct Peg;

#[derive(Component)]
pub struct PegToDespawn;

#[derive(Component)]
pub struct HitPeg;

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

fn peg_despawn_system(
    mut commands: Commands,
    mut pegs: Query<Entity, (With<Peg>, With<PegToDespawn>)>,
    balls: Query<&Ball>,
) {
    if !balls.is_empty() {
        return;
    }
    for entity in pegs.iter_mut() {
        commands.entity(entity).despawn();
    }
}

fn peg_hit_system(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut hit_pegs: Query<
        (Entity, &mut Handle<Image>, &mut Sprite),
        (With<Peg>, Without<HitPeg>, Added<HitByBall>),
    >,
) {
    for (entity, mut peg_image, mut peg_sprite) in hit_pegs.iter_mut() {
        let idx = fastrand::usize(..game_assets.peg_hit_sound.len());
        audio.play(game_assets.peg_hit_sound[idx].clone());

        *peg_image = game_assets.peg_hit_image.clone();
        peg_sprite.color = Color::GREEN;

        commands.entity(entity).insert(HitPeg).insert(PegToDespawn);
    }
}
