use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::DrawMode;
use bevy_prototype_lyon::prelude::GeometryBuilder;
use bevy_prototype_lyon::prelude::PathBuilder;
use bevy_prototype_lyon::prelude::StrokeMode;
use bevy_prototype_lyon::shapes::Circle;
use bevy_rapier2d::na;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::ColliderHandle;
use bevy_rapier2d::rapier::prelude::IntegrationParameters;
use bevy_rapier2d::rapier::prelude::RigidBodyBuilder;
use bevy_rapier2d::rapier::prelude::{
    BroadPhase, CCDSolver, ColliderBuilder, ColliderSet, ImpulseJointSet, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodySet,
};

use crate::ball::Ball;
use crate::ball::BallPhysicsBundle;
use crate::launcher::Launcher;
use crate::peg::PegToDespawn;
use crate::PIXELS_PER_METER;
use crate::PLAYER_BALL_RADIUS;

pub struct TrajectoryPlugin;

impl Plugin for TrajectoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_trajectory_world)
            .add_system(sync_trajectory_world_system)
            .add_system(draw_trajectory_system);
    }
}

pub struct TrajectoryWorld {
    scale: f32,
    scaled_shape_subdivision: u32,
    colliders: HashMap<Entity, ColliderHandle>,
    collider_set: ColliderSet,
    rigid_body_set: RigidBodySet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    gravity: na::Vector2<f32>,
}

impl TrajectoryWorld {
    fn new(scale: f32, scaled_shape_subdivision: u32) -> Self {
        TrajectoryWorld {
            scale,
            scaled_shape_subdivision,
            colliders: HashMap::new(),
            collider_set: ColliderSet::new(),
            rigid_body_set: RigidBodySet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            gravity: na::Vector2::new(0.0, -9.81 * 10.0 / scale),
        }
    }

    fn remove_collider(&mut self, entity: Entity) {
        if let Some(collider_handle) = self.colliders.get(&entity) {
            self.collider_set.remove(
                *collider_handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
        }
        self.colliders.remove(&entity);
    }

    fn add_collider(&mut self, entity: Entity, collider: &Collider, translation: Vec2) {
        let scaled_shape = collider
            .as_unscaled_typed_shape()
            .raw_scale_by(Vec2::ONE / self.scale, self.scaled_shape_subdivision)
            .unwrap();
        let rapier_collider = ColliderBuilder::new(scaled_shape)
            .translation((translation / self.scale).into())
            .build();
        let handle = self.collider_set.insert(rapier_collider);
        self.colliders.insert(entity, handle);
    }

    fn simulate_body_trajectory(
        &mut self,
        start_pos: Vec2,
        linvel: Vec2,
        collider: &Collider,
        restitution: &Restitution,
        mut max_collisions: usize,
    ) -> (Vec<Vec2>, Vec<Vec2>) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation((start_pos / self.scale).into())
            .linvel((linvel / self.scale).into())
            .build();
        let ball = collider
            .as_unscaled_typed_shape()
            .raw_scale_by(Vec2::ONE / self.scale, self.scaled_shape_subdivision)
            .unwrap();
        let collider = ColliderBuilder::new(ball)
            .restitution(restitution.coefficient)
            .restitution_combine_rule(bevy_rapier2d::rapier::prelude::CoefficientCombineRule::Max)
            .build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        let ball_collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        let integration_parameters = IntegrationParameters {
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        let physics_hooks = ();
        let event_handler = ();

        let mut trajectory_positions: Vec<bevy::prelude::Vec2> = Vec::with_capacity(300);
        let mut hit_positions: Vec<bevy::prelude::Vec2> = Vec::new();
        let mut encountered_colliders = HashSet::new();
        for _ in 0..300 {
            self.physics_pipeline.step(
                &self.gravity,
                &integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                &physics_hooks,
                &event_handler,
            );

            let body = &self.rigid_body_set[body_handle];
            let position = (*body.translation() * self.scale).into();

            trajectory_positions.push(position);
            if let Some(pair) = self.narrow_phase.contacts_with(ball_collider_handle).next() {
                let other_collider = if pair.collider1 != ball_collider_handle {
                    pair.collider1
                } else {
                    pair.collider2
                };
                if encountered_colliders.contains(&other_collider)
                    || pair.manifolds.get(0).map_or(true, |m| m.points.is_empty())
                {
                    continue;
                }
                encountered_colliders.insert(other_collider);
                hit_positions.push(position);
                max_collisions -= 1;
                if max_collisions == 0 {
                    break;
                }
            }
        }
        self.rigid_body_set.remove(
            body_handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );

        (trajectory_positions, hit_positions)
    }
}

pub fn init_trajectory_world(mut commands: Commands, rapier_config: Res<RapierConfiguration>) {
    commands.insert_resource(TrajectoryWorld::new(
        PIXELS_PER_METER,
        rapier_config.scaled_shape_subdivision,
    ));
}

// TODO: sync changed colliders and generalize removed pegs to any collider
pub fn sync_trajectory_world_system(
    mut trajectory_world: ResMut<TrajectoryWorld>,
    added_colliders: Query<(Entity, &Transform, &Collider), (Added<Collider>, Without<Ball>)>,
    // changed_colliders: Query<(Entity, &Collider), (Changed<Collider>, Without<Ball>)>,
    removed_pegs: Query<Entity, (Without<Ball>, With<PegToDespawn>)>,
) {
    for entity in removed_pegs.iter() {
        trajectory_world.remove_collider(entity);
    }
    for (entity, transform, collider) in added_colliders.iter() {
        trajectory_world.add_collider(entity, collider, transform.translation.truncate());
    }
}

#[derive(Component)]
struct Trajectory;

fn draw_trajectory_system(
    mut commands: Commands,
    mut trajectory_world: ResMut<TrajectoryWorld>,
    launcher: Query<(&Transform, &Launcher)>,
    lines: Query<Entity, With<Trajectory>>,
) {
    for line in lines.iter() {
        commands.entity(line).despawn();
    }
    // Let else and if let chains can't come soon enough...
    if let Ok((launcher_tr, launcher)) = launcher.get_single() {
        let ball_bundle = BallPhysicsBundle::new(launcher_tr.translation);
        let start_pos = launcher_tr.translation.truncate();
        let (trajectory_points, hit_points) = trajectory_world.simulate_body_trajectory(
            start_pos,
            launcher.get_impulse(),
            &ball_bundle.collider,
            &ball_bundle.restitution,
            2,
        );

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(start_pos);
        for point in trajectory_points {
            path_builder.line_to(point);
        }
        let line = path_builder.build();

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &line,
                DrawMode::Stroke(StrokeMode::new(Color::GREEN, 3.0)),
                Transform::default(),
            ))
            .insert(Trajectory);

        for point in hit_points.iter() {
            let shape = Circle {
                radius: PLAYER_BALL_RADIUS,
                center: *point,
            };
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shape,
                    DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
                    Transform::default(),
                ))
                .insert(Trajectory);
        }
    }
}
