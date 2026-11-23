//! Tests for the optional `geo` feature: conversions between clipper2-rust
//! paths and polytrees and the `geo-types` primitives used by the `geo` crate.

use clipper2_rust::geo::*;
use clipper2_rust::*;
use geo_types::{coord, line_string, polygon, Coord, LineString, MultiPolygon, Polygon};

fn square_d(x: f64, y: f64, size: f64) -> Polygon<f64> {
    polygon![
        (x: x, y: y),
        (x: x + size, y: y),
        (x: x + size, y: y + size),
        (x: x, y: y + size),
    ]
}

#[test]
fn test_point_coord_round_trip() {
    let c: Coord<f64> = coord! { x: 1.5, y: -2.25 };
    let p: PointD = c.into();
    assert_eq!(p, PointD::new(1.5, -2.25));
    let back: Coord<f64> = p.into();
    assert_eq!(back, c);

    let c64: Coord<i64> = coord! { x: 7, y: -9 };
    let p64: Point64 = c64.into();
    assert_eq!(p64, Point64::new(7, -9));
    assert_eq!(Coord::<i64>::from(p64), c64);
}

#[test]
fn test_path_from_ring_drops_closing_point() {
    let ring = square_d(0.0, 0.0, 10.0).exterior().clone();
    assert_eq!(ring.0.len(), 5, "geo rings repeat the first point");
    let path = path_from_ring(&ring);
    assert_eq!(
        path,
        vec![
            PointD::new(0.0, 0.0),
            PointD::new(10.0, 0.0),
            PointD::new(10.0, 10.0),
            PointD::new(0.0, 10.0),
        ]
    );
}

#[test]
fn test_path_from_line_string_keeps_all_points() {
    let ls: LineString<f64> = line_string![(x: 0.0, y: 0.0), (x: 5.0, y: 0.0), (x: 0.0, y: 0.0)];
    let path = path_from_line_string(&ls);
    assert_eq!(path.len(), 3);
    assert_eq!(line_string_from_path(&path), ls);
}

#[test]
fn test_paths_from_polygon_orients_exterior_positive_and_holes_negative() {
    // Exterior given clockwise and hole given counter-clockwise: the opposite
    // of what NonZero needs to see the hole.
    let exterior = line_string![
        (x: 0.0, y: 0.0), (x: 0.0, y: 10.0), (x: 10.0, y: 10.0), (x: 10.0, y: 0.0)
    ];
    let hole = line_string![
        (x: 2.0, y: 2.0), (x: 8.0, y: 2.0), (x: 8.0, y: 8.0), (x: 2.0, y: 8.0)
    ];
    let poly = Polygon::new(exterior, vec![hole]);
    let paths = paths_from_polygon(&poly);
    assert_eq!(paths.len(), 2);
    assert!(area(&paths[0]) > 0.0);
    assert!(area(&paths[1]) < 0.0);

    // NonZero union keeps the hole, so the filled area is 100 - 36.
    let united = union_d(&paths, &PathsD::new(), FillRule::NonZero, 2);
    assert!((area_paths(&united) - 64.0).abs() < 1e-9);
}

#[test]
fn test_union_d_tree_to_multi_polygon_with_hole_and_island() {
    // A frame (square with a square hole), plus a small island inside the hole.
    let frame = Polygon::new(
        square_d(0.0, 0.0, 30.0).exterior().clone(),
        vec![square_d(10.0, 10.0, 10.0).exterior().clone()],
    );
    let island = square_d(13.0, 13.0, 4.0);
    let subjects = paths_from_multi_polygon(&MultiPolygon::new(vec![frame, island]));

    let mut clipper = ClipperD::new(2);
    clipper.add_subject(&subjects);
    let mut tree = PolyTreeD::new();
    let mut open = PathsD::new();
    assert!(clipper.execute_tree(ClipType::Union, FillRule::NonZero, &mut tree, &mut open));

    let mp = multi_polygon_from_poly_tree_d(&tree);
    assert_eq!(mp.0.len(), 2, "frame and island are separate polygons");
    let frame_out = mp.0.iter().find(|p| p.interiors().len() == 1).unwrap();
    let island_out = mp.0.iter().find(|p| p.interiors().is_empty()).unwrap();
    assert!(frame_out.exterior().is_closed());
    assert!(frame_out.interiors()[0].is_closed());
    assert_eq!(frame_out.exterior().0.len(), 5);
    assert_eq!(island_out.exterior().0.len(), 5);

    let ext_area = area(&path_from_ring(frame_out.exterior())).abs();
    let hole_area = area(&path_from_ring(&frame_out.interiors()[0])).abs();
    assert!((ext_area - 900.0).abs() < 1e-9);
    assert!((hole_area - 100.0).abs() < 1e-9);
    assert!((area(&path_from_ring(island_out.exterior())).abs() - 16.0).abs() < 1e-9);
}

#[test]
fn test_poly_tree64_to_multi_polygon() {
    let a: Polygon<i64> = polygon![(x: 0, y: 0), (x: 20, y: 0), (x: 20, y: 20), (x: 0, y: 20)];
    let b: Polygon<i64> = polygon![(x: 5, y: 5), (x: 15, y: 5), (x: 15, y: 15), (x: 5, y: 15)];

    let mut clipper = Clipper64::new();
    clipper.add_subject(&paths_from_polygon(&a));
    clipper.add_clip(&paths_from_polygon(&b));
    let mut tree = PolyTree64::new();
    let mut open = Paths64::new();
    assert!(clipper.execute_tree(
        ClipType::Difference,
        FillRule::NonZero,
        &mut tree,
        &mut open
    ));

    let mp = multi_polygon_from_poly_tree64(&tree);
    assert_eq!(mp.0.len(), 1);
    assert_eq!(mp.0[0].interiors().len(), 1);
    assert_eq!(path_from_ring(mp.0[0].exterior()).len(), 4);
}

#[test]
fn test_open_paths_to_multi_line_string() {
    let lines = vec![path_from_line_string(&line_string![
        (x: -10.0, y: 50.0), (x: 110.0, y: 50.0)
    ])];
    let rect = vec![path_from_ring(square_d(0.0, 0.0, 100.0).exterior())];

    let mut clipper = ClipperD::new(2);
    clipper.add_open_subject(&lines);
    clipper.add_clip(&rect);
    let mut closed = PathsD::new();
    let mut open = PathsD::new();
    assert!(clipper.execute(
        ClipType::Intersection,
        FillRule::NonZero,
        &mut closed,
        Some(&mut open)
    ));

    let mls = multi_line_string_from_paths(&open);
    assert_eq!(mls.0.len(), 1);
    let mut xs: Vec<f64> = mls.0[0].0.iter().map(|c| c.x).collect();
    xs.sort_by(f64::total_cmp);
    assert_eq!(xs, vec![0.0, 100.0]);
}
