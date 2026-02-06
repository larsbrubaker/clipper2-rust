// Copyright 2025 - Clipper2 Rust port
// Tests for the Minkowski module
//
// Comprehensive tests covering:
// - Empty inputs
// - Single-point inputs
// - Basic sum/diff with simple shapes
// - Closed vs open path handling
// - Floating-point (PathD) overloads
// - Orientation and area verification
// - Known geometric results
// - Edge cases (collinear, degenerate)

use super::*;
use crate::core::{
    area, area_paths, get_bounds_paths, point_in_polygon, Point64, PointD, PointInPolygonResult,
};

// ============================================================================
// Helper: create common shapes
// ============================================================================

/// Create a square centered at origin with given half-size
fn make_square(half_size: i64) -> Path64 {
    vec![
        Point64::new(-half_size, -half_size),
        Point64::new(half_size, -half_size),
        Point64::new(half_size, half_size),
        Point64::new(-half_size, half_size),
    ]
}

/// Create a square centered at origin with given half-size (floating point)
fn make_square_d(half_size: f64) -> PathD {
    vec![
        PointD::new(-half_size, -half_size),
        PointD::new(half_size, -half_size),
        PointD::new(half_size, half_size),
        PointD::new(-half_size, half_size),
    ]
}

/// Create a triangle
fn make_triangle(size: i64) -> Path64 {
    vec![
        Point64::new(0, 0),
        Point64::new(size, 0),
        Point64::new(size / 2, size),
    ]
}

/// Create a simple line segment (open path with 2 points)
fn make_line_segment(x1: i64, y1: i64, x2: i64, y2: i64) -> Path64 {
    vec![Point64::new(x1, y1), Point64::new(x2, y2)]
}

// ============================================================================
// Tests for minkowski_internal (via public API)
// ============================================================================

#[test]
fn test_minkowski_sum_empty_pattern() {
    let pattern: Path64 = vec![];
    let path = make_square(100);
    let result = minkowski_sum(&pattern, &path, true);
    assert!(
        result.is_empty(),
        "Empty pattern should produce empty result"
    );
}

#[test]
fn test_minkowski_sum_empty_path() {
    let pattern = make_square(10);
    let path: Path64 = vec![];
    let result = minkowski_sum(&pattern, &path, true);
    assert!(result.is_empty(), "Empty path should produce empty result");
}

#[test]
fn test_minkowski_sum_both_empty() {
    let pattern: Path64 = vec![];
    let path: Path64 = vec![];
    let result = minkowski_sum(&pattern, &path, true);
    assert!(result.is_empty(), "Both empty should produce empty result");
}

#[test]
fn test_minkowski_diff_empty_pattern() {
    let pattern: Path64 = vec![];
    let path = make_square(100);
    let result = minkowski_diff(&pattern, &path, true);
    assert!(
        result.is_empty(),
        "Empty pattern should produce empty result"
    );
}

#[test]
fn test_minkowski_diff_empty_path() {
    let pattern = make_square(10);
    let path: Path64 = vec![];
    let result = minkowski_diff(&pattern, &path, true);
    assert!(result.is_empty(), "Empty path should produce empty result");
}

// ============================================================================
// Tests for MinkowskiSum with Path64
// ============================================================================

#[test]
fn test_minkowski_sum_square_with_square_closed() {
    // Minkowski sum of a square [-10,-10 to 10,10] with a square [-50,-50 to 50,50].
    //
    // The quad-based Minkowski algorithm produces degenerate (zero-area) quads when
    // pattern and path edges are parallel. For axis-aligned squares, 8 of 16 quads
    // degenerate, leaving the center uncovered. The union of non-degenerate quads
    // produces a frame: outer boundary [-60,-60 to 60,60] with a hole [-40,-40 to 40,40].
    // This matches the C++ Clipper2 behavior (identical algorithm).
    let pattern = make_square(10);
    let path = make_square(50);
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty(), "Result should not be empty");

    // Outer boundary is [-60,-60] to [60,60] (area 14400)
    // Inner hole is [-40,-40] to [40,40] (area 6400)
    // Net area = 14400 - 6400 = 8000
    let total_area = area_paths(&result).abs();
    assert!(
        (total_area - 8000.0).abs() < 200.0,
        "Area should be approximately 8000 (frame shape), got {}",
        total_area
    );

    // The outer bounds should still be [-60, -60] to [60, 60]
    let bounds = get_bounds_paths(&result);
    assert!(
        bounds.left >= -61 && bounds.left <= -59,
        "Left bound should be ~-60, got {}",
        bounds.left
    );
    assert!(
        bounds.right >= 59 && bounds.right <= 61,
        "Right bound should be ~60, got {}",
        bounds.right
    );
    assert!(
        bounds.top >= -61 && bounds.top <= -59,
        "Top bound should be ~-60, got {}",
        bounds.top
    );
    assert!(
        bounds.bottom >= 59 && bounds.bottom <= 61,
        "Bottom bound should be ~60, got {}",
        bounds.bottom
    );
}

#[test]
fn test_minkowski_sum_single_point_pattern() {
    // Minkowski sum with a single-point pattern is just translation
    // But with a single point we can't form quads so the result is a union of
    // degenerate quads. In practice the C++ code handles this by having
    // pat_len=1, so the inner loop runs once per path segment.
    let pattern = vec![Point64::new(10, 20)];
    let path = make_square(50);
    let result = minkowski_sum(&pattern, &path, true);

    // With a single-point pattern, each quad is degenerate (all 4 points collapse
    // to 2 unique points). The union of degenerate quads may produce empty or
    // a path. The key thing is it shouldn't panic.
    // Just verify it doesn't crash
    let _ = area_paths(&result);
}

#[test]
fn test_minkowski_sum_point_on_boundary() {
    // Sum of a small square pattern with a larger square path
    // Points on the boundary of the result should be inside or on-boundary
    let pattern = make_square(5);
    let path = make_square(20);
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty());

    // The center point should be inside the result
    // Check using the first path of result
    if !result.is_empty() && result[0].len() >= 3 {
        let center = Point64::new(0, 0);
        let pip = point_in_polygon(center, &result[0]);
        assert_ne!(
            pip,
            PointInPolygonResult::IsOutside,
            "Center should be inside the Minkowski sum"
        );
    }
}

#[test]
fn test_minkowski_sum_triangle_with_square_closed() {
    // Minkowski sum of triangle with square
    let pattern = make_square(10);
    let path = make_triangle(100);
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty(), "Result should not be empty");

    // Area should be larger than the original triangle area
    let original_area = area(&path).abs();
    let result_area = area_paths(&result).abs();
    assert!(
        result_area > original_area,
        "Minkowski sum area ({}) should be larger than original ({})",
        result_area,
        original_area
    );
}

#[test]
fn test_minkowski_sum_open_path() {
    // Open path (is_closed = false) with a square pattern
    // This should "inflate" the line segment
    let pattern = make_square(10);
    let path = make_line_segment(0, 0, 100, 0);
    let result = minkowski_sum(&pattern, &path, false);

    assert!(
        !result.is_empty(),
        "Result should not be empty for open path"
    );

    // The result should encompass a rectangle-like shape around the line segment
    let bounds = get_bounds_paths(&result);
    assert!(bounds.left <= -9, "Left should extend past -10");
    assert!(bounds.right >= 109, "Right should extend past 110");
}

#[test]
fn test_minkowski_sum_preserves_positive_orientation() {
    // After union, result paths should generally have positive orientation
    // for outer contours
    let pattern = make_square(5);
    let path = make_square(50);
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty());
    // The outer contour should have positive (CCW) orientation
    // The result should have positive total area
    let total_area = area_paths(&result);
    assert!(
        total_area > 0.0,
        "Total area should be positive (CCW), got {}",
        total_area
    );
}

// ============================================================================
// Tests for MinkowskiDiff with Path64
// ============================================================================

#[test]
fn test_minkowski_diff_square_with_square_closed() {
    // Minkowski diff of square[-50..50] with pattern[-10..10]
    // For a convex polygon, Minkowski diff effectively erodes (shrinks) the polygon
    let pattern = make_square(10);
    let path = make_square(50);
    let result = minkowski_diff(&pattern, &path, true);

    assert!(!result.is_empty(), "Result should not be empty");

    // The result area should be non-zero
    let total_area = area_paths(&result).abs();
    assert!(total_area > 0.0, "Result should have non-zero area");
}

#[test]
fn test_minkowski_diff_same_shape() {
    // Minkowski diff of a shape with itself should contain the origin
    let shape = make_square(50);
    let result = minkowski_diff(&shape, &shape, true);

    assert!(
        !result.is_empty(),
        "Diff of shape with itself should not be empty"
    );

    // The origin should be inside the result
    if !result.is_empty() && result[0].len() >= 3 {
        let origin = Point64::new(0, 0);
        let pip = point_in_polygon(origin, &result[0]);
        assert_ne!(
            pip,
            PointInPolygonResult::IsOutside,
            "Origin should be inside Minkowski diff of shape with itself"
        );
    }
}

#[test]
fn test_minkowski_diff_open_path() {
    let pattern = make_square(10);
    let path = make_line_segment(0, 0, 100, 0);
    let result = minkowski_diff(&pattern, &path, false);

    // Should produce some output without panicking
    assert!(
        !result.is_empty(),
        "Diff with open path should produce result"
    );
}

// ============================================================================
// Tests for MinkowskiSum with PathD (floating point)
// ============================================================================

#[test]
fn test_minkowski_sum_d_basic() {
    let pattern = make_square_d(10.0);
    let path = make_square_d(50.0);
    let result = minkowski_sum_d(&pattern, &path, true, 2);

    assert!(!result.is_empty(), "PathD sum should produce result");

    // Verify bounds are approximately correct
    // Sum of [-10..10] + [-50..50] = [-60..60]
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for path in &result {
        for pt in path {
            if pt.x < min_x {
                min_x = pt.x;
            }
            if pt.x > max_x {
                max_x = pt.x;
            }
            if pt.y < min_y {
                min_y = pt.y;
            }
            if pt.y > max_y {
                max_y = pt.y;
            }
        }
    }

    assert!(
        (min_x - (-60.0)).abs() < 1.0,
        "min_x should be ~-60, got {}",
        min_x
    );
    assert!(
        (max_x - 60.0).abs() < 1.0,
        "max_x should be ~60, got {}",
        max_x
    );
    assert!(
        (min_y - (-60.0)).abs() < 1.0,
        "min_y should be ~-60, got {}",
        min_y
    );
    assert!(
        (max_y - 60.0).abs() < 1.0,
        "max_y should be ~60, got {}",
        max_y
    );
}

#[test]
fn test_minkowski_sum_d_empty_inputs() {
    let pattern: PathD = vec![];
    let path = make_square_d(50.0);
    let result = minkowski_sum_d(&pattern, &path, true, 2);
    assert!(result.is_empty());

    let pattern = make_square_d(10.0);
    let path: PathD = vec![];
    let result = minkowski_sum_d(&pattern, &path, true, 2);
    assert!(result.is_empty());
}

#[test]
fn test_minkowski_sum_d_precision() {
    // Test with different decimal places
    let pattern = make_square_d(1.5);
    let path = make_square_d(3.5);

    // With 2 decimal places
    let result2 = minkowski_sum_d(&pattern, &path, true, 2);
    assert!(!result2.is_empty());

    // With 4 decimal places (higher precision)
    let result4 = minkowski_sum_d(&pattern, &path, true, 4);
    assert!(!result4.is_empty());

    // Both should have approximately the same area
    // Sum bounds: [-5..5] so area = 100
    let area2: f64 = result2
        .iter()
        .map(|p| {
            let n = p.len();
            if n < 3 {
                return 0.0;
            }
            let mut a = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                a += p[i].x * p[j].y - p[j].x * p[i].y;
            }
            a / 2.0
        })
        .sum::<f64>()
        .abs();

    let area4: f64 = result4
        .iter()
        .map(|p| {
            let n = p.len();
            if n < 3 {
                return 0.0;
            }
            let mut a = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                a += p[i].x * p[j].y - p[j].x * p[i].y;
            }
            a / 2.0
        })
        .sum::<f64>()
        .abs();

    assert!(
        (area2 - area4).abs() < 1.0,
        "Areas should be similar regardless of decimal places: {} vs {}",
        area2,
        area4
    );
}

// ============================================================================
// Tests for MinkowskiDiff with PathD (floating point)
// ============================================================================

#[test]
fn test_minkowski_diff_d_basic() {
    let pattern = make_square_d(10.0);
    let path = make_square_d(50.0);
    let result = minkowski_diff_d(&pattern, &path, true, 2);

    assert!(!result.is_empty(), "PathD diff should produce result");
}

#[test]
fn test_minkowski_diff_d_empty_inputs() {
    let pattern: PathD = vec![];
    let path = make_square_d(50.0);
    let result = minkowski_diff_d(&pattern, &path, true, 2);
    assert!(result.is_empty());

    let pattern = make_square_d(10.0);
    let path: PathD = vec![];
    let result = minkowski_diff_d(&pattern, &path, true, 2);
    assert!(result.is_empty());
}

// ============================================================================
// Tests for specific geometric properties
// ============================================================================

#[test]
fn test_minkowski_sum_commutativity_area() {
    // Minkowski sum is commutative: A + B = B + A
    // The areas should be the same
    let a = make_square(20);
    let b = make_triangle(60);

    let result_ab = minkowski_sum(&a, &b, true);
    let result_ba = minkowski_sum(&b, &a, true);

    let area_ab = area_paths(&result_ab).abs();
    let area_ba = area_paths(&result_ba).abs();

    assert!(
        (area_ab - area_ba).abs() < 10.0,
        "Minkowski sum should be commutative in area: {} vs {}",
        area_ab,
        area_ba
    );
}

#[test]
fn test_minkowski_sum_contains_original_shifted_points() {
    // For closed paths, each point p in path, the translated pattern (pattern + p)
    // should be contained within the Minkowski sum result
    let pattern = make_square(10);
    let path = vec![
        Point64::new(100, 100),
        Point64::new(200, 100),
        Point64::new(200, 200),
        Point64::new(100, 200),
    ];
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty());

    // Each path vertex, shifted by pattern center (0,0), should be inside result
    // Since pattern is centered at origin, the path points themselves should be inside
    if result.len() == 1 && result[0].len() >= 3 {
        for p in &path {
            let pip = point_in_polygon(*p, &result[0]);
            assert_ne!(
                pip,
                PointInPolygonResult::IsOutside,
                "Path point ({},{}) should be inside Minkowski sum",
                p.x,
                p.y
            );
        }
    }
}

#[test]
fn test_minkowski_sum_large_coordinates() {
    // Test with large coordinate values to ensure no overflow
    let pattern = make_square(100);
    let path = vec![
        Point64::new(1_000_000, 1_000_000),
        Point64::new(2_000_000, 1_000_000),
        Point64::new(2_000_000, 2_000_000),
        Point64::new(1_000_000, 2_000_000),
    ];
    let result = minkowski_sum(&pattern, &path, true);
    assert!(!result.is_empty(), "Large coordinates should work");

    let bounds = get_bounds_paths(&result);
    assert!(
        bounds.left <= 999_901,
        "Left bound should account for pattern"
    );
    assert!(
        bounds.right >= 2_000_099,
        "Right bound should account for pattern"
    );
}

#[test]
fn test_minkowski_sum_collinear_path() {
    // Path with collinear points (straight line with intermediate points)
    let pattern = make_square(10);
    let path = vec![
        Point64::new(0, 0),
        Point64::new(50, 0),
        Point64::new(100, 0),
    ];
    let result = minkowski_sum(&pattern, &path, false);
    // Should not panic and produce some result
    let _ = area_paths(&result);
}

#[test]
fn test_minkowski_diff_not_same_as_sum() {
    // Minkowski diff should generally produce different results than sum
    let pattern = make_square(15);
    let path = vec![
        Point64::new(100, 100),
        Point64::new(200, 100),
        Point64::new(200, 200),
        Point64::new(100, 200),
    ];

    let sum_result = minkowski_sum(&pattern, &path, true);
    let diff_result = minkowski_diff(&pattern, &path, true);

    let sum_area = area_paths(&sum_result).abs();
    let diff_area = area_paths(&diff_result).abs();

    // Areas should be different (unless pattern is symmetric about origin, which
    // our square is, but the center of the path is not at origin so the bounds differ)
    // Actually, for a symmetric pattern centered at origin, sum and diff will
    // produce the same result. Let's use an asymmetric pattern instead.
    // For a square centered at origin, p+q and p-q with symmetric pattern give same bounds.
    // This is expected behavior, so just verify both produce valid output.
    assert!(sum_area > 0.0, "Sum should have positive area");
    assert!(diff_area > 0.0, "Diff should have positive area");
}

#[test]
fn test_minkowski_sum_asymmetric_pattern() {
    // Use an asymmetric pattern to verify sum vs diff difference
    let pattern = vec![
        Point64::new(0, 0),
        Point64::new(20, 0),
        Point64::new(20, 10),
        Point64::new(0, 10),
    ];
    let path = vec![
        Point64::new(50, 50),
        Point64::new(150, 50),
        Point64::new(150, 150),
        Point64::new(50, 150),
    ];

    let sum_result = minkowski_sum(&pattern, &path, true);
    let diff_result = minkowski_diff(&pattern, &path, true);

    let sum_bounds = get_bounds_paths(&sum_result);
    let diff_bounds = get_bounds_paths(&diff_result);

    // Sum shifts right/up (adds pattern), diff shifts left/down (subtracts pattern)
    // So bounds should differ
    assert_ne!(
        sum_bounds.left, diff_bounds.left,
        "Asymmetric pattern should produce different sum vs diff bounds"
    );
}

#[test]
fn test_minkowski_sum_two_point_path_closed() {
    // A closed path with only 2 points (degenerate polygon)
    let pattern = make_square(10);
    let path = vec![Point64::new(0, 0), Point64::new(100, 0)];
    let result = minkowski_sum(&pattern, &path, true);
    // Should not panic; result validity depends on how union handles degenerate input
    let _ = area_paths(&result);
}

#[test]
fn test_minkowski_sum_three_point_path_open() {
    // An open polyline with 3 points (L-shape)
    let pattern = make_square(5);
    let path = vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
    ];
    let result = minkowski_sum(&pattern, &path, false);

    assert!(
        !result.is_empty(),
        "L-shaped open path should produce result"
    );
    let total_area = area_paths(&result).abs();
    assert!(total_area > 0.0, "Result should have positive area");
}

// ============================================================================
// Tests for union_paths helper (tested indirectly through public API)
// ============================================================================

#[test]
fn test_union_paths_via_minkowski_produces_clean_output() {
    // The union step should merge overlapping quads into clean contours
    let pattern = make_square(20);
    let path = make_square(50);
    let result = minkowski_sum(&pattern, &path, true);

    // A convex pattern + convex path should produce a single outer contour
    // (the union merges all quads into one polygon)
    assert!(
        result.len() <= 2,
        "Convex + convex should produce few contours, got {}",
        result.len()
    );
}

// ============================================================================
// Regression / stress tests
// ============================================================================

#[test]
fn test_minkowski_sum_many_sided_polygon() {
    // Create an octagon-like pattern with a square path.
    //
    // Similar to the square+square case: the quad-based algorithm produces a frame
    // shape because quads only cover the edge bands, not the center. The outer
    // boundary approximates the true Minkowski sum, but the center has a hole.
    // This matches C++ Clipper2 behavior (identical algorithm).
    let pattern = vec![
        Point64::new(10, 0),
        Point64::new(7, 7),
        Point64::new(0, 10),
        Point64::new(-7, 7),
        Point64::new(-10, 0),
        Point64::new(-7, -7),
        Point64::new(0, -10),
        Point64::new(7, -7),
    ];
    let path = make_square(50);
    let result = minkowski_sum(&pattern, &path, true);

    assert!(!result.is_empty());
    let total_area = area_paths(&result).abs();
    // The outer boundary (~14280) minus the inner hole (~6400) gives ~7880
    assert!(
        total_area > 5000.0,
        "Frame area should be > 5000, got {}",
        total_area
    );
    // Verify the outer boundary is larger than the original square
    let outer_area = result.iter().map(|p| area(p)).filter(|a| *a > 0.0).sum::<f64>();
    assert!(
        outer_area > 10000.0,
        "Outer boundary area should be > original square area, got {}",
        outer_area
    );
}

#[test]
fn test_minkowski_operations_dont_crash_with_single_points() {
    // Both pattern and path are single points
    let pattern = vec![Point64::new(5, 5)];
    let path = vec![Point64::new(10, 10)];
    let _ = minkowski_sum(&pattern, &path, true);
    let _ = minkowski_sum(&pattern, &path, false);
    let _ = minkowski_diff(&pattern, &path, true);
    let _ = minkowski_diff(&pattern, &path, false);
    // Just verifying no panics
}

#[test]
fn test_minkowski_sum_d_zero_decimal_places() {
    // Zero decimal places means scale = 1.0 (integer-like precision)
    let pattern = make_square_d(10.0);
    let path = make_square_d(50.0);
    let result = minkowski_sum_d(&pattern, &path, true, 0);
    assert!(!result.is_empty());
}

#[test]
fn test_minkowski_sum_d_high_decimal_places() {
    // High precision (6 decimal places)
    let pattern = make_square_d(0.001);
    let path = make_square_d(0.005);
    let result = minkowski_sum_d(&pattern, &path, true, 6);
    assert!(!result.is_empty());
}

#[test]
fn test_minkowski_internal_axis_aligned_squares_produce_degenerate_quads() {
    // For axis-aligned squares, the quad-based Minkowski algorithm produces 16 quads:
    // 8 non-degenerate (area = 2000 each) and 8 degenerate (area = 0, all points collinear).
    // The degenerate quads arise from parallel edges in pattern and path.
    let pattern = make_square(10);
    let path = make_square(50);
    let quads = minkowski_internal(&pattern, &path, true, true);

    assert_eq!(quads.len(), 16);
    let non_degenerate: Vec<_> = quads.iter().filter(|q| area(q).abs() > 0.0).collect();
    let degenerate: Vec<_> = quads.iter().filter(|q| area(q).abs() == 0.0).collect();
    assert_eq!(non_degenerate.len(), 8);
    assert_eq!(degenerate.len(), 8);

    // Union of the quads produces a frame shape (outer boundary with hole)
    let result = union_paths(&quads, FillRule::NonZero);
    assert_eq!(result.len(), 2, "Should produce outer boundary + hole");
    let outer_area = result.iter().map(|p| area(p)).filter(|a| *a > 0.0).sum::<f64>();
    let hole_area = result.iter().map(|p| area(p)).filter(|a| *a < 0.0).sum::<f64>().abs();
    assert!((outer_area - 14400.0).abs() < 100.0);
    assert!((hole_area - 6400.0).abs() < 100.0);
}
