use super::*;

#[test]
fn test_fill_rule_default() {
    assert_eq!(FillRule::default(), FillRule::EvenOdd);
}

#[test]
fn test_fill_rule_variants() {
    let rules = [
        FillRule::EvenOdd,
        FillRule::NonZero,
        FillRule::Positive,
        FillRule::Negative,
    ];
    assert_eq!(rules.len(), 4);

    // Test each variant is unique
    for i in 0..rules.len() {
        for j in (i + 1)..rules.len() {
            assert_ne!(rules[i], rules[j]);
        }
    }
}

#[test]
fn test_clipper2_exception() {
    let err = Clipper2Exception::new("test error");
    assert_eq!(err.description(), "test error");
    assert_eq!(err.to_string(), "Clipper2Exception: test error");
}

#[test]
fn test_point_creation() {
    let p1 = Point::new(10i32, 20i32);
    assert_eq!(p1.x, 10);
    assert_eq!(p1.y, 20);

    let p2 = Point::<f64>::zero();
    assert_eq!(p2.x, 0.0);
    assert_eq!(p2.y, 0.0);
}

#[test]
fn test_point_operations() {
    let p1 = Point::new(10i32, 20i32);
    let p2 = Point::new(5i32, 15i32);

    let sum = p1 + p2;
    assert_eq!(sum.x, 15);
    assert_eq!(sum.y, 35);

    let diff = p1 - p2;
    assert_eq!(diff.x, 5);
    assert_eq!(diff.y, 5);

    let neg = -p1;
    assert_eq!(neg.x, -10);
    assert_eq!(neg.y, -20);
}

#[test]
fn test_point_scale() {
    let p1 = Point::new(10i32, 20i32);
    let scaled = p1.scale(2.5f64);
    assert_eq!(scaled.x, 25.0);
    assert_eq!(scaled.y, 50.0);
}

#[test]
fn test_rect_creation() {
    let rect = Rect::new(0i32, 0i32, 100i32, 200i32);
    assert_eq!(rect.left, 0);
    assert_eq!(rect.top, 0);
    assert_eq!(rect.right, 100);
    assert_eq!(rect.bottom, 200);
}

#[test]
fn test_rect_properties() {
    let rect = Rect::new(10i32, 20i32, 110i32, 220i32);

    assert!(rect.is_valid());
    assert_eq!(rect.width(), 100);
    assert_eq!(rect.height(), 200);
    assert!(!rect.is_empty());

    let empty_rect = Rect::new(100i32, 200i32, 50i32, 150i32);
    assert!(empty_rect.is_valid()); // Geometrically invalid but not sentinel invalid
    assert!(empty_rect.is_empty());
}

#[test]
fn test_rect_modification() {
    let mut rect = Rect::new(0i32, 0i32, 50i32, 100i32);

    rect.set_width(200);
    assert_eq!(rect.right, 200);
    assert_eq!(rect.width(), 200);

    rect.set_height(300);
    assert_eq!(rect.bottom, 300);
    assert_eq!(rect.height(), 300);
}

#[test]
fn test_rect_scale() {
    let mut rect = RectD::new(10.0, 20.0, 30.0, 40.0);
    rect.scale(2.0);
    assert_eq!(rect.left, 20.0);
    assert_eq!(rect.top, 40.0);
    assert_eq!(rect.right, 60.0);
    assert_eq!(rect.bottom, 80.0);
}

#[test]
fn test_type_aliases() {
    let p64 = Point64::new(100, 200);
    let pd = PointD::new(10.5, 20.5);
    let r64 = Rect64::new(0, 0, 100, 200);
    let rd = RectD::new(0.0, 0.0, 100.0, 200.0);

    assert_eq!(p64.x, 100);
    assert_eq!(pd.x, 10.5);
    assert_eq!(r64.width(), 100);
    assert_eq!(rd.width(), 100.0);
}

#[test]
fn test_path_types() {
    let mut path64: Path64 = vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
    ];

    path64.push(Point64::new(0, 100));
    assert_eq!(path64.len(), 4);

    let paths64: Paths64 = vec![path64];
    assert_eq!(paths64.len(), 1);
    assert_eq!(paths64[0].len(), 4);
}

#[test]
fn test_invalid_points() {
    assert_eq!(INVALID_POINT64.x, i64::MAX);
    assert_eq!(INVALID_POINT64.y, i64::MAX);
    assert_eq!(INVALID_POINTD.x, f64::MAX);
    assert_eq!(INVALID_POINTD.y, f64::MAX);
}

#[test]
fn test_mid_point() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(100, 200);
    let mid = mid_point(p1, p2);
    assert_eq!(mid.x, 50);
    assert_eq!(mid.y, 100);

    let p3 = PointD::new(1.0, 3.0);
    let p4 = PointD::new(5.0, 7.0);
    let mid2 = mid_point(p3, p4);
    assert_eq!(mid2.x, 3.0);
    assert_eq!(mid2.y, 5.0);
}

#[test]
fn test_cross_product_three_points() {
    // Test with points that form a right turn (negative cross product)
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(1, 0);
    let p3 = Point64::new(1, 1);
    let cross = cross_product_three_points(p1, p2, p3);
    assert_eq!(cross, 1.0);

    // Test with points that form a left turn (positive cross product)
    let p4 = Point64::new(0, 0);
    let p5 = Point64::new(1, 0);
    let p6 = Point64::new(1, -1);
    let cross2 = cross_product_three_points(p4, p5, p6);
    assert_eq!(cross2, -1.0);

    // Test collinear points (zero cross product)
    let p7 = Point64::new(0, 0);
    let p8 = Point64::new(1, 1);
    let p9 = Point64::new(2, 2);
    let cross3 = cross_product_three_points(p7, p8, p9);
    assert_eq!(cross3, 0.0);
}

#[test]
fn test_cross_product_two_vectors() {
    let vec1 = Point64::new(1, 0);
    let vec2 = Point64::new(0, 1);
    let cross = cross_product_two_vectors(vec1, vec2);

    // Check actual calculation: vec1.y * vec2.x - vec2.y * vec1.x = 0*0 - 1*1 = -1
    assert_eq!(cross, -1.0);

    let vec3 = Point64::new(2, 3);
    let vec4 = Point64::new(4, 5);
    let cross2 = cross_product_two_vectors(vec3, vec4);
    // vec3.y * vec4.x - vec4.y * vec3.x = 3*4 - 5*2 = 12 - 10 = 2
    assert_eq!(cross2, 2.0);
}

#[test]
fn test_dot_product_three_points() {
    // Test with perpendicular vectors (dot product = 0)
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(1, 0);
    let p3 = Point64::new(1, 1);
    let dot = dot_product_three_points(p1, p2, p3);
    assert_eq!(dot, 0.0); // (1,0) . (0,1) = 0

    // Test with parallel vectors (positive dot product)
    let p4 = Point64::new(0, 0);
    let p5 = Point64::new(1, 0);
    let p6 = Point64::new(2, 0);
    let dot2 = dot_product_three_points(p4, p5, p6);
    assert_eq!(dot2, 1.0); // (1,0) . (1,0) = 1

    // Test with opposite vectors (negative dot product)
    let p7 = Point64::new(0, 0);
    let p8 = Point64::new(1, 0);
    let p9 = Point64::new(0, 0);
    let dot3 = dot_product_three_points(p7, p8, p9);
    assert_eq!(dot3, -1.0); // (1,0) . (-1,0) = -1
}

#[test]
fn test_dot_product_two_vectors() {
    let vec1 = Point64::new(3, 4);
    let vec2 = Point64::new(2, 1);
    let dot = dot_product_two_vectors(vec1, vec2);
    assert_eq!(dot, 10.0); // 3*2 + 4*1 = 6 + 4 = 10

    // Test with perpendicular vectors
    let vec3 = Point64::new(1, 0);
    let vec4 = Point64::new(0, 1);
    let dot2 = dot_product_two_vectors(vec3, vec4);
    assert_eq!(dot2, 0.0);
}

#[test]
fn test_rect_validity() {
    // Test valid rectangle creation
    let valid_rect = Rect64::new_with_validity(true);
    assert!(valid_rect.is_valid());
    assert_eq!(valid_rect.left, 0);
    assert_eq!(valid_rect.right, 0);

    // Test invalid rectangle creation
    let invalid_rect = Rect64::new_with_validity(false);
    assert!(!invalid_rect.is_valid());

    // Test invalid rectangle factory method
    let invalid_rect2 = Rect64::invalid();
    assert!(!invalid_rect2.is_valid());
}

#[test]
fn test_rect_midpoint() {
    let rect = Rect64::new(10, 20, 30, 40);
    let mid = rect.mid_point();
    assert_eq!(mid.x, 20); // (10 + 30) / 2
    assert_eq!(mid.y, 30); // (20 + 40) / 2
}

#[test]
fn test_rect_as_path() {
    let rect = Rect64::new(0, 0, 100, 200);
    let path = rect.as_path();
    assert_eq!(path.len(), 4);

    // Clockwise from top-left
    assert_eq!(path[0], Point64::new(0, 0)); // top-left
    assert_eq!(path[1], Point64::new(100, 0)); // top-right
    assert_eq!(path[2], Point64::new(100, 200)); // bottom-right
    assert_eq!(path[3], Point64::new(0, 200)); // bottom-left
}

#[test]
fn test_rect_contains_point() {
    let rect = Rect64::new(10, 10, 100, 100);

    // Point inside (exclusive bounds)
    assert!(rect.contains_point(&Point64::new(50, 50)));

    // Points on edges should not be contained (exclusive)
    assert!(!rect.contains_point(&Point64::new(10, 50))); // left edge
    assert!(!rect.contains_point(&Point64::new(100, 50))); // right edge
    assert!(!rect.contains_point(&Point64::new(50, 10))); // top edge
    assert!(!rect.contains_point(&Point64::new(50, 100))); // bottom edge

    // Points outside
    assert!(!rect.contains_point(&Point64::new(5, 50)));
    assert!(!rect.contains_point(&Point64::new(150, 50)));
}

#[test]
fn test_rect_contains_rect() {
    let outer = Rect64::new(0, 0, 100, 100);
    let inner = Rect64::new(10, 10, 90, 90);
    let overlapping = Rect64::new(50, 50, 150, 150);
    let outside = Rect64::new(200, 200, 300, 300);

    assert!(outer.contains_rect(&inner));
    assert!(!outer.contains_rect(&overlapping));
    assert!(!outer.contains_rect(&outside));

    // Same rectangle should contain itself
    assert!(outer.contains_rect(&outer));
}

#[test]
fn test_rect_intersects() {
    let rect1 = Rect64::new(0, 0, 100, 100);
    let rect2 = Rect64::new(50, 50, 150, 150); // overlapping
    let rect3 = Rect64::new(200, 200, 300, 300); // separate
    let rect4 = Rect64::new(100, 0, 200, 100); // touching edge

    assert!(rect1.intersects(&rect2));
    assert!(!rect1.intersects(&rect3));
    assert!(rect1.intersects(&rect4)); // touching edges do intersect

    // Rectangle intersects with itself
    assert!(rect1.intersects(&rect1));
}

#[test]
fn test_rect_equality() {
    let rect1 = Rect64::new(10, 20, 30, 40);
    let rect2 = Rect64::new(10, 20, 30, 40);
    let rect3 = Rect64::new(10, 20, 30, 41);

    assert_eq!(rect1, rect2);
    assert_ne!(rect1, rect3);
}

#[test]
fn test_rect_union_operator() {
    let mut rect1 = Rect64::new(0, 0, 50, 50);
    let rect2 = Rect64::new(25, 25, 100, 100);

    rect1 += rect2;

    // Result should be bounding box of both rectangles
    assert_eq!(rect1.left, 0);
    assert_eq!(rect1.top, 0);
    assert_eq!(rect1.right, 100);
    assert_eq!(rect1.bottom, 100);
}

#[test]
fn test_constants() {
    use constants::*;

    assert!((PI - std::f64::consts::PI).abs() < 0.000_000_1);
    assert_eq!(CLIPPER2_MAX_DEC_PRECISION, 8);
    assert_eq!(MIN_COORD, -MAX_COORD);
    assert_eq!(INVALID, i64::MAX);
    // Constants are verified at compile time - these runtime checks are redundant
}

#[test]
fn test_do_error() {
    use errors::*;

    let result = do_error(PRECISION_ERROR_I);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().description(), PRECISION_ERROR);

    let result2 = do_error(SCALE_ERROR_I);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err().description(), SCALE_ERROR);

    let result3 = do_error(999); // unknown error
    assert!(result3.is_err());
    assert_eq!(result3.unwrap_err().description(), "Unknown error");
}

#[test]
fn test_tri_sign() {
    assert_eq!(tri_sign(10), 1);
    assert_eq!(tri_sign(-10), -1);
    assert_eq!(tri_sign(0), 0);
    assert_eq!(tri_sign(i64::MAX), 1);
    assert_eq!(tri_sign(i64::MIN), -1);
    assert_eq!(tri_sign(1), 1);
    assert_eq!(tri_sign(-1), -1);
}

#[test]
fn test_multiply_u64() {
    // Test simple cases
    let result = multiply_u64(0, 0);
    assert_eq!(result.hi, 0);
    assert_eq!(result.lo, 0);

    let result = multiply_u64(1, 1);
    assert_eq!(result.hi, 0);
    assert_eq!(result.lo, 1);

    let result = multiply_u64(10, 20);
    assert_eq!(result.hi, 0);
    assert_eq!(result.lo, 200);

    // Test case that would overflow 64-bit
    let result = multiply_u64(u64::MAX, 2);
    assert_eq!(result.hi, 1);
    assert_eq!(result.lo, u64::MAX - 1);

    // Test maximum values
    let result = multiply_u64(u64::MAX, u64::MAX);
    assert_eq!(result.hi, u64::MAX - 1);
    assert_eq!(result.lo, 1);
}

#[test]
fn test_products_are_equal() {
    // Test basic equality
    assert!(products_are_equal(2, 3, 6, 1));
    assert!(products_are_equal(2, 3, 1, 6));
    assert!(products_are_equal(4, 5, 10, 2));

    // Test basic inequality
    assert!(!products_are_equal(2, 3, 7, 1));
    assert!(!products_are_equal(4, 5, 10, 3));

    // Test with zero values
    assert!(products_are_equal(0, 5, 0, 10));
    assert!(products_are_equal(5, 0, 10, 0));
    assert!(products_are_equal(0, 5, 1, 0)); // Both products are 0
    assert!(!products_are_equal(0, 5, 1, 1)); // 0 != 1

    // Test with negative values
    assert!(products_are_equal(-2, 3, 2, -3));
    assert!(products_are_equal(-2, -3, 2, 3));
    assert!(!products_are_equal(-2, 3, 2, 3));

    // Test large values that might cause overflow
    let large = 1000000000i64;
    assert!(products_are_equal(large, 2, 2 * large, 1));
    assert!(products_are_equal(large, large, large * large, 1));

    // Test edge cases with max values
    assert!(products_are_equal(i64::MAX, 0, 0, i64::MAX));
    assert!(products_are_equal(i64::MIN, 0, 0, i64::MIN));

    // Test sign differentiation - this is important for the algorithm
    assert!(products_are_equal(1, -1, -1, 1)); // Both products are -1
    assert!(products_are_equal(-1, -1, 1, 1)); // Both positive results
    assert!(!products_are_equal(1, -1, 1, 1)); // -1 != 1
}

#[test]
fn test_strip_duplicates_path() {
    // Test open path with duplicates
    let mut open_path = vec![
        Point64::new(0, 0),
        Point64::new(0, 0), // duplicate
        Point64::new(10, 10),
        Point64::new(10, 10), // duplicate
        Point64::new(20, 20),
    ];
    strip_duplicates_path(&mut open_path, false);
    assert_eq!(open_path.len(), 3);
    assert_eq!(open_path[0], Point64::new(0, 0));
    assert_eq!(open_path[1], Point64::new(10, 10));
    assert_eq!(open_path[2], Point64::new(20, 20));

    // Test closed path with duplicates including wrap-around
    let mut closed_path = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
        Point64::new(0, 0), // should be removed for closed path
    ];
    strip_duplicates_path(&mut closed_path, true);
    assert_eq!(closed_path.len(), 4);
    assert_eq!(closed_path[0], Point64::new(0, 0));
    assert_eq!(closed_path[3], Point64::new(0, 10));

    // Test path with no duplicates
    let mut no_dups = vec![
        Point64::new(0, 0),
        Point64::new(10, 10),
        Point64::new(20, 20),
    ];
    let original = no_dups.clone();
    strip_duplicates_path(&mut no_dups, false);
    assert_eq!(no_dups, original);

    // Test empty path
    let mut empty: Path64 = vec![];
    strip_duplicates_path(&mut empty, true);
    assert!(empty.is_empty());

    // Test single point path
    let mut single = vec![Point64::new(0, 0)];
    strip_duplicates_path(&mut single, true);
    assert_eq!(single.len(), 1);
}

#[test]
fn test_strip_duplicates_paths() {
    let mut paths = vec![
        vec![
            Point64::new(0, 0),
            Point64::new(0, 0), // duplicate
            Point64::new(10, 10),
        ],
        vec![
            Point64::new(20, 20),
            Point64::new(30, 30),
            Point64::new(20, 20), // wrap-around duplicate
        ],
    ];

    strip_duplicates_paths(&mut paths, true);

    // First path should have duplicate removed
    assert_eq!(paths[0].len(), 2);
    assert_eq!(paths[0][0], Point64::new(0, 0));
    assert_eq!(paths[0][1], Point64::new(10, 10));

    // Second path should have wrap-around duplicate removed
    assert_eq!(paths[1].len(), 2);
    assert_eq!(paths[1][0], Point64::new(20, 20));
    assert_eq!(paths[1][1], Point64::new(30, 30));
}

#[test]
fn test_check_precision_range() {
    use constants::CLIPPER2_MAX_DEC_PRECISION;
    use errors::PRECISION_ERROR_I;

    // Test valid precision - should not change
    let mut precision = 5;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, 5);
    assert_eq!(error_code, 0);

    // Test maximum valid precision
    let mut precision = CLIPPER2_MAX_DEC_PRECISION;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, 0);

    // Test minimum valid precision
    let mut precision = -CLIPPER2_MAX_DEC_PRECISION;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, 0);

    // Test positive overflow - should clamp and set error
    let mut precision = CLIPPER2_MAX_DEC_PRECISION + 1;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, PRECISION_ERROR_I);

    // Test negative overflow - should clamp and set error
    let mut precision = -CLIPPER2_MAX_DEC_PRECISION - 1;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, PRECISION_ERROR_I);

    // Test extreme positive value
    let mut precision = i32::MAX;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, PRECISION_ERROR_I);

    // Test extreme negative value
    let mut precision = i32::MIN;
    let mut error_code = 0;
    check_precision_range(&mut precision, &mut error_code);
    assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
    assert_eq!(error_code, PRECISION_ERROR_I);
}

#[test]
fn test_check_precision_range_simple() {
    use constants::CLIPPER2_MAX_DEC_PRECISION;

    // Test convenience function
    let mut precision = CLIPPER2_MAX_DEC_PRECISION + 5;
    check_precision_range_simple(&mut precision);
    assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);

    let mut precision = -CLIPPER2_MAX_DEC_PRECISION - 3;
    check_precision_range_simple(&mut precision);
    assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);

    let mut precision = 3;
    check_precision_range_simple(&mut precision);
    assert_eq!(precision, 3); // Should remain unchanged
}

#[test]
fn test_get_bounds_path() {
    // Test basic rectangular path
    let path: Path64 = vec![
        Point64::new(10, 20),
        Point64::new(100, 30),
        Point64::new(50, 80),
        Point64::new(0, 10),
    ];

    let bounds = get_bounds_path(&path);
    assert_eq!(bounds.left, 0);
    assert_eq!(bounds.top, 10);
    assert_eq!(bounds.right, 100);
    assert_eq!(bounds.bottom, 80);

    // Test single point path
    let single_path: Path64 = vec![Point64::new(42, 37)];
    let single_bounds = get_bounds_path(&single_path);
    assert_eq!(single_bounds.left, 42);
    assert_eq!(single_bounds.top, 37);
    assert_eq!(single_bounds.right, 42);
    assert_eq!(single_bounds.bottom, 37);

    // Test empty path - should return invalid bounds
    let empty_path: Path64 = vec![];
    let empty_bounds = get_bounds_path(&empty_path);
    assert_eq!(empty_bounds.left, i64::MAX);
    assert_eq!(empty_bounds.top, i64::MAX);
    assert_eq!(empty_bounds.right, i64::MIN);
    assert_eq!(empty_bounds.bottom, i64::MIN);
}

#[test]
fn test_get_bounds_path_double() {
    // Test with floating-point path
    let path: PathD = vec![
        PointD::new(10.5, 20.7),
        PointD::new(100.3, 30.1),
        PointD::new(50.9, 80.4),
        PointD::new(0.2, 10.8),
    ];

    let bounds = get_bounds_path(&path);
    assert_eq!(bounds.left, 0.2);
    assert_eq!(bounds.top, 10.8);
    assert_eq!(bounds.right, 100.3);
    assert_eq!(bounds.bottom, 80.4);
}

#[test]
fn test_get_bounds_paths() {
    // Test multiple paths
    let paths: Paths64 = vec![
        vec![Point64::new(0, 0), Point64::new(50, 25)],
        vec![Point64::new(25, 50), Point64::new(100, 75)],
        vec![Point64::new(-10, -5), Point64::new(30, 40)],
    ];

    let bounds = get_bounds_paths(&paths);
    assert_eq!(bounds.left, -10);
    assert_eq!(bounds.top, -5);
    assert_eq!(bounds.right, 100);
    assert_eq!(bounds.bottom, 75);

    // Test empty paths
    let empty_paths: Paths64 = vec![];
    let empty_bounds = get_bounds_paths(&empty_paths);
    assert_eq!(empty_bounds.left, i64::MAX);
    assert_eq!(empty_bounds.right, i64::MIN);

    // Test paths with empty paths inside
    let mixed_paths: Paths64 = vec![
        vec![Point64::new(10, 20)],
        vec![], // empty path
        vec![Point64::new(30, 40)],
    ];
    let mixed_bounds = get_bounds_paths(&mixed_paths);
    assert_eq!(mixed_bounds.left, 10);
    assert_eq!(mixed_bounds.top, 20);
    assert_eq!(mixed_bounds.right, 30);
    assert_eq!(mixed_bounds.bottom, 40);
}

#[test]
fn test_get_bounds_path_convert() {
    // Test converting from i32 path to i64 bounds
    let path32: Path<i32> = vec![
        Point::new(10i32, 20i32),
        Point::new(100i32, 30i32),
        Point::new(50i32, 80i32),
    ];

    let bounds64: Rect64 = get_bounds_path_convert(&path32);
    assert_eq!(bounds64.left, 10i64);
    assert_eq!(bounds64.top, 20i64);
    assert_eq!(bounds64.right, 100i64);
    assert_eq!(bounds64.bottom, 80i64);

    // Test converting from f32 path to f64 bounds
    let pathf32: Path<f32> = vec![Point::new(10.5f32, 20.7f32), Point::new(100.3f32, 30.1f32)];

    let boundsf64: RectD = get_bounds_path_convert(&pathf32);
    // Use a more generous epsilon for f32 to f64 conversion
    const TOLERANCE: f64 = 1e-6;
    assert!((boundsf64.left - 10.5).abs() < TOLERANCE);
    assert!((boundsf64.top - 20.700000762939453).abs() < TOLERANCE); // f32 precision loss
    assert!((boundsf64.right - 100.30000305175781).abs() < TOLERANCE);
    assert!((boundsf64.bottom - 30.100000381469727).abs() < TOLERANCE);
}

#[test]
fn test_get_bounds_paths_convert() {
    // Test converting multiple paths
    let paths32: Paths<i32> = vec![
        vec![Point::new(10i32, 20i32), Point::new(50i32, 25i32)],
        vec![Point::new(25i32, 50i32), Point::new(100i32, 75i32)],
    ];

    let bounds64: Rect64 = get_bounds_paths_convert(&paths32);
    assert_eq!(bounds64.left, 10i64);
    assert_eq!(bounds64.top, 20i64);
    assert_eq!(bounds64.right, 100i64);
    assert_eq!(bounds64.bottom, 75i64);
}

#[test]
fn test_get_bounds_extreme_values() {
    // Test with extreme coordinate values
    let extreme_path: Path64 = vec![
        Point64::new(i64::MIN + 100, i64::MIN + 200),
        Point64::new(i64::MAX - 100, i64::MAX - 200),
        Point64::new(0, 0),
    ];

    let bounds = get_bounds_path(&extreme_path);
    assert_eq!(bounds.left, i64::MIN + 100);
    assert_eq!(bounds.top, i64::MIN + 200);
    assert_eq!(bounds.right, i64::MAX - 100);
    assert_eq!(bounds.bottom, i64::MAX - 200);
}

#[test]
fn test_get_bounds_negative_coordinates() {
    // Test with all negative coordinates
    let negative_path: Path64 = vec![
        Point64::new(-100, -200),
        Point64::new(-50, -150),
        Point64::new(-75, -175),
    ];

    let bounds = get_bounds_path(&negative_path);
    assert_eq!(bounds.left, -100);
    assert_eq!(bounds.top, -200);
    assert_eq!(bounds.right, -50);
    assert_eq!(bounds.bottom, -150);
}

#[test]
fn test_get_bounds_identical_points() {
    // Test with identical points (degenerate case)
    let identical_path: Path64 = vec![
        Point64::new(42, 37),
        Point64::new(42, 37),
        Point64::new(42, 37),
    ];

    let bounds = get_bounds_path(&identical_path);
    assert_eq!(bounds.left, 42);
    assert_eq!(bounds.top, 37);
    assert_eq!(bounds.right, 42);
    assert_eq!(bounds.bottom, 37);

    // Verify it results in a valid zero-area rectangle
    assert_eq!(bounds.width(), 0);
    assert_eq!(bounds.height(), 0);
    assert!(bounds.is_empty()); // Zero-size rectangles are empty when left==right or top==bottom
}

#[test]
fn test_sqr() {
    // Test basic integer squaring
    assert_eq!(sqr(5i64), 25.0);
    assert_eq!(sqr(-3i32), 9.0);
    assert_eq!(sqr(0i64), 0.0);

    // Test floating point squaring
    assert_eq!(sqr(2.5f64), 6.25);
    assert_eq!(sqr(-1.5f32), 2.25);

    // Test large values
    assert_eq!(sqr(1000i64), 1000000.0);
}

#[test]
fn test_distance_sqr() {
    // Test basic distance
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(3, 4);
    assert_eq!(distance_sqr(p1, p2), 25.0); // 3^2 + 4^2 = 9 + 16 = 25

    // Test zero distance
    let p3 = Point64::new(10, 20);
    let p4 = Point64::new(10, 20);
    assert_eq!(distance_sqr(p3, p4), 0.0);

    // Test negative coordinates
    let p5 = Point64::new(-5, -5);
    let p6 = Point64::new(-8, -1);
    assert_eq!(distance_sqr(p5, p6), 25.0); // (-3)^2 + 4^2 = 9 + 16 = 25

    // Test with floating point
    let pf1 = PointD::new(0.0, 0.0);
    let pf2 = PointD::new(3.0, 4.0);
    assert_eq!(distance_sqr(pf1, pf2), 25.0);
}

#[test]
fn test_perpendicular_distance_from_line_sqr() {
    // Test perpendicular distance to horizontal line
    let pt = Point64::new(5, 10);
    let line1 = Point64::new(0, 5);
    let line2 = Point64::new(10, 5);

    let dist_sqr = perpendicular_distance_from_line_sqr(pt, line1, line2);
    assert_eq!(dist_sqr, 25.0); // Distance of 5 squared = 25

    // Test point on line (should be 0 distance)
    let pt_on_line = Point64::new(5, 5);
    let dist_on_line = perpendicular_distance_from_line_sqr(pt_on_line, line1, line2);
    assert_eq!(dist_on_line, 0.0);

    // Test degenerate line (both points same)
    let degenerate_dist = perpendicular_distance_from_line_sqr(pt, line1, line1);
    assert_eq!(degenerate_dist, 0.0);
}

#[test]
fn test_area() {
    // Test triangle area
    let triangle: Path64 = vec![Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, 10)];
    assert_eq!(area(&triangle), 50.0); // Area = 0.5 * base * height = 0.5 * 10 * 10 = 50

    // Test rectangle area (clockwise, should be negative)
    let rectangle: Path64 = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
    ];
    assert_eq!(area(&rectangle), 100.0); // This gives positive because of the shoelace formula

    // Test rectangle area (counterclockwise, should be positive)
    let rectangle_ccw: Path64 = vec![
        Point64::new(0, 0),
        Point64::new(0, 10),
        Point64::new(10, 10),
        Point64::new(10, 0),
    ];
    assert_eq!(area(&rectangle_ccw), -100.0); // This gives negative because of the shoelace formula

    // Test empty path
    let empty: Path64 = vec![];
    assert_eq!(area(&empty), 0.0);

    // Test path with less than 3 points
    let line: Path64 = vec![Point64::new(0, 0), Point64::new(10, 10)];
    assert_eq!(area(&line), 0.0);
}

#[test]
fn test_area_paths() {
    let triangle1: Path64 = vec![Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, 10)];

    let triangle2: Path64 = vec![Point64::new(0, 0), Point64::new(0, 10), Point64::new(-5, 5)];

    // Calculate areas before moving into paths
    let area1 = area(&triangle1);
    let area2 = area(&triangle2);
    let expected_total = area1 + area2;

    let paths = vec![triangle1, triangle2];
    assert_eq!(area_paths(&paths), expected_total);

    // Test empty paths
    let empty_paths: Paths64 = vec![];
    assert_eq!(area_paths(&empty_paths), 0.0);
}

#[test]
fn test_is_positive() {
    // Test rectangle with positive area (this path gives positive area)
    let positive_rect: Path64 = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
    ];
    assert!(is_positive(&positive_rect));

    // Test rectangle with negative area (this path gives negative area)
    let negative_rect: Path64 = vec![
        Point64::new(0, 0),
        Point64::new(0, 10),
        Point64::new(10, 10),
        Point64::new(10, 0),
    ];
    assert!(!is_positive(&negative_rect));

    // Test degenerate case (zero area should be considered positive)
    let line: Path64 = vec![Point64::new(0, 0), Point64::new(10, 10)];
    assert!(is_positive(&line)); // Zero area returns true (>= 0.0)

    // Test empty path
    let empty: Path64 = vec![];
    assert!(is_positive(&empty)); // Zero area returns true
}
