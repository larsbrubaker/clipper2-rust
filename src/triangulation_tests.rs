//! Geometric predicates used by the triangulation sweep.
use super::*;

#[test]
fn segment_interior_excludes_endpoints_and_noncollinear_points() {
    for (a, b, middle) in [
        (Point64::new(0, 0), Point64::new(10, 0), Point64::new(5, 0)),
        (Point64::new(0, 0), Point64::new(0, 10), Point64::new(0, 5)),
        (Point64::new(0, 0), Point64::new(10, 10), Point64::new(5, 5)),
    ] {
        assert!(on_segment_interior(a, b, middle));
        assert!(on_segment_interior(b, a, middle));
        assert!(!on_segment_interior(a, b, a));
        assert!(!on_segment_interior(a, b, b));
        assert!(!on_segment_interior(a, b, Point64::new(11, 11)));
        assert!(!on_segment_interior(a, b, Point64::new(4, 3)));
    }
    let offset = 1_i64 << 60;
    assert!(on_segment_interior(
        Point64::new(offset, offset),
        Point64::new(offset + 10, offset + 10),
        Point64::new(offset + 5, offset + 5),
    ));
    assert!(!on_segment_interior(
        Point64::new(1, 1),
        Point64::new(1, 1),
        Point64::new(1, 1),
    ));
}

#[test]
fn segment_distance_matches_cpp_for_projection_and_endpoints() {
    let offset = 1_i64 << 60;
    for base in [0, offset, -offset] {
        let a = Point64::new(base, base);
        let b = Point64::new(base + 10, base);
        assert_eq!(squared_distance(a, b), 100.0);
        assert_eq!(shortest_dist(Point64::new(base + 5, base + 3), a, b), 9.0);
        assert_eq!(shortest_dist(Point64::new(base - 4, base + 3), a, b), 25.0);
        assert_eq!(shortest_dist(Point64::new(base + 14, base + 3), a, b), 25.0);
    }
}

#[test]
fn strict_intersections_exclude_parallel_and_touching_segments() {
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 10);
    assert!(segments_intersect(
        a,
        b,
        Point64::new(0, 10),
        Point64::new(10, 0)
    ));
    assert!(!segments_intersect(a, b, a, Point64::new(10, 0)));
    assert!(!segments_intersect(
        a,
        b,
        Point64::new(5, 5),
        Point64::new(15, 15)
    ));
    assert!(!segments_intersect(
        a,
        b,
        Point64::new(20, 0),
        Point64::new(20, 30)
    ));
}

#[test]
fn incircle_sign_matches_cpp_and_reverses_with_winding() {
    let a = Point64::new(0, 0);
    let b = Point64::new(10, 0);
    let c = Point64::new(0, 10);
    let inside = Point64::new(5, 5);
    assert_eq!(in_circle(a, b, c, inside), 5000.0);
    assert_eq!(in_circle(c, b, a, inside), -5000.0);
    assert_eq!(in_circle(a, b, c, Point64::new(20, 20)), -40000.0);
    assert_eq!(in_circle(a, b, c, Point64::new(10, 10)), 0.0);
}

#[test]
fn local_minimum_search_rejects_short_and_horizontal_paths() {
    for path in [
        vec![],
        vec![Point64::new(0, 0)],
        crate::make_path64(&[0, 0, 10, 0, 20, 0]),
    ] {
        assert!(!find_loc_min(&path, &mut 0));
    }
    let path = crate::make_path64(&[0, 0, 10, 0, 10, 10, 0, 10]);
    let mut i = 0;
    assert!(find_loc_min(&path, &mut i));
    // C++ selects index 3, at the left end of the bottom horizontal edge.
    assert_eq!(i, 3);
}
