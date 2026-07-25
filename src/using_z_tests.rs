// Tests for the using_z feature (C++ USINGZ equivalent).
// These verify exact behavioral match with the C++ implementation:
// Point z semantics from clipper.core.h, z-callback and vertex z-inheritance
// from clipper.engine.cpp (SetZ), and the ClipperD descaling proxy from
// clipper.engine.h (ZCB).

use crate::clipper::{make_path64, make_path_z, make_path_zd};
use crate::core::{get_segment_intersect_pt, mid_point, Point64, PointD};
use crate::engine::ClipType;
use crate::engine_public::{Clipper64, ClipperD};
use crate::{FillRule, Paths64, PathsD};

// ----------------------------------------------------------------------------
// Point z semantics (clipper.core.h)
// ----------------------------------------------------------------------------

#[test]
fn test_point_equality_ignores_z() {
    // C++ operator== compares only x and y under USINGZ
    let a = Point64::new_z(10, 20, 5);
    let b = Point64::new_z(10, 20, 99);
    assert_eq!(a, b);
    assert_ne!(a, Point64::new_z(10, 21, 5));
}

#[test]
fn test_point_arithmetic_zeroes_z() {
    // C++ operator+/-/unary- construct via Point(x, y), which zeroes z
    let a = Point64::new_z(10, 20, 5);
    let b = Point64::new_z(1, 2, 7);
    assert_eq!(a.add_point(b).z, 0);
    assert_eq!(a.sub_point(b).z, 0);
    assert_eq!(a.negate().z, 0);
    // mid_point also constructs via Point(x, y)
    assert_eq!(mid_point(a, b).z, 0);
}

#[test]
fn test_point_scale_preserves_z() {
    // C++ operator* is the one arithmetic op that carries z through
    let a = PointD::new_z(10.0, 20.0, 5);
    let s: PointD = a.scale(2.0f64);
    assert_eq!(s.z, 5);
}

#[test]
fn test_segment_intersect_pt_zeroes_z() {
    // C++ GetSegmentIntersectPt explicitly writes ip.z = 0
    let mut ip = Point64::new_z(0, 0, 77);
    let found = get_segment_intersect_pt(
        Point64::new(0, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
        Point64::new(10, 0),
        &mut ip,
    );
    assert!(found);
    assert_eq!(ip.z, 0);
}

#[test]
fn test_make_path_z() {
    let path = make_path_z(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(path.len(), 2);
    assert_eq!((path[0].x, path[0].y, path[0].z), (1, 2, 3));
    assert_eq!((path[1].x, path[1].y, path[1].z), (4, 5, 6));
}

#[test]
fn test_make_path_zd() {
    // C++ MakePathZD truncates the z double with static_cast<int64_t>
    let path = make_path_zd(&[1.5, 2.5, 3.9, 4.5, 5.5, -6.9]);
    assert_eq!(path.len(), 2);
    assert_eq!(path[0].z, 3);
    assert_eq!(path[1].z, -6);
}

// ----------------------------------------------------------------------------
// Engine z-callback (clipper.engine.cpp SetZ / IntersectEdges)
// ----------------------------------------------------------------------------

/// Self-intersecting pentagram from the C++ UsingZ example: NonZero union
/// produces one outer ring whose 5 crossing points are new intersections
/// (callback fires) and whose 5 tips are original vertices (callback does not).
fn pentagram() -> Paths64 {
    vec![make_path64(&[100, 50, 10, 79, 65, 2, 65, 98, 10, 21])]
}

#[test]
fn test_z_callback_fires_at_intersections_64() {
    let mut c = Clipper64::new();
    c.add_subject(&pentagram());
    c.set_z_callback(|_, _, _, _, pt| pt.z = 1);
    let mut solution = Paths64::new();
    c.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);

    assert_eq!(solution.len(), 1);
    assert_eq!(solution[0].len(), 10);
    let flagged = solution[0].iter().filter(|pt| pt.z == 1).count();
    let unflagged = solution[0].iter().filter(|pt| pt.z == 0).count();
    assert_eq!(
        flagged, 5,
        "each pentagram crossing must invoke the callback"
    );
    assert_eq!(unflagged, 5, "original tips must keep their input z");
}

#[test]
fn test_z_callback_receives_edge_vertices_64() {
    // Every callback invocation passes the four defining edge vertices; for
    // the pentagram all of them are original input vertices, so each must
    // match an input point exactly.
    let subject = pentagram();
    let inputs = subject[0].clone();
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.set_z_callback(move |e1b, e1t, e2b, e2t, pt| {
        for v in [e1b, e1t, e2b, e2t] {
            assert!(
                inputs.iter().any(|p| p == v),
                "callback edge vertex {},{} is not an input vertex",
                v.x,
                v.y
            );
        }
        pt.z = 1;
    });
    let mut solution = Paths64::new();
    c.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);
    assert_eq!(solution[0].iter().filter(|pt| pt.z == 1).count(), 5);
}

#[test]
fn test_default_z_used_when_no_vertex_matches() {
    // C++ SetZ assigns DefaultZ when the intersection matches no edge vertex,
    // and it does so before the callback runs; a no-op callback preserves it.
    let mut c = Clipper64::new();
    c.base.default_z = 42;
    c.add_subject(&pentagram());
    c.set_z_callback(|_, _, _, _, _| {});
    let mut solution = Paths64::new();
    c.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);
    let flagged = solution[0].iter().filter(|pt| pt.z == 42).count();
    assert_eq!(flagged, 5, "unmatched intersections must receive default_z");
}

#[test]
fn test_z_inheritance_from_matching_vertex() {
    // When the intersection coincides with an edge vertex, SetZ inherits that
    // vertex's z (checking subject edges before clip edges). The clip square's
    // corner (50,50) lies inside the subject and its corners (50,0)/(0,50)
    // style crossings land exactly on clip vertices here: subject and clip
    // share the vertex (100, 100), which is also an intersection of the two
    // boundaries, so the output point there must carry the subject's z.
    let subject = vec![make_path_z(&[0, 0, 7, 100, 0, 7, 100, 100, 7, 0, 100, 7])];
    let clip = vec![make_path_z(&[
        50, 50, 9, 150, 50, 9, 150, 150, 9, 50, 150, 9,
    ])];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.add_clip(&clip);
    // no-op callback: the z visible in the solution is whatever SetZ assigned
    c.set_z_callback(|_, _, _, _, _| {});
    let mut solution = Paths64::new();
    c.execute(
        ClipType::Intersection,
        FillRule::NonZero,
        &mut solution,
        None,
    );

    assert_eq!(solution.len(), 1);
    // Solution is the square (50,50)-(100,100). The two boundary crossings
    // (100,50) and (50,100) match no input vertex -> default_z (0). The
    // corners (100,100) and (50,50) are original vertices carried through
    // from subject (z=7) and clip (z=9) respectively.
    for pt in &solution[0] {
        match (pt.x, pt.y) {
            (100, 100) => assert_eq!(pt.z, 7, "subject corner keeps subject z"),
            (50, 50) => assert_eq!(pt.z, 9, "clip corner keeps clip z"),
            (100, 50) | (50, 100) => assert_eq!(pt.z, 0, "crossings get default_z"),
            other => panic!("unexpected solution vertex {:?}", other),
        }
    }
}

#[test]
fn test_z_callback_fires_for_xor() {
    // Regression for the IntersectEdges Xor arm: C++ captures the
    // AddLocalMinPoly result and calls SetZ on it (clipper.engine.cpp
    // ClipType::Xor case); the port originally discarded it.
    let subject = vec![
        make_path64(&[0, 0, 100, 0, 100, 100, 0, 100]),
        make_path64(&[50, 50, 150, 50, 150, 150, 50, 150]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.set_z_callback(|_, _, _, _, pt| pt.z = 1);
    let mut solution = Paths64::new();
    c.execute(ClipType::Xor, FillRule::EvenOdd, &mut solution, None);

    // The two boundary crossings (100,50) and (50,100) each appear in two
    // rings of the Xor result; every occurrence must have been flagged.
    let mut crossings = 0;
    for path in &solution {
        for pt in path {
            if (pt.x, pt.y) == (100, 50) || (pt.x, pt.y) == (50, 100) {
                assert_eq!(pt.z, 1, "crossing {},{} missed the callback", pt.x, pt.y);
                crossings += 1;
            }
        }
    }
    assert_eq!(crossings, 4);
}

// ----------------------------------------------------------------------------
// ClipperD descaling proxy (clipper.engine.h ZCB)
// ----------------------------------------------------------------------------

#[test]
fn test_clipper_d_callback_sees_descaled_coordinates() {
    let subject: PathsD = vec![crate::clipper::make_path_d(&[
        100.0, 50.0, 10.0, 79.0, 65.0, 2.0, 65.0, 98.0, 10.0, 21.0,
    ])];
    let mut c = ClipperD::new(2);
    c.add_subject(&subject);
    c.set_z_callback(|e1b, e1t, e2b, e2t, pt| {
        // With precision 2 the engine works on x100 coordinates; the proxy
        // must descale them back to the caller's space before invoking us.
        for v in [e1b, e1t, e2b, e2t] {
            assert!(
                v.x <= 100.0 && v.y <= 100.0,
                "coordinate not descaled: {},{}",
                v.x,
                v.y
            );
        }
        pt.z = 1;
    });
    let mut solution = PathsD::new();
    c.execute(ClipType::Union, FillRule::NonZero, &mut solution, None);

    // build_path_d_from_outpt must preserve z through the f64 conversion
    assert_eq!(solution.len(), 1);
    assert_eq!(solution[0].iter().filter(|pt| pt.z == 1).count(), 5);
}
