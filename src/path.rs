use bevy::prelude::*;

use crate::{
    common::GameState,
    spline::{Segment, SegmentType},
};

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_path_points,
                tessellate_path_segments,
                (draw_path, move_path_agents),
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .register_type::<Path>()
        .register_type::<PathAgent>()
        .register_type::<PathPoint>();
    }
}

#[derive(Reflect, Default, Clone, Copy)]
pub enum PathEasingFunction {
    None,
    #[default]
    Constant,
}

struct RawPathPoint {
    pos: Vec2,
    segment: Segment,
    speed_multiplier: f32,
    easing_function: PathEasingFunction,
    entity: Entity,
}

#[derive(Component, Reflect, Default, Clone)]
pub struct PathPoint {
    pub segment_type: SegmentType,
    pub speed_multiplier: f32,
    pub easing_function: PathEasingFunction,
}

impl PathPoint {
    fn as_raw(&self, translation: Vec3, self_entity: Entity) -> RawPathPoint {
        let Self {
            segment_type,
            speed_multiplier,
            easing_function,
        } = *self;
        RawPathPoint {
            pos: translation.truncate(),
            entity: self_entity,
            segment: Segment::new(segment_type),
            speed_multiplier,
            easing_function,
        }
    }
}

#[derive(Component, Default, Reflect)]
pub struct Path {
    #[reflect(ignore)]
    points: Vec<RawPathPoint>,
    pub move_speed: f32,
    pub looped: bool,
}

impl Path {
    pub fn new(move_speed: f32, looped: bool) -> Self {
        Self {
            move_speed,
            looped,
            ..Default::default()
        }
    }
    pub fn get_world_pos(&self, t: f32) -> Vec2 {
        let point_idx = (t.floor() as usize).clamp(0, self.points.len() - 1);
        let point = &self.points[point_idx];

        if let PathEasingFunction::Constant = point.easing_function {
            point
                .segment
                .get_pos_cached(t - point_idx as f32)
                .unwrap_or(self.points[point_idx].pos)
        } else {
            let Some(neigbors) = self.get_neigbors_positions(point_idx) else {return point.pos};
            point.segment.get_pos(t - point_idx as f32, &neigbors)
        }
    }

    pub fn tessellate_segments(&mut self) {
        for i in 0..self.points.len() {
            let neighbors = self.get_neigbors_positions(i);
            self.points[i].segment.tessellate(neighbors.as_ref());
        }
    }

    fn get_neigbors_positions(&self, point_idx: usize) -> Option<[Vec2; 4]> {
        if !self.looped && point_idx >= self.points.len() - 1 {
            return None;
        }
        let p1 = self.points[point_idx].pos;
        let p2 = self.points[(point_idx + 1) % self.points.len()].pos;
        let (p0, p3) = if self.looped {
            (
                self.points[point_idx
                    .checked_add_signed(-1)
                    .unwrap_or(self.points.len() - 1)]
                .pos,
                self.points[(point_idx + 2) % self.points.len()].pos,
            )
        } else {
            let dir = (p2 - p1).normalize_or_zero();
            let p0 = if point_idx > 0 {
                self.points[point_idx - 1].pos
            } else {
                p1 - dir
            };
            let p3 = if point_idx < self.points.len() - 2 {
                self.points[point_idx + 2].pos
            } else {
                p2 + dir
            };
            (p0, p3)
        };
        Some([p0, p1, p2, p3])
    }

    pub fn move_agent_along_path(&self, mut t: f32, time_delta: f32) -> f32 {
        let mut point_idx = (t.floor() as usize).clamp(0, self.points.len() - 1);
        t = t.fract();

        let mut agent_distance = self.move_speed * time_delta;
        loop {
            let segment = &self.points[point_idx].segment;
            let seg_distance = (1.0 - t) * segment.len();
            if seg_distance < f32::EPSILON || seg_distance.is_nan() {
                break;
            }
            if seg_distance > agent_distance {
                t += agent_distance / segment.len();
                break;
            }
            agent_distance -= seg_distance;
            t = 0.0;
            point_idx += 1;
            if point_idx >= self.points.len() {
                if self.looped {
                    point_idx = 0
                } else {
                    break;
                }
            }
        }
        t + point_idx as f32
    }
}

#[derive(Component, Reflect)]
pub struct PathAgent {
    pub t: f32,
}

fn move_path_agents(
    paths: Query<&Path>,
    mut agents: Query<(&mut PathAgent, &mut Transform, &Parent)>,
    time: Res<Time>,
) {
    for (mut agent, mut tr, path) in agents.iter_mut() {
        let Ok(path) = paths.get(path.get()) else {continue};

        agent.t = path.move_agent_along_path(agent.t, time.delta_seconds());
        tr.translation = path.get_world_pos(agent.t).extend(tr.translation.z)
    }
}

fn draw_path(paths: Query<(&Path, &Transform, &ComputedVisibility)>, mut gizmos: Gizmos) {
    for (path, tr, cv) in paths.iter() {
        if !cv.is_visible() {
            continue;
        }
        let path_pos = tr.translation.truncate();
        for point in &path.points {
            gizmos.circle_2d(point.pos + path_pos, 5.0, Color::RED);
            let segment = &point.segment;
            if segment.len() == 0.0 {
                continue;
            }
            let seg_points = segment.points();
            for (p1, p2) in seg_points[..seg_points.len()].iter().zip(&seg_points[1..]) {
                gizmos.line_2d(*p1 + path_pos, *p2 + path_pos, Color::WHITE);
            }
        }
    }
}

fn tessellate_path_segments(mut paths: Query<&mut Path, Or<(Changed<Path>, Changed<Transform>)>>) {
    for mut path in paths.iter_mut() {
        path.tessellate_segments();
    }
}

fn sync_path_points(
    mut paths: Query<&mut Path>,
    changed_points: Query<
        (Entity, &PathPoint, &GlobalTransform, &Parent),
        Or<(Changed<PathPoint>, Changed<GlobalTransform>)>,
    >,
    mut removed_points: RemovedComponents<PathPoint>,
) {
    for (point_e, point, tr, parent) in changed_points.iter() {
        let Ok(mut path) = paths.get_mut(parent.get()) else {continue};
        let new_raw_point = point.as_raw(tr.translation(), point_e);
        if let Some(raw_point) = path.points.iter_mut().find(|p| p.entity == point_e) {
            *raw_point = new_raw_point;
        } else {
            path.points.push(new_raw_point);
        };
    }
    for removed_point in removed_points.iter() {
        // Not optimal
        for mut path in paths.iter_mut() {
            // Avoid triggering change detection unless point is actually removed
            if let Some(idx) = path
                .bypass_change_detection()
                .points
                .iter()
                .position(|e| e.entity == removed_point)
            {
                path.points.remove(idx);
            }
        }
    }
}
