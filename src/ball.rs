use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{common::GameAssets, PLAYER_BALL_RADIUS};

#[derive(Component)]
pub struct Ball;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ball_despawn_system);
    }
}

fn ball_despawn_system(mut commands: Commands, balls: Query<(Entity, &Transform), With<Ball>>) {
    for (entity, tr) in balls.iter() {
        if tr.translation.x.abs() > 1000.0 || tr.translation.y.abs() > 1000.0 {
            commands.entity(entity).despawn()
        }
    }
}

#[derive(Bundle)]
pub struct BallPhysicsBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub restitution: Restitution,
    pub ccd: Ccd,
    pub transform: Transform,
    pub mass: MassProperties,
}

impl BallPhysicsBundle {
    pub fn new(translation: Vec3) -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(PLAYER_BALL_RADIUS),
            restitution: Restitution {
                coefficient: 0.9,
                combine_rule: CoefficientCombineRule::Max,
            },
            ccd: Ccd::enabled(),
            transform: Transform::from_translation(translation),
            mass: MassProperties {
                mass: 5.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    #[bundle]
    pub ball_physics: BallPhysicsBundle,

    // SpriteBundle without tranform
    pub sprite: Sprite,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,

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
            texture: game_assets.ball_image.clone(),
            visibility: Default::default(),

            name: Name::new("Ball"),
            ball: Ball,
        }
    }
}
