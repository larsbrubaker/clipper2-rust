//! Path Offset (Inflate/Shrink) module
//!
//! Direct port from clipper.offset.h and clipper.offset.cpp
//! Copyright (c) Angus Johnson 2010-2025
//!
//! This module provides path offsetting (inflating/deflating) operations.
//! It supports different join types (Square, Bevel, Round, Miter) and
//! end types (Polygon, Joined, Butt, Square, Round).

use crate::core::*;
use crate::engine::*;
use crate::engine_public::*;

use std::f64::consts::PI;

// ============================================================================
// Constants
// ============================================================================

const FLOATING_POINT_TOLERANCE: f64 = 1e-12;

/// Arc approximation constant (1/500).
/// When arc_tolerance is undefined (0), curve imprecision will be relative
/// to the size of the offset (delta).
/// See https://www.angusj.com/clipper2/Docs/Trigonometry.htm
const ARC_CONST: f64 = 0.002;

// ============================================================================
// Enums
// ============================================================================

/// Join type for path offset operations
/// Direct port from clipper.offset.h line 19
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinType {
    /// Joins are 'squared' at exactly the offset distance (more complex code)
    Square,
    /// Similar to Square, but the offset distance varies with angle (simple code & faster)
    Bevel,
    /// Round joins using arc approximation
    Round,
    /// Miter joins with configurable limit
    Miter,
}

/// End type for path offset operations
/// Direct port from clipper.offset.h line 23
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndType {
    /// Offsets only one side of a closed path
    Polygon,
    /// Offsets both sides of a path, with joined ends
    Joined,
    /// Offsets both sides of a path, with square blunt ends
    Butt,
    /// Offsets both sides of a path, with square extended ends
    Square,
    /// Offsets both sides of a path, with round extended ends
    Round,
}

/// Callback function type for variable delta offset per vertex
/// Direct port from clipper.offset.h line 30
pub type DeltaCallback64 = Box<dyn Fn(&Path64, &PathD, usize, usize) -> f64>;

// ============================================================================
// Helper functions (module-level)
// ============================================================================

/// Find the lowest (largest Y) point among closed paths, and determine
/// if the path containing that point has negative area.
/// Direct port from clipper.offset.cpp line 36
pub fn get_lowest_closed_path_info(
    paths: &Paths64,
    idx: &mut Option<usize>,
    is_neg_area: &mut bool,
) {
    *idx = None;
    let mut bot_pt = Point64::new(i64::MAX, i64::MIN);
    for i in 0..paths.len() {
        let mut a: f64 = f64::MAX;
        for pt in &paths[i] {
            if (pt.y < bot_pt.y) || ((pt.y == bot_pt.y) && (pt.x >= bot_pt.x)) {
                continue;
            }
            if a == f64::MAX {
                a = area(&paths[i]);
                if a == 0.0 {
                    break; // invalid closed path
                }
                *is_neg_area = a < 0.0;
            }
            *idx = Some(i);
            bot_pt.x = pt.x;
            bot_pt.y = pt.y;
        }
    }
}

/// Hypotenuse calculation
/// Direct port from clipper.offset.cpp line 60
#[inline]
fn hypot(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

/// Get unit normal perpendicular to the direction from pt1 to pt2
/// Direct port from clipper.offset.cpp line 70
#[inline]
fn get_unit_normal(pt1: &Point64, pt2: &Point64) -> PointD {
    if pt1 == pt2 {
        return PointD::new(0.0, 0.0);
    }
    let dx = (pt2.x - pt1.x) as f64;
    let dy = (pt2.y - pt1.y) as f64;
    let inverse_hypot = 1.0 / hypot(dx, dy);
    PointD::new(dy * inverse_hypot, -dx * inverse_hypot)
}

/// Check if a value is approximately zero
/// Direct port from clipper.offset.cpp line 81
#[inline]
fn almost_zero(value: f64, epsilon: f64) -> bool {
    value.abs() < epsilon
}

/// Normalize a vector to unit length
/// Direct port from clipper.offset.cpp line 86
#[inline]
fn normalize_vector(vec: &PointD) -> PointD {
    let h = hypot(vec.x, vec.y);
    if almost_zero(h, 0.001) {
        return PointD::new(0.0, 0.0);
    }
    let inverse_hypot = 1.0 / h;
    PointD::new(vec.x * inverse_hypot, vec.y * inverse_hypot)
}

/// Get the average of two unit vectors, normalized
/// Direct port from clipper.offset.cpp line 94
#[inline]
fn get_avg_unit_vector(vec1: &PointD, vec2: &PointD) -> PointD {
    normalize_vector(&PointD::new(vec1.x + vec2.x, vec1.y + vec2.y))
}

/// Check if the end type represents a closed path
/// Direct port from clipper.offset.cpp line 99
#[inline]
fn is_closed_path(et: EndType) -> bool {
    et == EndType::Polygon || et == EndType::Joined
}

/// Get perpendicular point (integer result)
/// Direct port from clipper.offset.cpp line 104
#[inline]
fn get_perpendic(pt: &Point64, norm: &PointD, delta: f64) -> Point64 {
    Point64::new(
        (pt.x as f64 + norm.x * delta).round() as i64,
        (pt.y as f64 + norm.y * delta).round() as i64,
    )
}

/// Get perpendicular point (floating-point result)
/// Direct port from clipper.offset.cpp line 113
#[inline]
fn get_perpendic_d(pt: &Point64, norm: &PointD, delta: f64) -> PointD {
    PointD::new(pt.x as f64 + norm.x * delta, pt.y as f64 + norm.y * delta)
}

/// Negate all points in a PathD
/// Direct port from clipper.offset.cpp line 122
#[inline]
fn negate_path(path: &mut PathD) {
    for pt in path.iter_mut() {
        pt.x = -pt.x;
        pt.y = -pt.y;
    }
}

// ============================================================================
// Group struct
// ============================================================================

/// A group of paths sharing the same join and end types
/// Direct port from clipper.offset.h line 35 (ClipperOffset::Group)
pub struct Group {
    pub paths_in: Paths64,
    pub lowest_path_idx: Option<usize>,
    pub is_reversed: bool,
    pub join_type: JoinType,
    pub end_type: EndType,
}

impl Group {
    /// Create a new Group from paths with given join/end types
    /// Direct port from clipper.offset.cpp line 139
    pub fn new(paths: &Paths64, join_type: JoinType, end_type: EndType) -> Self {
        let mut paths_in = paths.clone();
        let is_joined = end_type == EndType::Polygon || end_type == EndType::Joined;

        for p in paths_in.iter_mut() {
            strip_duplicates_path(p, is_joined);
        }

        let mut lowest_path_idx = None;
        let mut is_reversed = false;

        if end_type == EndType::Polygon {
            let mut is_neg_area = false;
            get_lowest_closed_path_info(&paths_in, &mut lowest_path_idx, &mut is_neg_area);
            // the lowermost path must be an outer path, so if its orientation is negative,
            // then flag the whole group as 'reversed' (will negate delta etc.)
            // as this is much more efficient than reversing every path.
            is_reversed = lowest_path_idx.is_some() && is_neg_area;
        }

        Self {
            paths_in,
            lowest_path_idx,
            is_reversed,
            join_type,
            end_type,
        }
    }
}

// ============================================================================
// ClipperOffset struct
// ============================================================================

/// Path offset (inflate/shrink) engine
/// Direct port from clipper.offset.h line 32
pub struct ClipperOffset {
    // Private state
    error_code_: i32,
    delta_: f64,
    group_delta_: f64,
    temp_lim_: f64,
    steps_per_rad_: f64,
    step_sin_: f64,
    step_cos_: f64,
    norms: PathD,
    path_out: Path64,
    solution: Paths64,
    solution_tree: Option<PolyTree64>,
    groups_: Vec<Group>,
    join_type_: JoinType,
    end_type_: EndType,

    // Public configuration
    miter_limit_: f64,
    arc_tolerance_: f64,
    preserve_collinear_: bool,
    reverse_solution_: bool,

    delta_callback64_: Option<DeltaCallback64>,
}

impl ClipperOffset {
    /// Create a new ClipperOffset with given parameters
    /// Direct port from clipper.offset.h line 85
    pub fn new(
        miter_limit: f64,
        arc_tolerance: f64,
        preserve_collinear: bool,
        reverse_solution: bool,
    ) -> Self {
        Self {
            error_code_: 0,
            delta_: 0.0,
            group_delta_: 0.0,
            temp_lim_: 0.0,
            steps_per_rad_: 0.0,
            step_sin_: 0.0,
            step_cos_: 0.0,
            norms: PathD::new(),
            path_out: Path64::new(),
            solution: Paths64::new(),
            solution_tree: None,
            groups_: Vec::new(),
            join_type_: JoinType::Bevel,
            end_type_: EndType::Polygon,
            miter_limit_: miter_limit,
            arc_tolerance_: arc_tolerance,
            preserve_collinear_: preserve_collinear,
            reverse_solution_: reverse_solution,
            delta_callback64_: None,
        }
    }

    /// Get the error code
    /// Direct port from clipper.offset.h line 95
    pub fn error_code(&self) -> i32 {
        self.error_code_
    }

    /// Add a single path with join and end types
    /// Direct port from clipper.offset.cpp line 168
    pub fn add_path(&mut self, path: &Path64, jt: JoinType, et: EndType) {
        let paths = vec![path.clone()];
        self.groups_.push(Group::new(&paths, jt, et));
    }

    /// Add multiple paths with join and end types
    /// Direct port from clipper.offset.cpp line 173
    pub fn add_paths(&mut self, paths: &Paths64, jt: JoinType, et: EndType) {
        if paths.is_empty() {
            return;
        }
        self.groups_.push(Group::new(paths, jt, et));
    }

    /// Clear all groups and normals
    /// Direct port from clipper.offset.h line 98
    pub fn clear(&mut self) {
        self.groups_.clear();
        self.norms.clear();
    }

    /// Execute offset operation, returning paths
    /// Direct port from clipper.offset.cpp line 636
    pub fn execute(&mut self, delta: f64, paths64: &mut Paths64) {
        paths64.clear();
        self.solution = Paths64::new();
        self.solution_tree = None;
        self.execute_internal(delta);
        *paths64 = std::mem::take(&mut self.solution);
    }

    /// Execute offset operation, returning polytree
    /// Direct port from clipper.offset.cpp line 645
    pub fn execute_tree(&mut self, delta: f64, polytree: &mut PolyTree64) {
        polytree.clear();
        self.solution_tree = Some(PolyTree64::new());
        self.solution = Paths64::new();
        self.execute_internal(delta);
        *polytree = self.solution_tree.take().unwrap_or_else(PolyTree64::new);
    }

    /// Execute offset operation with delta callback
    /// Direct port from clipper.offset.cpp line 655
    pub fn execute_with_callback(&mut self, delta_cb: DeltaCallback64, paths: &mut Paths64) {
        self.delta_callback64_ = Some(delta_cb);
        self.execute(1.0, paths);
    }

    // Property accessors matching C++ interface

    /// Get miter limit
    pub fn miter_limit(&self) -> f64 {
        self.miter_limit_
    }

    /// Set miter limit
    pub fn set_miter_limit(&mut self, miter_limit: f64) {
        self.miter_limit_ = miter_limit;
    }

    /// Get arc tolerance
    pub fn arc_tolerance(&self) -> f64 {
        self.arc_tolerance_
    }

    /// Set arc tolerance
    pub fn set_arc_tolerance(&mut self, arc_tolerance: f64) {
        self.arc_tolerance_ = arc_tolerance;
    }

    /// Get preserve collinear setting
    pub fn preserve_collinear(&self) -> bool {
        self.preserve_collinear_
    }

    /// Set preserve collinear setting
    pub fn set_preserve_collinear(&mut self, preserve_collinear: bool) {
        self.preserve_collinear_ = preserve_collinear;
    }

    /// Get reverse solution setting
    pub fn reverse_solution(&self) -> bool {
        self.reverse_solution_
    }

    /// Set reverse solution setting
    pub fn set_reverse_solution(&mut self, reverse_solution: bool) {
        self.reverse_solution_ = reverse_solution;
    }

    /// Set the delta callback
    pub fn set_delta_callback(&mut self, cb: DeltaCallback64) {
        self.delta_callback64_ = Some(cb);
    }

    // ========================================================================
    // Private methods
    // ========================================================================

    /// Build the normal vectors for each edge of a path
    /// Direct port from clipper.offset.cpp line 179
    fn build_normals(&mut self, path: &Path64) {
        self.norms.clear();
        if path.is_empty() {
            return;
        }
        self.norms.reserve(path.len());
        for i in 0..path.len() - 1 {
            self.norms.push(get_unit_normal(&path[i], &path[i + 1]));
        }
        self.norms
            .push(get_unit_normal(path.last().unwrap(), &path[0]));
    }

    /// Bevel join: simple straight cut between two offset edges
    /// Direct port from clipper.offset.cpp line 190
    fn do_bevel(&mut self, path: &Path64, j: usize, k: usize) {
        let pt1: PointD;
        let pt2: PointD;
        if j == k {
            let abs_delta = self.group_delta_.abs();
            pt1 = PointD::new(
                path[j].x as f64 - abs_delta * self.norms[j].x,
                path[j].y as f64 - abs_delta * self.norms[j].y,
            );
            pt2 = PointD::new(
                path[j].x as f64 + abs_delta * self.norms[j].x,
                path[j].y as f64 + abs_delta * self.norms[j].y,
            );
        } else {
            pt1 = PointD::new(
                path[j].x as f64 + self.group_delta_ * self.norms[k].x,
                path[j].y as f64 + self.group_delta_ * self.norms[k].y,
            );
            pt2 = PointD::new(
                path[j].x as f64 + self.group_delta_ * self.norms[j].x,
                path[j].y as f64 + self.group_delta_ * self.norms[j].y,
            );
        }
        self.path_out
            .push(Point64::new(pt1.x.round() as i64, pt1.y.round() as i64));
        self.path_out
            .push(Point64::new(pt2.x.round() as i64, pt2.y.round() as i64));
    }

    /// Square join: offset with perpendicular extensions
    /// Direct port from clipper.offset.cpp line 218
    fn do_square(&mut self, path: &Path64, j: usize, k: usize) {
        let vec: PointD;
        if j == k {
            vec = PointD::new(self.norms[j].y, -self.norms[j].x);
        } else {
            vec = get_avg_unit_vector(
                &PointD::new(-self.norms[k].y, self.norms[k].x),
                &PointD::new(self.norms[j].y, -self.norms[j].x),
            );
        }

        let abs_delta = self.group_delta_.abs();

        // offset the original vertex delta units along unit vector
        let pt_q = PointD::new(path[j].x as f64, path[j].y as f64);
        let pt_q = translate_point(&pt_q, abs_delta * vec.x, abs_delta * vec.y);
        // get perpendicular vertices
        let pt1 = translate_point(
            &pt_q,
            self.group_delta_ * vec.y,
            self.group_delta_ * -vec.x,
        );
        let pt2 = translate_point(
            &pt_q,
            self.group_delta_ * -vec.y,
            self.group_delta_ * vec.x,
        );
        // get 2 vertices along one edge offset
        let pt3 = get_perpendic_d(&path[k], &self.norms[k], self.group_delta_);
        if j == k {
            let pt4 = PointD::new(
                pt3.x + vec.x * self.group_delta_,
                pt3.y + vec.y * self.group_delta_,
            );
            let mut pt = pt_q;
            get_segment_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            // get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(Point64::new(
                reflected.x.round() as i64,
                reflected.y.round() as i64,
            ));
            self.path_out
                .push(Point64::new(pt.x.round() as i64, pt.y.round() as i64));
        } else {
            let pt4 = get_perpendic_d(&path[j], &self.norms[k], self.group_delta_);
            let mut pt = pt_q;
            get_segment_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            self.path_out
                .push(Point64::new(pt.x.round() as i64, pt.y.round() as i64));
            // get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(Point64::new(
                reflected.x.round() as i64,
                reflected.y.round() as i64,
            ));
        }
    }

    /// Miter join: sharp corner extension
    /// Direct port from clipper.offset.cpp line 258
    fn do_miter(&mut self, path: &Path64, j: usize, k: usize, cos_a: f64) {
        let q = self.group_delta_ / (cos_a + 1.0);
        self.path_out.push(Point64::new(
            (path[j].x as f64 + (self.norms[k].x + self.norms[j].x) * q).round() as i64,
            (path[j].y as f64 + (self.norms[k].y + self.norms[j].y) * q).round() as i64,
        ));
    }

    /// Round join: arc approximation between two offset edges
    /// Direct port from clipper.offset.cpp line 273
    fn do_round(&mut self, path: &Path64, j: usize, k: usize, angle: f64) {
        if self.delta_callback64_.is_some() {
            // when deltaCallback64_ is assigned, group_delta_ won't be constant,
            // so we'll need to do the following calculations for *every* vertex.
            let abs_delta = self.group_delta_.abs();
            let arc_tol = if self.arc_tolerance_ > FLOATING_POINT_TOLERANCE {
                self.arc_tolerance_.min(abs_delta)
            } else {
                abs_delta * ARC_CONST
            };
            let steps_per_360 =
                (PI / (1.0 - arc_tol / abs_delta).acos()).min(abs_delta * PI);
            self.step_sin_ = (2.0 * PI / steps_per_360).sin();
            self.step_cos_ = (2.0 * PI / steps_per_360).cos();
            if self.group_delta_ < 0.0 {
                self.step_sin_ = -self.step_sin_;
            }
            self.steps_per_rad_ = steps_per_360 / (2.0 * PI);
        }

        let pt = path[j];
        let mut offset_vec = PointD::new(
            self.norms[k].x * self.group_delta_,
            self.norms[k].y * self.group_delta_,
        );

        if j == k {
            offset_vec = offset_vec.negate();
        }

        self.path_out.push(Point64::new(
            (pt.x as f64 + offset_vec.x).round() as i64,
            (pt.y as f64 + offset_vec.y).round() as i64,
        ));

        let steps = (self.steps_per_rad_ * angle.abs()).ceil() as i32; // #448, #456
        for _ in 1..steps {
            offset_vec = PointD::new(
                offset_vec.x * self.step_cos_ - self.step_sin_ * offset_vec.y,
                offset_vec.x * self.step_sin_ + offset_vec.y * self.step_cos_,
            );
            self.path_out.push(Point64::new(
                (pt.x as f64 + offset_vec.x).round() as i64,
                (pt.y as f64 + offset_vec.y).round() as i64,
            ));
        }
        self.path_out
            .push(get_perpendic(&path[j], &self.norms[j], self.group_delta_));
    }

    /// Offset a single point (vertex) of a path
    /// Direct port from clipper.offset.cpp line 311
    fn offset_point(&mut self, group: &Group, path: &Path64, j: usize, k: usize) {
        // Let A = change in angle where edges join
        // A == 0: ie no change in angle (flat join)
        // A == PI: edges 'spike'
        // sin(A) < 0: right turning
        // cos(A) < 0: change in angle is more than 90 degree

        if path[j] == path[k] {
            return;
        }

        let mut sin_a = cross_product_two_vectors(self.norms[j], self.norms[k]);
        let cos_a = dot_product_two_vectors(self.norms[j], self.norms[k]);
        if sin_a > 1.0 {
            sin_a = 1.0;
        } else if sin_a < -1.0 {
            sin_a = -1.0;
        }

        if let Some(ref cb) = self.delta_callback64_ {
            self.group_delta_ = cb(path, &self.norms, j, k);
            if group.is_reversed {
                self.group_delta_ = -self.group_delta_;
            }
        }
        if self.group_delta_.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[j]);
            return;
        }

        if cos_a > -0.999 && (sin_a * self.group_delta_ < 0.0) {
            // is concave (#593)
            // by far the simplest way to construct concave joins, especially those joining very
            // short segments, is to insert 3 points that produce negative regions. These regions
            // will be removed later by the finishing union operation. This is also the best way
            // to ensure that path reversals (ie over-shrunk paths) are removed.
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[k], self.group_delta_));
            self.path_out.push(path[j]); // (#405, #873, #916)
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[j], self.group_delta_));
        } else if cos_a > 0.999 && self.join_type_ != JoinType::Round {
            // almost straight - less than 2.5 degree (#424, #482, #526 & #724)
            self.do_miter(path, j, k, cos_a);
        } else if self.join_type_ == JoinType::Miter {
            // miter unless the angle is sufficiently acute to exceed ML
            if cos_a > self.temp_lim_ - 1.0 {
                self.do_miter(path, j, k, cos_a);
            } else {
                self.do_square(path, j, k);
            }
        } else if self.join_type_ == JoinType::Round {
            self.do_round(path, j, k, sin_a.atan2(cos_a));
        } else if self.join_type_ == JoinType::Bevel {
            self.do_bevel(path, j, k);
        } else {
            self.do_square(path, j, k);
        }
    }

    /// Offset a closed polygon path
    /// Direct port from clipper.offset.cpp line 372
    fn offset_polygon(&mut self, group: &Group, path: &Path64) {
        self.path_out.clear();
        let len = path.len();
        let mut k = len - 1;
        for j in 0..len {
            self.offset_point(group, path, j, k);
            k = j;
        }
        self.solution.push(self.path_out.clone());
    }

    /// Offset an open path with joined ends (offset both sides)
    /// Direct port from clipper.offset.cpp line 380
    fn offset_open_joined(&mut self, group: &Group, path: &Path64) {
        self.offset_polygon(group, path);
        let mut reverse_path = path.clone();
        reverse_path.reverse();

        // rebuild normals
        self.norms.reverse();
        self.norms.push(self.norms[0]);
        self.norms.remove(0);
        negate_path(&mut self.norms);

        self.offset_polygon(group, &reverse_path);
    }

    /// Offset an open path (offset both sides with end caps)
    /// Direct port from clipper.offset.cpp line 395
    fn offset_open_path(&mut self, group: &Group, path: &Path64) {
        // do the line start cap
        if let Some(ref cb) = self.delta_callback64_ {
            self.group_delta_ = cb(path, &self.norms, 0, 0);
        }

        if self.group_delta_.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[0]);
        } else {
            match self.end_type_ {
                EndType::Butt => self.do_bevel(path, 0, 0),
                EndType::Round => self.do_round(path, 0, 0, PI),
                _ => self.do_square(path, 0, 0),
            }
        }

        let high_i = path.len() - 1;
        // offset the left side going forward
        let mut k = 0;
        for j in 1..high_i {
            self.offset_point(group, path, j, k);
            k = j;
        }

        // reverse normals
        for i in (1..=high_i).rev() {
            self.norms[i] = PointD::new(-self.norms[i - 1].x, -self.norms[i - 1].y);
        }
        self.norms[0] = self.norms[high_i];

        // do the line end cap
        if let Some(ref cb) = self.delta_callback64_ {
            self.group_delta_ = cb(path, &self.norms, high_i, high_i);
        }

        if self.group_delta_.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[high_i]);
        } else {
            match self.end_type_ {
                EndType::Butt => self.do_bevel(path, high_i, high_i),
                EndType::Round => self.do_round(path, high_i, high_i, PI),
                _ => self.do_square(path, high_i, high_i),
            }
        }

        let mut j = high_i - 1;
        let mut k = high_i;
        while j > 0 {
            self.offset_point(group, path, j, k);
            k = j;
            j -= 1;
        }
        self.solution.push(self.path_out.clone());
    }

    /// Process offset for a single group
    /// Direct port from clipper.offset.cpp line 455
    fn do_group_offset(&mut self, group_idx: usize) {
        let end_type = self.groups_[group_idx].end_type;
        let join_type = self.groups_[group_idx].join_type;
        let is_reversed = self.groups_[group_idx].is_reversed;
        let has_lowest = self.groups_[group_idx].lowest_path_idx.is_some();

        if end_type == EndType::Polygon {
            // a straight path (2 points) can now also be 'polygon' offset
            // where the ends will be treated as (180 deg.) joins
            if !has_lowest {
                self.delta_ = self.delta_.abs();
            }
            self.group_delta_ = if is_reversed {
                -self.delta_
            } else {
                self.delta_
            };
        } else {
            self.group_delta_ = self.delta_.abs();
        }

        let abs_delta = self.group_delta_.abs();
        self.join_type_ = join_type;
        self.end_type_ = end_type;

        if join_type == JoinType::Round || end_type == EndType::Round {
            // calculate the number of steps required to approximate a circle
            let arc_tol = if self.arc_tolerance_ > FLOATING_POINT_TOLERANCE {
                self.arc_tolerance_.min(abs_delta)
            } else {
                abs_delta * ARC_CONST
            };

            let steps_per_360 =
                (PI / (1.0 - arc_tol / abs_delta).acos()).min(abs_delta * PI);
            self.step_sin_ = (2.0 * PI / steps_per_360).sin();
            self.step_cos_ = (2.0 * PI / steps_per_360).cos();
            if self.group_delta_ < 0.0 {
                self.step_sin_ = -self.step_sin_;
            }
            self.steps_per_rad_ = steps_per_360 / (2.0 * PI);
        }

        let path_count = self.groups_[group_idx].paths_in.len();
        for path_idx in 0..path_count {
            let path = self.groups_[group_idx].paths_in[path_idx].clone();
            let path_len = path.len();
            self.path_out.clear();

            if path_len == 1 {
                // single point
                if self.delta_callback64_.is_some() {
                    let cb_result = {
                        let cb = self.delta_callback64_.as_ref().unwrap();
                        cb(&path, &self.norms, 0, 0)
                    };
                    self.group_delta_ = cb_result;
                    if self.groups_[group_idx].is_reversed {
                        self.group_delta_ = -self.group_delta_;
                    }
                }

                if self.group_delta_ < 1.0 {
                    continue;
                }
                let pt = path[0];
                let abs_delta = self.group_delta_.abs();
                // single vertex: build a circle or square
                if self.groups_[group_idx].join_type == JoinType::Round {
                    let radius = abs_delta;
                    let steps = if self.steps_per_rad_ > 0.0 {
                        (self.steps_per_rad_ * 2.0 * PI).ceil() as usize
                    } else {
                        0
                    };
                    self.path_out = ellipse_point64(pt, radius, radius, steps);
                } else {
                    let d = abs_delta.ceil() as i64;
                    let r = Rect64::new(pt.x - d, pt.y - d, pt.x + d, pt.y + d);
                    self.path_out = r.as_path();
                }

                self.solution.push(self.path_out.clone());
                continue;
            } // end of offsetting a single point

            if path_len == 2 && self.groups_[group_idx].end_type == EndType::Joined {
                self.end_type_ = if self.groups_[group_idx].join_type == JoinType::Round {
                    EndType::Round
                } else {
                    EndType::Square
                };
            }

            self.build_normals(&path);

            // We need to work with a snapshot of end_type_ since offset methods use self
            let current_end_type = self.end_type_;
            let group_snapshot = GroupSnapshot {
                is_reversed: self.groups_[group_idx].is_reversed,
            };

            match current_end_type {
                EndType::Polygon => {
                    let g = Group {
                        paths_in: Paths64::new(),
                        lowest_path_idx: None,
                        is_reversed: group_snapshot.is_reversed,
                        join_type,
                        end_type,
                    };
                    self.offset_polygon(&g, &path);
                }
                EndType::Joined => {
                    let g = Group {
                        paths_in: Paths64::new(),
                        lowest_path_idx: None,
                        is_reversed: group_snapshot.is_reversed,
                        join_type,
                        end_type,
                    };
                    self.offset_open_joined(&g, &path);
                }
                _ => {
                    let g = Group {
                        paths_in: Paths64::new(),
                        lowest_path_idx: None,
                        is_reversed: group_snapshot.is_reversed,
                        join_type,
                        end_type,
                    };
                    self.offset_open_path(&g, &path);
                }
            }
        }
    }

    /// Calculate total solution capacity
    /// Direct port from clipper.offset.cpp line 553
    fn calc_solution_capacity(&self) -> usize {
        let mut result = 0;
        for g in &self.groups_ {
            result += if g.end_type == EndType::Joined {
                g.paths_in.len() * 2
            } else {
                g.paths_in.len()
            };
        }
        result
    }

    /// Check if the orientation of polygon groups is reversed
    /// Direct port from clipper.offset.cpp line 561
    fn check_reverse_orientation(&self) -> bool {
        // nb: this assumes there's consistency in orientation between groups
        for g in &self.groups_ {
            if g.end_type == EndType::Polygon {
                return g.is_reversed;
            }
        }
        false
    }

    /// Main internal execution method
    /// Direct port from clipper.offset.cpp line 574
    fn execute_internal(&mut self, delta: f64) {
        self.error_code_ = 0;
        if self.groups_.is_empty() {
            return;
        }
        self.solution.reserve(self.calc_solution_capacity());

        if delta.abs() < 0.5 {
            // ie: offset is insignificant
            let mut sol_size = 0;
            for group in &self.groups_ {
                sol_size += group.paths_in.len();
            }
            self.solution.reserve(sol_size);
            for group in &self.groups_ {
                for p in &group.paths_in {
                    self.solution.push(p.clone());
                }
            }
        } else {
            self.temp_lim_ = if self.miter_limit_ <= 1.0 {
                2.0
            } else {
                2.0 / (self.miter_limit_ * self.miter_limit_)
            };

            self.delta_ = delta;
            let group_count = self.groups_.len();
            for i in 0..group_count {
                self.do_group_offset(i);
                if self.error_code_ != 0 {
                    self.solution.clear();
                    // continue checking other groups
                }
            }
        }

        if self.solution.is_empty() {
            return;
        }

        let paths_reversed = self.check_reverse_orientation();
        // clean up self-intersections
        let mut c = Clipper64::new();
        c.set_preserve_collinear(self.preserve_collinear_);
        // the solution should retain the orientation of the input
        c.set_reverse_solution(self.reverse_solution_ != paths_reversed);

        c.add_subject(&self.solution);

        if let Some(ref mut tree) = self.solution_tree {
            if paths_reversed {
                c.execute_tree(ClipType::Union, FillRule::Negative, tree, &mut Paths64::new());
            } else {
                c.execute_tree(ClipType::Union, FillRule::Positive, tree, &mut Paths64::new());
            }
        } else {
            let mut result = Paths64::new();
            if paths_reversed {
                c.execute(
                    ClipType::Union,
                    FillRule::Negative,
                    &mut result,
                    None,
                );
            } else {
                c.execute(
                    ClipType::Union,
                    FillRule::Positive,
                    &mut result,
                    None,
                );
            }
            self.solution = result;
        }
    }
}

impl Default for ClipperOffset {
    fn default() -> Self {
        Self::new(2.0, 0.0, false, false)
    }
}

/// Helper struct to snapshot group properties needed during offset
struct GroupSnapshot {
    is_reversed: bool,
}

// Include tests from separate file
#[cfg(test)]
#[path = "offset_tests.rs"]
mod tests;
