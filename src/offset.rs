/*******************************************************************************
* Author    :  Angus Johnson (original C++), Rust port                        *
* Date      :  2025                                                           *
* Website   :  https://www.angusj.com                                         *
* Copyright :  Angus Johnson 2010-2025                                        *
* Purpose   :  Path Offset (Inflate/Shrink)                                   *
* License   :  https://www.boost.org/LICENSE_1_0.txt                          *
*******************************************************************************/

//! Path offset (inflate/shrink) module.
//!
//! Direct port from clipper.offset.h / clipper.offset.cpp.
//! Provides `ClipperOffset` for inflating/shrinking paths by a specified delta.

use crate::core::{
    area, constants, cross_product_two_vectors, dot_product_two_vectors, ellipse_point64,
    get_segment_intersect_pt_d, reflect_point, strip_duplicates_path, translate_point, Path64,
    PathD, Paths64, Point64, PointD, Rect64,
};
use crate::engine::ClipType;
use crate::engine_public::{Clipper64, PolyTree64};
use crate::FillRule;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FLOATING_POINT_TOLERANCE: f64 = 1e-12;

/// Default arc approximation constant (1/500).
/// When arc_tolerance is 0, the calculated default arc tolerance
/// (offset_radius * arc_const) generally produces good (smooth) arc
/// approximations without producing excessively small segment lengths.
const ARC_CONST: f64 = 0.002;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Join type for path offsetting.
/// Direct port from clipper.offset.h line 19.
///
/// - `Square`: Joins are 'squared' at exactly the offset distance (more complex code)
/// - `Bevel`: Similar to Square, but offset distance varies with angle (simple & faster)
/// - `Round`: Joins are rounded (arc approximation)
/// - `Miter`: Joins extend to a point, limited by miter_limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinType {
    Square,
    Bevel,
    Round,
    Miter,
}

/// End type for open path offsetting.
/// Direct port from clipper.offset.h line 23.
///
/// - `Polygon`: Offsets only one side of a closed path
/// - `Joined`: Offsets both sides of a path, with joined ends
/// - `Butt`: Offsets both sides of a path, with square blunt ends
/// - `Square`: Offsets both sides of a path, with square extended ends
/// - `Round`: Offsets both sides of a path, with round extended ends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndType {
    Polygon,
    Joined,
    Butt,
    Square,
    Round,
}

/// Delta callback type for variable offset.
/// Called for each vertex to determine the offset delta at that point.
/// Parameters: (path, path_normals, curr_idx, prev_idx) -> delta
pub type DeltaCallback64 = Box<dyn Fn(&Path64, &PathD, usize, usize) -> f64>;

// ---------------------------------------------------------------------------
// Helper functions (module-level, matching C++ file-scope functions)
// ---------------------------------------------------------------------------

/// Find the lowest (highest y) closed path and determine its orientation.
/// Direct port from clipper.offset.cpp GetLowestClosedPathInfo.
fn get_lowest_closed_path_info(paths: &Paths64) -> (Option<usize>, bool) {
    let mut idx: Option<usize> = None;
    let mut bot_pt = Point64::new(i64::MAX, i64::MIN);
    let mut is_neg_area = false;

    for (i, path_i) in paths.iter().enumerate() {
        let mut a: f64 = f64::MAX;
        for pt in path_i {
            if (pt.y < bot_pt.y) || (pt.y == bot_pt.y && pt.x >= bot_pt.x) {
                continue;
            }
            if a == f64::MAX {
                a = area(path_i);
                if a == 0.0 {
                    break; // invalid closed path
                }
                is_neg_area = a < 0.0;
            }
            idx = Some(i);
            bot_pt.x = pt.x;
            bot_pt.y = pt.y;
        }
    }
    (idx, is_neg_area)
}

/// Hypotenuse calculation.
/// Direct port from clipper.offset.cpp Hypot.
#[inline]
fn hypot_xy(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

/// Get unit normal vector between two points.
/// Direct port from clipper.offset.cpp GetUnitNormal.
#[inline]
fn get_unit_normal(pt1: &Point64, pt2: &Point64) -> PointD {
    if pt1 == pt2 {
        return PointD::new(0.0, 0.0);
    }
    let dx = (pt2.x - pt1.x) as f64;
    let dy = (pt2.y - pt1.y) as f64;
    let inverse_hypot = 1.0 / hypot_xy(dx, dy);
    PointD::new(dy * inverse_hypot, -dx * inverse_hypot)
}

/// Check if a floating-point value is approximately zero.
/// Direct port from clipper.offset.cpp AlmostZero.
#[inline]
fn almost_zero(value: f64, epsilon: f64) -> bool {
    value.abs() < epsilon
}

/// Normalize a 2D vector to unit length.
/// Direct port from clipper.offset.cpp NormalizeVector.
#[inline]
fn normalize_vector(vec: &PointD) -> PointD {
    let h = hypot_xy(vec.x, vec.y);
    if almost_zero(h, 0.001) {
        return PointD::new(0.0, 0.0);
    }
    let inverse_hypot = 1.0 / h;
    PointD::new(vec.x * inverse_hypot, vec.y * inverse_hypot)
}

/// Get the average of two unit vectors, normalized.
/// Direct port from clipper.offset.cpp GetAvgUnitVector.
#[inline]
fn get_avg_unit_vector(vec1: &PointD, vec2: &PointD) -> PointD {
    normalize_vector(&PointD::new(vec1.x + vec2.x, vec1.y + vec2.y))
}

/// Check if the end type represents a closed path.
/// Direct port from clipper.offset.cpp IsClosedPath.
#[inline]
#[allow(dead_code)]
fn is_closed_path(et: EndType) -> bool {
    et == EndType::Polygon || et == EndType::Joined
}

/// Get perpendicular offset point (returns Point64, rounds coordinates).
/// Direct port from clipper.offset.cpp GetPerpendic.
#[inline]
fn get_perpendic(pt: &Point64, norm: &PointD, delta: f64) -> Point64 {
    Point64::new(
        (pt.x as f64 + norm.x * delta).round() as i64,
        (pt.y as f64 + norm.y * delta).round() as i64,
    )
}

/// Get perpendicular offset point (returns PointD, no rounding).
/// Direct port from clipper.offset.cpp GetPerpendicD.
#[inline]
fn get_perpendic_d(pt: &Point64, norm: &PointD, delta: f64) -> PointD {
    PointD::new(pt.x as f64 + norm.x * delta, pt.y as f64 + norm.y * delta)
}

/// Negate all coordinates in a PathD.
/// Direct port from clipper.offset.cpp NegatePath.
#[inline]
fn negate_path(path: &mut PathD) {
    for pt in path.iter_mut() {
        pt.x = -pt.x;
        pt.y = -pt.y;
    }
}

/// Convert PointD to Point64 by rounding.
/// Helper for C++ implicit conversions where path_out.emplace_back(PointD) rounds.
#[inline]
fn point64_from_f(x: f64, y: f64) -> Point64 {
    Point64::new(x.round() as i64, y.round() as i64)
}

// ---------------------------------------------------------------------------
// Group struct (internal)
// ---------------------------------------------------------------------------

/// Internal grouping of paths with shared join/end types.
/// Direct port from ClipperOffset::Group (clipper.offset.h line 35-43).
struct Group {
    paths_in: Paths64,
    lowest_path_idx: Option<usize>,
    is_reversed: bool,
    join_type: JoinType,
    end_type: EndType,
}

impl Group {
    /// Construct a new Group, stripping duplicates and determining orientation.
    /// Direct port from ClipperOffset::Group constructor (clipper.offset.cpp line 139-162).
    fn new(paths: &Paths64, join_type: JoinType, end_type: EndType) -> Self {
        let mut paths_in = paths.clone();
        let is_joined = end_type == EndType::Polygon || end_type == EndType::Joined;

        for p in paths_in.iter_mut() {
            strip_duplicates_path(p, is_joined);
        }

        let (lowest_path_idx, is_reversed) = if end_type == EndType::Polygon {
            let (idx, is_neg_area) = get_lowest_closed_path_info(&paths_in);
            // The lowermost path must be an outer path, so if its orientation is
            // negative, then flag the whole group as 'reversed' (will negate delta etc.)
            // as this is much more efficient than reversing every path.
            let is_reversed = idx.is_some() && is_neg_area;
            (idx, is_reversed)
        } else {
            (None, false)
        };

        Group {
            paths_in,
            lowest_path_idx,
            is_reversed,
            join_type,
            end_type,
        }
    }
}

// ---------------------------------------------------------------------------
// ClipperOffset
// ---------------------------------------------------------------------------

/// Path offset (inflate/shrink) engine.
/// Direct port from ClipperOffset class (clipper.offset.h / clipper.offset.cpp).
///
/// Inflates (or shrinks) both open and closed paths using the specified join type
/// and end type. After building the raw offset, a Clipper64 union is used to
/// clean up self-intersections.
pub struct ClipperOffset {
    // Internal state
    error_code: i32,
    delta: f64,
    group_delta: f64,
    temp_lim: f64,
    steps_per_rad: f64,
    step_sin: f64,
    step_cos: f64,
    norms: PathD,
    path_out: Path64,
    solution: Paths64,
    groups: Vec<Group>,
    join_type: JoinType,
    end_type: EndType,

    // User-configurable parameters
    miter_limit: f64,
    arc_tolerance: f64,
    preserve_collinear: bool,
    reverse_solution: bool,

    // Callbacks
    delta_callback: Option<DeltaCallback64>,
}

impl ClipperOffset {
    // ------------------------------------------------------------------
    // Construction / configuration
    // ------------------------------------------------------------------

    /// Create a new ClipperOffset with the given parameters.
    /// Direct port from ClipperOffset constructor (clipper.offset.h line 85-91).
    pub fn new(
        miter_limit: f64,
        arc_tolerance: f64,
        preserve_collinear: bool,
        reverse_solution: bool,
    ) -> Self {
        ClipperOffset {
            error_code: 0,
            delta: 0.0,
            group_delta: 0.0,
            temp_lim: 0.0,
            steps_per_rad: 0.0,
            step_sin: 0.0,
            step_cos: 0.0,
            norms: PathD::new(),
            path_out: Path64::new(),
            solution: Paths64::new(),
            groups: Vec::new(),
            join_type: JoinType::Bevel,
            end_type: EndType::Polygon,
            miter_limit,
            arc_tolerance,
            preserve_collinear,
            reverse_solution,
            delta_callback: None,
        }
    }

    /// Create a new ClipperOffset with default parameters.
    /// miter_limit = 2.0, arc_tolerance = 0.0,
    /// preserve_collinear = false, reverse_solution = false.
    pub fn new_default() -> Self {
        Self::new(2.0, 0.0, false, false)
    }

    /// Get the error code from the last operation.
    pub fn error_code(&self) -> i32 {
        self.error_code
    }

    /// Get the miter limit.
    pub fn miter_limit(&self) -> f64 {
        self.miter_limit
    }

    /// Set the miter limit.
    pub fn set_miter_limit(&mut self, miter_limit: f64) {
        self.miter_limit = miter_limit;
    }

    /// Get the arc tolerance.
    pub fn arc_tolerance(&self) -> f64 {
        self.arc_tolerance
    }

    /// Set the arc tolerance.
    /// Needed for rounded offsets. See offset_trigonometry2.svg.
    pub fn set_arc_tolerance(&mut self, arc_tolerance: f64) {
        self.arc_tolerance = arc_tolerance;
    }

    /// Get the preserve_collinear flag.
    pub fn preserve_collinear(&self) -> bool {
        self.preserve_collinear
    }

    /// Set the preserve_collinear flag.
    pub fn set_preserve_collinear(&mut self, preserve_collinear: bool) {
        self.preserve_collinear = preserve_collinear;
    }

    /// Get the reverse_solution flag.
    pub fn reverse_solution(&self) -> bool {
        self.reverse_solution
    }

    /// Set the reverse_solution flag.
    pub fn set_reverse_solution(&mut self, reverse_solution: bool) {
        self.reverse_solution = reverse_solution;
    }

    /// Set the delta callback for variable offset.
    /// Direct port from ClipperOffset::SetDeltaCallback.
    pub fn set_delta_callback(&mut self, cb: Option<DeltaCallback64>) {
        self.delta_callback = cb;
    }

    // ------------------------------------------------------------------
    // Path input
    // ------------------------------------------------------------------

    /// Add a single path with the given join type and end type.
    /// Direct port from ClipperOffset::AddPath (clipper.offset.cpp line 168-171).
    pub fn add_path(&mut self, path: &Path64, jt: JoinType, et: EndType) {
        self.groups.push(Group::new(&vec![path.clone()], jt, et));
    }

    /// Add multiple paths with the given join type and end type.
    /// Direct port from ClipperOffset::AddPaths (clipper.offset.cpp line 173-177).
    pub fn add_paths(&mut self, paths: &Paths64, jt: JoinType, et: EndType) {
        if paths.is_empty() {
            return;
        }
        self.groups.push(Group::new(paths, jt, et));
    }

    /// Clear all groups and normals.
    /// Direct port from ClipperOffset::Clear (clipper.offset.h line 98).
    pub fn clear(&mut self) {
        self.groups.clear();
        self.norms.clear();
    }

    // ------------------------------------------------------------------
    // Execution
    // ------------------------------------------------------------------

    /// Execute the offset operation, storing results in `paths`.
    /// Direct port from ClipperOffset::Execute(double, Paths64&)
    /// (clipper.offset.cpp line 636-642).
    pub fn execute(&mut self, delta: f64, paths: &mut Paths64) {
        paths.clear();
        self.solution.clear();
        self.execute_internal(delta, None);
        std::mem::swap(&mut self.solution, paths);
    }

    /// Execute the offset operation, storing results in a `PolyTree64`.
    /// Direct port from ClipperOffset::Execute(double, PolyTree64&)
    /// (clipper.offset.cpp line 645-653).
    pub fn execute_tree(&mut self, delta: f64, polytree: &mut PolyTree64) {
        polytree.clear();
        self.solution.clear();
        self.execute_internal(delta, Some(polytree));
        self.solution.clear();
    }

    /// Execute using a delta callback for variable offset.
    /// Direct port from ClipperOffset::Execute(DeltaCallback64, Paths64&)
    /// (clipper.offset.cpp line 655-659).
    pub fn execute_with_callback(&mut self, delta_cb: DeltaCallback64, paths: &mut Paths64) {
        self.delta_callback = Some(delta_cb);
        self.execute(1.0, paths);
    }

    // ------------------------------------------------------------------
    // Internal execution
    // ------------------------------------------------------------------

    /// Calculate the expected solution capacity.
    /// Direct port from ClipperOffset::CalcSolutionCapacity (clipper.offset.cpp line 553-559).
    fn calc_solution_capacity(&self) -> usize {
        let mut result = 0;
        for g in &self.groups {
            result += if g.end_type == EndType::Joined {
                g.paths_in.len() * 2
            } else {
                g.paths_in.len()
            };
        }
        result
    }

    /// Check if the groups have reversed orientation.
    /// Direct port from ClipperOffset::CheckReverseOrientation (clipper.offset.cpp line 561-572).
    fn check_reverse_orientation(&self) -> bool {
        // nb: this assumes there's consistency in orientation between groups
        for g in &self.groups {
            if g.end_type == EndType::Polygon {
                return g.is_reversed;
            }
        }
        false
    }

    /// Core internal execution logic.
    /// Direct port from ClipperOffset::ExecuteInternal (clipper.offset.cpp line 574-634).
    fn execute_internal(&mut self, delta: f64, polytree: Option<&mut PolyTree64>) {
        self.error_code = 0;
        if self.groups.is_empty() {
            return;
        }
        self.solution.reserve(self.calc_solution_capacity());

        if delta.abs() < 0.5 {
            // offset is insignificant - just copy paths
            let mut sol_size = 0;
            for group in &self.groups {
                sol_size += group.paths_in.len();
            }
            self.solution.reserve(sol_size);
            for group in &self.groups {
                self.solution.extend(group.paths_in.iter().cloned());
            }
        } else {
            self.temp_lim = if self.miter_limit <= 1.0 {
                2.0
            } else {
                2.0 / (self.miter_limit * self.miter_limit)
            };

            self.delta = delta;
            // Process each group - we need indices because do_group_offset
            // borrows self mutably
            for i in 0..self.groups.len() {
                self.do_group_offset(i);
                if self.error_code != 0 {
                    self.solution.clear();
                }
            }
        }

        if self.solution.is_empty() {
            return;
        }

        let paths_reversed = self.check_reverse_orientation();
        // Clean up self-intersections using Clipper64 union
        let mut c = Clipper64::new();
        c.set_preserve_collinear(self.preserve_collinear);
        // The solution should retain the orientation of the input
        c.set_reverse_solution(self.reverse_solution != paths_reversed);
        c.add_subject(&self.solution);

        let fill_rule = if paths_reversed {
            FillRule::Negative
        } else {
            FillRule::Positive
        };

        if let Some(tree) = polytree {
            let mut open_paths = Paths64::new();
            c.execute_tree(ClipType::Union, fill_rule, tree, &mut open_paths);
        } else {
            c.execute(ClipType::Union, fill_rule, &mut self.solution, None);
        }
    }

    // ------------------------------------------------------------------
    // Build normals
    // ------------------------------------------------------------------

    /// Build unit normals for each edge of a path.
    /// Direct port from ClipperOffset::BuildNormals (clipper.offset.cpp line 179-188).
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

    // ------------------------------------------------------------------
    // Join type implementations
    // ------------------------------------------------------------------

    /// Bevel join implementation.
    /// Direct port from ClipperOffset::DoBevel (clipper.offset.cpp line 190-216).
    fn do_bevel(&mut self, path: &Path64, j: usize, k: usize) {
        let pt1: PointD;
        let pt2: PointD;
        if j == k {
            let abs_delta = self.group_delta.abs();
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
                path[j].x as f64 + self.group_delta * self.norms[k].x,
                path[j].y as f64 + self.group_delta * self.norms[k].y,
            );
            pt2 = PointD::new(
                path[j].x as f64 + self.group_delta * self.norms[j].x,
                path[j].y as f64 + self.group_delta * self.norms[j].y,
            );
        }
        self.path_out.push(point64_from_f(pt1.x, pt1.y));
        self.path_out.push(point64_from_f(pt2.x, pt2.y));
    }

    /// Square join implementation.
    /// Direct port from ClipperOffset::DoSquare (clipper.offset.cpp line 218-256).
    fn do_square(&mut self, path: &Path64, j: usize, k: usize) {
        let vec: PointD = if j == k {
            PointD::new(self.norms[j].y, -self.norms[j].x)
        } else {
            get_avg_unit_vector(
                &PointD::new(-self.norms[k].y, self.norms[k].x),
                &PointD::new(self.norms[j].y, -self.norms[j].x),
            )
        };

        let abs_delta = self.group_delta.abs();

        // Now offset the original vertex delta units along unit vector
        let pt_q = PointD::new(path[j].x as f64, path[j].y as f64);
        let pt_q = translate_point(&pt_q, abs_delta * vec.x, abs_delta * vec.y);
        // Get perpendicular vertices
        let pt1 = translate_point(&pt_q, self.group_delta * vec.y, self.group_delta * -vec.x);
        let pt2 = translate_point(&pt_q, self.group_delta * -vec.y, self.group_delta * vec.x);
        // Get 2 vertices along one edge offset
        let pt3 = get_perpendic_d(&path[k], &self.norms[k], self.group_delta);

        if j == k {
            let pt4 = PointD::new(
                pt3.x + vec.x * self.group_delta,
                pt3.y + vec.y * self.group_delta,
            );
            let mut pt = pt_q;
            get_segment_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            // Get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(point64_from_f(reflected.x, reflected.y));
            self.path_out.push(point64_from_f(pt.x, pt.y));
        } else {
            let pt4 = get_perpendic_d(&path[j], &self.norms[k], self.group_delta);
            let mut pt = pt_q;
            get_segment_intersect_pt_d(pt1, pt2, pt3, pt4, &mut pt);
            self.path_out.push(point64_from_f(pt.x, pt.y));
            // Get the second intersect point through reflection
            let reflected = reflect_point(&pt, &pt_q);
            self.path_out.push(point64_from_f(reflected.x, reflected.y));
        }
    }

    /// Miter join implementation.
    /// Direct port from ClipperOffset::DoMiter (clipper.offset.cpp line 258-271).
    fn do_miter(&mut self, path: &Path64, j: usize, k: usize, cos_a: f64) {
        let q = self.group_delta / (cos_a + 1.0);
        self.path_out.push(point64_from_f(
            path[j].x as f64 + (self.norms[k].x + self.norms[j].x) * q,
            path[j].y as f64 + (self.norms[k].y + self.norms[j].y) * q,
        ));
    }

    /// Round join implementation.
    /// Direct port from ClipperOffset::DoRound (clipper.offset.cpp line 273-309).
    fn do_round(&mut self, path: &Path64, j: usize, k: usize, angle: f64) {
        if self.delta_callback.is_some() {
            // When delta_callback is assigned, group_delta won't be constant,
            // so we need to do these calculations for *every* vertex.
            let abs_delta = self.group_delta.abs();
            let arc_tol = if self.arc_tolerance > FLOATING_POINT_TOLERANCE {
                abs_delta.min(self.arc_tolerance)
            } else {
                abs_delta * ARC_CONST
            };
            let steps_per_360 =
                (constants::PI / (1.0 - arc_tol / abs_delta).acos()).min(abs_delta * constants::PI);
            self.step_sin = (2.0 * constants::PI / steps_per_360).sin();
            self.step_cos = (2.0 * constants::PI / steps_per_360).cos();
            if self.group_delta < 0.0 {
                self.step_sin = -self.step_sin;
            }
            self.steps_per_rad = steps_per_360 / (2.0 * constants::PI);
        }

        let pt = path[j];
        let mut offset_vec = PointD::new(
            self.norms[k].x * self.group_delta,
            self.norms[k].y * self.group_delta,
        );

        if j == k {
            offset_vec = offset_vec.negate();
        }
        self.path_out.push(point64_from_f(
            pt.x as f64 + offset_vec.x,
            pt.y as f64 + offset_vec.y,
        ));

        let steps = (self.steps_per_rad * angle.abs()).ceil() as i32; // #448, #456
        for _ in 1..steps {
            // ie 1 less than steps
            offset_vec = PointD::new(
                offset_vec.x * self.step_cos - self.step_sin * offset_vec.y,
                offset_vec.x * self.step_sin + offset_vec.y * self.step_cos,
            );
            self.path_out.push(point64_from_f(
                pt.x as f64 + offset_vec.x,
                pt.y as f64 + offset_vec.y,
            ));
        }
        self.path_out
            .push(get_perpendic(&path[j], &self.norms[j], self.group_delta));
    }

    // ------------------------------------------------------------------
    // Offset operations
    // ------------------------------------------------------------------

    /// Offset a single vertex.
    /// Direct port from ClipperOffset::OffsetPoint (clipper.offset.cpp line 311-370).
    fn offset_point(&mut self, group_idx: usize, path: &Path64, j: usize, k: usize) {
        // Let A = change in angle where edges join
        // A == 0: ie no change in angle (flat join)
        // A == PI: edges 'spike'
        // sin(A) < 0: right turning
        // cos(A) < 0: change in angle is more than 90 degree

        if path[j] == path[k] {
            return;
        }

        let sin_a = cross_product_two_vectors(self.norms[j], self.norms[k]);
        let cos_a = dot_product_two_vectors(self.norms[j], self.norms[k]);
        let sin_a = sin_a.clamp(-1.0, 1.0);

        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, j, k);
            if self.groups[group_idx].is_reversed {
                self.group_delta = -self.group_delta;
            }
        }
        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[j]);
            return;
        }

        if cos_a > -0.999 && (sin_a * self.group_delta < 0.0) {
            // test for concavity first (#593)
            // Is concave.
            // By far the simplest way to construct concave joins, especially those
            // joining very short segments, is to insert 3 points that produce negative
            // regions. These regions will be removed later by the finishing union
            // operation. This is also the best way to ensure that path reversals
            // (ie over-shrunk paths) are removed.
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[k], self.group_delta));
            self.path_out.push(path[j]); // (#405, #873, #916)
            self.path_out
                .push(get_perpendic(&path[j], &self.norms[j], self.group_delta));
        } else if cos_a > 0.999 && self.join_type != JoinType::Round {
            // Almost straight - less than 2.5 degree (#424, #482, #526 & #724)
            self.do_miter(path, j, k, cos_a);
        } else if self.join_type == JoinType::Miter {
            // Miter unless the angle is sufficiently acute to exceed ML
            if cos_a > self.temp_lim - 1.0 {
                self.do_miter(path, j, k, cos_a);
            } else {
                self.do_square(path, j, k);
            }
        } else if self.join_type == JoinType::Round {
            self.do_round(path, j, k, sin_a.atan2(cos_a));
        } else if self.join_type == JoinType::Bevel {
            self.do_bevel(path, j, k);
        } else {
            self.do_square(path, j, k);
        }
    }

    /// Offset a closed polygon.
    /// Direct port from ClipperOffset::OffsetPolygon (clipper.offset.cpp line 372-378).
    fn offset_polygon(&mut self, group_idx: usize, path: &Path64) {
        self.path_out.clear();
        let len = path.len();
        if len == 0 {
            return;
        }
        let mut k = len - 1;
        for j in 0..len {
            self.offset_point(group_idx, path, j, k);
            k = j;
        }
        let path_out = std::mem::take(&mut self.path_out);
        self.solution.push(path_out);
    }

    /// Offset an open path with joined ends.
    /// Direct port from ClipperOffset::OffsetOpenJoined (clipper.offset.cpp line 380-393).
    fn offset_open_joined(&mut self, group_idx: usize, path: &Path64) {
        self.offset_polygon(group_idx, path);
        let mut reverse_path = path.clone();
        reverse_path.reverse();

        // Rebuild normals
        self.norms.reverse();
        self.norms.push(self.norms[0]);
        self.norms.remove(0);
        negate_path(&mut self.norms);

        self.offset_polygon(group_idx, &reverse_path);
    }

    /// Offset an open path.
    /// Direct port from ClipperOffset::OffsetOpenPath (clipper.offset.cpp line 395-453).
    fn offset_open_path(&mut self, group_idx: usize, path: &Path64) {
        self.path_out.clear();

        // Do the line start cap
        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, 0, 0);
        }

        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[0]);
        } else {
            match self.end_type {
                EndType::Butt => self.do_bevel(path, 0, 0),
                EndType::Round => self.do_round(path, 0, 0, constants::PI),
                _ => self.do_square(path, 0, 0),
            }
        }

        let high_i = path.len() - 1;
        // Offset the left side going forward
        let mut k = 0;
        for j in 1..high_i {
            self.offset_point(group_idx, path, j, k);
            k = j;
        }

        // Reverse normals
        for i in (1..=high_i).rev() {
            self.norms[i] = PointD::new(-self.norms[i - 1].x, -self.norms[i - 1].y);
        }
        self.norms[0] = self.norms[high_i];

        // Do the line end cap
        if let Some(ref cb) = self.delta_callback {
            self.group_delta = cb(path, &self.norms, high_i, high_i);
        }

        if self.group_delta.abs() <= FLOATING_POINT_TOLERANCE {
            self.path_out.push(path[high_i]);
        } else {
            match self.end_type {
                EndType::Butt => self.do_bevel(path, high_i, high_i),
                EndType::Round => self.do_round(path, high_i, high_i, constants::PI),
                _ => self.do_square(path, high_i, high_i),
            }
        }

        // Offset the right side going backward
        let mut k = high_i;
        for j in (1..high_i).rev() {
            self.offset_point(group_idx, path, j, k);
            k = j;
        }
        let path_out = std::mem::take(&mut self.path_out);
        self.solution.push(path_out);
    }

    // ------------------------------------------------------------------
    // Group offset
    // ------------------------------------------------------------------

    /// Process a single group of paths.
    /// Direct port from ClipperOffset::DoGroupOffset (clipper.offset.cpp line 455-539).
    fn do_group_offset(&mut self, group_idx: usize) {
        let group_end_type = self.groups[group_idx].end_type;
        let group_join_type = self.groups[group_idx].join_type;
        let group_is_reversed = self.groups[group_idx].is_reversed;
        let group_lowest = self.groups[group_idx].lowest_path_idx;

        if group_end_type == EndType::Polygon {
            // A straight path (2 points) can now also be 'polygon' offset
            // where the ends will be treated as (180 deg.) joins
            if group_lowest.is_none() {
                self.delta = self.delta.abs();
            }
            self.group_delta = if group_is_reversed {
                -self.delta
            } else {
                self.delta
            };
        } else {
            self.group_delta = self.delta.abs();
        }

        let abs_delta = self.group_delta.abs();
        self.join_type = group_join_type;
        self.end_type = group_end_type;

        if group_join_type == JoinType::Round || group_end_type == EndType::Round {
            // Calculate the number of steps required to approximate a circle
            let arc_tol = if self.arc_tolerance > FLOATING_POINT_TOLERANCE {
                abs_delta.min(self.arc_tolerance)
            } else {
                abs_delta * ARC_CONST
            };

            let steps_per_360 =
                (constants::PI / (1.0 - arc_tol / abs_delta).acos()).min(abs_delta * constants::PI);
            self.step_sin = (2.0 * constants::PI / steps_per_360).sin();
            self.step_cos = (2.0 * constants::PI / steps_per_360).cos();
            if self.group_delta < 0.0 {
                self.step_sin = -self.step_sin;
            }
            self.steps_per_rad = steps_per_360 / (2.0 * constants::PI);
        }

        // Iterate over paths in the group
        let paths_count = self.groups[group_idx].paths_in.len();
        for path_idx in 0..paths_count {
            let path = self.groups[group_idx].paths_in[path_idx].clone();
            let path_len = path.len();
            self.path_out.clear();

            if path_len == 1 {
                // Single point
                if self.delta_callback.is_some() {
                    let cb_result = if let Some(ref cb) = self.delta_callback {
                        cb(&path, &self.norms, 0, 0)
                    } else {
                        0.0
                    };
                    self.group_delta = cb_result;
                    if group_is_reversed {
                        self.group_delta = -self.group_delta;
                    }
                }

                if self.group_delta < 1.0 {
                    continue;
                }
                let pt = path[0];
                let abs_delta_local = self.group_delta.abs();

                // Single vertex: build a circle or square
                if group_join_type == JoinType::Round {
                    let radius = abs_delta_local;
                    let steps = if self.steps_per_rad > 0.0 {
                        (self.steps_per_rad * 2.0 * constants::PI).ceil() as usize
                    } else {
                        0
                    };
                    self.path_out = ellipse_point64(pt, radius, radius, steps);
                } else {
                    let d = abs_delta_local.ceil() as i64;
                    let r = Rect64::new(pt.x - d, pt.y - d, pt.x + d, pt.y + d);
                    self.path_out = r.as_path();
                }

                let path_out = std::mem::take(&mut self.path_out);
                self.solution.push(path_out);
                continue;
            } // end of offsetting a single point

            if path_len == 2 && group_end_type == EndType::Joined {
                self.end_type = if group_join_type == JoinType::Round {
                    EndType::Round
                } else {
                    EndType::Square
                };
            }

            self.build_normals(&path);
            if self.end_type == EndType::Polygon {
                self.offset_polygon(group_idx, &path);
            } else if self.end_type == EndType::Joined {
                self.offset_open_joined(group_idx, &path);
            } else {
                self.offset_open_path(group_idx, &path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "offset_tests.rs"]
mod tests;
