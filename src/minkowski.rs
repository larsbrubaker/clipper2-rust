// Copyright 2025 - Clipper2 Rust port
// Ported from clipper.minkowski.h by Angus Johnson
// Original Copyright: Angus Johnson 2010-2023
// License: https://www.boost.org/LICENSE_1_0.txt
//
// Purpose: Minkowski Sum and Difference operations

use crate::core::{is_positive, scale_path, scale_paths, Path64, PathD, Paths64, PathsD};
use crate::engine::ClipType;
use crate::engine_public::Clipper64;
use crate::FillRule;

// ============================================================================
// Internal helper functions (equivalent to C++ detail namespace)
// ============================================================================

/// Core Minkowski operation that generates quads from pattern translated along path.
///
/// Direct port from C++ detail::Minkowski (clipper.minkowski.h lines 20-72).
///
/// For each point on `path`, translates the `pattern` (adding or subtracting
/// depending on `is_sum`), then builds quadrilateral polygons connecting
/// adjacent translated copies. The resulting quads are ensured to have
/// positive orientation.
///
/// # Arguments
/// * `pattern` - The pattern path to convolve
/// * `path` - The path along which the pattern is translated
/// * `is_sum` - If true, computes sum (p + pattern); if false, computes difference (p - pattern)
/// * `is_closed` - If true, the path is treated as closed (last point connects to first)
fn minkowski_internal(pattern: &Path64, path: &Path64, is_sum: bool, is_closed: bool) -> Paths64 {
    let delta: usize = if is_closed { 0 } else { 1 };
    let pat_len = pattern.len();
    let path_len = path.len();

    if pat_len == 0 || path_len == 0 {
        return Paths64::new();
    }

    // Build translated copies of pattern at each path point
    let mut tmp: Vec<Path64> = Vec::with_capacity(path_len);

    if is_sum {
        for p in path.iter() {
            let path2: Path64 = pattern.iter().map(|pt2| *p + *pt2).collect();
            tmp.push(path2);
        }
    } else {
        for p in path.iter() {
            let path2: Path64 = pattern.iter().map(|pt2| *p - *pt2).collect();
            tmp.push(path2);
        }
    }

    // Build quad polygons connecting adjacent translated copies
    // Each quad connects: tmp[g][h], tmp[i][h], tmp[i][j], tmp[g][j]
    let result_capacity = (path_len - delta) * pat_len;
    let mut result: Paths64 = Vec::with_capacity(result_capacity);

    let mut g: usize = if is_closed { path_len - 1 } else { 0 };

    let mut i = delta;
    while i < path_len {
        let mut h: usize = pat_len - 1;
        for j in 0..pat_len {
            let mut quad: Path64 = Vec::with_capacity(4);
            quad.push(tmp[g][h]);
            quad.push(tmp[i][h]);
            quad.push(tmp[i][j]);
            quad.push(tmp[g][j]);

            if !is_positive(&quad) {
                quad.reverse();
            }
            result.push(quad);
            h = j;
        }
        g = i;
        i += 1;
    }

    result
}

/// Union a set of paths using the clipping engine.
///
/// Direct port from C++ detail::Union (clipper.minkowski.h lines 74-81).
///
/// # Arguments
/// * `subjects` - The paths to union
/// * `fill_rule` - The fill rule to use for the union operation
fn union_paths(subjects: &Paths64, fill_rule: FillRule) -> Paths64 {
    let mut result = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(subjects);
    clipper.execute(ClipType::Union, fill_rule, &mut result, None);
    result
}

// ============================================================================
// Public API functions
// ============================================================================

/// Compute the Minkowski Sum of a pattern and path using integer coordinates.
///
/// Direct port from C++ MinkowskiSum (Path64 overload, clipper.minkowski.h lines 85-88).
///
/// The Minkowski Sum is the set of all points that are the sum of any point
/// in the pattern and any point in the path. Geometrically, it can be thought
/// of as sweeping the pattern along the path.
///
/// # Arguments
/// * `pattern` - The pattern path (typically a small convex polygon)
/// * `path` - The path along which to sweep
/// * `is_closed` - Whether the path should be treated as closed
///
/// # Returns
/// The Minkowski sum as a set of paths (unioned into a clean result)
pub fn minkowski_sum(pattern: &Path64, path: &Path64, is_closed: bool) -> Paths64 {
    union_paths(
        &minkowski_internal(pattern, path, true, is_closed),
        FillRule::NonZero,
    )
}

/// Compute the Minkowski Sum of a pattern and path using floating-point coordinates.
///
/// Direct port from C++ MinkowskiSum (PathD overload, clipper.minkowski.h lines 90-98).
///
/// Internally scales to integer coordinates, performs the operation, then scales back.
///
/// # Arguments
/// * `pattern` - The pattern path in floating-point coordinates
/// * `path` - The path along which to sweep
/// * `is_closed` - Whether the path should be treated as closed
/// * `decimal_places` - Number of decimal places of precision (default 2 in C++)
///
/// # Returns
/// The Minkowski sum as a set of paths in floating-point coordinates
pub fn minkowski_sum_d(
    pattern: &PathD,
    path: &PathD,
    is_closed: bool,
    decimal_places: i32,
) -> PathsD {
    let mut error_code: i32 = 0;
    let scale = 10f64.powi(decimal_places);

    let pat64: Path64 = scale_path(pattern, scale, scale, &mut error_code);
    let path64: Path64 = scale_path(path, scale, scale, &mut error_code);

    let tmp = union_paths(
        &minkowski_internal(&pat64, &path64, true, is_closed),
        FillRule::NonZero,
    );

    let inv_scale = 1.0 / scale;
    scale_paths(&tmp, inv_scale, inv_scale, &mut error_code)
}

/// Compute the Minkowski Difference of a pattern and path using integer coordinates.
///
/// Direct port from C++ MinkowskiDiff (Path64 overload, clipper.minkowski.h lines 100-103).
///
/// The Minkowski Difference is similar to the Minkowski Sum but subtracts
/// pattern points from path points instead of adding them.
///
/// # Arguments
/// * `pattern` - The pattern path
/// * `path` - The path from which to subtract
/// * `is_closed` - Whether the path should be treated as closed
///
/// # Returns
/// The Minkowski difference as a set of paths (unioned into a clean result)
pub fn minkowski_diff(pattern: &Path64, path: &Path64, is_closed: bool) -> Paths64 {
    union_paths(
        &minkowski_internal(pattern, path, false, is_closed),
        FillRule::NonZero,
    )
}

/// Compute the Minkowski Difference of a pattern and path using floating-point coordinates.
///
/// Direct port from C++ MinkowskiDiff (PathD overload, clipper.minkowski.h lines 105-113).
///
/// Internally scales to integer coordinates, performs the operation, then scales back.
///
/// # Arguments
/// * `pattern` - The pattern path in floating-point coordinates
/// * `path` - The path from which to subtract
/// * `is_closed` - Whether the path should be treated as closed
/// * `decimal_places` - Number of decimal places of precision (default 2 in C++)
///
/// # Returns
/// The Minkowski difference as a set of paths in floating-point coordinates
pub fn minkowski_diff_d(
    pattern: &PathD,
    path: &PathD,
    is_closed: bool,
    decimal_places: i32,
) -> PathsD {
    let mut error_code: i32 = 0;
    let scale = 10f64.powi(decimal_places);

    let pat64: Path64 = scale_path(pattern, scale, scale, &mut error_code);
    let path64: Path64 = scale_path(path, scale, scale, &mut error_code);

    let tmp = union_paths(
        &minkowski_internal(&pat64, &path64, false, is_closed),
        FillRule::NonZero,
    );

    let inv_scale = 1.0 / scale;
    scale_paths(&tmp, inv_scale, inv_scale, &mut error_code)
}

#[cfg(test)]
#[path = "minkowski_tests.rs"]
mod tests;
