use bevy::prelude::*;

use crate::{common::GameState, input::GameInput};

// TODO:
// - [ ] Efficient distance from point to spline
// - [ ] Use bevy::math splines
// - [ ] Efficeint draw splines
// - [ ] Add movement easing using bevy::math spline

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_path).run_if(in_state(GameState::InGame)))
            .add_systems(Startup, path_test)
            .register_type::<Path>()
            .register_type::<PathPoint>()
            .register_type::<PathType>();
    }
}

fn path_test(mut commands: Commands) {
    commands.spawn((
        Path {
            points: vec![
                PathPoint {
                    pos: Vec2::new(0.0, 0.0),
                    ..Default::default()
                },
                PathPoint {
                    pos: Vec2::new(100.0, 0.0),
                    ..Default::default()
                },
                PathPoint {
                    pos: Vec2::new(200.0, 0.0),
                    ..Default::default()
                },
                PathPoint {
                    pos: Vec2::new(300.0, 0.0),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        Name::new("Path"),
    ));
}

#[derive(Default, Reflect)]
pub enum PathType {
    Linear,
    #[default]
    CatmullRom,
}

#[derive(Default, Reflect)]
pub struct PathPoint {
    pos: Vec2,
    move_speed_mul: f32,
    move_ease_method: (),
    path_type: PathType,
}

#[derive(Component, Default, Reflect)]
pub struct Path {
    points: Vec<PathPoint>,
    looped: bool,
}

#[derive(Component)]
pub struct PathAgent {
    pub path: Entity,
    pub t: f32,
    pub point_idx: usize,
}

/// Implementation of centripetal Catmull–Rom spline.
/// See: https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline#Definition
fn catmull_rom_spline(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, alpha: f32, t: f32) -> Vec2 {
    let t0 = 0.0;
    let t1 = (p1 - p0).length_squared().powf(alpha * 0.5) + t0;
    let t2 = (p2 - p1).length_squared().powf(alpha * 0.5) + t1;
    let t3 = (p3 - p2).length_squared().powf(alpha * 0.5) + t2;

    let t = t1 * t + (1.0 - t) * t2;

    let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
    let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
    let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;

    let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
    let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;

    (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2
}

fn draw_path(paths: Query<&Path>, mut gizmos: Gizmos, inp: Res<GameInput>) {
    let path = paths.single();
    let points_num = path.points.len();

    for point in &path.points {
        gizmos.circle_2d(point.pos, 5.0, Color::RED);
    }

    let mut centers = Vec::new();
    for i in 0..points_num - (!path.looped as usize) {
        let p1 = &path.points[i];
        let p2 = &path.points[(i + 1) % points_num];
        if let PathType::CatmullRom = &p1.path_type {
            let p0 = &path.points[i.checked_add_signed(-1).unwrap_or(points_num - 1)];
            let p3 = &path.points[(i + 2) % points_num];
            let seg_points = (1..100)
                .map(|t| catmull_rom_spline(p0.pos, p1.pos, p2.pos, p3.pos, 0.5, t as f32 / 100.0))
                .collect::<Vec<_>>();
            for (p1, p2) in seg_points[..seg_points.len() - 1]
                .iter()
                .zip(&seg_points[1..])
            {
                gizmos.line_2d(*p1, *p2, Color::GREEN)
            }
            let center = seg_points.iter().cloned().reduce(|acc, v| acc + v).unwrap()
                / seg_points.len() as f32;
            // let center = catmull_rom_spline(p0.pos, p1.pos, p2.pos, p3.pos, 0.5, 0.5);
            centers.push(center);
            centers.push(p1.pos);
            centers.push(p2.pos);
        } else {
            gizmos.line_2d(p1.pos, p2.pos, Color::WHITE);
            centers.push((p1.pos + p2.pos) / 2.0)
        }
    }
    let mut closest = Vec2::ZERO;
    let mut closest_length = f32::INFINITY;
    for c in centers.iter() {
        let len = (inp.cursor_position - *c).length();
        if len < closest_length {
            closest = *c;
            closest_length = len;
        }
    }
    gizmos.line_2d(closest, inp.cursor_position, Color::BLUE);
}
