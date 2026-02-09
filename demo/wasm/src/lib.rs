use wasm_bindgen::prelude::*;

use clipper2_rust::core::{
    area, area_paths, ellipse_point64, get_bounds_paths, is_positive, point_in_polygon, Path64,
    Paths64, Point64, PointInPolygonResult, Rect64,
};
use clipper2_rust::engine::ClipType;
use clipper2_rust::engine_public::{Clipper64, PolyTree64};
use clipper2_rust::minkowski::{minkowski_diff, minkowski_sum};
use clipper2_rust::offset::{EndType, JoinType};
use clipper2_rust::{
    boolean_op_64, inflate_paths_64, ramer_douglas_peucker, rect_clip_64, rect_clip_lines_64,
    simplify_path, FillRule,
};

// ============================================================================
// Buffer encoding/decoding
// ============================================================================
// Paths encoded as flat f64 arrays: [n_points, x0, y0, x1, y1, ..., n_points, ...]
// Single paths encoded as: [x0, y0, x1, y1, ...]

fn decode_paths(buf: &[f64]) -> Paths64 {
    let mut paths = Paths64::new();
    let mut i = 0;
    while i < buf.len() {
        let n = buf[i] as usize;
        i += 1;
        let mut path = Path64::with_capacity(n);
        for _ in 0..n {
            if i + 1 < buf.len() {
                path.push(Point64::new(buf[i] as i64, buf[i + 1] as i64));
                i += 2;
            }
        }
        paths.push(path);
    }
    paths
}

fn decode_single_path(buf: &[f64]) -> Path64 {
    let mut path = Path64::with_capacity(buf.len() / 2);
    let mut i = 0;
    while i + 1 < buf.len() {
        path.push(Point64::new(buf[i] as i64, buf[i + 1] as i64));
        i += 2;
    }
    path
}

fn encode_paths(paths: &Paths64) -> Vec<f64> {
    let total: usize = paths.iter().map(|p| 1 + p.len() * 2).sum();
    let mut buf = Vec::with_capacity(total);
    for path in paths {
        buf.push(path.len() as f64);
        for pt in path {
            buf.push(pt.x as f64);
            buf.push(pt.y as f64);
        }
    }
    buf
}

fn encode_single_path(path: &Path64) -> Vec<f64> {
    let mut buf = Vec::with_capacity(path.len() * 2);
    for pt in path {
        buf.push(pt.x as f64);
        buf.push(pt.y as f64);
    }
    buf
}

fn clip_type_from_u8(v: u8) -> ClipType {
    match v {
        1 => ClipType::Intersection,
        2 => ClipType::Union,
        3 => ClipType::Difference,
        4 => ClipType::Xor,
        _ => ClipType::Intersection,
    }
}

fn fill_rule_from_u8(v: u8) -> FillRule {
    match v {
        0 => FillRule::EvenOdd,
        1 => FillRule::NonZero,
        2 => FillRule::Positive,
        3 => FillRule::Negative,
        _ => FillRule::EvenOdd,
    }
}

fn join_type_from_u8(v: u8) -> JoinType {
    match v {
        0 => JoinType::Square,
        1 => JoinType::Bevel,
        2 => JoinType::Round,
        3 => JoinType::Miter,
        _ => JoinType::Round,
    }
}

fn end_type_from_u8(v: u8) -> EndType {
    match v {
        0 => EndType::Polygon,
        1 => EndType::Joined,
        2 => EndType::Butt,
        3 => EndType::Square,
        4 => EndType::Round,
        _ => EndType::Polygon,
    }
}

// ============================================================================
// WASM Exports
// ============================================================================

#[wasm_bindgen]
pub fn boolean_op(clip_type: u8, fill_rule: u8, subjects: &[f64], clips: &[f64]) -> Vec<f64> {
    let subj = decode_paths(subjects);
    let clp = decode_paths(clips);
    let result = boolean_op_64(
        clip_type_from_u8(clip_type),
        fill_rule_from_u8(fill_rule),
        &subj,
        &clp,
    );
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn inflate_paths(
    paths: &[f64],
    delta: f64,
    join_type: u8,
    end_type: u8,
    miter_limit: f64,
    arc_tolerance: f64,
) -> Vec<f64> {
    let p = decode_paths(paths);
    let result = inflate_paths_64(
        &p,
        delta,
        join_type_from_u8(join_type),
        end_type_from_u8(end_type),
        miter_limit,
        arc_tolerance,
    );
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn rect_clip(left: f64, top: f64, right: f64, bottom: f64, paths: &[f64]) -> Vec<f64> {
    let rect = Rect64::new(left as i64, top as i64, right as i64, bottom as i64);
    let p = decode_paths(paths);
    let result = rect_clip_64(&rect, &p);
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn rect_clip_lines(left: f64, top: f64, right: f64, bottom: f64, lines: &[f64]) -> Vec<f64> {
    let rect = Rect64::new(left as i64, top as i64, right as i64, bottom as i64);
    let l = decode_paths(lines);
    let result = rect_clip_lines_64(&rect, &l);
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn mink_sum(pattern: &[f64], path: &[f64], is_closed: bool) -> Vec<f64> {
    let pat = decode_single_path(pattern);
    let p = decode_single_path(path);
    let result = minkowski_sum(&pat, &p, is_closed);
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn mink_diff(pattern: &[f64], path: &[f64], is_closed: bool) -> Vec<f64> {
    let pat = decode_single_path(pattern);
    let p = decode_single_path(path);
    let result = minkowski_diff(&pat, &p, is_closed);
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn simplify(paths: &[f64], epsilon: f64, is_closed: bool) -> Vec<f64> {
    let p = decode_paths(paths);
    let mut result = Paths64::new();
    for path in &p {
        result.push(simplify_path(path, epsilon, is_closed));
    }
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn rdp_simplify(paths: &[f64], epsilon: f64) -> Vec<f64> {
    let p = decode_paths(paths);
    let mut result = Paths64::new();
    for path in &p {
        result.push(ramer_douglas_peucker(path, epsilon));
    }
    encode_paths(&result)
}

#[wasm_bindgen]
pub fn pip_test(px: f64, py: f64, polygon: &[f64]) -> i32 {
    let poly = decode_single_path(polygon);
    let pt = Point64::new(px as i64, py as i64);
    match point_in_polygon(pt, &poly) {
        PointInPolygonResult::IsInside => 1,
        PointInPolygonResult::IsOutside => -1,
        PointInPolygonResult::IsOn => 0,
    }
}

#[wasm_bindgen]
pub fn path_area(paths: &[f64]) -> f64 {
    let p = decode_paths(paths);
    area_paths(&p)
}

#[wasm_bindgen]
pub fn single_area(path: &[f64]) -> f64 {
    let p = decode_single_path(path);
    area(&p)
}

#[wasm_bindgen]
pub fn is_positive_path(path: &[f64]) -> bool {
    let p = decode_single_path(path);
    is_positive(&p)
}

#[wasm_bindgen]
pub fn bounds(paths: &[f64]) -> Vec<f64> {
    let p = decode_paths(paths);
    let r = get_bounds_paths(&p);
    vec![r.left as f64, r.top as f64, r.right as f64, r.bottom as f64]
}

#[wasm_bindgen]
pub fn make_ellipse(cx: f64, cy: f64, rx: f64, ry: f64, steps: u32) -> Vec<f64> {
    let path = ellipse_point64(
        Point64::new(cx as i64, cy as i64),
        rx,
        ry,
        if steps == 0 { 0 } else { steps as usize },
    );
    encode_single_path(&path)
}

#[wasm_bindgen]
pub fn make_star(cx: f64, cy: f64, outer_r: f64, inner_r: f64, points: u32) -> Vec<f64> {
    let n = points as usize;
    let mut path = Path64::with_capacity(n * 2);
    for i in 0..n {
        let angle_outer =
            (i as f64) * 2.0 * std::f64::consts::PI / (n as f64) - std::f64::consts::PI / 2.0;
        let angle_inner = angle_outer + std::f64::consts::PI / (n as f64);
        path.push(Point64::new(
            (cx + outer_r * angle_outer.cos()) as i64,
            (cy + outer_r * angle_outer.sin()) as i64,
        ));
        path.push(Point64::new(
            (cx + inner_r * angle_inner.cos()) as i64,
            (cy + inner_r * angle_inner.sin()) as i64,
        ));
    }
    encode_single_path(&path)
}

/// Boolean op returning PolyTree as JSON.
/// Format: { "children": [ { "polygon": [[x,y],...], "is_hole": bool, "depth": n, "children": [...] } ] }
#[wasm_bindgen]
pub fn polytree_op(clip_type: u8, fill_rule: u8, subjects: &[f64], clips: &[f64]) -> String {
    let subj = decode_paths(subjects);
    let clp = decode_paths(clips);
    let mut tree = PolyTree64::new();
    let mut open_paths = Paths64::new();
    let mut clipper = Clipper64::new();
    clipper.add_subject(&subj);
    clipper.add_clip(&clp);
    clipper.execute_tree(
        clip_type_from_u8(clip_type),
        fill_rule_from_u8(fill_rule),
        &mut tree,
        &mut open_paths,
    );

    fn node_to_json(tree: &PolyTree64, idx: usize, depth: usize) -> String {
        let node = &tree.nodes[idx];
        let poly = node.polygon();
        let mut pts = String::from("[");
        for (i, pt) in poly.iter().enumerate() {
            if i > 0 {
                pts.push(',');
            }
            pts.push_str(&format!("[{},{}]", pt.x, pt.y));
        }
        pts.push(']');
        let is_hole = depth % 2 == 1;
        let mut children_json = String::from("[");
        for (i, &child_idx) in node.children().iter().enumerate() {
            if i > 0 {
                children_json.push(',');
            }
            children_json.push_str(&node_to_json(tree, child_idx, depth + 1));
        }
        children_json.push(']');
        format!(
            r#"{{"polygon":{},"is_hole":{},"depth":{},"children":{}}}"#,
            pts, is_hole, depth, children_json
        )
    }

    let root = &tree.nodes[0];
    let mut result = String::from(r#"{"children":["#);
    for (i, &child_idx) in root.children().iter().enumerate() {
        if i > 0 {
            result.push(',');
        }
        result.push_str(&node_to_json(&tree, child_idx, 1));
    }
    result.push_str("]}");
    result
}
