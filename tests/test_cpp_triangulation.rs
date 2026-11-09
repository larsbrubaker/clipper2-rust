//! Regression and coverage tests for the C++ triangulation port.
use clipper2_rust::{area, make_path64, triangulate_64, triangulate_d, Paths64, TriangulateResult};

fn check_area(paths: &Paths64, expected: f64) {
    for delaunay in [false, true] {
        let mut triangles = Vec::new();
        assert_eq!(
            triangulate_64(paths, &mut triangles, delaunay),
            TriangulateResult::Success
        );
        assert!(triangles.iter().all(|t| t.len() == 3 && area(t) > 0.0));
        assert_eq!(
            triangles.iter().map(|t| area(t).abs()).sum::<f64>(),
            expected
        );
        // Area alone can hide overlapping triangles and missing coverage.
        let expected_region =
            clipper2_rust::union_subjects_64(paths, clipper2_rust::FillRule::NonZero);
        let actual_region =
            clipper2_rust::union_subjects_64(&triangles, clipper2_rust::FillRule::NonZero);
        assert!(clipper2_rust::xor_64(
            &expected_region,
            &actual_region,
            clipper2_rust::FillRule::NonZero
        )
        .is_empty());
    }
}

#[test]
fn edge_through_vertex_issue_12() {
    let paths = vec![make_path64(&[
        -209, 3155, 716, 3534, 3324, 2439, 298, 9844, 507, 6689,
    ])];
    check_area(&paths, 9498178.0);
}

#[test]
fn edge_through_hole_vertices_issue_12() {
    let paths = vec![
        make_path64(&[
            320313, 10000, 310000, 160313, 99688, 150000, -313, 90000, 10000, 59688,
        ]),
        make_path64(&[
            210000, 150000, 220000, 130000, 220000, 150000, 310000, 150000, 10000, 70000,
        ]),
    ];
    check_area(&paths, 30025200468.5);
}

#[test]
fn simple_concave_holes_and_disjoint_paths() {
    let outer = make_path64(&[0, 0, 100, 0, 100, 100, 0, 100]);
    let hole = make_path64(&[20, 20, 20, 80, 80, 80, 80, 20]);
    check_area(&vec![outer.clone()], 10000.0);
    check_area(&vec![outer.clone(), hole], 6400.0);
    check_area(
        &vec![outer, make_path64(&[200, 0, 300, 0, 300, 100, 200, 100])],
        20000.0,
    );
    check_area(
        &vec![make_path64(&[
            0, 0, 100, 0, 100, 40, 40, 40, 40, 100, 0, 100,
        ])],
        6400.0,
    );
}

#[test]
fn empty_collinear_and_intersecting_input() {
    let mut triangles = vec![make_path64(&[0, 0, 10, 0, 0, 10])];
    assert_eq!(
        triangulate_64(&vec![], &mut triangles, true),
        TriangulateResult::NoPolygons
    );
    assert!(triangles.is_empty());
    assert_eq!(
        triangulate_64(
            &vec![make_path64(&[0, 0, 50, 50, 100, 100])],
            &mut triangles,
            true
        ),
        TriangulateResult::NoPolygons
    );
    assert_eq!(
        triangulate_64(
            &vec![make_path64(&[0, 0, 100, 100, 0, 100, 100, 0])],
            &mut triangles,
            true
        ),
        TriangulateResult::PathsIntersect
    );
    assert!(triangles.is_empty());
}

#[test]
fn float_paths_and_precision_clamping() {
    let paths = vec![vec![
        clipper2_rust::PointD::new(0.1, 0.1),
        clipper2_rust::PointD::new(10.1, 0.1),
        clipper2_rust::PointD::new(0.1, 10.1),
    ]];
    for precision in [-4, 0, 1, 8, 12] {
        let mut triangles = vec![];
        assert_eq!(
            triangulate_d(&paths, precision, &mut triangles, true),
            TriangulateResult::Success
        );
        assert_eq!(triangles.len(), 1);
        assert!((area(&triangles[0]) - 50.0).abs() < 1e-9);
    }
}

#[test]
fn large_coordinate_triangle_preserves_integer_differences() {
    // Corrected C++ returns this triangle in both modes. Converting the
    // coordinates to f64 before subtracting collapses these distinct vertices.
    let offset = 1_i64 << 60;
    let subject = make_path64(&[offset, offset, offset + 32, offset, offset, offset + 64]);
    for delaunay in [false, true] {
        let mut triangles = vec![];
        assert_eq!(
            triangulate_64(&vec![subject.clone()], &mut triangles, delaunay),
            TriangulateResult::Success
        );
        assert_eq!(
            canonical_triangles(triangles),
            canonical_triangles(vec![subject.clone()])
        );
    }
}

// Canonicalize vertex order, not geometry: different arena traversal orders may
// rotate triangle vertices or reorder triangles without changing the mesh.
fn canonical_triangles(paths: Paths64) -> Vec<Vec<(i64, i64)>> {
    let mut triangles: Vec<_> = paths
        .into_iter()
        .map(|p| {
            let mut vertices: Vec<_> = p.into_iter().map(|p| (p.x, p.y)).collect();
            vertices.sort_unstable();
            vertices
        })
        .collect();
    triangles.sort_unstable();
    triangles
}

fn check_cpp_reference(data: &str) {
    for (line_no, line) in data.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut values = line.split_whitespace().map(|s| s.parse::<i64>().unwrap());
        let delaunay = values.next().unwrap() != 0;
        let expected_status = values.next().unwrap();
        let n = values.next().unwrap();
        let mut paths = vec![];
        for _ in 0..n {
            let len = values.next().unwrap();
            let mut p = vec![];
            for _ in 0..len {
                p.push(clipper2_rust::Point64::new(
                    values.next().unwrap(),
                    values.next().unwrap(),
                ));
            }
            paths.push(p);
        }
        let n = values.next().unwrap();
        let mut expected = vec![];
        for _ in 0..n {
            let mut t = vec![];
            for _ in 0..3 {
                t.push(clipper2_rust::Point64::new(
                    values.next().unwrap(),
                    values.next().unwrap(),
                ));
            }
            expected.push(t);
        }
        assert!(values.next().is_none());
        let mut actual = vec![];
        let status = triangulate_64(&paths, &mut actual, delaunay);
        let status_code = status as i64;
        assert_eq!(
            status_code,
            expected_status,
            "reference line {}",
            line_no + 1
        );
        assert_eq!(
            canonical_triangles(actual),
            canonical_triangles(expected),
            "reference line {}",
            line_no + 1
        );
    }
}

#[test]
fn matches_corrected_cpp_reference() {
    check_cpp_reference(include_str!("data/Triangulation.txt"));
}

#[test]
fn winding_and_y_mirroring() {
    let subject = make_path64(&[-209, 3155, 716, 3534, 3324, 2439, 298, 9844, 507, 6689]);
    for reverse in [false, true] {
        for mirror in [false, true] {
            let mut p = subject.clone();
            if reverse {
                p.reverse();
            }
            if mirror {
                for v in &mut p {
                    v.y = -v.y;
                }
            }
            check_area(&vec![p], 9498178.0);
        }
    }
}

#[test]
fn collinear_vertices_with_either_winding() {
    let cases = [
        (
            make_path64(&[
                6000, 0, 8000, 6000, 4000, 4000, 0, 2000, 5000, 5000, 10000, 8000,
            ]),
            8000000.0,
        ),
        (
            make_path64(&[0, 0, 2000, 4000, 3000, 3000, 5000, 1000]),
            9000000.0,
        ),
    ];
    for (path, expected) in cases {
        let mut reversed = path.clone();
        reversed.reverse();
        check_area(&vec![path], expected);
        check_area(&vec![reversed], expected);
    }
}

#[test]
fn spike_at_first_local_minimum_is_ignored() {
    let path = make_path64(&[0, 0, 10, 0, 10, 10, 5, 10, 5, 20, 5, 10, 4, 2, 3, 10, 0, 10]);
    check_area(&vec![path], 92.0);
}

#[test]
fn horizontal_spike_at_path_closure_is_ignored() {
    check_area(&vec![make_path64(&[4, 4, 0, 0, 8, 0, 4, 4, 2, 4])], 16.0);
}

// Union output at unit coordinates can retrace an edge because of rounding.
#[test]
fn retraced_edge_from_union_output_is_ignored() {
    let path = make_path64(&[7, 4, 4, 1, 7, 2, 9, 3, 8, 3, 7, 3, 7, 4, 6, 4]);
    check_area(&vec![path], 4.0);
}

// The first path is a triangle with a unit-length edge, which is too small to
// triangulate. One of its edges meets the second path within rounding tolerance.
#[test]
fn ignored_tiny_path_meeting_another_path() {
    let paths = vec![
        make_path64(&[1, 3, 2, 3, 1, 5, 1, 4]),
        make_path64(&[2, 2, 4, 2, 2, 5, 2, 4, 1, 2]),
    ];
    for delaunay in [false, true] {
        let mut triangles = vec![];
        assert_eq!(
            triangulate_64(&paths, &mut triangles, delaunay),
            TriangulateResult::Success
        );
        assert_eq!(triangles.iter().map(area).sum::<f64>(), 4.5);
    }
}

// A small triangle cut from a path that touches another is part of its area.
#[test]
fn small_loop_of_touching_path_is_kept() {
    check_area(
        &vec![
            make_path64(&[3, 2, 2, 3, 0, 5, 2, 2]),
            make_path64(&[4, 3, 4, 4, 2, 3, 3, 2]),
        ],
        3.5,
    );
}

// C++ only checked edges for crossings against an inner minimum's connecting edge
// when the edge lay inside that edge's x range, which fails when it heads right.
#[test]
fn inner_minimum_edge_avoids_edges_on_its_right() {
    check_area(
        &vec![make_path64(&[
            744413, 460911, 582000, 563000, 675000, 699000, 483935, 477767, 486556, 455276, 566000,
            496000, 729568, 441477,
        ])],
        14506790027.5,
    );
    check_area(
        &vec![make_path64(&[
            20000, 21000, 27000, 13000, 48939, 25683, 31000, 59000, 41790, 59378, 38000, 74000,
            5000, 5000,
        ])],
        938833000.0,
    );
}

// The vertex at (0, 3000) lies on the edge from (0, 4000) to (0, 2000).
#[test]
fn vertex_on_edge_of_same_path() {
    check_area(
        &vec![make_path64(&[
            5000, 0, 0, 3000, 3000, 3000, 0, 4000, 0, 2000, 2000, 0,
        ])],
        7000000.0,
    );
}

// The path passes through its lowest point twice, which decides its orientation.
#[test]
fn path_touching_itself_at_lowest_point() {
    let mut path = make_path64(&[
        3000, 1000, 3000, 2000, 5000, 5000, 3000, 3000, 2000, 3000, 5000, 5000, 5000, 2000, 4000,
        1000,
    ]);
    check_area(&vec![path.clone()], 5500000.0);
    path.reverse();
    check_area(&vec![path], 5500000.0);
}

// Edges traced in opposite directions by two paths bound nothing: the area on
// both sides of them is either filled (first case) or empty (second case).
#[test]
fn edges_shared_in_opposite_directions_cancel() {
    check_area(
        &vec![
            make_path64(&[10000, 12000, 10000, 13000, 10000, 19000, 2000, 2000]),
            make_path64(&[10000, 7000, 13000, 7000, 10000, 13000]),
        ],
        37000000.0,
    );
    check_area(
        &vec![
            make_path64(&[4000, 5000, 0, 5000, 2333, 2666, 2000, 4000, 3000, 2000]),
            make_path64(&[3000, 2000, 2000, 4000, 2333, 2666]),
        ],
        6001500.0,
    );
}

// At (2000, 10000) a lobe sits in the notch of a reflex vertex. The area below
// that point is filled, so it still needs a connecting edge.
#[test]
fn lobe_touching_reflex_vertex_keeps_area_below() {
    check_area(
        &vec![
            make_path64(&[
                2000, 10000, 2000, 6000, 6571, 4857, 5200, 7600, 2000, 10000, 5058, 7882, 0, 18000,
                1000, 3000,
            ]),
            make_path64(&[
                5058, 7882, 5200, 7600, 10000, 4000, 6571, 4857, 8000, 2000, 15000, 1000,
            ]),
        ],
        47998800.0,
    );
}

#[cfg(feature = "using_z")]
#[test]
fn preserves_vertex_z_values() {
    let mut p = make_path64(&[0, 0, 100, 0, 100, 100, 0, 100]);
    for (i, v) in p.iter_mut().enumerate() {
        v.z = i as i64 + 10;
    }
    for delaunay in [false, true] {
        let mut triangles = vec![];
        assert_eq!(
            triangulate_64(&vec![p.clone()], &mut triangles, delaunay),
            TriangulateResult::Success
        );
        for v in triangles.iter().flatten() {
            let original = p.iter().find(|q| q.x == v.x && q.y == v.y).unwrap();
            assert_eq!(v.z, original.z);
        }
    }
}
