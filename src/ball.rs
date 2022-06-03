use bevy::{prelude::*, utils::HashSet};
use bevy_kira_audio::Audio;
use bevy_rapier2d::prelude::*;

use crate::common::IngameState;
use crate::{common::GameAssets, PLAYER_BALL_RADIUS};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BallCollisionEvent>()
            .add_system_set(
                SystemSet::on_update(IngameState::Ball)
                    .with_system(ball_collision_system)
                    .with_system(ball_despawn_system.before(ball_collision_system)),
            )
            .add_system(ball_hitsound_system.before(ball_collision_system));
    }
}

#[derive(Component)]
pub struct Ball;

pub struct BallCollisionEvent(pub Entity);

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
                coefficient: 0.95,
                combine_rule: CoefficientCombineRule::Max,
            },
            ccd: Ccd::enabled(),
            transform: Transform::from_translation(translation),
            mass: MassProperties {
                mass: 1.0,
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

fn ball_despawn_system(
    mut commands: Commands,
    mut state: ResMut<State<IngameState>>,
    balls: Query<(Entity, &Transform), With<Ball>>,
) {
    if balls.is_empty() {
        state.set(IngameState::Cleanup).unwrap();
        return;
    }
    for (entity, tr) in balls.iter() {
        if tr.translation.x.abs() > 400.0 || tr.translation.y.abs() > 400.0 {
            commands.entity(entity).despawn()
        }
    }
}

pub fn ball_collision_system(
    mut hit_events: EventWriter<BallCollisionEvent>,
    rapier_context: Res<RapierContext>,
    balls: Query<Entity, With<Ball>>,
    mut last_contact_entities: Local<HashSet<Entity>>,
) {
    let mut contact_entities = HashSet::new();
    balls.for_each(|ball| {
        for contact_pair in rapier_context.contacts_with(ball) {
            let other_collider = if contact_pair.collider1() == ball {
                contact_pair.collider2()
            } else {
                contact_pair.collider1()
            };

            contact_entities.insert(other_collider);
            if last_contact_entities.contains(&other_collider)
                || contact_pair
                    .manifold(0)
                    .map_or(true, |m| m.num_points() == 0)
            {
                continue;
            }
            last_contact_entities.insert(other_collider);
            hit_events.send(BallCollisionEvent(other_collider));
        }
    });
    last_contact_entities.retain(|e| contact_entities.contains(e));
}

pub fn ball_hitsound_system(
    game_assets: Res<GameAssets>,
    mut hit_by_ball: EventReader<BallCollisionEvent>,
    audio: Res<Audio>,
) {
    for _ in hit_by_ball.iter() {
        let idx = fastrand::usize(..game_assets.ball_hit_sound.len());
        audio.play(game_assets.ball_hit_sound[idx].clone());
    }
}
