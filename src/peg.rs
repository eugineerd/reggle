use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_rapier2d::prelude::*;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::{Animator, EaseFunction, Tween};
use std::collections::VecDeque;
use std::time::Duration;

use crate::common::{GameState, GameStats, InGameState};
use crate::sounds::{play_collision_sound, CollisionSound, SoundType};
use crate::{assets::GameAssets, PEG_RADIUS};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PegDespawnQueue::default())
            .add_systems(OnEnter(GameState::InGame), spawn_peg_system)
            .add_systems(
                Update,
                (
                    peg_hit_system
                        .run_if(in_state(InGameState::Ball))
                        .after(play_collision_sound),
                    peg_despawn_system.run_if(in_state(InGameState::Cleanup)),
                ),
            );
    }
}

#[derive(Component, Default)]
pub struct Peg {
    pub is_target: bool,
    pub is_hit: bool,
}

#[derive(Resource, Default)]
pub struct PegDespawnQueue(VecDeque<Entity>);

#[derive(Bundle)]
pub struct PegBundle {
    #[bundle()]
    pub sprite_bundle: SpriteBundle,

    pub collider: Collider,
    pub name: Name,
    pub collision_sound: CollisionSound,
    pub peg: Peg,
}

impl PegBundle {
    pub fn new(transform: Transform, game_assets: &GameAssets) -> Self {
        Self {
            collider: Collider::ball(PEG_RADIUS),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PEG_RADIUS * 2.0, PEG_RADIUS * 2.0)),
                    ..Default::default()
                },
                transform: transform,
                texture: game_assets.peg.image.clone(),
                ..Default::default()
            },
            name: Name::new("Peg"),
            collision_sound: CollisionSound {
                sound: SoundType::Random(game_assets.peg.hit_sound.clone()),
                ..Default::default()
            },
            peg: Peg::default(),
        }
    }
}

fn spawn_peg_system(mut commands: Commands, game_assets: Res<GameAssets>) {
    let pegs_count = 15 * 7;
    let target_pegs_count = pegs_count / 10;

    let mut transforms = Vec::with_capacity(pegs_count);
    for i in 0..15 {
        for j in 1..8 {
            let t = Transform::from_xyz(
                (14 / 2 - i) as f32 * PEG_RADIUS * 5.0,
                -j as f32 * PEG_RADIUS * 5.0,
                0.0,
            );
            transforms.push(t);
        }
    }

    fastrand::shuffle(&mut transforms);

    let mut pegs = Vec::with_capacity(pegs_count - target_pegs_count);
    let mut target_pegs = Vec::with_capacity(target_pegs_count);
    for (i, t) in transforms.iter().enumerate() {
        if i <= target_pegs_count {
            let mut b = PegBundle::new(t.clone(), &game_assets);
            b.sprite_bundle.sprite.color = Color::ORANGE;
            b.peg.is_target = true;
            target_pegs.push(b)
        } else {
            pegs.push(PegBundle::new(t.clone(), &game_assets))
        }
    }

    commands.spawn_batch(pegs);
    commands.spawn_batch(target_pegs);
}

fn peg_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<Audio>,
    game_assets: Res<GameAssets>,
    mut game_stats: ResMut<GameStats>,
    mut peg_sprites: Query<&mut Sprite, With<Peg>>,
    mut despawn_timer: Local<Option<Timer>>,
    mut peg_despawn_queue: ResMut<PegDespawnQueue>,
) {
    let despawn_timer =
        despawn_timer.get_or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Repeating));
    despawn_timer.tick(time.delta());
    if let Some(peg) = peg_despawn_queue.0.front() {
        let inflated_size = PEG_RADIUS * 2.0 + despawn_timer.percent() * PEG_RADIUS * 1.5;
        peg_sprites.get_mut(*peg).unwrap().custom_size = Some(Vec2::splat(inflated_size));
        if despawn_timer.just_finished() {
            audio.play(game_assets.peg.pop_sound.clone());
            game_stats.player_score += 1;
            commands.entity(*peg).despawn();
            peg_despawn_queue.0.pop_front();
        }
    } else {
        despawn_timer.reset();
        peg_despawn_queue.0.clear();
        commands.insert_resource(NextState(Some(InGameState::Launcher)));
    }
}

fn peg_hit_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    game_assets: Res<GameAssets>,
    mut pegs: Query<(
        Entity,
        &mut Handle<Image>,
        &mut Sprite,
        &Transform,
        &mut CollisionSound,
        &mut Peg,
    )>,
    mut peg_despawn_queue: ResMut<PegDespawnQueue>,
) {
    for event in collision_events.iter() {
        let CollisionEvent::Started(e1, e2, _) = event else {continue};
        let mut comps = pegs.get_mut(*e1);
        if comps.is_err() {
            comps = pegs.get_mut(*e2);
        }
        let Ok((entity, mut peg_image, mut peg_sprite, tr, mut cs, mut peg)) = comps else {continue};

        if peg.is_hit {
            continue;
        }

        peg.is_hit = true;
        *peg_image = game_assets.peg.hit_image.clone();
        if peg_sprite.color == Color::ORANGE {
            peg_sprite.color = Color::ORANGE_RED;
        } else {
            peg_sprite.color = Color::rgb(0.5, 0.6, 1.0);
        }
        cs.sound = SoundType::None;
        let hit_tween = Tween::new(
            EaseFunction::CubicIn,
            Duration::from_secs_f32(0.1),
            TransformScaleLens {
                start: tr.scale,
                end: tr.scale * 1.5,
            },
        )
        .then(Tween::new(
            EaseFunction::CubicOut,
            Duration::from_secs_f32(0.1),
            TransformScaleLens {
                end: tr.scale,
                start: tr.scale * 1.5,
            },
        ));
        commands.entity(entity).insert(Animator::new(hit_tween));
        peg_despawn_queue.0.push_back(entity);
    }
}
