//! Conversions between clipper2-rust paths and the [`geo-types`] primitives used by
//! the [`geo`] crate. Enabled by the `geo` feature.
//!
//! This module is an interop layer, not part of the Clipper2 port: the clipping engine
//! still works only on [`Path`] and [`Paths`], and nothing here changes its results.
//!
//! The two models differ in three ways, and the functions below bridge each one:
//!
//! - **Closing point.** A geo ring repeats its first point at the end; a Clipper path
//!   does not. [`path_from_ring`] drops the repeat and the polygon builders add it back.
//!   [`path_from_line_string`] keeps every point, for open paths.
//! - **Holes.** A geo [`Polygon`] owns its holes; Clipper marks holes only by
//!   orientation. [`paths_from_polygon`] makes the exterior positive (counter-clockwise
//!   with y up) and every hole negative, so `NonZero`, `Positive` and `EvenOdd` all see
//!   the holes the polygon describes. Going back needs the nesting, so results come
//!   from a [`PolyTree64`] or [`PolyTreeD`]: [`multi_polygon_from_poly_tree64`] and
//!   [`multi_polygon_from_poly_tree_d`] turn each solid node and its direct hole
//!   children into one polygon, and an island inside a hole becomes its own polygon.
//! - **Orphan rule.** [`Path`] is an alias for `Vec<Point<T>>`, which this crate cannot
//!   implement `From` for, so paths convert through functions; only [`Point`] and
//!   [`Coord`] convert with `From`.
//!
//! Under the `using_z` feature a point built from a [`Coord`] has `z == 0`, and the z
//! value is dropped when converting back.
//!
//! [`geo-types`]: https://docs.rs/geo-types
//! [`geo`]: https://docs.rs/geo

use crate::core::{area, Path, Paths, Point, ToF64};
use crate::engine_public::{PolyTree64, PolyTreeD};
use geo_types::{Coord, CoordNum, LineString, MultiLineString, MultiPolygon, Polygon};

impl<T: CoordNum> From<Coord<T>> for Point<T> {
    fn from(c: Coord<T>) -> Self {
        Point::new(c.x, c.y)
    }
}

impl<T: CoordNum> From<Point<T>> for Coord<T> {
    fn from(p: Point<T>) -> Self {
        Coord { x: p.x, y: p.y }
    }
}

/// Convert a geo line string to a Clipper path, keeping every point. Use this for open
/// paths; use [`path_from_ring`] for polygon rings.
pub fn path_from_line_string<T: CoordNum>(line: &LineString<T>) -> Path<T> {
    line.0.iter().map(|&c| Point::from(c)).collect()
}

/// Convert a geo polygon ring to a Clipper path, dropping the closing point that repeats
/// the first one. An open line string converts with all its points.
pub fn path_from_ring<T: CoordNum>(ring: &LineString<T>) -> Path<T> {
    let mut path = path_from_line_string(ring);
    if path.len() > 1 && ring.is_closed() {
        path.pop();
    }
    path
}

/// Convert a Clipper path to an open geo line string with the same points.
pub fn line_string_from_path<T: CoordNum>(path: &Path<T>) -> LineString<T> {
    LineString::new(path.iter().map(|&p| Coord::from(p)).collect())
}

/// Convert Clipper open paths, such as the open solution of an `execute` call, to a geo
/// multi line string.
pub fn multi_line_string_from_paths<T: CoordNum>(paths: &Paths<T>) -> MultiLineString<T> {
    MultiLineString::new(paths.iter().map(line_string_from_path).collect())
}

/// Convert a geo polygon to Clipper paths: the exterior first, made positive, then each
/// hole, made negative. Rings with fewer than three distinct points are skipped.
pub fn paths_from_polygon<T: CoordNum + ToF64>(polygon: &Polygon<T>) -> Paths<T> {
    let mut paths = Paths::new();
    push_oriented_ring(&mut paths, polygon.exterior(), true);
    for hole in polygon.interiors() {
        push_oriented_ring(&mut paths, hole, false);
    }
    paths
}

/// Convert a geo multi polygon to Clipper paths, as [`paths_from_polygon`] does for each
/// of its polygons.
pub fn paths_from_multi_polygon<T: CoordNum + ToF64>(multi: &MultiPolygon<T>) -> Paths<T> {
    multi.0.iter().flat_map(paths_from_polygon).collect()
}

fn push_oriented_ring<T: CoordNum + ToF64>(
    paths: &mut Paths<T>,
    ring: &LineString<T>,
    positive: bool,
) {
    let mut path = path_from_ring(ring);
    if path.len() < 3 {
        return;
    }
    if (area(&path) > 0.0) != positive {
        path.reverse();
    }
    paths.push(path);
}

fn polygon_from_paths<T: CoordNum>(outer: &Path<T>, holes: &[&Path<T>]) -> Polygon<T> {
    // Polygon::new closes every ring by repeating its first point.
    Polygon::new(
        line_string_from_path(outer),
        holes.iter().map(|h| line_string_from_path(h)).collect(),
    )
}

/// Convert a [`PolyTree64`] to a geo multi polygon. Each solid node becomes a polygon
/// whose interiors are its direct hole children; islands nested inside those holes
/// become separate polygons. Rings keep the orientation Clipper produced.
pub fn multi_polygon_from_poly_tree64(tree: &PolyTree64) -> MultiPolygon<i64> {
    let mut polygons = Vec::new();
    for idx in 1..tree.nodes.len() {
        let node = &tree.nodes[idx];
        if node.polygon().is_empty() || tree.is_hole(idx) {
            continue;
        }
        let holes: Vec<&Path<i64>> = node
            .children()
            .iter()
            .map(|&c| tree.nodes[c].polygon())
            .filter(|p| !p.is_empty())
            .collect();
        polygons.push(polygon_from_paths(node.polygon(), &holes));
    }
    MultiPolygon::new(polygons)
}

/// Convert a [`PolyTreeD`] to a geo multi polygon, as [`multi_polygon_from_poly_tree64`]
/// does for integer trees.
pub fn multi_polygon_from_poly_tree_d(tree: &PolyTreeD) -> MultiPolygon<f64> {
    let mut polygons = Vec::new();
    for idx in 1..tree.nodes.len() {
        let node = &tree.nodes[idx];
        if node.polygon().is_empty() || tree.is_hole(idx) {
            continue;
        }
        let holes: Vec<&Path<f64>> = node
            .children()
            .iter()
            .map(|&c| tree.nodes[c].polygon())
            .filter(|p| !p.is_empty())
            .collect();
        polygons.push(polygon_from_paths(node.polygon(), &holes));
    }
    MultiPolygon::new(polygons)
}
