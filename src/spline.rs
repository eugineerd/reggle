use bevy::prelude::*;

const SEGMENTS_MAX_ITER_NUM: usize = 10;
const SEGMENTS_ANGLE_TOL: f32 = 0.4;
const SEGMENTS_MAX_STEP: f32 = 0.1;

#[derive(Reflect, Clone, Copy)]
pub enum SegmentType {
    None,
    Linear,
    CatmullRom(f32),
}

impl Default for SegmentType {
    fn default() -> Self {
        SegmentType::CatmullRom(0.5)
    }
}

#[derive(Default, Reflect, Clone)]
pub struct Segment {
    pub typ: SegmentType,
    len: f32,
    #[reflect(ignore)]
    points: Vec<Vec2>,
    #[reflect(ignore)]
    distnaces: Vec<f32>,
}

impl Segment {
    pub fn new(typ: SegmentType) -> Self {
        Self {
            typ,
            ..Default::default()
        }
    }
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }
    pub fn len(&self) -> f32 {
        self.len
    }
    pub fn get_pos(&self, mut t: f32, neighbors: &[Vec2; 4]) -> Vec2 {
        t = t.clamp(0.0, 1.0);
        match self.typ {
            SegmentType::None => neighbors[1],
            SegmentType::Linear => {
                let p2 = neighbors[2];
                neighbors[1] + (p2 - neighbors[1]) * t
            }
            SegmentType::CatmullRom(alpha) => catmull_rom_spline(
                neighbors[0],
                neighbors[1],
                neighbors[2],
                neighbors[3],
                alpha,
                t,
            ),
        }
    }

    pub fn get_pos_cached(&self, mut t: f32) -> Option<Vec2> {
        t = t.clamp(0.0, 1.0);
        let points = self.points();
        let distances = &self.distnaces;
        if points.is_empty() {
            return None;
        }
        if points.len() == 2 {
            return Some(points[0] + (points[1] - points[0]) * t);
        }

        let target_distnace = self.len() * t;
        let mut start = 0;
        let mut end = distances.len();
        let mut idx = (start + end) / 2;
        while start < idx {
            if target_distnace <= distances[idx] {
                end = idx;
            } else {
                start = idx;
            }
            idx = (start + end) / 2;
        }
        if idx == points.len() - 1 {
            return Some(points[idx]);
        }

        let line_vec = points[idx + 1] - points[idx];
        let line_length = line_vec.length();
        let frac = if line_length < f32::EPSILON {
            0.5
        } else {
            (target_distnace - distances[idx]) / line_length
        };
        Some(points[idx] + line_vec * frac)
    }

    pub fn get_pos_and_vel(&self, mut t: f32, neighbors: &[Vec2; 4]) -> (Vec2, Vec2) {
        t = t.clamp(0.0, 1.0);
        match self.typ {
            SegmentType::None => (neighbors[1], Vec2::ZERO),
            SegmentType::Linear => {
                let p1 = neighbors[1];
                let p2 = neighbors[2];
                (p1 + (p2 - p1) * t, (p2 - p1).normalize_or_zero())
            }
            SegmentType::CatmullRom(alpha) => catmull_rom_spline_with_derivative(
                neighbors[0],
                neighbors[1],
                neighbors[2],
                neighbors[3],
                alpha,
                t,
            ),
        }
    }

    pub fn tessellate(&mut self, neighbors: Option<&[Vec2; 4]>) {
        self.points.clear();
        self.distnaces.clear();
        self.len = 0.0;
        let Some(neighbors) = neighbors else {return};
        if let SegmentType::Linear = self.typ {
            self.points.push(neighbors[1]);
            self.points.push(neighbors[2]);
            self.len = (neighbors[2] - neighbors[1]).length();
            self.distnaces.push(self.len);
            return;
        }
        let mut t_last = 0.0f32;
        let (mut x_last, mut dt_last) = self.get_pos_and_vel(t_last, neighbors);
        self.points.push(x_last);
        while t_last < 1.0 {
            let mut t = (t_last + SEGMENTS_MAX_STEP).min(1.0);
            let (mut x, mut dt) = self.get_pos_and_vel(t, neighbors);
            let mut iter_num = 0;
            while dt_last.angle_between(dt).abs() > SEGMENTS_ANGLE_TOL
                && iter_num < SEGMENTS_MAX_ITER_NUM
            {
                iter_num += 1;
                t = (t_last + t) / 2.0;
                (x, dt) = self.get_pos_and_vel(t, neighbors);
            }
            self.distnaces.push(self.len);
            self.len += (x - x_last).length();
            self.points.push(x);
            (t_last, x_last, dt_last) = (t, x, dt);
        }
    }
}

/// Implementation of centripetal Catmull–Rom spline.
/// See: https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline#Definition
fn catmull_rom_spline(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, mut alpha: f32, mut t: f32) -> Vec2 {
    t = t.clamp(0.0, 1.0);
    alpha = alpha.clamp(0.0, 1.0);

    let t0 = 0.0;
    let t1 = (p1 - p0).length_squared().powf(alpha * 0.5) + t0;
    let t2 = (p2 - p1).length_squared().powf(alpha * 0.5) + t1;
    let t3 = (p3 - p2).length_squared().powf(alpha * 0.5) + t2;

    t = t1 + t * (t2 - t1);

    let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
    let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
    let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;

    let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
    let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;

    (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2
}

fn catmull_rom_spline_with_derivative(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    mut alpha: f32,
    mut t: f32,
) -> (Vec2, Vec2) {
    t = t.clamp(0.0, 1.0);
    alpha = alpha.clamp(0.0, 1.0);

    let t0 = 0.0;
    let t1 = (p1 - p0).length_squared().powf(alpha * 0.5) + t0;
    let t2 = (p2 - p1).length_squared().powf(alpha * 0.5) + t1;
    let t3 = (p3 - p2).length_squared().powf(alpha * 0.5) + t2;

    t = t1 + t * (t2 - t1);

    let a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1;
    let a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2;
    let a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3;

    let b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2;
    let b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3;

    let c = (t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2;

    let da1 = 1.0 / (t1 - t0) * (p1 - p0);
    let da2 = 1.0 / (t2 - t1) * (p2 - p1);
    let da3 = 1.0 / (t3 - t2) * (p3 - p2);

    let db1 = 1.0 / (t2 - t0) * (a2 - a1) + (t2 - t) / (t2 - t0) * da1 + (t - t0) / (t2 - t0) * da2;
    let db2 = 1.0 / (t3 - t1) * (a3 - a2) + (t3 - t) / (t3 - t1) * da2 + (t - t1) / (t3 - t1) * da3;

    let dc = 1.0 / (t2 - t1) * (b2 - b1) + (t2 - t) / (t2 - t1) * db1 + (t - t1) / (t2 - t1) * db2;

    (c, dc)
}
