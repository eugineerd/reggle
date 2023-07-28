use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_rapier2d::prelude::*;

use crate::common::{GameState, InGameState};
use crate::peg::PegDespawnEvent;
use crate::sounds::{CollisionSound, SoundType};
use crate::{assets::GameAssets, PLAYER_BALL_RADIUS};
use crate::{ARENA_POS, ARENA_SIZE};

const BALL_STUCK_VEL_SQ: f32 = 200.0;
const BALL_STUCK_SECS: u64 = 1;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (ball_despawn_system, unstuck_ball)
                .run_if(in_state(InGameState::Ball))
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
    pub vel: Velocity,
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
            acc: ActiveEvents::COLLISION_EVENTS,
            vel: Velocity::default(),
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

// Idea: run this system in a physics simulation that is in the future.
// That will allow us to remove blocks on which the ball would've stuck
// without making the player wait.
fn unstuck_ball(
    rapier_ctx: Res<RapierContext>,
    mut despawn_events: EventWriter<PegDespawnEvent>,
    balls: Query<(Entity, &Velocity), With<Ball>>,
    time: Res<Time>,
    mut collision_time: Local<HashMap<(Entity, Entity), Duration>>,
) {
    let mut to_retain = HashSet::new();
    for (ball_e, ball_vel) in balls.iter() {
        let is_low_vel = ball_vel.linvel.length_squared() < BALL_STUCK_VEL_SQ;
        for contact in rapier_ctx.contacts_with(ball_e) {
            if !contact.has_any_active_contacts() {
                continue;
            }
            let collider_e = if contact.collider1() != ball_e {
                contact.collider1()
            } else {
                contact.collider2()
            };
            let key = (ball_e, collider_e);
            to_retain.insert(key);
            let Some(duration) = collision_time.get_mut(&key) else {
                collision_time.insert((ball_e, collider_e), Duration::default());
                continue
            };
            *duration += time.delta();
            if *duration > Duration::from_secs(BALL_STUCK_SECS) && is_low_vel {
                despawn_events.send(PegDespawnEvent(collider_e));
            }
        }
    }
    collision_time.retain(|k, _| to_retain.contains(k));
}
