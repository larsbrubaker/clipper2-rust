use super::*;

#[test]
fn test_get_segment_intersect_pt_basic() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(10, 10);
    let p3 = Point64::new(0, 10);
    let p4 = Point64::new(10, 0);
    let mut ip = Point64::new(0, 0);

    // Crossing lines should intersect at (5, 5)
    let result = get_segment_intersect_pt(p1, p2, p3, p4, &mut ip);
    assert!(result);
    assert_eq!(ip.x, 5);
    assert_eq!(ip.y, 5);
}

#[test]
fn test_get_segment_intersect_pt_parallel() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(10, 0);
    let p3 = Point64::new(0, 5);
    let p4 = Point64::new(10, 5);
    let mut ip = Point64::new(0, 0);

    // Parallel lines should not intersect
    let result = get_segment_intersect_pt(p1, p2, p3, p4, &mut ip);
    assert!(!result);
}

#[test]
fn test_get_segment_intersect_pt_endpoint() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(10, 0);
    let p3 = Point64::new(0, 0);
    let p4 = Point64::new(0, 10);
    let mut ip = Point64::new(0, 0);

    // Lines that meet at endpoint should return endpoint
    let result = get_segment_intersect_pt(p1, p2, p3, p4, &mut ip);
    assert!(result);
    assert_eq!(ip.x, 0);
    assert_eq!(ip.y, 0);
}

#[test]
fn test_is_collinear_true() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(5, 5);
    let p3 = Point64::new(10, 10);

    // Points on a diagonal line should be collinear
    assert!(is_collinear(p1, p2, p3));

    // Test horizontal line
    let p4 = Point64::new(0, 5);
    let p5 = Point64::new(5, 5);
    let p6 = Point64::new(10, 5);
    assert!(is_collinear(p4, p5, p6));

    // Test vertical line
    let p7 = Point64::new(5, 0);
    let p8 = Point64::new(5, 5);
    let p9 = Point64::new(5, 10);
    assert!(is_collinear(p7, p8, p9));
}

#[test]
fn test_is_collinear_false() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(5, 5);
    let p3 = Point64::new(10, 5);

    // Points forming a right angle should not be collinear
    assert!(!is_collinear(p1, p2, p3));
}

#[test]
fn test_point_in_polygon_inside() {
    // Square polygon
    let square = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
    ];

    // Point inside square
    let inside_pt = Point64::new(5, 5);
    assert_eq!(
        point_in_polygon(inside_pt, &square),
        PointInPolygonResult::IsInside
    );
}

#[test]
fn test_point_in_polygon_outside() {
    // Square polygon
    let square = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
    ];

    // Point outside square
    let outside_pt = Point64::new(15, 15);
    assert_eq!(
        point_in_polygon(outside_pt, &square),
        PointInPolygonResult::IsOutside
    );

    // Point to the left
    let left_pt = Point64::new(-5, 5);
    assert_eq!(
        point_in_polygon(left_pt, &square),
        PointInPolygonResult::IsOutside
    );
}

#[test]
fn test_point_in_polygon_on_edge() {
    // Square polygon
    let square = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
    ];

    // Point on edge
    let edge_pt = Point64::new(5, 0);
    assert_eq!(
        point_in_polygon(edge_pt, &square),
        PointInPolygonResult::IsOn
    );

    // Point at vertex
    let vertex_pt = Point64::new(0, 0);
    assert_eq!(
        point_in_polygon(vertex_pt, &square),
        PointInPolygonResult::IsOn
    );
}

#[test]
fn test_point_in_polygon_triangle() {
    // Triangle polygon
    let triangle = vec![Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, 10)];

    // Point inside triangle
    let inside_pt = Point64::new(5, 3);
    assert_eq!(
        point_in_polygon(inside_pt, &triangle),
        PointInPolygonResult::IsInside
    );

    // Point outside triangle
    let outside_pt = Point64::new(1, 8);
    assert_eq!(
        point_in_polygon(outside_pt, &triangle),
        PointInPolygonResult::IsOutside
    );
}

#[test]
fn test_point_in_polygon_degenerate() {
    // Line (only 2 points) - should always be outside
    let line = vec![Point64::new(0, 0), Point64::new(10, 0)];

    let test_pt = Point64::new(5, 0);
    assert_eq!(
        point_in_polygon(test_pt, &line),
        PointInPolygonResult::IsOutside
    );
}

#[test]
fn test_get_location_basic() {
    let rect = Rect64::new(10, 10, 50, 50);
    let mut loc = Location::Inside;

    // Point inside rectangle
    let inside_pt = Point64::new(25, 25);
    let result = get_location(&rect, &inside_pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Inside);

    // Point to the left
    let left_pt = Point64::new(5, 25);
    let result = get_location(&rect, &left_pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Left);

    // Point to the right
    let right_pt = Point64::new(55, 25);
    let result = get_location(&rect, &right_pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Right);

    // Point above
    let top_pt = Point64::new(25, 5);
    let result = get_location(&rect, &top_pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Top);

    // Point below
    let bottom_pt = Point64::new(25, 55);
    let result = get_location(&rect, &bottom_pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Bottom);
}

#[test]
fn test_get_location_on_edge() {
    let rect = Rect64::new(10, 10, 50, 50);
    let mut loc = Location::Inside;

    // Point on left edge
    let left_edge_pt = Point64::new(10, 25);
    let result = get_location(&rect, &left_edge_pt, &mut loc);
    assert!(!result); // Returns false when on edge
    assert_eq!(loc, Location::Left);

    // Point on right edge
    let right_edge_pt = Point64::new(50, 25);
    let result = get_location(&rect, &right_edge_pt, &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Right);

    // Point on top edge
    let top_edge_pt = Point64::new(25, 10);
    let result = get_location(&rect, &top_edge_pt, &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Top);

    // Point on bottom edge
    let bottom_edge_pt = Point64::new(25, 50);
    let result = get_location(&rect, &bottom_edge_pt, &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Bottom);
}

#[test]
fn test_is_horizontal() {
    let p1 = Point64::new(0, 5);
    let p2 = Point64::new(10, 5);
    assert!(is_horizontal(&p1, &p2));

    let p3 = Point64::new(0, 5);
    let p4 = Point64::new(10, 10);
    assert!(!is_horizontal(&p3, &p4));
}

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

#[test]
fn test_location_variants() {
    let locations = [
        Location::Left,
        Location::Top,
        Location::Right,
        Location::Bottom,
        Location::Inside,
    ];
    assert_eq!(locations.len(), 5);

    // Test each variant is unique
    for i in 0..locations.len() {
        for j in (i + 1)..locations.len() {
            assert_ne!(locations[i], locations[j]);
        }
    }
}

#[test]
fn test_location_copy_clone() {
    let loc = Location::Inside;
    let copied = loc;
    let cloned = loc;

    assert_eq!(loc, copied);
    assert_eq!(loc, cloned);
    assert_eq!(copied, cloned);
}

#[test]
fn test_location_debug() {
    assert_eq!(format!("{:?}", Location::Left), "Left");
    assert_eq!(format!("{:?}", Location::Top), "Top");
    assert_eq!(format!("{:?}", Location::Right), "Right");
    assert_eq!(format!("{:?}", Location::Bottom), "Bottom");
    assert_eq!(format!("{:?}", Location::Inside), "Inside");
}

#[test]
fn test_location_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(Location::Left, "left");
    map.insert(Location::Top, "top");
    map.insert(Location::Right, "right");
    map.insert(Location::Bottom, "bottom");
    map.insert(Location::Inside, "inside");

    assert_eq!(map[&Location::Left], "left");
    assert_eq!(map[&Location::Top], "top");
    assert_eq!(map[&Location::Right], "right");
    assert_eq!(map[&Location::Bottom], "bottom");
    assert_eq!(map[&Location::Inside], "inside");
    assert_eq!(map.len(), 5);
}

#[test]
fn test_location_ordering_properties() {
    let locations = [
        Location::Left,
        Location::Top,
        Location::Right,
        Location::Bottom,
        Location::Inside,
    ];

    // Test that all locations can be compared for equality
    for loc1 in &locations {
        for loc2 in &locations {
            let eq = loc1 == loc2;
            let ne = loc1 != loc2;
            assert_eq!(eq, !ne); // == and != should be opposites
            if std::ptr::eq(loc1, loc2) {
                assert!(eq);
            }
        }
    }
}

#[test]
fn test_get_location_inside() {
    let rect = Rect64::new(10, 10, 100, 100);
    let pt = Point64::new(50, 50);
    let mut loc = Location::Left;

    let result = get_location(&rect, &pt, &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Inside);
}

#[test]
fn test_get_location_left_edge() {
    let rect = Rect64::new(10, 10, 100, 100);
    let pt = Point64::new(10, 50);
    let mut loc = Location::Inside;

    let result = get_location(&rect, &pt, &mut loc);
    assert!(!result); // On edge should return false
    assert_eq!(loc, Location::Left);
}

#[test]
fn test_get_location_right_edge() {
    let rect = Rect64::new(10, 10, 100, 100);
    let pt = Point64::new(100, 50);
    let mut loc = Location::Inside;

    let result = get_location(&rect, &pt, &mut loc);
    assert!(!result); // On edge should return false
    assert_eq!(loc, Location::Right);
}

#[test]
fn test_get_location_top_edge() {
    let rect = Rect64::new(10, 10, 100, 100);
    let pt = Point64::new(50, 10);
    let mut loc = Location::Inside;

    let result = get_location(&rect, &pt, &mut loc);
    assert!(!result); // On edge should return false
    assert_eq!(loc, Location::Top);
}

#[test]
fn test_get_location_bottom_edge() {
    let rect = Rect64::new(10, 10, 100, 100);
    let pt = Point64::new(50, 100);
    let mut loc = Location::Inside;

    let result = get_location(&rect, &pt, &mut loc);
    assert!(!result); // On edge should return false
    assert_eq!(loc, Location::Bottom);
}

#[test]
fn test_get_location_corners_on_edge() {
    let rect = Rect64::new(10, 10, 100, 100);
    let mut loc = Location::Inside;

    // Top-left corner (on left edge)
    let result = get_location(&rect, &Point64::new(10, 10), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Left);

    // Top-right corner (on right edge - C++ implementation prioritizes x over y)
    let result = get_location(&rect, &Point64::new(100, 10), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Right);

    // Bottom-left corner (on left edge)
    let result = get_location(&rect, &Point64::new(10, 100), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Left);

    // Bottom-right corner (on right edge - C++ implementation prioritizes x over y)
    let result = get_location(&rect, &Point64::new(100, 100), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Right);
}

#[test]
fn test_get_location_outside_regions() {
    let rect = Rect64::new(10, 10, 100, 100);
    let mut loc = Location::Inside;

    // Left region
    let result = get_location(&rect, &Point64::new(5, 50), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Left);

    // Right region
    let result = get_location(&rect, &Point64::new(150, 50), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Right);

    // Top region
    let result = get_location(&rect, &Point64::new(50, 5), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Top);

    // Bottom region
    let result = get_location(&rect, &Point64::new(50, 150), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Bottom);
}

#[test]
fn test_get_location_diagonal_outside() {
    let rect = Rect64::new(10, 10, 100, 100);
    let mut loc = Location::Inside;

    // Top-left diagonal (should be Left since x < left)
    let result = get_location(&rect, &Point64::new(5, 5), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Left);

    // Top-right diagonal (should be Right since x > right)
    let result = get_location(&rect, &Point64::new(150, 5), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Right);

    // Bottom-left diagonal (should be Left since x < left)
    let result = get_location(&rect, &Point64::new(5, 150), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Left);

    // Bottom-right diagonal (should be Right since x > right)
    let result = get_location(&rect, &Point64::new(150, 150), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Right);
}

#[test]
fn test_get_location_edge_cases() {
    let rect = Rect64::new(0, 0, 10, 10);
    let mut loc = Location::Inside;

    // Point at origin (on left edge)
    let result = get_location(&rect, &Point64::new(0, 0), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Left);

    // Single point rectangle
    let single_rect = Rect64::new(5, 5, 5, 5);
    let result = get_location(&single_rect, &Point64::new(5, 5), &mut loc);
    assert!(!result);
    assert_eq!(loc, Location::Left);

    // Point just inside
    let result = get_location(&rect, &Point64::new(1, 1), &mut loc);
    assert!(result);
    assert_eq!(loc, Location::Inside);
}

#[test]
fn test_get_location_comprehensive_coverage() {
    let rect = Rect64::new(10, 10, 100, 100);
    let mut loc = Location::Inside;

    // Test a grid of points to ensure comprehensive coverage
    let test_points = [
        // Inside
        (Point64::new(50, 50), true, Location::Inside),
        (Point64::new(11, 11), true, Location::Inside),
        (Point64::new(99, 99), true, Location::Inside),
        // On edges (should return false)
        (Point64::new(10, 50), false, Location::Left),
        (Point64::new(100, 50), false, Location::Right),
        (Point64::new(50, 10), false, Location::Top),
        (Point64::new(50, 100), false, Location::Bottom),
        // Outside
        (Point64::new(5, 50), true, Location::Left),
        (Point64::new(150, 50), true, Location::Right),
        (Point64::new(50, 5), true, Location::Top),
        (Point64::new(50, 150), true, Location::Bottom),
    ];

    for (point, expected_result, expected_location) in test_points.iter() {
        let result = get_location(&rect, point, &mut loc);
        assert_eq!(
            result, *expected_result,
            "Point {:?} should return {}",
            point, expected_result
        );
        assert_eq!(
            loc, *expected_location,
            "Point {:?} should be at location {:?}",
            point, expected_location
        );
    }
}

#[test]
fn test_is_horizontal_true() {
    let pt1 = Point64::new(10, 50);
    let pt2 = Point64::new(100, 50);
    assert!(is_horizontal(&pt1, &pt2));

    // Test with same point
    assert!(is_horizontal(&pt1, &pt1));

    // Test with zero y-coordinates
    let pt3 = Point64::new(-10, 0);
    let pt4 = Point64::new(20, 0);
    assert!(is_horizontal(&pt3, &pt4));

    // Test with negative y-coordinates
    let pt5 = Point64::new(0, -25);
    let pt6 = Point64::new(100, -25);
    assert!(is_horizontal(&pt5, &pt6));
}

#[test]
fn test_is_horizontal_false() {
    let pt1 = Point64::new(10, 50);
    let pt2 = Point64::new(10, 60);
    assert!(!is_horizontal(&pt1, &pt2));

    // Test diagonal
    let pt3 = Point64::new(0, 0);
    let pt4 = Point64::new(10, 10);
    assert!(!is_horizontal(&pt3, &pt4));

    // Test with different y values
    let pt5 = Point64::new(100, 25);
    let pt6 = Point64::new(100, 26);
    assert!(!is_horizontal(&pt5, &pt6));
}

#[test]
fn test_is_horizontal_edge_cases() {
    // Test with extreme values
    let pt1 = Point64::new(i64::MIN, 0);
    let pt2 = Point64::new(i64::MAX, 0);
    assert!(is_horizontal(&pt1, &pt2));

    let pt3 = Point64::new(0, i64::MIN);
    let pt4 = Point64::new(0, i64::MAX);
    assert!(!is_horizontal(&pt3, &pt4));

    // Test with same x, different y
    let pt5 = Point64::new(42, -100);
    let pt6 = Point64::new(42, 100);
    assert!(!is_horizontal(&pt5, &pt6));

    // Test with different x, same y
    let pt7 = Point64::new(-100, 42);
    let pt8 = Point64::new(100, 42);
    assert!(is_horizontal(&pt7, &pt8));
}

#[test]
fn test_is_horizontal_symmetry() {
    let pt1 = Point64::new(10, 20);
    let pt2 = Point64::new(30, 20);
    let pt3 = Point64::new(10, 25);

    // Function should be symmetric
    assert_eq!(is_horizontal(&pt1, &pt2), is_horizontal(&pt2, &pt1));
    assert_eq!(is_horizontal(&pt1, &pt3), is_horizontal(&pt3, &pt1));

    // Test multiple symmetric cases
    let test_pairs = [
        (Point64::new(0, 10), Point64::new(100, 10)),
        (Point64::new(-50, -20), Point64::new(75, -20)),
        (Point64::new(5, 5), Point64::new(5, 15)),
        (Point64::new(-10, 0), Point64::new(-5, 0)),
    ];

    for (p1, p2) in test_pairs.iter() {
        assert_eq!(
            is_horizontal(p1, p2),
            is_horizontal(p2, p1),
            "Symmetry failed for points {:?} and {:?}",
            p1,
            p2
        );
    }
}

#[test]
fn test_is_horizontal_comprehensive() {
    // Create a comprehensive test with various combinations
    let horizontal_pairs = [
        (Point64::new(0, 0), Point64::new(1, 0)),
        (Point64::new(-10, 5), Point64::new(20, 5)),
        (Point64::new(100, -50), Point64::new(-100, -50)),
        (Point64::new(42, 42), Point64::new(42, 42)), // same point
    ];

    let non_horizontal_pairs = [
        (Point64::new(0, 0), Point64::new(0, 1)),
        (Point64::new(10, 10), Point64::new(15, 15)),
        (Point64::new(-5, 20), Point64::new(-5, 19)),
        (Point64::new(100, 0), Point64::new(100, -1)),
    ];

    for (p1, p2) in horizontal_pairs.iter() {
        assert!(
            is_horizontal(p1, p2),
            "Expected horizontal line for points {:?} and {:?}",
            p1,
            p2
        );
    }

    for (p1, p2) in non_horizontal_pairs.iter() {
        assert!(
            !is_horizontal(p1, p2),
            "Expected non-horizontal line for points {:?} and {:?}",
            p1,
            p2
        );
    }
}

// ============================================================================
// Tests for Phase 1: Missing core.rs functions
// ============================================================================

// --- ScaleRect tests ---

#[test]
fn test_scale_rect_i64_to_f64() {
    let rect = Rect64::new(10, 20, 30, 40);
    let scaled: RectD = scale_rect(&rect, 2.0);
    assert_eq!(scaled.left, 20.0);
    assert_eq!(scaled.top, 40.0);
    assert_eq!(scaled.right, 60.0);
    assert_eq!(scaled.bottom, 80.0);
}

#[test]
fn test_scale_rect_f64_to_i64() {
    let rect = RectD::new(10.5, 20.5, 30.5, 40.5);
    let scaled: Rect64 = scale_rect(&rect, 2.0);
    // Should round: 10.5*2=21.0, 20.5*2=41.0, etc.
    assert_eq!(scaled.left, 21);
    assert_eq!(scaled.top, 41);
    assert_eq!(scaled.right, 61);
    assert_eq!(scaled.bottom, 81);
}

#[test]
fn test_scale_rect_fractional() {
    let rect = RectD::new(1.0, 2.0, 3.0, 4.0);
    let scaled: RectD = scale_rect(&rect, 0.5);
    assert_eq!(scaled.left, 0.5);
    assert_eq!(scaled.top, 1.0);
    assert_eq!(scaled.right, 1.5);
    assert_eq!(scaled.bottom, 2.0);
}

#[test]
fn test_scale_rect_identity() {
    let rect = Rect64::new(10, 20, 30, 40);
    let scaled: Rect64 = scale_rect(&rect, 1.0);
    assert_eq!(scaled, rect);
}

// --- ScalePath tests ---

#[test]
fn test_scale_path_d_to_i64() {
    let path: PathD = vec![
        PointD::new(1.5, 2.5),
        PointD::new(3.5, 4.5),
        PointD::new(5.5, 6.5),
    ];
    let mut error_code = 0;
    let result: Path64 = scale_path(&path, 2.0, 2.0, &mut error_code);
    assert_eq!(error_code, 0);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Point64::new(3, 5)); // 1.5*2=3.0, 2.5*2=5.0
    assert_eq!(result[1], Point64::new(7, 9)); // 3.5*2=7.0, 4.5*2=9.0
    assert_eq!(result[2], Point64::new(11, 13)); // 5.5*2=11.0, 6.5*2=13.0
}

#[test]
fn test_scale_path_i64_to_d() {
    let path: Path64 = vec![Point64::new(10, 20), Point64::new(30, 40)];
    let mut error_code = 0;
    let result: PathD = scale_path(&path, 0.1, 0.1, &mut error_code);
    assert_eq!(error_code, 0);
    assert_eq!(result.len(), 2);
    assert!((result[0].x - 1.0).abs() < 1e-10);
    assert!((result[0].y - 2.0).abs() < 1e-10);
    assert!((result[1].x - 3.0).abs() < 1e-10);
    assert!((result[1].y - 4.0).abs() < 1e-10);
}

#[test]
fn test_scale_path_different_xy_scales() {
    let path: PathD = vec![PointD::new(10.0, 20.0)];
    let mut error_code = 0;
    let result: Path64 = scale_path(&path, 2.0, 3.0, &mut error_code);
    assert_eq!(error_code, 0);
    assert_eq!(result[0], Point64::new(20, 60));
}

#[test]
fn test_scale_path_zero_scale_error() {
    let path: PathD = vec![PointD::new(10.0, 20.0)];
    let mut error_code = 0;
    let result: Path64 = scale_path(&path, 0.0, 2.0, &mut error_code);
    // Should set error code but still produce result (non-fatal)
    assert_ne!(error_code & errors::SCALE_ERROR_I, 0);
    // With zero scale treated as 1.0, x should be 10, y should be 40
    assert_eq!(result[0], Point64::new(10, 40));
}

#[test]
fn test_scale_path_empty() {
    let path: PathD = vec![];
    let mut error_code = 0;
    let result: Path64 = scale_path(&path, 2.0, 2.0, &mut error_code);
    assert_eq!(error_code, 0);
    assert!(result.is_empty());
}

#[test]
fn test_scale_path_uniform() {
    let path: Path64 = vec![Point64::new(10, 20), Point64::new(30, 40)];
    let mut error_code = 0;
    let result: PathD = scale_path_uniform(&path, 0.5, &mut error_code);
    assert_eq!(error_code, 0);
    assert!((result[0].x - 5.0).abs() < 1e-10);
    assert!((result[0].y - 10.0).abs() < 1e-10);
}

// --- ScalePaths tests ---

#[test]
fn test_scale_paths_basic() {
    let paths: PathsD = vec![
        vec![PointD::new(1.0, 2.0), PointD::new(3.0, 4.0)],
        vec![PointD::new(5.0, 6.0)],
    ];
    let mut error_code = 0;
    let result: Paths64 = scale_paths(&paths, 10.0, 10.0, &mut error_code);
    assert_eq!(error_code, 0);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0], Point64::new(10, 20));
    assert_eq!(result[0][1], Point64::new(30, 40));
    assert_eq!(result[1][0], Point64::new(50, 60));
}

#[test]
fn test_scale_paths_range_error() {
    // Create paths that when scaled would exceed MAX_COORD
    let large_val = (constants::MAX_COORD / 2) as f64;
    let paths: PathsD = vec![vec![PointD::new(large_val, large_val)]];
    let mut error_code = 0;
    let result: Paths64 = scale_paths(&paths, 10.0, 10.0, &mut error_code);
    assert_ne!(error_code & errors::RANGE_ERROR_I, 0);
    assert!(result.is_empty());
}

#[test]
fn test_scale_paths_uniform() {
    let paths: Paths64 = vec![vec![Point64::new(100, 200)]];
    let mut error_code = 0;
    let result: PathsD = scale_paths_uniform(&paths, 0.01, &mut error_code);
    assert_eq!(error_code, 0);
    assert!((result[0][0].x - 1.0).abs() < 1e-10);
    assert!((result[0][0].y - 2.0).abs() < 1e-10);
}

// --- TransformPath tests ---

#[test]
fn test_transform_path_d_to_i64() {
    let path: PathD = vec![PointD::new(1.4, 2.6), PointD::new(3.5, 4.5)];
    let result: Path64 = transform_path(&path);
    assert_eq!(result[0], Point64::new(1, 3)); // rounds
    assert_eq!(result[1], Point64::new(4, 5)); // rounds 3.5->4, 4.5->5 (banker's rounding: 4)
}

#[test]
fn test_transform_path_i64_to_d() {
    let path: Path64 = vec![Point64::new(10, 20)];
    let result: PathD = transform_path(&path);
    assert_eq!(result[0].x, 10.0);
    assert_eq!(result[0].y, 20.0);
}

#[test]
fn test_transform_paths_basic() {
    let paths: Paths64 = vec![
        vec![Point64::new(1, 2), Point64::new(3, 4)],
        vec![Point64::new(5, 6)],
    ];
    let result: PathsD = transform_paths(&paths);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][0].x, 1.0);
    assert_eq!(result[0][1].y, 4.0);
    assert_eq!(result[1][0].x, 5.0);
}

#[test]
fn test_transform_path_empty() {
    let path: PathD = vec![];
    let result: Path64 = transform_path(&path);
    assert!(result.is_empty());
}

#[test]
fn test_transform_paths_empty() {
    let paths: PathsD = vec![];
    let result: Paths64 = transform_paths(&paths);
    assert!(result.is_empty());
}

// --- NearEqual tests ---

#[test]
fn test_near_equal_same_point() {
    let p = Point64::new(10, 20);
    assert!(near_equal(&p, &p, 1.0));
}

#[test]
fn test_near_equal_close_points() {
    let p1 = PointD::new(1.0, 1.0);
    let p2 = PointD::new(1.0, 1.5);
    assert!(near_equal(&p1, &p2, 1.0)); // dist_sqr = 0.25 < 1.0
    assert!(!near_equal(&p1, &p2, 0.1)); // dist_sqr = 0.25 >= 0.1
}

#[test]
fn test_near_equal_far_points() {
    let p1 = Point64::new(0, 0);
    let p2 = Point64::new(100, 100);
    assert!(!near_equal(&p1, &p2, 100.0));
}

#[test]
fn test_near_equal_threshold_boundary() {
    let p1 = PointD::new(0.0, 0.0);
    let p2 = PointD::new(1.0, 0.0);
    // dist_sqr = 1.0, threshold = 1.0 -> false (strictly less than)
    assert!(!near_equal(&p1, &p2, 1.0));
    // dist_sqr = 1.0, threshold = 1.01 -> true
    assert!(near_equal(&p1, &p2, 1.01));
}

// --- StripNearEqual tests ---

#[test]
fn test_strip_near_equal_basic() {
    let path: PathD = vec![
        PointD::new(0.0, 0.0),
        PointD::new(0.1, 0.1),
        PointD::new(10.0, 10.0),
        PointD::new(10.05, 10.05),
        PointD::new(20.0, 20.0),
    ];
    // With threshold of 1.0 (max_dist_sqrd), 0.1^2 + 0.1^2 = 0.02 < 1.0
    let result = strip_near_equal(&path, 1.0, false);
    assert_eq!(result.len(), 3); // should keep (0,0), (10,10), (20,20)
    assert_eq!(result[0], PointD::new(0.0, 0.0));
    assert_eq!(result[1], PointD::new(10.0, 10.0));
    assert_eq!(result[2], PointD::new(20.0, 20.0));
}

#[test]
fn test_strip_near_equal_closed_path() {
    let path: PathD = vec![
        PointD::new(0.0, 0.0),
        PointD::new(10.0, 0.0),
        PointD::new(10.0, 10.0),
        PointD::new(0.05, 0.05), // near first point
    ];
    let result = strip_near_equal(&path, 1.0, true);
    assert_eq!(result.len(), 3); // last point removed (near first)
}

#[test]
fn test_strip_near_equal_empty() {
    let path: PathD = vec![];
    let result = strip_near_equal(&path, 1.0, false);
    assert!(result.is_empty());
}

#[test]
fn test_strip_near_equal_single_point() {
    let path: PathD = vec![PointD::new(1.0, 2.0)];
    let result = strip_near_equal(&path, 1.0, false);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_strip_near_equal_paths_basic() {
    let paths: PathsD = vec![
        vec![
            PointD::new(0.0, 0.0),
            PointD::new(0.01, 0.01),
            PointD::new(10.0, 10.0),
        ],
        vec![
            PointD::new(5.0, 5.0),
            PointD::new(5.0, 5.0),
            PointD::new(15.0, 15.0),
        ],
    ];
    let result = strip_near_equal_paths(&paths, 1.0, false);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].len(), 2); // stripped near-equal
    assert_eq!(result[1].len(), 2); // stripped duplicate
}

// --- TranslatePoint tests ---

#[test]
fn test_translate_point_i64() {
    let pt = Point64::new(10, 20);
    let result = translate_point(&pt, 5.0, -3.0);
    assert_eq!(result, Point64::new(15, 17));
}

#[test]
fn test_translate_point_f64() {
    let pt = PointD::new(1.0, 2.0);
    let result = translate_point(&pt, 0.5, 0.5);
    assert!((result.x - 1.5).abs() < 1e-10);
    assert!((result.y - 2.5).abs() < 1e-10);
}

#[test]
fn test_translate_point_zero() {
    let pt = Point64::new(42, 99);
    let result = translate_point(&pt, 0.0, 0.0);
    assert_eq!(result, pt);
}

#[test]
fn test_translate_point_negative() {
    let pt = Point64::new(100, 200);
    let result = translate_point(&pt, -50.0, -100.0);
    assert_eq!(result, Point64::new(50, 100));
}

// --- ReflectPoint tests ---

#[test]
fn test_reflect_point_basic() {
    let pt = Point64::new(10, 20);
    let pivot = Point64::new(15, 25);
    let result = reflect_point(&pt, &pivot);
    assert_eq!(result, Point64::new(20, 30));
}

#[test]
fn test_reflect_point_origin() {
    let pt = Point64::new(5, 5);
    let pivot = Point64::new(0, 0);
    let result = reflect_point(&pt, &pivot);
    assert_eq!(result, Point64::new(-5, -5));
}

#[test]
fn test_reflect_point_same() {
    let pt = Point64::new(10, 20);
    let result = reflect_point(&pt, &pt);
    assert_eq!(result, pt);
}

#[test]
fn test_reflect_point_f64() {
    let pt = PointD::new(1.0, 2.0);
    let pivot = PointD::new(3.0, 4.0);
    let result = reflect_point(&pt, &pivot);
    assert!((result.x - 5.0).abs() < 1e-10);
    assert!((result.y - 6.0).abs() < 1e-10);
}

// --- GetSign tests ---

#[test]
fn test_get_sign_positive() {
    assert_eq!(get_sign(&5i64), 1);
    assert_eq!(get_sign(&1i64), 1);
    assert_eq!(get_sign(&100.0f64), 1);
    assert_eq!(get_sign(&0.001f64), 1);
}

#[test]
fn test_get_sign_negative() {
    assert_eq!(get_sign(&-5i64), -1);
    assert_eq!(get_sign(&-1i64), -1);
    assert_eq!(get_sign(&-100.0f64), -1);
    assert_eq!(get_sign(&-0.001f64), -1);
}

#[test]
fn test_get_sign_zero() {
    assert_eq!(get_sign(&0i64), 0);
    assert_eq!(get_sign(&0.0f64), 0);
}

// --- CrossProductSign tests ---

#[test]
fn test_cross_product_sign_positive() {
    // Counter-clockwise turn -> positive cross product
    let pt1 = Point64::new(0, 0);
    let pt2 = Point64::new(10, 0);
    let pt3 = Point64::new(10, 10);
    assert_eq!(cross_product_sign(pt1, pt2, pt3), 1);
}

#[test]
fn test_cross_product_sign_negative() {
    // Clockwise turn -> negative cross product
    let pt1 = Point64::new(0, 0);
    let pt2 = Point64::new(10, 0);
    let pt3 = Point64::new(10, -10);
    assert_eq!(cross_product_sign(pt1, pt2, pt3), -1);
}

#[test]
fn test_cross_product_sign_collinear() {
    let pt1 = Point64::new(0, 0);
    let pt2 = Point64::new(5, 5);
    let pt3 = Point64::new(10, 10);
    assert_eq!(cross_product_sign(pt1, pt2, pt3), 0);
}

#[test]
fn test_cross_product_sign_large_values() {
    // Test with values that would overflow 64-bit multiplication
    let large = i64::MAX / 4;
    let pt1 = Point64::new(0, 0);
    let pt2 = Point64::new(large, 0);
    let pt3 = Point64::new(large, large);
    assert_eq!(cross_product_sign(pt1, pt2, pt3), 1);
}

#[test]
fn test_cross_product_sign_consistency_with_cross_product() {
    // Verify sign matches the floating-point cross product
    let test_cases = [
        (Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, 5)),
        (Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, -5)),
        (
            Point64::new(0, 0),
            Point64::new(10, 10),
            Point64::new(20, 20),
        ),
        (
            Point64::new(-100, -100),
            Point64::new(100, 50),
            Point64::new(50, 200),
        ),
    ];

    for (pt1, pt2, pt3) in test_cases {
        let cp = cross_product_three_points(pt1, pt2, pt3);
        let cps = cross_product_sign(pt1, pt2, pt3);
        if cp > 0.0 {
            assert_eq!(cps, 1, "Mismatch for {:?}, {:?}, {:?}", pt1, pt2, pt3);
        } else if cp < 0.0 {
            assert_eq!(cps, -1, "Mismatch for {:?}, {:?}, {:?}", pt1, pt2, pt3);
        } else {
            assert_eq!(cps, 0, "Mismatch for {:?}, {:?}, {:?}", pt1, pt2, pt3);
        }
    }
}

// --- SegmentsIntersect tests ---

#[test]
fn test_segments_intersect_crossing() {
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 10);
    let c = Point64::new(0, 10);
    let d = Point64::new(10, 0);
    assert!(segments_intersect(a, b, c, d, false));
    assert!(segments_intersect(a, b, c, d, true));
}

#[test]
fn test_segments_intersect_parallel() {
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 0);
    let c = Point64::new(0, 5);
    let d = Point64::new(10, 5);
    assert!(!segments_intersect(a, b, c, d, false));
    assert!(!segments_intersect(a, b, c, d, true));
}

#[test]
fn test_segments_intersect_non_crossing() {
    let a = Point64::new(0, 0);
    let b = Point64::new(5, 5);
    let c = Point64::new(6, 0);
    let d = Point64::new(10, 0);
    assert!(!segments_intersect(a, b, c, d, false));
}

#[test]
fn test_segments_intersect_t_shape_inclusive() {
    // Segment endpoint touches the other segment
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 0);
    let c = Point64::new(5, 5);
    let d = Point64::new(5, 0);
    // With inclusive=true, touching at endpoint should count
    assert!(segments_intersect(a, b, c, d, true));
    // With inclusive=false, should not count
    assert!(!segments_intersect(a, b, c, d, false));
}

#[test]
fn test_segments_intersect_collinear() {
    // Collinear segments (overlapping)
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 0);
    let c = Point64::new(5, 0);
    let d = Point64::new(15, 0);
    // Collinear segments should return false even with inclusive
    assert!(!segments_intersect(a, b, c, d, true));
}

// --- GetClosestPointOnSegment tests ---

#[test]
fn test_closest_point_on_segment_midpoint() {
    let off = Point64::new(5, 10);
    let seg1 = Point64::new(0, 0);
    let seg2 = Point64::new(10, 0);
    let result = get_closest_point_on_segment(off, seg1, seg2);
    assert_eq!(result, Point64::new(5, 0));
}

#[test]
fn test_closest_point_on_segment_start() {
    let off = Point64::new(-5, 5);
    let seg1 = Point64::new(0, 0);
    let seg2 = Point64::new(10, 0);
    let result = get_closest_point_on_segment(off, seg1, seg2);
    assert_eq!(result, Point64::new(0, 0));
}

#[test]
fn test_closest_point_on_segment_end() {
    let off = Point64::new(15, 5);
    let seg1 = Point64::new(0, 0);
    let seg2 = Point64::new(10, 0);
    let result = get_closest_point_on_segment(off, seg1, seg2);
    assert_eq!(result, Point64::new(10, 0));
}

#[test]
fn test_closest_point_on_segment_degenerate() {
    let off = Point64::new(5, 5);
    let seg1 = Point64::new(3, 3);
    let seg2 = Point64::new(3, 3); // degenerate segment (point)
    let result = get_closest_point_on_segment(off, seg1, seg2);
    assert_eq!(result, Point64::new(3, 3));
}

#[test]
fn test_closest_point_on_segment_f64() {
    let off = PointD::new(5.0, 10.0);
    let seg1 = PointD::new(0.0, 0.0);
    let seg2 = PointD::new(10.0, 0.0);
    let result = get_closest_point_on_segment(off, seg1, seg2);
    assert!((result.x - 5.0).abs() < 1e-10);
    assert!((result.y - 0.0).abs() < 1e-10);
}

#[test]
fn test_closest_point_on_segment_diagonal() {
    let off = PointD::new(0.0, 10.0);
    let seg1 = PointD::new(0.0, 0.0);
    let seg2 = PointD::new(10.0, 10.0);
    let result = get_closest_point_on_segment(off, seg1, seg2);
    // Projection of (0,10) onto line from (0,0) to (10,10)
    // q = ((0-0)*10 + (10-0)*10) / (100+100) = 100/200 = 0.5
    // result = (0 + 0.5*10, 0 + 0.5*10) = (5, 5)
    assert!((result.x - 5.0).abs() < 1e-10);
    assert!((result.y - 5.0).abs() < 1e-10);
}

// --- FromF64 trait tests ---

#[test]
fn test_from_f64_i64_rounding() {
    assert_eq!(i64::from_f64(1.4), 1);
    assert_eq!(i64::from_f64(1.5), 2);
    assert_eq!(i64::from_f64(1.6), 2);
    assert_eq!(i64::from_f64(-1.4), -1);
    assert_eq!(i64::from_f64(-1.5), -2);
    assert_eq!(i64::from_f64(-1.6), -2);
}

#[test]
fn test_from_f64_f64_exact() {
    assert_eq!(f64::from_f64(1.4), 1.4);
    assert_eq!(f64::from_f64(-1.5), -1.5);
    assert_eq!(f64::from_f64(0.0), 0.0);
}

#[test]
fn test_is_integral() {
    assert!(i64::is_integral());
    assert!(!f64::is_integral());
}
