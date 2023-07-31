use bevy::prelude::*;

use crate::{common::GameState, input::GameInput};

// TODO:
// - [ ] Efficient distance from point to spline
// - [-] Use bevy::math splines
// - [x] Efficeint draw splines
// - [ ] Add movement easing using bevy::math spline

const SEGMENTS_MAX_ITER_NUM: usize = 10;
const SEGMENTS_ANGLE_TOL: f32 = 0.4;
const SEGMENTS_MAX_STEP: f32 = 0.1;

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (cache_path_segments, draw_path).run_if(in_state(GameState::InGame)),
        )
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
                    pos: Vec2::new(-100.0, 400.0),
                    ..Default::default()
                },
                PathPoint {
                    pos: Vec2::new(100.0, -200.0),
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

#[derive(Reflect)]
pub enum PathType {
    Linear,
    CatmullRom(f32),
}

impl Default for PathType {
    fn default() -> Self {
        return PathType::CatmullRom(0.5);
    }
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

#[derive(Component, Default)]
struct PathSegmentCache {
    segments: Vec<Vec<Vec2>>,
}

#[derive(Component)]
pub struct PathAgent {
    pub path: Entity,
    pub t: f32,
    pub point_idx: usize,
}

/// Implementation of centripetal Catmull–Rom spline.
/// See: https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline#Definition
fn catmull_rom_spline(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    mut alpha: f32,
    mut t: f32,
    compute_derivative: bool,
) -> (Vec2, Vec2) {
    t = t.clamp(0.0, 1.0);
    alpha = alpha.clamp(0.0, 1.0);

    let t0 = 0.0;
    let t1 = (p1 - p0).length_squared().powf(alpha * 0.5) + t0;
    let t2 = (p2 - p1).length_squared().powf(alpha * 0.5) + t1;
    let t3 = (p3 - p2).length_squared().powf(alpha * 0.5) + t2;

    t = t1 * t + (1.0 - t) * t2;

    let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
    let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
    let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;

    let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
    let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;

    let c = (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2;

    let mut dc = Vec2::ZERO;
    if compute_derivative {
        let da1 = 1.0 / (t1 - t0) * (p1 - p0);
        let da2 = 1.0 / (t2 - t1) * (p2 - p1);
        let da3 = 1.0 / (t3 - t2) * (p3 - p2);

        let db1 =
            1.0 / (t2 - t0) * (a2 - a1) + (t2 - t) / (t2 - t0) * da1 + (t - t0) / (t2 - t0) * da2;
        let db2 =
            1.0 / (t3 - t1) * (a3 - a2) + (t3 - t) / (t3 - t1) * da2 + (t - t1) / (t3 - t1) * da3;

        dc = 1.0 / (t2 - t1) * (b2 - b1) + (t2 - t) / (t2 - t1) * db1 + (t - t1) / (t2 - t1) * db2
    }

    (c, dc)
}

fn draw_path(paths: Query<(&Path, &PathSegmentCache)>, mut gizmos: Gizmos, inp: Res<GameInput>) {
    let Ok((path, seg_cache)) = paths.get_single() else {return};

    for point in &path.points {
        gizmos.circle_2d(point.pos, 5.0, Color::RED);
    }

    for seg in &seg_cache.segments {
        for (p1, p2) in seg[..seg.len()].iter().zip(&seg[1..]) {
            gizmos.line_2d(*p1, *p2, Color::WHITE);
        }
    }
}

fn cache_path_segments(
    mut commands: Commands,
    paths: Query<(Entity, &Path), Or<(Added<Path>, Changed<Path>)>>,
) {
    for (e, path) in paths.iter() {
        let points_num = path.points.len();
        let mut cache = PathSegmentCache::default();

        for i in 0..points_num - (!path.looped as usize) {
            let p1 = &path.points[i];
            let p2 = &path.points[(i + 1) % points_num];
            let mut segment = Vec::new();
            match p1.path_type {
                PathType::CatmullRom(alpha) => {
                    let p0 = &path.points[i.checked_add_signed(-1).unwrap_or(points_num - 1)];
                    let p3 = &path.points[(i + 2) % points_num];
                    let segment_spline =
                        |t| catmull_rom_spline(p0.pos, p1.pos, p2.pos, p3.pos, alpha, t, true);
                    let mut t_last = 0.0f32;
                    let (mut x_last, mut dt_last) = segment_spline(t_last);
                    segment.push(x_last);
                    while t_last < 1.0 {
                        let mut t = (t_last + SEGMENTS_MAX_STEP).min(1.0);
                        let (mut x, mut dt) = segment_spline(t);
                        let mut iter_num = 0;
                        while dt_last.angle_between(dt).abs() > SEGMENTS_ANGLE_TOL
                            && iter_num < SEGMENTS_MAX_ITER_NUM
                        {
                            iter_num += 1;
                            t = (t_last + t) / 2.0;
                            (x, dt) = segment_spline(t);
                        }
                        (t_last, x_last, dt_last) = (t, x, dt);
                        segment.push(x_last);
                    }
                }
                PathType::Linear => {
                    segment.push(p1.pos);
                    segment.push(p2.pos);
                }
            }
            cache.segments.push(segment);
        }
        if let Some(mut c) = commands.get_entity(e) {
            c.insert(cache);
        }
    }
}
