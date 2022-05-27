use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{input::InputState, PLAYER_BALL_RADIUS};

#[derive(Component)]
pub struct Ball;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_ball_system);
    }
}

fn spawn_ball_system(mut commands: Commands, input_state: Res<InputState>) {
    if !input_state.spawn_ball_action {
        return;
    }
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(PLAYER_BALL_RADIUS))
        .insert(Restitution {
            coefficient: 0.9,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Velocity {
            linvel: (input_state.cursor_position - Vec2::new(0.0, 150.0)).normalize_or_zero()
                * 200.0,
            ..Default::default()
        })
        .insert(ExternalImpulse::default())
        .insert(Transform::from_xyz(0.0, 150.0, 0.0))
        .insert(Ccd::enabled())
        .insert(MassProperties {
            mass: 1.0,
            ..Default::default()
        })
        .insert(Ball);
}
