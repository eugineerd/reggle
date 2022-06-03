use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

use crate::ball::BallCollisionEvent;
use crate::common::{GameState, IngameState};
use crate::{
    input_state::{GameAction, InputState},
    GameAssets, PEG_RADIUS,
};

pub struct PegPlugin;

impl Plugin for PegPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PegsToDespawn::default())
            .add_system_set(SystemSet::on_update(GameState::Ingame).with_system(spawn_peg_system))
            .add_system_set(SystemSet::on_update(IngameState::Ball).with_system(peg_hit_system))
            .add_system_set(
                SystemSet::on_update(IngameState::Cleanup).with_system(peg_despawn_system),
            );
    }
}

#[derive(Component)]
pub struct Peg;

#[derive(Component)]
pub struct HitPeg;

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
    time: Res<Time>,
    mut state: ResMut<State<IngameState>>,
    mut pegs_to_despawn: ResMut<PegsToDespawn>,
) {
    pegs_to_despawn.despawn_timer.tick(time.delta());
    if pegs_to_despawn.despawn_timer.just_finished() {
        if let Some(peg) = pegs_to_despawn.queue.front() {
            commands.entity(*peg).despawn();
            pegs_to_despawn.queue.pop_front();
        } else {
            pegs_to_despawn.despawn_timer.reset();
            pegs_to_despawn.queue.clear();
            pegs_to_despawn.set.clear();
            state.set(IngameState::Launcher).unwrap();
        }
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
                peg_sprite.color = Color::rgb(0.5, 0.6, 1.0);
                let idx = fastrand::usize(..game_assets.peg_hit_sound.len());
                audio.play(game_assets.peg_hit_sound[idx].clone());
                pegs_to_despawn.set.insert(entity);
                pegs_to_despawn.queue.push_back(entity);
            }
        }
    }
}
