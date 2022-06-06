use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

use crate::ball::BallCollisionEvent;
use crate::common::{GameState, IngameState};
use crate::{GameAssets, PEG_RADIUS};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PegsToDespawn::default())
            .insert_resource(PegConfig {
                target_pegs_count: 20,
            })
            .add_system_set(SystemSet::on_enter(GameState::Ingame).with_system(spawn_peg_system))
            .add_system_set(
                SystemSet::on_update(IngameState::AllocatePegs)
                    .with_system(select_target_pegs_system.after(spawn_peg_system)),
            )
            .add_system_set(SystemSet::on_update(IngameState::Ball).with_system(peg_hit_system))
            .add_system_set(
                SystemSet::on_update(IngameState::Cleanup).with_system(peg_despawn_system),
            );
    }
}

pub struct PegConfig {
    pub target_pegs_count: usize,
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
            global_transform: GlobalTransform::default(),
            image_handle,
            visibility: Visibility::default(),
            name: Name::new("Peg"),
            peg: Peg,
        }
    }
}

fn spawn_peg_system(mut commands: Commands, game_assets: Res<GameAssets>) {
    let image_handle = game_assets.peg_image.clone();
    commands.spawn_batch((0..=14).flat_map(move |i| {
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
    }));
}

fn select_target_pegs_system(
    mut commands: Commands,
    peg_config: Res<PegConfig>,
    mut state: ResMut<State<IngameState>>,
    mut pegs: Query<(Entity, &mut Sprite), Added<Peg>>,
) {
    let mut pegs_vec: Vec<_> = pegs.iter_mut().collect();
    fastrand::shuffle(&mut pegs_vec);
    let mut pegs_left = peg_config.target_pegs_count;
    for (entity, sprite) in &mut pegs_vec {
        if pegs_left == 0 {
            break;
        }
        commands.entity(*entity).insert(TargetPeg);
        sprite.color = Color::ORANGE;
        pegs_left -= 1;
    }
    state.set(IngameState::Launcher).unwrap();
}

fn peg_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<Audio>,
    game_assets: Res<GameAssets>,
    mut state: ResMut<State<IngameState>>,
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
            commands.entity(*peg).despawn();
            pegs_to_despawn.queue.pop_front();
        }
    } else {
        pegs_to_despawn.despawn_timer.reset();
        pegs_to_despawn.queue.clear();
        pegs_to_despawn.set.clear();
        state.set(IngameState::Launcher).unwrap();
    }
}

fn peg_hit_system(
    mut hit_by_ball: EventReader<BallCollisionEvent>,
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut pegs: Query<(Entity, &mut Handle<Image>, &mut Sprite), With<Peg>>,
    mut pegs_to_despawn: ResMut<PegsToDespawn>,
) {
    for event in hit_by_ball.iter() {
        if let Ok((entity, mut peg_image, mut peg_sprite)) = pegs.get_mut(event.0) {
            if !pegs_to_despawn.set.contains(&entity) {
                *peg_image = game_assets.peg_hit_image.clone();
                if peg_sprite.color == Color::ORANGE {
                    peg_sprite.color = Color::ORANGE_RED;
                } else {
                    peg_sprite.color = Color::rgb(0.5, 0.6, 1.0);
                }
                let idx = fastrand::usize(..game_assets.peg_hit_sound.len());
                audio.play(game_assets.peg_hit_sound[idx].clone());
                pegs_to_despawn.set.insert(entity);
                pegs_to_despawn.queue.push_back(entity);
            }
        }
    }
}
