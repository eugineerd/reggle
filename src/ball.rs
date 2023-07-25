use bevy::{prelude::*, utils::HashSet};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_rapier2d::prelude::*;

use crate::common::{GameState, InGameState};
use crate::sounds::{CollisionSound, SoundType};
use crate::{assets::GameAssets, PLAYER_BALL_RADIUS};
use crate::{ARENA_POS, ARENA_SIZE};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (ball_despawn_system.run_if(in_state(InGameState::Ball)),)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Ball;

#[derive(Bundle)]
pub struct BallPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub restitution: Restitution,
    pub ccd: Ccd,
    pub transform: Transform,
    pub acc: ActiveEvents,
}

impl BallPhysicsBundle {
    pub fn new(translation: Vec3) -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(PLAYER_BALL_RADIUS),
            restitution: Restitution {
                coefficient: 0.95,
                combine_rule: CoefficientCombineRule::Max,
            },
            ccd: Ccd::enabled(),
            transform: Transform::from_translation(translation),
            acc: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    #[bundle()]
    pub ball_physics: BallPhysicsBundle,

    // SpriteBundle without tranform
    pub sprite: Sprite,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,

    pub collision_sound: CollisionSound,
    pub name: Name,
    pub ball: Ball,
}

impl BallBundle {
    pub fn new(translation: Vec3, game_assets: &GameAssets) -> Self {
        Self {
            ball_physics: BallPhysicsBundle::new(translation),

            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    PLAYER_BALL_RADIUS * 2.0,
                    PLAYER_BALL_RADIUS * 2.0,
                )),
                ..Default::default()
            },
            global_transform: Default::default(),
            texture: game_assets.ball.image.clone(),
            visibility: Default::default(),
            computed_visibility: Default::default(),

            collision_sound: CollisionSound {
                sound: SoundType::Random(game_assets.ball.hit_sound.clone()),
                volume: 0.5,
                ..Default::default()
            },
            name: Name::new("Ball"),
            ball: Ball,
        }
    }
}

fn ball_despawn_system(mut commands: Commands, balls: Query<(Entity, &Transform), With<Ball>>) {
    if balls.is_empty() {
        commands.insert_resource(NextState(Some(InGameState::Cleanup)));
        return;
    }
    for (entity, tr) in balls.iter() {
        if tr.translation.x > (ARENA_POS.x + ARENA_SIZE.x / 2.0)
            || tr.translation.x < (ARENA_POS.x - ARENA_SIZE.x / 2.0)
            || tr.translation.y > (ARENA_POS.y + ARENA_SIZE.y / 2.0)
            || tr.translation.y < (ARENA_POS.y - ARENA_SIZE.y / 2.0)
        {
            commands.entity(entity).despawn()
        }
    }
}
