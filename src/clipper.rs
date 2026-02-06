/*******************************************************************************
* Author    :  Angus Johnson (original C++), Rust port                        *
* Date      :  2025                                                           *
* Website   :  https://www.angusj.com                                         *
* Copyright :  Angus Johnson 2010-2025                                        *
* Purpose   :  Simple public API for the Clipper Library                      *
* License   :  https://www.boost.org/LICENSE_1_0.txt                          *
*******************************************************************************/

//! Public API convenience functions for the Clipper2 library.
//!
//! Direct port from clipper.h. These functions provide a simple interface
//! for common polygon operations: boolean operations, path offsetting,
//! rect clipping, path simplification, and various geometric utilities.

use crate::core::{
    check_precision_range, constants, cross_product_three_points, distance_sqr, is_collinear,
    perpendic_dist_from_line_sqrd, point_in_polygon, scale_path, scale_paths, scale_rect, sqr,
    FromF64, Path, Path64, PathD, Paths, Paths64, PathsD, Point, Point64, PointInPolygonResult,
    Rect64, RectD, ToF64,
};
use crate::engine::ClipType;
use crate::engine_public::{Clipper64, ClipperD, PolyTree64, PolyTreeD};
use crate::offset::{ClipperOffset, EndType, JoinType};
use crate::rectclip::{RectClip64, RectClipLines64};
use crate::FillRule;
use num_traits::Num;

// ============================================================================
// Boolean Operations (Paths64)
// ============================================================================

/// Perform a boolean operation on Paths64.
/// Direct port from clipper.h BooleanOp (Paths64 overload).
pub fn boolean_op_64(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &Paths64,
    clips: &Paths64,
) -> Paths64 {
    let mut result = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    clipper.execute(clip_type, fill_rule, &mut result, None);
    result
}

/// Perform a boolean operation on Paths64 with PolyTree64 output.
/// Direct port from clipper.h BooleanOp (PolyTree64 overload).
pub fn boolean_op_tree_64(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &Paths64,
    clips: &Paths64,
    solution: &mut PolyTree64,
) {
    let mut sol_open = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    clipper.execute_tree(clip_type, fill_rule, solution, &mut sol_open);
}

/// Perform a boolean operation on PathsD.
/// Direct port from clipper.h BooleanOp (PathsD overload).
pub fn boolean_op_d(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &PathsD,
    clips: &PathsD,
    precision: i32,
) -> PathsD {
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    let mut result = PathsD::new();
    if error_code != 0 {
        return result;
    }
    let mut clipper = ClipperD::new(precision);
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    clipper.execute(clip_type, fill_rule, &mut result, None);
    result
}

/// Perform a boolean operation on PathsD with PolyTreeD output.
/// Direct port from clipper.h BooleanOp (PolyTreeD overload).
pub fn boolean_op_tree_d(
    clip_type: ClipType,
    fill_rule: FillRule,
    subjects: &PathsD,
    clips: &PathsD,
    polytree: &mut PolyTreeD,
    precision: i32,
) {
    polytree.clear();
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if error_code != 0 {
        return;
    }
    let mut clipper = ClipperD::new(precision);
    clipper.add_subject(subjects);
    clipper.add_clip(clips);
    let mut open_paths = PathsD::new();
    clipper.execute_tree(clip_type, fill_rule, polytree, &mut open_paths);
}

// ============================================================================
// Intersect
// ============================================================================

/// Compute the intersection of subjects and clips (Paths64).
/// Direct port from clipper.h Intersect.
pub fn intersect_64(subjects: &Paths64, clips: &Paths64, fill_rule: FillRule) -> Paths64 {
    boolean_op_64(ClipType::Intersection, fill_rule, subjects, clips)
}

/// Compute the intersection of subjects and clips (PathsD).
/// Direct port from clipper.h Intersect (PathsD overload).
pub fn intersect_d(
    subjects: &PathsD,
    clips: &PathsD,
    fill_rule: FillRule,
    precision: i32,
) -> PathsD {
    boolean_op_d(
        ClipType::Intersection,
        fill_rule,
        subjects,
        clips,
        precision,
    )
}

// ============================================================================
// Union
// ============================================================================

/// Compute the union of subjects and clips (Paths64).
/// Direct port from clipper.h Union.
pub fn union_64(subjects: &Paths64, clips: &Paths64, fill_rule: FillRule) -> Paths64 {
    boolean_op_64(ClipType::Union, fill_rule, subjects, clips)
}

/// Compute the union of subjects and clips (PathsD).
/// Direct port from clipper.h Union (PathsD overload).
pub fn union_d(subjects: &PathsD, clips: &PathsD, fill_rule: FillRule, precision: i32) -> PathsD {
    boolean_op_d(ClipType::Union, fill_rule, subjects, clips, precision)
}

/// Compute the union of subjects only (no clips) (Paths64).
/// Direct port from clipper.h Union (subjects-only overload).
pub fn union_subjects_64(subjects: &Paths64, fill_rule: FillRule) -> Paths64 {
    let mut result = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.execute(ClipType::Union, fill_rule, &mut result, None);
    result
}

/// Compute the union of subjects only (no clips) (PathsD).
/// Direct port from clipper.h Union (subjects-only PathsD overload).
pub fn union_subjects_d(subjects: &PathsD, fill_rule: FillRule, precision: i32) -> PathsD {
    let mut result = PathsD::new();
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if error_code != 0 {
        return result;
    }
    let mut clipper = ClipperD::new(precision);
    clipper.add_subject(subjects);
    clipper.execute(ClipType::Union, fill_rule, &mut result, None);
    result
}

// ============================================================================
// Difference
// ============================================================================

/// Compute the difference of subjects minus clips (Paths64).
/// Direct port from clipper.h Difference.
pub fn difference_64(subjects: &Paths64, clips: &Paths64, fill_rule: FillRule) -> Paths64 {
    boolean_op_64(ClipType::Difference, fill_rule, subjects, clips)
}

/// Compute the difference of subjects minus clips (PathsD).
/// Direct port from clipper.h Difference (PathsD overload).
pub fn difference_d(
    subjects: &PathsD,
    clips: &PathsD,
    fill_rule: FillRule,
    precision: i32,
) -> PathsD {
    boolean_op_d(ClipType::Difference, fill_rule, subjects, clips, precision)
}

// ============================================================================
// Xor
// ============================================================================

/// Compute the symmetric difference (Xor) of subjects and clips (Paths64).
/// Direct port from clipper.h Xor.
pub fn xor_64(subjects: &Paths64, clips: &Paths64, fill_rule: FillRule) -> Paths64 {
    boolean_op_64(ClipType::Xor, fill_rule, subjects, clips)
}

/// Compute the symmetric difference (Xor) of subjects and clips (PathsD).
/// Direct port from clipper.h Xor (PathsD overload).
pub fn xor_d(subjects: &PathsD, clips: &PathsD, fill_rule: FillRule, precision: i32) -> PathsD {
    boolean_op_d(ClipType::Xor, fill_rule, subjects, clips, precision)
}

// ============================================================================
// InflatePaths
// ============================================================================

/// Inflate (or deflate) paths by a delta amount (Paths64).
/// Direct port from clipper.h InflatePaths.
pub fn inflate_paths_64(
    paths: &Paths64,
    delta: f64,
    jt: JoinType,
    et: EndType,
    miter_limit: f64,
    arc_tolerance: f64,
) -> Paths64 {
    if delta == 0.0 {
        return paths.clone();
    }
    let mut clip_offset = ClipperOffset::new(miter_limit, arc_tolerance, false, false);
    clip_offset.add_paths(paths, jt, et);
    let mut solution = Paths64::new();
    clip_offset.execute(delta, &mut solution);
    solution
}

/// Inflate (or deflate) paths by a delta amount (PathsD).
/// Direct port from clipper.h InflatePaths (PathsD overload).
pub fn inflate_paths_d(
    paths: &PathsD,
    delta: f64,
    jt: JoinType,
    et: EndType,
    miter_limit: f64,
    precision: i32,
    arc_tolerance: f64,
) -> PathsD {
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if delta == 0.0 {
        return paths.clone();
    }
    if error_code != 0 {
        return PathsD::new();
    }
    let scale = 10f64.powi(precision);
    let mut clip_offset = ClipperOffset::new(miter_limit, arc_tolerance * scale, false, false);
    let scaled_paths: Paths64 = scale_paths(paths, scale, scale, &mut error_code);
    if error_code != 0 {
        return PathsD::new();
    }
    clip_offset.add_paths(&scaled_paths, jt, et);
    let mut solution = Paths64::new();
    clip_offset.execute(delta * scale, &mut solution);
    scale_paths(&solution, 1.0 / scale, 1.0 / scale, &mut error_code)
}

// ============================================================================
// TranslatePath / TranslatePaths
// ============================================================================

/// Translate all points in a path by (dx, dy).
/// Direct port from clipper.h TranslatePath.
pub fn translate_path<T>(path: &Path<T>, dx: T, dy: T) -> Path<T>
where
    T: Copy + std::ops::Add<Output = T>,
{
    let mut result = Vec::with_capacity(path.len());
    for pt in path {
        result.push(Point {
            x: pt.x + dx,
            y: pt.y + dy,
        });
    }
    result
}

/// Translate all paths by (dx, dy).
/// Direct port from clipper.h TranslatePaths.
pub fn translate_paths<T>(paths: &Paths<T>, dx: T, dy: T) -> Paths<T>
where
    T: Copy + std::ops::Add<Output = T>,
{
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(translate_path(path, dx, dy));
    }
    result
}

// ============================================================================
// RectClip
// ============================================================================

/// Clip paths to a rectangle (Paths64).
/// Direct port from clipper.h RectClip.
pub fn rect_clip_64(rect: &Rect64, paths: &Paths64) -> Paths64 {
    if rect.is_empty() || paths.is_empty() {
        return Paths64::new();
    }
    let mut rc = RectClip64::new(*rect);
    rc.execute(paths)
}

/// Clip a single path to a rectangle (Paths64 output).
/// Direct port from clipper.h RectClip (single path overload).
pub fn rect_clip_path_64(rect: &Rect64, path: &Path64) -> Paths64 {
    if rect.is_empty() || path.is_empty() {
        return Paths64::new();
    }
    let mut rc = RectClip64::new(*rect);
    rc.execute(&vec![path.clone()])
}

/// Clip paths to a rectangle (PathsD).
/// Direct port from clipper.h RectClip (PathsD overload).
pub fn rect_clip_d(rect: &RectD, paths: &PathsD, precision: i32) -> PathsD {
    if rect.is_empty() || paths.is_empty() {
        return PathsD::new();
    }
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if error_code != 0 {
        return PathsD::new();
    }
    let scale = 10f64.powi(precision);
    let r: Rect64 = scale_rect(rect, scale);
    let mut rc = RectClip64::new(r);
    let pp: Paths64 = scale_paths(paths, scale, scale, &mut error_code);
    if error_code != 0 {
        return PathsD::new();
    }
    let result = rc.execute(&pp);
    scale_paths(&result, 1.0 / scale, 1.0 / scale, &mut error_code)
}

/// Clip a single path to a rectangle (PathsD).
/// Direct port from clipper.h RectClip (single PathD overload).
pub fn rect_clip_path_d(rect: &RectD, path: &PathD, precision: i32) -> PathsD {
    rect_clip_d(rect, &vec![path.clone()], precision)
}

// ============================================================================
// RectClipLines
// ============================================================================

/// Clip open lines to a rectangle (Paths64).
/// Direct port from clipper.h RectClipLines.
pub fn rect_clip_lines_64(rect: &Rect64, lines: &Paths64) -> Paths64 {
    if rect.is_empty() || lines.is_empty() {
        return Paths64::new();
    }
    let mut rcl = RectClipLines64::new(*rect);
    rcl.execute(lines)
}

/// Clip a single open line to a rectangle (Paths64 output).
/// Direct port from clipper.h RectClipLines (single path overload).
pub fn rect_clip_line_64(rect: &Rect64, line: &Path64) -> Paths64 {
    rect_clip_lines_64(rect, &vec![line.clone()])
}

/// Clip open lines to a rectangle (PathsD).
/// Direct port from clipper.h RectClipLines (PathsD overload).
pub fn rect_clip_lines_d(rect: &RectD, lines: &PathsD, precision: i32) -> PathsD {
    if rect.is_empty() || lines.is_empty() {
        return PathsD::new();
    }
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if error_code != 0 {
        return PathsD::new();
    }
    let scale = 10f64.powi(precision);
    let r: Rect64 = scale_rect(rect, scale);
    let mut rcl = RectClipLines64::new(r);
    let p: Paths64 = scale_paths(lines, scale, scale, &mut error_code);
    if error_code != 0 {
        return PathsD::new();
    }
    let result = rcl.execute(&p);
    scale_paths(&result, 1.0 / scale, 1.0 / scale, &mut error_code)
}

/// Clip a single open line to a rectangle (PathsD).
/// Direct port from clipper.h RectClipLines (single PathD overload).
pub fn rect_clip_line_d(rect: &RectD, line: &PathD, precision: i32) -> PathsD {
    rect_clip_lines_d(rect, &vec![line.clone()], precision)
}

// ============================================================================
// PolyTree conversion
// ============================================================================

/// Helper: recursively collect paths from a PolyPath64 node.
fn poly_path_to_paths64(tree: &PolyTree64, node_idx: usize, paths: &mut Paths64) {
    let polygon = tree.nodes[node_idx].polygon().clone();
    if !polygon.is_empty() {
        paths.push(polygon);
    }
    for &child_idx in tree.nodes[node_idx].children() {
        poly_path_to_paths64(tree, child_idx, paths);
    }
}

/// Helper: recursively collect paths from a PolyPathD node.
fn poly_path_to_paths_d(tree: &PolyTreeD, node_idx: usize, paths: &mut PathsD) {
    let polygon = tree.nodes[node_idx].polygon().clone();
    if !polygon.is_empty() {
        paths.push(polygon);
    }
    for &child_idx in tree.nodes[node_idx].children() {
        poly_path_to_paths_d(tree, child_idx, paths);
    }
}

/// Convert a PolyTree64 to a flat list of Paths64.
/// Direct port from clipper.h PolyTreeToPaths64.
pub fn poly_tree_to_paths64(polytree: &PolyTree64) -> Paths64 {
    let mut result = Paths64::new();
    let root = &polytree.nodes[0];
    for &child_idx in root.children() {
        poly_path_to_paths64(polytree, child_idx, &mut result);
    }
    result
}

/// Convert a PolyTreeD to a flat list of PathsD.
/// Direct port from clipper.h PolyTreeToPathsD.
pub fn poly_tree_to_paths_d(polytree: &PolyTreeD) -> PathsD {
    let mut result = PathsD::new();
    let root = &polytree.nodes[0];
    for &child_idx in root.children() {
        poly_path_to_paths_d(polytree, child_idx, &mut result);
    }
    result
}

/// Check that all children in a PolyTree64 are fully contained by their parents.
/// Direct port from clipper.h CheckPolytreeFullyContainsChildren.
pub fn check_polytree_fully_contains_children(polytree: &PolyTree64) -> bool {
    let root = &polytree.nodes[0];
    for &child_idx in root.children() {
        if polytree.nodes[child_idx].count() > 0
            && !poly_path64_contains_children(polytree, child_idx)
        {
            return false;
        }
    }
    true
}

/// Helper: check if a PolyPath64 node's children are all contained within it.
/// Direct port from clipper.h details::PolyPath64ContainsChildren.
fn poly_path64_contains_children(tree: &PolyTree64, node_idx: usize) -> bool {
    let parent_polygon = tree.nodes[node_idx].polygon();
    for &child_idx in tree.nodes[node_idx].children() {
        let child_polygon = tree.nodes[child_idx].polygon();
        // Return false if this child isn't fully contained by its parent.
        // Checking for a single vertex outside is a bit too crude since
        // it doesn't account for rounding errors. It's better to check
        // for consecutive vertices found outside the parent's polygon.
        let mut outside_cnt: i32 = 0;
        for pt in child_polygon {
            let result = point_in_polygon(*pt, parent_polygon);
            if result == PointInPolygonResult::IsInside {
                outside_cnt -= 1;
            } else if result == PointInPolygonResult::IsOutside {
                outside_cnt += 1;
            }
            if outside_cnt > 1 {
                return false;
            } else if outside_cnt < -1 {
                break;
            }
        }

        // Now check any nested children too
        if tree.nodes[child_idx].count() > 0 && !poly_path64_contains_children(tree, child_idx) {
            return false;
        }
    }
    true
}

// ============================================================================
// MakePath
// ============================================================================

/// Create a Path64 from a flat slice of coordinate pairs [x0, y0, x1, y1, ...].
/// Direct port from clipper.h MakePath.
pub fn make_path64(coords: &[i64]) -> Path64 {
    let size = coords.len() - coords.len() % 2;
    let mut result = Path64::with_capacity(size / 2);
    let mut i = 0;
    while i < size {
        result.push(Point64::new(coords[i], coords[i + 1]));
        i += 2;
    }
    result
}

/// Create a PathD from a flat slice of coordinate pairs [x0, y0, x1, y1, ...].
/// Direct port from clipper.h MakePathD.
pub fn make_path_d(coords: &[f64]) -> PathD {
    let size = coords.len() - coords.len() % 2;
    let mut result = PathD::with_capacity(size / 2);
    let mut i = 0;
    while i < size {
        result.push(Point::<f64>::new(coords[i], coords[i + 1]));
        i += 2;
    }
    result
}

// ============================================================================
// TrimCollinear
// ============================================================================

/// Remove collinear points from a Path64.
/// Direct port from clipper.h TrimCollinear.
pub fn trim_collinear_64(p: &Path64, is_open_path: bool) -> Path64 {
    let len = p.len();
    if len < 3 {
        if !is_open_path || len < 2 || p[0] == p[1] {
            return Path64::new();
        } else {
            return p.clone();
        }
    }

    let mut dst = Path64::with_capacity(len);
    let mut src_idx: usize = 0;
    let mut stop = len - 1;

    if !is_open_path {
        while src_idx != stop && is_collinear(p[stop], p[src_idx], p[src_idx + 1]) {
            src_idx += 1;
        }
        while src_idx != stop && is_collinear(p[stop - 1], p[stop], p[src_idx]) {
            stop -= 1;
        }
        if src_idx == stop {
            return Path64::new();
        }
    }

    let mut prev_idx = src_idx;
    dst.push(p[prev_idx]);
    src_idx += 1;

    while src_idx < stop {
        if !is_collinear(p[prev_idx], p[src_idx], p[src_idx + 1]) {
            prev_idx = src_idx;
            dst.push(p[prev_idx]);
        }
        src_idx += 1;
    }

    if is_open_path || !is_collinear(p[prev_idx], p[stop], dst[0]) {
        dst.push(p[stop]);
    } else {
        while dst.len() > 2 && is_collinear(dst[dst.len() - 1], dst[dst.len() - 2], dst[0]) {
            dst.pop();
        }
        if dst.len() < 3 {
            return Path64::new();
        }
    }
    dst
}

/// Remove collinear points from a PathD (scales to integer for precision).
/// Direct port from clipper.h TrimCollinear (PathD overload).
pub fn trim_collinear_d(path: &PathD, precision: i32, is_open_path: bool) -> PathD {
    let mut error_code = 0;
    let mut prec = precision;
    check_precision_range(&mut prec, &mut error_code);
    if error_code != 0 {
        return PathD::new();
    }
    let scale = 10f64.powi(precision);
    let p: Path64 = scale_path(path, scale, scale, &mut error_code);
    if error_code != 0 {
        return PathD::new();
    }
    let p = trim_collinear_64(&p, is_open_path);
    scale_path(&p, 1.0 / scale, 1.0 / scale, &mut error_code)
}

// ============================================================================
// Distance / Length
// ============================================================================

/// Compute the distance between two points.
/// Direct port from clipper.h Distance.
pub fn distance<T>(pt1: Point<T>, pt2: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    distance_sqr(pt1, pt2).sqrt()
}

/// Compute the total length of a path.
/// Direct port from clipper.h Length.
pub fn path_length<T>(path: &Path<T>, is_closed_path: bool) -> f64
where
    T: Copy + ToF64,
{
    let mut result = 0.0;
    if path.len() < 2 {
        return result;
    }
    for i in 0..path.len() - 1 {
        result += distance(path[i], path[i + 1]);
    }
    if is_closed_path {
        result += distance(path[path.len() - 1], path[0]);
    }
    result
}

// ============================================================================
// NearCollinear
// ============================================================================

/// Check if three points are nearly collinear within a tolerance.
/// Direct port from clipper.h NearCollinear.
pub fn near_collinear<T>(
    pt1: Point<T>,
    pt2: Point<T>,
    pt3: Point<T>,
    sin_sqrd_min_angle_rads: f64,
) -> bool
where
    T: Copy + ToF64,
{
    let cp = cross_product_three_points(pt1, pt2, pt3).abs();
    (cp * cp) / (distance_sqr(pt1, pt2) * distance_sqr(pt2, pt3)) < sin_sqrd_min_angle_rads
}

// ============================================================================
// SimplifyPath / SimplifyPaths
// ============================================================================

/// Helper: get next non-flagged index (wrapping).
fn get_next(current: usize, high: usize, flags: &[bool]) -> usize {
    let mut c = current + 1;
    while c <= high && flags[c] {
        c += 1;
    }
    if c <= high {
        return c;
    }
    c = 0;
    while flags[c] {
        c += 1;
    }
    c
}

/// Helper: get prior non-flagged index (wrapping).
fn get_prior(current: usize, high: usize, flags: &[bool]) -> usize {
    let mut c = if current == 0 { high } else { current - 1 };
    while c > 0 && flags[c] {
        c -= 1;
    }
    if !flags[c] {
        return c;
    }
    c = high;
    while flags[c] {
        c -= 1;
    }
    c
}

/// Simplify a path by removing vertices that are within epsilon distance
/// of an imaginary line between their neighbors.
/// Direct port from clipper.h SimplifyPath.
pub fn simplify_path<T>(path: &Path<T>, epsilon: f64, is_closed_path: bool) -> Path<T>
where
    T: Copy + ToF64 + FromF64 + Num + PartialEq,
{
    let len = path.len();
    if len < 4 {
        return path.clone();
    }
    let high = len - 1;
    let eps_sqr = sqr(epsilon);

    let mut flags = vec![false; len];
    let mut dist_sqr = vec![0.0f64; len];

    if is_closed_path {
        dist_sqr[0] = perpendic_dist_from_line_sqrd(path[0], path[high], path[1]);
        dist_sqr[high] = perpendic_dist_from_line_sqrd(path[high], path[0], path[high - 1]);
    } else {
        dist_sqr[0] = constants::MAX_DBL;
        dist_sqr[high] = constants::MAX_DBL;
    }
    for i in 1..high {
        dist_sqr[i] = perpendic_dist_from_line_sqrd(path[i], path[i - 1], path[i + 1]);
    }

    let mut curr = 0usize;
    loop {
        if dist_sqr[curr] > eps_sqr {
            let start = curr;
            loop {
                curr = get_next(curr, high, &flags);
                if curr == start || dist_sqr[curr] <= eps_sqr {
                    break;
                }
            }
            if curr == start {
                break;
            }
        }

        let prior = get_prior(curr, high, &flags);
        let mut next = get_next(curr, high, &flags);
        if next == prior {
            break;
        }

        let prior2;
        let prior_for_update;
        if dist_sqr[next] < dist_sqr[curr] {
            prior_for_update = curr;
            curr = next;
            next = get_next(next, high, &flags);
            prior2 = get_prior(prior_for_update, high, &flags);
        } else {
            prior_for_update = prior;
            prior2 = get_prior(prior, high, &flags);
        }

        flags[curr] = true;
        curr = next;
        next = get_next(next, high, &flags);

        if is_closed_path || (curr != high && curr != 0) {
            dist_sqr[curr] =
                perpendic_dist_from_line_sqrd(path[curr], path[prior_for_update], path[next]);
        }
        if is_closed_path || (prior_for_update != 0 && prior_for_update != high) {
            dist_sqr[prior_for_update] =
                perpendic_dist_from_line_sqrd(path[prior_for_update], path[prior2], path[curr]);
        }
    }

    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        if !flags[i] {
            result.push(path[i]);
        }
    }
    result
}

/// Simplify multiple paths.
/// Direct port from clipper.h SimplifyPaths.
pub fn simplify_paths<T>(paths: &Paths<T>, epsilon: f64, is_closed_path: bool) -> Paths<T>
where
    T: Copy + ToF64 + FromF64 + Num + PartialEq,
{
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(simplify_path(path, epsilon, is_closed_path));
    }
    result
}

// Note: path2_contains_path1 is already implemented in engine_fns.rs
// and re-exported from the crate root.

// ============================================================================
// Ramer-Douglas-Peucker
// ============================================================================

/// Recursive helper for Ramer-Douglas-Peucker algorithm.
/// Direct port from clipper.h RDP.
fn rdp<T>(path: &Path<T>, begin: usize, end: usize, eps_sqrd: f64, flags: &mut Vec<bool>)
where
    T: Copy + ToF64 + PartialEq,
{
    let mut idx = 0;
    let mut max_d = 0.0;
    let mut actual_end = end;

    // Handle duplicate endpoints
    while actual_end > begin && path[begin] == path[actual_end] {
        flags[actual_end] = false;
        actual_end -= 1;
    }

    for i in (begin + 1)..actual_end {
        let d = perpendic_dist_from_line_sqrd(path[i], path[begin], path[actual_end]);
        if d <= max_d {
            continue;
        }
        max_d = d;
        idx = i;
    }

    if max_d <= eps_sqrd {
        return;
    }

    flags[idx] = true;
    if idx > begin + 1 {
        rdp(path, begin, idx, eps_sqrd, flags);
    }
    if idx < actual_end - 1 {
        rdp(path, idx, actual_end, eps_sqrd, flags);
    }
}

/// Simplify a path using the Ramer-Douglas-Peucker algorithm.
/// Direct port from clipper.h RamerDouglasPeucker.
pub fn ramer_douglas_peucker<T>(path: &Path<T>, epsilon: f64) -> Path<T>
where
    T: Copy + ToF64 + PartialEq,
{
    let len = path.len();
    if len < 5 {
        return path.clone();
    }
    let mut flags = vec![false; len];
    flags[0] = true;
    flags[len - 1] = true;
    rdp(path, 0, len - 1, sqr(epsilon), &mut flags);
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        if flags[i] {
            result.push(path[i]);
        }
    }
    result
}

/// Simplify multiple paths using the Ramer-Douglas-Peucker algorithm.
/// Direct port from clipper.h RamerDouglasPeucker (Paths overload).
pub fn ramer_douglas_peucker_paths<T>(paths: &Paths<T>, epsilon: f64) -> Paths<T>
where
    T: Copy + ToF64 + PartialEq,
{
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(ramer_douglas_peucker(path, epsilon));
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "clipper_tests.rs"]
mod tests;
