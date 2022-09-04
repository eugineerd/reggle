use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_kira_audio::{Audio, AudioControl};
use bevy_rapier2d::prelude::*;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, Tween, TweeningType};
use iyes_loopless::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

use crate::ball::BallCollisionEvent;
use crate::common::{GameState, GameStats, InGameState};
use crate::{GameAssets, PEG_RADIUS};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PegsToDespawn::default())
            .add_enter_system(GameState::InGame, spawn_peg_system.exclusive_system())
            .add_system(peg_hit_system.run_in_state(InGameState::Ball))
            .add_system(peg_despawn_system.run_in_state(InGameState::Cleanup));
    }
}

#[derive(Component)]
pub struct Peg;

#[derive(Component)]
pub struct HitPeg;

#[derive(Component)]
pub struct TargetPeg;

pub struct PegsToDespawn {
    set: HashSet<Entity>,
    despawn_timer: Timer,
    queue: VecDeque<Entity>,
}

impl Default for PegsToDespawn {
    fn default() -> Self {
        Self {
            set: Default::default(),
            despawn_timer: Timer::from_seconds(0.1, true),
            queue: Default::default(),
        }
    }
}

#[derive(Bundle)]
pub struct PegBundle {
    pub collider: Collider,
    pub transform: Transform,
    pub sprite: Sprite,
    pub global_transform: GlobalTransform,
    pub image_handle: Handle<Image>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub name: Name,
    pub peg: Peg,
}

impl PegBundle {
    pub fn new(transform: Transform, image_handle: Handle<Image>) -> Self {
        Self {
            collider: Collider::ball(PEG_RADIUS),
            transform,
            sprite: Sprite {
                custom_size: Some(Vec2::new(PEG_RADIUS * 2.0, PEG_RADIUS * 2.0)),
                ..Default::default()
            },
            global_transform: Default::default(),
            image_handle,
            visibility: Default::default(),
            computed_visibility: Default::default(),
            name: Name::new("Peg"),
            peg: Peg,
        }
    }
}

fn spawn_peg_system(world: &mut World) {
    let image_handle = world.resource::<GameAssets>().peg_image.clone();
    let mut pegs_entities: Vec<Entity> = world
        .spawn_batch((0..=14).flat_map(move |i| {
            (1..=7).map({
                let image_handle = image_handle.clone();
                move |j| {
                    let transform = Transform::from_xyz(
                        (14 / 2 - i) as f32 * PEG_RADIUS * 5.0,
                        -j as f32 * PEG_RADIUS * 5.0,
                        0.0,
                    );
                    PegBundle::new(transform, image_handle.clone())
                }
            })
        }))
        .collect();

    fastrand::shuffle(&mut pegs_entities);
    let target_pegs_num = world.resource::<GameStats>().target_pegs_left;
    for e in pegs_entities.iter().take(target_pegs_num) {
        if let Some(mut sprite) = world.entity_mut(*e).insert(TargetPeg).get_mut::<Sprite>() {
            sprite.color = Color::ORANGE
        }
    }
}

fn peg_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<Audio>,
    game_assets: Res<GameAssets>,
    mut game_stats: ResMut<GameStats>,
    mut peg_sprites: Query<&mut Sprite, With<Peg>>,
    mut pegs_to_despawn: ResMut<PegsToDespawn>,
) {
    pegs_to_despawn.despawn_timer.tick(time.delta());
    if let Some(peg) = pegs_to_despawn.queue.front() {
        let inflated_size =
            PEG_RADIUS * 2.0 + pegs_to_despawn.despawn_timer.percent() * PEG_RADIUS * 1.5;
        peg_sprites.get_mut(*peg).unwrap().custom_size = Some(Vec2::splat(inflated_size));
        if pegs_to_despawn.despawn_timer.just_finished() {
            audio.play(game_assets.peg_pop_sound.clone());
            game_stats.player_score += 1;
            commands.entity(*peg).despawn();
            pegs_to_despawn.queue.pop_front();
        }
    } else {
        pegs_to_despawn.despawn_timer.reset();
        pegs_to_despawn.queue.clear();
        pegs_to_despawn.set.clear();
        commands.insert_resource(NextState(InGameState::Launcher));
    }
}

fn peg_hit_system(
    mut commands: Commands,
    mut hit_by_ball: EventReader<BallCollisionEvent>,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut pegs: Query<(Entity, &mut Handle<Image>, &mut Sprite, &Transform), With<Peg>>,
    mut pegs_to_despawn: ResMut<PegsToDespawn>,
) {
    for event in hit_by_ball.iter() {
        if let Ok((entity, mut peg_image, mut peg_sprite, tr)) = pegs.get_mut(event.0) {
            if !pegs_to_despawn.set.contains(&entity) {
                *peg_image = game_assets.peg_hit_image.clone();
                if peg_sprite.color == Color::ORANGE {
                    peg_sprite.color = Color::ORANGE_RED;
                } else {
                    peg_sprite.color = Color::rgb(0.5, 0.6, 1.0);
                }
                let hit_tween = Tween::new(
                    EaseFunction::CubicIn,
                    TweeningType::Once,
                    Duration::from_secs_f32(0.1),
                    TransformScaleLens {
                        start: tr.scale,
                        end: tr.scale * 1.5,
                    },
                )
                .then(Tween::new(
                    EaseFunction::CubicOut,
                    TweeningType::Once,
                    Duration::from_secs_f32(0.1),
                    TransformScaleLens {
                        end: tr.scale,
                        start: tr.scale * 1.5,
                    },
                ));
                commands.entity(entity).insert(Animator::new(hit_tween));
                let idx = fastrand::usize(..game_assets.peg_hit_sound.len());
                audio.play(game_assets.peg_hit_sound[idx].clone());
                pegs_to_despawn.set.insert(entity);
                pegs_to_despawn.queue.push_back(entity);
            }
        }
    }
}
