// Copyright Angus Johnson 2010-2026. Distributed under BSL-1.0.
//! Constrained Delaunay triangulation, ported from Angus Johnson's
//! `clipper.triangulation.cpp` (Boost Software License 1.0).
//!
//! Uses index arenas instead of C++ pointers, with the same sweep, intersection
//! repair, and optional Delaunay edge flips. New edges stop at or avoid live
//! vertices on their interiors, preventing overlapping triangles.
//!
//! Outer paths must have positive signed area and holes negative signed area
//! (CCW outers / CW holes in Cartesian coordinates). Reversing all paths is
//! supported. Self-intersecting paths should first be resolved with `union`.

use crate::core::{cross_product_sign, scale_paths_uniform, sqr, Path64, Paths64, PathsD, Point64};

/// Outcome of triangulation, matching the C++ `TriangulateResult` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TriangulateResult {
    /// Triangulation completed successfully.
    Success = 0,
    /// A required connecting edge could not be found.
    Fail = 1,
    /// No polygon with at least three usable vertices was supplied.
    NoPolygons = 2,
    /// Input edges intersect beyond the rounding tolerance.
    PathsIntersect = 3,
}

/// Triangulate integer polygons, replacing `solution` with positive-area triangles.
///
/// Holes must have winding opposite to their containing outer boundary. Set
/// `use_delaunay` to legalize unconstrained edges after the sweep. On failure,
/// `solution` is empty. With `using_z`, existing vertex Z values are preserved.
pub fn triangulate_64(
    paths: &Paths64,
    solution: &mut Paths64,
    use_delaunay: bool,
) -> TriangulateResult {
    solution.clear();
    let mut d = Delaunay::new(use_delaunay);
    match d.execute(paths) {
        Ok(triangles) => {
            *solution = triangles;
            TriangulateResult::Success
        }
        Err(result) => result,
    }
}

/// Triangulate floating-point polygons by scaling to integer coordinates.
///
/// Precision is clamped to 0..=8 decimal places, matching C++ triangulation.
pub fn triangulate_d(
    paths: &PathsD,
    dec_places: i32,
    solution: &mut PathsD,
    use_delaunay: bool,
) -> TriangulateResult {
    let scale = 10_f64.powi(dec_places.clamp(0, 8));
    let mut ec = 0;
    let input: Paths64 = scale_paths_uniform(paths, scale, &mut ec);
    let mut triangles = Vec::new();
    let result = triangulate_64(&input, &mut triangles, use_delaunay);
    *solution = scale_paths_uniform(&triangles, 1.0 / scale, &mut ec);
    result
}

// Arena indices never move. NONE is a missing link and is checked before indexing.
const NONE: usize = usize::MAX;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EdgeKind {
    Loose,
    Ascend,
    Descend,
}

struct Vertex {
    pt: Point64,
    edges: Vec<usize>,
    inner_lm: bool,
}

#[derive(Clone, Copy)]
struct Edge {
    vl: usize,
    vr: usize,
    vb: usize,
    vt: usize,
    kind: EdgeKind,
    tri_a: usize,
    tri_b: usize,
    active: bool,
    next: usize,
    prev: usize,
}

struct Delaunay {
    vertices: Vec<Vertex>,
    // Sorted traversal order is separate from the vertex arena: indices stay stable.
    vertex_order: Vec<usize>,
    edges: Vec<Edge>,
    edge_order: Vec<usize>,
    triangles: Vec<[usize; 3]>,
    pending: Vec<usize>,
    horizontals: Vec<usize>,
    minima: Vec<usize>,
    lowermost: usize,
    first_active: usize,
    use_delaunay: bool,
}

fn left_turn(a: Point64, b: Point64, c: Point64) -> bool {
    cross_product_sign(a, b, c) < 0
}
fn right_turn(a: Point64, b: Point64, c: Point64) -> bool {
    cross_product_sign(a, b, c) > 0
}

fn on_segment_interior(a: Point64, b: Point64, v: Point64) -> bool {
    v != a
        && v != b
        && v.x >= a.x.min(b.x)
        && v.x <= a.x.max(b.x)
        && v.y >= a.y.min(b.y)
        && v.y <= a.y.max(b.y)
        && cross_product_sign(a, v, b) == 0
}

fn angle(a: Point64, b: Point64, c: Point64) -> f64 {
    let (abx, aby) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let (bcx, bcy) = ((b.x - c.x) as f64, (b.y - c.y) as f64);
    (abx * bcy - aby * bcx).atan2(abx * bcx + aby * bcy)
}

fn in_circle(a: Point64, b: Point64, c: Point64, d: Point64) -> f64 {
    let (a0, a1) = ((a.x - d.x) as f64, (a.y - d.y) as f64);
    let (b0, b1) = ((b.x - d.x) as f64, (b.y - d.y) as f64);
    let (c0, c1) = ((c.x - d.x) as f64, (c.y - d.y) as f64);
    let a2 = a0 * a0 + a1 * a1;
    let b2 = b0 * b0 + b1 * b1;
    let c2 = c0 * c0 + c1 * c1;
    a0 * (b1 * c2 - c1 * b2) - b0 * (a1 * c2 - c1 * a2) + c0 * (a1 * b2 - b1 * a2)
}

// Match C++ DistSqr/DistanceSqr: subtract integers before converting to f64.
// Converting coordinates first loses short edges at large coordinate offsets.
fn squared_distance(a: Point64, b: Point64) -> f64 {
    sqr(a.x - b.x) + sqr(a.y - b.y)
}

fn shortest_dist(pt: Point64, a: Point64, b: Point64) -> f64 {
    let (dx, dy) = ((b.x - a.x) as f64, (b.y - a.y) as f64);
    let (ax, ay) = ((pt.x - a.x) as f64, (pt.y - a.y) as f64);
    let q = ax * dx + ay * dy;
    if q < 0.0 {
        squared_distance(pt, a)
    } else if q > dx * dx + dy * dy {
        squared_distance(pt, b)
    } else {
        sqr(ax * dy - dx * ay) / (dx * dx + dy * dy)
    }
}

// Match the triangulator's strict intersection predicate, including its endpoint
// exclusions and f64 calculations (distinct from core::segs_intersect).
fn segments_intersect(a: Point64, b: Point64, c: Point64, d: Point64) -> bool {
    if a == c || b == c || b == d {
        return false;
    }
    let (dy1, dx1) = ((b.y - a.y) as f64, (b.x - a.x) as f64);
    let (dy2, dx2) = ((d.y - c.y) as f64, (d.x - c.x) as f64);
    let cp = dy1 * dx2 - dy2 * dx1;
    if cp == 0.0 {
        return false;
    }
    let t = (a.x - c.x) as f64 * dy2 - (a.y - c.y) as f64 * dx2;
    if t >= 0.0 {
        if cp < 0.0 || t >= cp {
            return false;
        }
    } else if cp > 0.0 || t <= cp {
        return false;
    }
    let t = (a.x - c.x) as f64 * dy1 - (a.y - c.y) as f64 * dx1;
    if t >= 0.0 {
        cp > 0.0 && t < cp
    } else {
        cp < 0.0 && t > cp
    }
}

fn find_loc_min(path: &Path64, idx: &mut usize) -> bool {
    let len = path.len();
    if len < 3 {
        return false;
    }
    let start = *idx;
    let mut n = (*idx + 1) % len;
    while path[n].y <= path[*idx].y {
        *idx = n;
        n = (n + 1) % len;
        if *idx == start {
            return false;
        }
    }
    while path[n].y >= path[*idx].y {
        *idx = n;
        n = (n + 1) % len;
    }
    true
}

impl Delaunay {
    fn new(use_delaunay: bool) -> Self {
        Self {
            vertices: vec![],
            vertex_order: vec![],
            edges: vec![],
            edge_order: vec![],
            triangles: vec![],
            pending: vec![],
            horizontals: vec![],
            minima: vec![],
            lowermost: NONE,
            first_active: NONE,
            use_delaunay,
        }
    }
    fn pt(&self, v: usize) -> Point64 {
        self.vertices[v].pt
    }
    fn add_vertex(&mut self, pt: Point64) -> usize {
        let v = self.vertices.len();
        self.vertices.push(Vertex {
            pt,
            edges: Vec::with_capacity(2),
            inner_lm: false,
        });
        self.vertex_order.push(v);
        v
    }
    fn horizontal(&self, e: usize) -> bool {
        self.pt(self.edges[e].vb).y == self.pt(self.edges[e].vt).y
    }
    fn completed(&self, e: usize) -> bool {
        let e = &self.edges[e];
        e.tri_a != NONE && (e.tri_b != NONE || e.kind != EdgeKind::Loose)
    }
    fn loc_min_angle(&self, v: usize) -> f64 {
        let es = &self.vertices[v].edges;
        let (asc, des) = if self.edges[es[0]].kind == EdgeKind::Ascend {
            (0, 1)
        } else {
            (1, 0)
        };
        angle(
            self.pt(self.edges[es[des]].vt),
            self.pt(v),
            self.pt(self.edges[es[asc]].vt),
        )
    }
    fn remove_edge_from_vertex(&mut self, v: usize, e: usize) {
        let es = &mut self.vertices[v].edges;
        let i = es
            .iter()
            .position(|&x| x == e)
            .expect("edge must belong to vertex");
        es.remove(i);
    }
    fn add_active(&mut self, e: usize) {
        if self.edges[e].active {
            return;
        }
        self.edges[e].prev = NONE;
        self.edges[e].next = self.first_active;
        self.edges[e].active = true;
        if self.first_active != NONE {
            self.edges[self.first_active].prev = e;
        }
        self.first_active = e;
    }
    fn remove_active(&mut self, e: usize) {
        let edge = self.edges[e];
        self.remove_edge_from_vertex(edge.vb, e);
        self.remove_edge_from_vertex(edge.vt, e);
        if edge.next != NONE {
            self.edges[edge.next].prev = edge.prev;
        }
        if edge.prev != NONE {
            self.edges[edge.prev].next = edge.next;
        }
        self.edges[e].active = false;
        if self.first_active == e {
            self.first_active = edge.next;
        }
    }
    fn create_edge(&mut self, v1: usize, v2: usize, kind: EdgeKind) -> usize {
        let (vb, vt) = if self.pt(v1).y >= self.pt(v2).y {
            (v1, v2)
        } else {
            (v2, v1)
        };
        let (vl, vr) = if self.pt(v1).x <= self.pt(v2).x {
            (v1, v2)
        } else {
            (v2, v1)
        };
        let e = self.edges.len();
        self.edges.push(Edge {
            vl,
            vr,
            vb,
            vt,
            kind,
            tri_a: NONE,
            tri_b: NONE,
            active: false,
            next: NONE,
            prev: NONE,
        });
        self.edge_order.push(e);
        self.vertices[v1].edges.push(e);
        self.vertices[v2].edges.push(e);
        if kind == EdgeKind::Loose {
            self.pending.push(e);
            self.add_active(e);
        }
        e
    }
    fn create_triangle(&mut self, es: [usize; 3]) {
        let t = self.triangles.len();
        self.triangles.push(es);
        for e in es {
            if self.edges[e].tri_a != NONE {
                self.edges[e].tri_b = t;
                self.remove_active(e);
            } else {
                self.edges[e].tri_a = t;
                if self.edges[e].kind != EdgeKind::Loose {
                    self.remove_active(e);
                }
            }
        }
    }
    fn linking_edge(&self, v1: usize, v2: usize, prefer_ascending: bool) -> usize {
        let mut res = NONE;
        for &e in &self.vertices[v1].edges {
            let edge = &self.edges[e];
            if edge.vl == v2 || edge.vr == v2 {
                if edge.kind == EdgeKind::Loose
                    || (edge.kind == EdgeKind::Ascend) == prefer_ascending
                {
                    return e;
                }
                res = e;
            }
        }
        res
    }
    fn path_from_triangle(&self, t: [usize; 3]) -> Path64 {
        let e = self.edges[t[0]];
        let other = self.edges[t[1]];
        let a = self.pt(e.vl);
        let b = self.pt(e.vr);
        let c = if self.pt(other.vl) == a || self.pt(other.vl) == b {
            self.pt(other.vr)
        } else {
            self.pt(other.vl)
        };
        vec![a, b, c]
    }
    fn has_vertex_on_segment_interior(&self, a: Point64, b: Point64) -> bool {
        self.vertex_order
            .iter()
            .any(|&v| !self.vertices[v].edges.is_empty() && on_segment_interior(a, b, self.pt(v)))
    }
    fn force_legal(&mut self, e: usize) {
        let edge = self.edges[e];
        if edge.tri_a == NONE || edge.tri_b == NONE {
            return;
        }
        let mut va = NONE;
        let mut vb = NONE;
        let mut ea = [NONE; 3];
        let mut eb = [NONE; 3];
        for (t, slot) in [(edge.tri_a, 1), (edge.tri_b, 2)] {
            for x in self.triangles[t] {
                if x == e {
                    continue;
                }
                let other = self.edges[x];
                if other.vl == edge.vl || other.vr == edge.vl {
                    ea[slot] = x;
                    let v = if other.vl == edge.vl {
                        other.vr
                    } else {
                        other.vl
                    };
                    if slot == 1 {
                        va = v;
                    } else {
                        vb = v;
                    }
                } else {
                    eb[slot] = x;
                }
            }
        }
        let (a, l, r, b) = (self.pt(va), self.pt(edge.vl), self.pt(edge.vr), self.pt(vb));
        if cross_product_sign(a, l, r) == 0 {
            return;
        }
        let ict = in_circle(a, l, r, b);
        if ict == 0.0 || right_turn(a, l, r) == (ict < 0.0) {
            return;
        }
        // After legalizing, vl/vr no longer denote spatial left/right.
        self.edges[e].vl = va;
        self.edges[e].vr = vb;
        ea[0] = e;
        eb[0] = e;
        self.triangles[edge.tri_a] = ea;
        self.triangles[edge.tri_b] = eb;
        for (es, new_t, old_t) in [(ea, edge.tri_a, edge.tri_b), (eb, edge.tri_b, edge.tri_a)] {
            for &x in &es[1..] {
                if self.edges[x].kind == EdgeKind::Loose {
                    self.pending.push(x);
                }
                if self.edges[x].tri_a == new_t || self.edges[x].tri_b == new_t {
                    continue;
                }
                if self.edges[x].tri_a == old_t {
                    self.edges[x].tri_a = new_t;
                } else {
                    assert_eq!(self.edges[x].tri_b, old_t);
                    self.edges[x].tri_b = new_t;
                }
            }
        }
    }
    fn remove_intersection(&mut self, e1: usize, e2: usize) -> bool {
        let a = self.edges[e1];
        let b = self.edges[e2];
        let mut v = a.vl;
        let mut tmp = e2;
        let mut d = shortest_dist(self.pt(a.vl), self.pt(b.vl), self.pt(b.vr));
        for (vx, ex, s1, s2) in [
            (a.vr, e2, b.vl, b.vr),
            (b.vl, e1, a.vl, a.vr),
            (b.vr, e1, a.vl, a.vr),
        ] {
            let d2 = shortest_dist(self.pt(vx), self.pt(s1), self.pt(s2));
            if d2 < d {
                d = d2;
                v = vx;
                tmp = ex;
            }
        }
        if d > 1.0 {
            return false;
        }
        let v2 = self.edges[tmp].vt;
        self.remove_edge_from_vertex(v2, tmp);
        if self.edges[tmp].vl == v2 {
            self.edges[tmp].vl = v;
        } else {
            self.edges[tmp].vr = v;
        }
        self.edges[tmp].vt = v;
        self.vertices[v].edges.push(tmp);
        self.vertices[v].inner_lm = false;
        let bottom = self.edges[tmp].vb;
        if self.vertices[bottom].inner_lm && self.loc_min_angle(bottom) <= 0.0 {
            self.vertices[bottom].inner_lm = false;
        }
        self.create_edge(v, v2, self.edges[tmp].kind);
        true
    }
    fn fixup_edge_intersects(&mut self) -> bool {
        let mut i = 0;
        while i < self.edge_order.len() {
            let e1 = self.edge_order[i];
            let mut j = i + 1;
            while j < self.edge_order.len() {
                let e2 = self.edge_order[j];
                let a = self.edges[e1];
                let b = self.edges[e2];
                if self.pt(b.vl).x >= self.pt(a.vr).x {
                    break;
                }
                if self.pt(b.vt).y < self.pt(a.vb).y
                    && self.pt(b.vb).y > self.pt(a.vt).y
                    && segments_intersect(
                        self.pt(b.vl),
                        self.pt(b.vr),
                        self.pt(a.vl),
                        self.pt(a.vr),
                    )
                    && !self.remove_intersection(e2, e1)
                {
                    return false;
                }
                j += 1;
            }
            i += 1;
        }
        true
    }
    fn split_edge(&mut self, long: usize, short: usize) {
        let old = self.edges[long].vt;
        let new = self.edges[short].vt;
        self.remove_edge_from_vertex(old, long);
        self.edges[long].vt = new;
        if self.edges[long].vl == old {
            self.edges[long].vl = new;
        } else {
            self.edges[long].vr = new;
        }
        self.vertices[new].edges.push(long);
        self.create_edge(new, old, self.edges[long].kind);
    }
    fn merge_duplicates(&mut self) {
        let mut v1 = self.vertex_order[0];
        for i in 1..self.vertex_order.len() {
            let v2 = self.vertex_order[i];
            if self.pt(v1) != self.pt(v2) {
                v1 = v2;
                continue;
            }
            if !self.vertices[v1].inner_lm || !self.vertices[v2].inner_lm {
                self.vertices[v1].inner_lm = false;
            }
            let edges = std::mem::take(&mut self.vertices[v2].edges);
            for &e in &edges {
                if self.edges[e].vb == v2 {
                    self.edges[e].vb = v1;
                } else {
                    self.edges[e].vt = v1;
                }
                if self.edges[e].vl == v2 {
                    self.edges[e].vl = v1;
                } else {
                    self.edges[e].vr = v1;
                }
            }
            self.vertices[v1].edges.extend(edges);
            let len = self.vertices[v1].edges.len();
            for j in 0..len {
                let e1 = self.vertices[v1].edges[j];
                if self.horizontal(e1) || self.edges[e1].vb != v1 {
                    continue;
                }
                for k in j + 1..len {
                    let e2 = self.vertices[v1].edges[k];
                    let a = self.edges[e1];
                    let b = self.edges[e2];
                    if b.vb != v1
                        || self.pt(a.vt).y == self.pt(b.vt).y
                        || cross_product_sign(self.pt(a.vt), self.pt(v1), self.pt(b.vt)) != 0
                    {
                        continue;
                    }
                    if self.pt(a.vt).y < self.pt(b.vt).y {
                        self.split_edge(e1, e2);
                    } else {
                        self.split_edge(e2, e1);
                    }
                    break;
                }
            }
        }
    }
    fn create_inner_loc_min_edge(&mut self, above: usize) -> usize {
        let p = self.pt(above);
        let mut e = self.first_active;
        let mut below = NONE;
        let mut best_d = -1.0;
        while e != NONE {
            let edge = self.edges[e];
            if self.pt(edge.vl).x <= p.x
                && self.pt(edge.vr).x >= p.x
                && self.pt(edge.vb).y >= p.y
                && edge.vb != above
                && edge.vt != above
                && !left_turn(self.pt(edge.vl), p, self.pt(edge.vr))
            {
                let d = shortest_dist(p, self.pt(edge.vl), self.pt(edge.vr));
                if below == NONE || d < best_d {
                    below = e;
                    best_d = d;
                }
            }
            e = edge.next;
        }
        if below == NONE {
            return NONE;
        }
        let edge = self.edges[below];
        let mut best = if self.pt(edge.vt).y <= p.y {
            edge.vb
        } else {
            edge.vt
        };
        let mut best_pt = self.pt(best);
        let left = best_pt.x < p.x;
        e = self.first_active;
        while e != NONE {
            let edge = self.edges[e];
            let in_x = if left {
                self.pt(edge.vr).x > best_pt.x && self.pt(edge.vl).x < p.x
            } else {
                self.pt(edge.vr).x < best_pt.x && self.pt(edge.vl).x > p.x
            };
            if in_x
                && self.pt(edge.vb).y > p.y
                && self.pt(edge.vt).y < best_pt.y
                && segments_intersect(self.pt(edge.vb), self.pt(edge.vt), best_pt, p)
            {
                best = if self.pt(edge.vt).y > p.y {
                    edge.vt
                } else {
                    edge.vb
                };
                best_pt = self.pt(best);
            }
            e = edge.next;
        }
        // Stop at the nearest live vertex on this segment. Shortening
        // the same segment cannot turn an earlier rejected vertex into a match.
        for &v in &self.vertex_order {
            if v != above
                && v != best
                && !self.vertices[v].edges.is_empty()
                && on_segment_interior(p, self.pt(best), self.pt(v))
            {
                best = v;
            }
        }
        self.create_edge(best, above, EdgeKind::Loose)
    }
    fn horizontal_between(&self, v1: usize, v2: usize) -> bool {
        let a = self.pt(v1);
        let b = self.pt(v2);
        let (l, r) = (a.x.min(b.x), a.x.max(b.x));
        let mut e = self.first_active;
        while e != NONE {
            let edge = self.edges[e];
            let left = self.pt(edge.vl);
            let right = self.pt(edge.vr);
            if left.y == a.y
                && right.y == a.y
                && left.x >= l
                && right.x <= r
                && (left.x != l || left.x != r)
            {
                return true;
            }
            e = edge.next;
        }
        false
    }
    // The C++ left/right methods differ only in turn signs, boundary kinds,
    // preferred linking-edge direction, and triangle edge order.
    fn do_triangulate(&mut self, mut edge: usize, mut pivot: usize, min_y: i64, left: bool) {
        // Tail iteration replaces the C++ recursion without changing its order.
        loop {
            let v = if self.edges[edge].vb == pivot {
                self.edges[edge].vt
            } else {
                self.edges[edge].vb
            };
            let mut alt = NONE;
            let mut e_alt = NONE;
            for &e in &self.vertices[pivot].edges {
                if e == edge || !self.edges[e].active {
                    continue;
                }
                let vx = if self.edges[e].vt == pivot {
                    self.edges[e].vb
                } else {
                    self.edges[e].vt
                };
                if vx == v {
                    continue;
                }
                let cps = cross_product_sign(self.pt(v), self.pt(pivot), self.pt(vx));
                if cps == 0 {
                    if (self.pt(v).x > self.pt(pivot).x) == (self.pt(pivot).x > self.pt(vx).x) {
                        continue;
                    }
                } else {
                    let wrong = if left { cps > 0 } else { cps < 0 };
                    let worse = alt != NONE
                        && if left {
                            !left_turn(self.pt(vx), self.pt(pivot), self.pt(alt))
                        } else {
                            !right_turn(self.pt(vx), self.pt(pivot), self.pt(alt))
                        };
                    if wrong || worse {
                        continue;
                    }
                }
                alt = vx;
                e_alt = e;
            }
            if alt == NONE || self.pt(alt).y < min_y {
                return;
            }
            let kind = self.edges[e_alt].kind;
            if self.pt(alt).y < self.pt(pivot).y {
                if kind
                    == if left {
                        EdgeKind::Ascend
                    } else {
                        EdgeKind::Descend
                    }
                {
                    return;
                }
            } else if self.pt(alt).y > self.pt(pivot).y
                && kind
                    == if left {
                        EdgeKind::Descend
                    } else {
                        EdgeKind::Ascend
                    }
            {
                return;
            }
            let prefer_ascending = if left {
                self.pt(alt).y < self.pt(v).y
            } else {
                self.pt(alt).y > self.pt(v).y
            };
            let mut ex = self.linking_edge(alt, v, prefer_ascending);
            if ex == NONE {
                if self.pt(alt).y == self.pt(v).y
                    && self.pt(v).y == min_y
                    && self.horizontal_between(alt, v)
                {
                    return;
                }
                // Never create a loose edge through another live vertex.
                if self.has_vertex_on_segment_interior(self.pt(alt), self.pt(v)) {
                    return;
                }
                ex = self.create_edge(alt, v, EdgeKind::Loose);
            }
            self.create_triangle(if left {
                [edge, e_alt, ex]
            } else {
                [edge, ex, e_alt]
            });
            if self.completed(ex) {
                return;
            }
            edge = ex;
            pivot = alt;
        }
    }
    fn execute(&mut self, paths: &Paths64) -> Result<Paths64, TriangulateResult> {
        let count = paths.iter().map(Vec::len).sum();
        self.vertices.reserve(count);
        self.edges.reserve(count);
        for p in paths {
            self.add_path(p);
        }
        if self.vertices.len() < 3 {
            return Err(TriangulateResult::NoPolygons);
        }
        if self.vertices[self.lowermost].inner_lm {
            while let Some(v) = self.minima.pop() {
                self.vertices[v].inner_lm = !self.vertices[v].inner_lm;
            }
            for edge in &mut self.edges {
                edge.kind = if edge.kind == EdgeKind::Ascend {
                    EdgeKind::Descend
                } else {
                    EdgeKind::Ascend
                };
            }
        } else {
            self.minima.clear();
        }
        self.edge_order
            .sort_unstable_by_key(|&e| self.vertices[self.edges[e].vl].pt.x);
        if !self.fixup_edge_intersects() {
            return Err(TriangulateResult::PathsIntersect);
        }
        self.vertex_order.sort_unstable_by(|&a, &b| {
            let a = self.vertices[a].pt;
            let b = self.vertices[b].pt;
            b.y.cmp(&a.y).then_with(|| a.x.cmp(&b.x))
        });
        self.merge_duplicates();
        let mut curr_y = self.pt(self.vertex_order[0]).y;
        for i in 0..self.vertex_order.len() {
            let v = self.vertex_order[i];
            if self.vertices[v].edges.is_empty() {
                continue;
            }
            if self.pt(v).y != curr_y {
                while let Some(lm) = self.minima.pop() {
                    let e = self.create_inner_loc_min_edge(lm);
                    if e == NONE {
                        return Err(TriangulateResult::Fail);
                    }
                    let bottom = self.edges[e].vb;
                    if self.horizontal(e) {
                        let left = self.edges[e].vl == bottom;
                        self.do_triangulate(e, bottom, curr_y, left);
                    } else {
                        self.do_triangulate(e, bottom, curr_y, true);
                        if !self.completed(e) {
                            self.do_triangulate(e, bottom, curr_y, false);
                        }
                    }
                    let es = &self.vertices[lm].edges;
                    let (a, b) = (es[0], es[1]);
                    self.add_active(a);
                    self.add_active(b);
                }
                while let Some(e) = self.horizontals.pop() {
                    if self.completed(e) {
                        continue;
                    }
                    let edge = self.edges[e];
                    if edge.vb == edge.vl {
                        if edge.kind == EdgeKind::Ascend {
                            self.do_triangulate(e, edge.vb, curr_y, true);
                        }
                    } else if edge.kind == EdgeKind::Descend {
                        self.do_triangulate(e, edge.vb, curr_y, false);
                    }
                }
                curr_y = self.pt(v).y;
            }
            // Processing may add/remove edges. Descending indices preserve the
            // same traversal and mutation semantics as the C++ implementation.
            let mut j = self.vertices[v].edges.len();
            while j > 0 {
                j -= 1;
                if j >= self.vertices[v].edges.len() {
                    continue;
                }
                let e = self.vertices[v].edges[j];
                let edge = self.edges[e];
                if self.completed(e) || edge.kind == EdgeKind::Loose {
                    continue;
                }
                if v == edge.vb {
                    if self.horizontal(e) {
                        self.horizontals.push(e);
                    }
                    if !self.vertices[v].inner_lm {
                        self.add_active(e);
                    }
                } else if self.horizontal(e) {
                    self.horizontals.push(e);
                } else {
                    self.do_triangulate(e, edge.vb, self.pt(v).y, edge.kind == EdgeKind::Ascend);
                }
            }
            if self.vertices[v].inner_lm {
                self.minima.push(v);
            }
        }
        while let Some(e) = self.horizontals.pop() {
            let edge = self.edges[e];
            if !self.completed(e) && edge.vb == edge.vl {
                self.do_triangulate(e, edge.vb, curr_y, true);
            }
        }
        if self.use_delaunay {
            while let Some(e) = self.pending.pop() {
                self.force_legal(e);
            }
        }
        let mut result = Vec::with_capacity(self.triangles.len());
        for &t in &self.triangles {
            let mut p = self.path_from_triangle(t);
            let cps = cross_product_sign(p[0], p[1], p[2]);
            if cps == 0 {
                continue;
            }
            if cps < 0 {
                p.reverse();
            }
            result.push(p);
        }
        Ok(result)
    }
    fn add_path(&mut self, path: &Path64) {
        let len = path.len();
        let mut i0 = 0;
        if !find_loc_min(path, &mut i0) {
            return;
        }
        let mut prev = (i0 + len - 1) % len;
        while path[prev] == path[i0] {
            prev = (prev + len - 1) % len;
        }
        let mut next = (i0 + 1) % len;
        let mut i = i0;
        while cross_product_sign(path[prev], path[i], path[next]) == 0 {
            find_loc_min(path, &mut i);
            if i == i0 {
                return;
            }
            prev = (i + len - 1) % len;
            while path[prev] == path[i] {
                prev = (prev + len - 1) % len;
            }
            next = (i + 1) % len;
        }
        let start = self.vertices.len();
        let v0 = self.add_vertex(path[i]);
        if left_turn(path[prev], path[i], path[next]) {
            self.vertices[v0].inner_lm = true;
        }
        let mut v_prev = v0;
        i = next;
        loop {
            self.minima.push(v_prev);
            if self.lowermost == NONE
                || self.pt(v_prev).y > self.pt(self.lowermost).y
                || (self.pt(v_prev).y == self.pt(self.lowermost).y
                    && self.pt(v_prev).x < self.pt(self.lowermost).x)
            {
                self.lowermost = v_prev;
            }
            next = (i + 1) % len;
            if cross_product_sign(self.pt(v_prev), path[i], path[next]) == 0 {
                i = next;
                continue;
            }
            while path[i].y <= self.pt(v_prev).y {
                let v = self.add_vertex(path[i]);
                self.create_edge(v_prev, v, EdgeKind::Ascend);
                v_prev = v;
                i = next;
                next = (i + 1) % len;
                while cross_product_sign(self.pt(v_prev), path[i], path[next]) == 0 {
                    i = next;
                    next = (i + 1) % len;
                }
            }
            let mut v_prev_prev = v_prev;
            while i != i0 && path[i].y >= self.pt(v_prev).y {
                let v = self.add_vertex(path[i]);
                self.create_edge(v, v_prev, EdgeKind::Descend);
                v_prev_prev = v_prev;
                v_prev = v;
                i = next;
                next = (i + 1) % len;
                while cross_product_sign(self.pt(v_prev), path[i], path[next]) == 0 {
                    i = next;
                    next = (i + 1) % len;
                }
            }
            if i == i0 {
                break;
            }
            if left_turn(self.pt(v_prev_prev), self.pt(v_prev), path[i]) {
                self.vertices[v_prev].inner_lm = true;
            }
        }
        self.create_edge(v0, v_prev, EdgeKind::Descend);
        let count = self.vertices.len() - start;
        if count < 3
            || (count == 3
                && (squared_distance(self.pt(start), self.pt(start + 1)) <= 1.0
                    || squared_distance(self.pt(start + 1), self.pt(start + 2)) <= 1.0
                    || squared_distance(self.pt(start + 2), self.pt(start)) <= 1.0))
        {
            for v in &mut self.vertices[start..] {
                v.edges.clear();
            }
        }
    }
}

#[cfg(test)]
#[path = "triangulation_tests.rs"]
mod tests;
