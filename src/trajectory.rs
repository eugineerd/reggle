use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, PathBuilder, ShapeBundle, Stroke};
use bevy_prototype_lyon::shapes::Circle;
use bevy_rapier2d::na;
use bevy_rapier2d::prelude::{Collider, RapierConfiguration, Restitution, RigidBody};
use bevy_rapier2d::rapier::prelude::{
    BroadPhase, CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, ImpulseJointSet,
    IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
    RigidBodyBuilder, RigidBodySet,
};

use crate::ball::BallPhysicsBundle;
use crate::common::{GameState, InGameState};
use crate::launcher::Launcher;
use crate::PIXELS_PER_METER;
use crate::PLAYER_BALL_RADIUS;

pub struct TrajectoryPlugin;

impl Plugin for TrajectoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), init_trajectory_world)
            .add_systems(
                Update,
                (draw_trajectory_system, despawn_trajectory_line)
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(InGameState::Launcher)),
            )
            .add_systems(
                OnExit(InGameState::Launcher),
                despawn_trajectory_line.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                PostUpdate,
                sync_colliders_system.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
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

    trajectory_points: Vec<Vec2>,
    collision_points: Vec<Vec2>,
}

impl TrajectoryWorld {
    fn new(scale: f32, scaled_shape_subdivision: u32, gravity: Vec2) -> Self {
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
            gravity: gravity.into(),

            trajectory_points: Vec::new(),
            collision_points: Vec::new(),
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

    fn update_collider(&mut self, entity: Entity, collider: &Collider) {
        if let Some(handle) = self.colliders.get(&entity) {
            if let Some(rapier_collider) = self.collider_set.get_mut(*handle) {
                let scaled_shape = collider
                    .as_unscaled_typed_shape()
                    .raw_scale_by(Vec2::ONE / self.scale, self.scaled_shape_subdivision)
                    .unwrap();
                rapier_collider.set_shape(scaled_shape);
            }
        };
    }

    fn move_collider(&mut self, entity: Entity, translation: Vec2) {
        if let Some(handle) = self.colliders.get(&entity) {
            if let Some(rapier_collider) = self.collider_set.get_mut(*handle) {
                rapier_collider.set_translation((translation / self.scale).into());
            }
        };
    }

    fn simulate_body_trajectory(
        &mut self,
        start_pos: Vec2,
        linvel: Vec2,
        collider: &Collider,
        restitution: &Restitution,
        mut max_collisions: usize,
        max_trajectory_points: usize,
    ) -> (&[Vec2], &[Vec2]) {
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
        let body_collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        let integration_parameters = IntegrationParameters {
            dt: 1.0 / 60.0,
            ..Default::default()
        };

        self.trajectory_points.clear();
        self.collision_points.clear();
        let mut encountered_colliders = HashSet::new();
        for _ in 0..max_trajectory_points {
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
                None,
                &(),
                &(),
            );

            let body = &self.rigid_body_set[body_handle];
            let position = (*body.translation() * self.scale).into();

            self.trajectory_points.push(position);
            if let Some(pair) = self.narrow_phase.contacts_with(body_collider_handle).next() {
                let other_collider = if pair.collider1 != body_collider_handle {
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
                self.collision_points.push(position);
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

        (&self.trajectory_points, &self.collision_points)
    }
}

pub fn init_trajectory_world(mut commands: Commands, rapier_config: Res<RapierConfiguration>) {
    commands.insert_resource(TrajectoryWorld::new(
        PIXELS_PER_METER,
        rapier_config.scaled_shape_subdivision,
        rapier_config.gravity / PIXELS_PER_METER,
    ));
}

pub fn sync_colliders_system(
    mut trajectory_world: ResMut<TrajectoryWorld>,
    added_colliders: Query<(Entity, &Transform, &Collider), Added<Collider>>,
    changed_colliders: Query<(Entity, &Collider), Changed<Collider>>,
    moved_colliders: Query<(Entity, &Transform), (Changed<Transform>, With<Collider>)>,
    mut removed_colliders: RemovedComponents<Collider>,
) {
    changed_colliders.for_each(|(entity, collider)| {
        trajectory_world.update_collider(entity, collider);
    });
    moved_colliders.for_each(|(entity, transform)| {
        trajectory_world.move_collider(entity, transform.translation.truncate());
    });
    added_colliders.for_each(|(entity, transform, collider)| {
        trajectory_world.add_collider(entity, collider, transform.translation.truncate());
    });
    for entity in removed_colliders.iter() {
        trajectory_world.remove_collider(entity);
    }
}

#[derive(Component)]
pub struct TrajectoryLine;

fn despawn_trajectory_line(mut commands: Commands, lines: Query<Entity, With<TrajectoryLine>>) {
    for line in lines.iter() {
        commands.entity(line).despawn();
    }
}

fn draw_trajectory_system(
    mut commands: Commands,
    mut trajectory_world: ResMut<TrajectoryWorld>,
    launcher: Query<(&Transform, &Launcher)>,
) {
    if let Ok((launcher_tr, launcher)) = launcher.get_single() {
        let ball_bundle = BallPhysicsBundle::new(launcher_tr.translation);
        let start_pos = launcher_tr.translation.truncate();
        let (trajectory_points, collision_points) = trajectory_world.simulate_body_trajectory(
            start_pos,
            launcher.get_impulse(),
            &ball_bundle.collider,
            &ball_bundle.restitution,
            1,
            200,
        );

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(start_pos);
        for point in trajectory_points {
            path_builder.line_to(*point);
        }
        let line = path_builder.build();

        commands.spawn((
            TrajectoryLine,
            ShapeBundle {
                path: GeometryBuilder::build_as(&line),
                transform: Transform::from_xyz(0., 0., -1.0),
                ..Default::default()
            },
            Stroke::new(Color::WHITE, 2.0),
        ));

        for point in collision_points.iter() {
            let shape = Circle {
                radius: PLAYER_BALL_RADIUS,
                center: *point,
            };
            commands.spawn((
                TrajectoryLine,
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    ..Default::default()
                },
                Fill::color(Color::RED),
            ));
        }
    }
}
