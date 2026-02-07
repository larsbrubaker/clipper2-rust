use super::*;
use crate::engine_public::*;

// ============================================================================
// Enum tests
// ============================================================================

#[test]
fn test_clip_type_default() {
    let ct: ClipType = Default::default();
    assert_eq!(ct, ClipType::NoClip);
}

#[test]
fn test_clip_type_variants() {
    assert_ne!(ClipType::Intersection, ClipType::Union);
    assert_ne!(ClipType::Difference, ClipType::Xor);
    assert_eq!(ClipType::NoClip, ClipType::NoClip);
}

#[test]
fn test_path_type_variants() {
    assert_ne!(PathType::Subject, PathType::Clip);
    assert_eq!(PathType::Subject, PathType::Subject);
}

#[test]
fn test_join_with_default() {
    let jw: JoinWith = Default::default();
    assert_eq!(jw, JoinWith::NoJoin);
}

#[test]
fn test_vertex_flags_operations() {
    let empty = VertexFlags::EMPTY;
    let open_start = VertexFlags::OPEN_START;
    let open_end = VertexFlags::OPEN_END;
    let local_max = VertexFlags::LOCAL_MAX;
    let local_min = VertexFlags::LOCAL_MIN;

    // Test bitwise OR
    let combined = open_start | open_end;
    assert_ne!(combined, VertexFlags::EMPTY);

    // Test bitwise AND
    assert_eq!(combined & open_start, open_start);
    assert_eq!(combined & local_max, VertexFlags::EMPTY);

    // Test multiple flags
    let all = open_start | open_end | local_max | local_min;
    assert_ne!(all & open_start, empty);
    assert_ne!(all & local_min, empty);
}

#[test]
fn test_vertex_flags_default() {
    let vf: VertexFlags = Default::default();
    assert_eq!(vf, VertexFlags::EMPTY);
}

// ============================================================================
// Data structure tests
// ============================================================================

#[test]
fn test_vertex_new() {
    let v = Vertex::new(Point64::new(10, 20));
    assert_eq!(v.pt, Point64::new(10, 20));
    assert_eq!(v.next, NONE);
    assert_eq!(v.prev, NONE);
    assert_eq!(v.flags, VertexFlags::EMPTY);
}

#[test]
fn test_outpt_new() {
    let op = OutPt::new(Point64::new(5, 10), 0);
    assert_eq!(op.pt, Point64::new(5, 10));
    assert_eq!(op.outrec, 0);
    assert!(op.horz.is_none());
}

#[test]
fn test_outrec_new() {
    let or = OutRec::new(42);
    assert_eq!(or.idx, 42);
    assert!(or.owner.is_none());
    assert!(or.front_edge.is_none());
    assert!(or.back_edge.is_none());
    assert!(or.pts.is_none());
    assert!(!or.is_open);
}

#[test]
fn test_active_new() {
    let a = Active::new();
    assert_eq!(a.wind_dx, 1);
    assert_eq!(a.wind_cnt, 0);
    assert_eq!(a.wind_cnt2, 0);
    assert!(a.outrec.is_none());
    assert!(a.prev_in_ael.is_none());
    assert!(a.next_in_ael.is_none());
    assert_eq!(a.join_with, JoinWith::NoJoin);
}

#[test]
fn test_local_minima_new() {
    let lm = LocalMinima::new(5, PathType::Subject, false);
    assert_eq!(lm.vertex, 5);
    assert_eq!(lm.polytype, PathType::Subject);
    assert!(!lm.is_open);
}

#[test]
fn test_intersect_node_new() {
    let in1 = IntersectNode::new();
    assert_eq!(in1.pt, Point64::new(0, 0));
    assert_eq!(in1.edge1, NONE);

    let in2 = IntersectNode::with_edges(1, 2, Point64::new(5, 5));
    assert_eq!(in2.pt, Point64::new(5, 5));
    assert_eq!(in2.edge1, 1);
    assert_eq!(in2.edge2, 2);
}

#[test]
fn test_horz_segment_new() {
    let hs = HorzSegment::new();
    assert!(hs.left_op.is_none());
    assert!(hs.right_op.is_none());
    assert!(hs.left_to_right);

    let hs2 = HorzSegment::with_op(10);
    assert_eq!(hs2.left_op, Some(10));
    assert!(hs2.right_op.is_none());
}

#[test]
fn test_horz_join_new() {
    let hj = HorzJoin::new();
    assert!(hj.op1.is_none());
    assert!(hj.op2.is_none());

    let hj2 = HorzJoin::with_ops(3, 7);
    assert_eq!(hj2.op1, Some(3));
    assert_eq!(hj2.op2, Some(7));
}

// ============================================================================
// Free function tests
// ============================================================================

#[test]
fn test_is_odd() {
    assert!(is_odd(1));
    assert!(is_odd(3));
    assert!(is_odd(-1));
    assert!(!is_odd(0));
    assert!(!is_odd(2));
    assert!(!is_odd(-2));
}

#[test]
fn test_get_dx() {
    // Vertical line going down
    assert_eq!(get_dx(Point64::new(0, 0), Point64::new(0, 10)), 0.0);

    // 45-degree line
    let dx = get_dx(Point64::new(0, 0), Point64::new(10, 10));
    assert!((dx - 1.0).abs() < 1e-10);

    // Horizontal line right
    let dx = get_dx(Point64::new(0, 0), Point64::new(10, 0));
    assert_eq!(dx, -f64::MAX);

    // Horizontal line left
    let dx = get_dx(Point64::new(10, 0), Point64::new(0, 0));
    assert_eq!(dx, f64::MAX);
}

#[test]
fn test_top_x() {
    let mut e = Active::new();
    e.bot = Point64::new(0, 100);
    e.top = Point64::new(100, 0);
    e.dx = get_dx(e.bot, e.top);

    // At top
    assert_eq!(top_x(&e, 0), 100);
    // At bottom
    assert_eq!(top_x(&e, 100), 0);
    // At middle
    let x = top_x(&e, 50);
    assert_eq!(x, 50);
}

#[test]
fn test_is_horizontal_active() {
    let mut e = Active::new();
    e.bot = Point64::new(0, 10);
    e.top = Point64::new(10, 10);
    assert!(is_horizontal_active(&e));

    e.top = Point64::new(10, 20);
    assert!(!is_horizontal_active(&e));
}

#[test]
fn test_is_heading_right_left_horz() {
    let mut e = Active::new();
    e.dx = -f64::MAX;
    assert!(is_heading_right_horz(&e));
    assert!(!is_heading_left_horz(&e));

    e.dx = f64::MAX;
    assert!(!is_heading_right_horz(&e));
    assert!(is_heading_left_horz(&e));
}

#[test]
fn test_intersect_list_sort() {
    let a = IntersectNode::with_edges(0, 1, Point64::new(5, 10));
    let b = IntersectNode::with_edges(2, 3, Point64::new(3, 20));
    let c = IntersectNode::with_edges(4, 5, Point64::new(7, 10));

    let mut nodes = [a, b, c];
    nodes.sort_by(intersect_list_sort);

    // Should sort by y descending (larger y first), then x ascending
    assert_eq!(nodes[0].pt, Point64::new(3, 20));
    assert_eq!(nodes[1].pt, Point64::new(5, 10));
    assert_eq!(nodes[2].pt, Point64::new(7, 10));
}

// ============================================================================
// ClipperBase tests
// ============================================================================

#[test]
fn test_clipper_base_new() {
    let cb = ClipperBase::new();
    assert_eq!(cb.cliptype, ClipType::NoClip);
    assert_eq!(cb.fillrule, FillRule::EvenOdd);
    assert!(cb.preserve_collinear);
    assert!(!cb.reverse_solution);
    assert_eq!(cb.error_code, 0);
    assert!(cb.succeeded);
}

#[test]
fn test_clipper_base_add_path_closed() {
    let mut cb = ClipperBase::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
        Point64::new(0, 100),
    ];
    cb.add_path(&path, PathType::Subject, false);
    assert!(!cb.vertex_arena.is_empty());
    assert!(!cb.minima_list.is_empty());
    assert!(!cb.has_open_paths);
}

#[test]
fn test_clipper_base_add_path_open() {
    let mut cb = ClipperBase::new();
    let path = vec![Point64::new(0, 0), Point64::new(100, 100)];
    cb.add_path(&path, PathType::Subject, true);
    assert!(!cb.vertex_arena.is_empty());
    assert!(cb.has_open_paths);
}

#[test]
fn test_clipper_base_add_path_too_short() {
    let mut cb = ClipperBase::new();
    let path = vec![Point64::new(0, 0)];
    cb.add_path(&path, PathType::Subject, false);
    assert!(cb.vertex_arena.is_empty());
    assert!(cb.minima_list.is_empty());
}

#[test]
fn test_clipper_base_add_paths() {
    let mut cb = ClipperBase::new();
    let paths = vec![
        vec![
            Point64::new(0, 0),
            Point64::new(100, 0),
            Point64::new(100, 100),
        ],
        vec![
            Point64::new(200, 200),
            Point64::new(300, 200),
            Point64::new(300, 300),
        ],
    ];
    cb.add_paths(&paths, PathType::Subject, false);
    assert_eq!(cb.vertex_arena.len(), 6);
}

#[test]
fn test_clipper_base_clear() {
    let mut cb = ClipperBase::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
    ];
    cb.add_path(&path, PathType::Subject, false);
    assert!(!cb.vertex_arena.is_empty());

    cb.clear();
    assert!(cb.vertex_arena.is_empty());
    assert!(cb.minima_list.is_empty());
    assert!(cb.active_arena.is_empty());
}

#[test]
fn test_clipper_base_scanline() {
    let mut cb = ClipperBase::new();
    cb.insert_scanline(10);
    cb.insert_scanline(20);
    cb.insert_scanline(5);
    cb.insert_scanline(10); // duplicate

    // Max-heap: pops largest y first (matching C++ std::priority_queue behavior)
    assert_eq!(cb.pop_scanline(), Some(20));
    assert_eq!(cb.pop_scanline(), Some(10)); // duplicate removed
    assert_eq!(cb.pop_scanline(), Some(5));
    assert_eq!(cb.pop_scanline(), None);
}

#[test]
fn test_clipper_base_new_out_rec() {
    let mut cb = ClipperBase::new();
    let idx1 = cb.new_out_rec();
    let idx2 = cb.new_out_rec();
    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(cb.outrec_list[idx1].idx, 0);
    assert_eq!(cb.outrec_list[idx2].idx, 1);
}

#[test]
fn test_clipper_base_new_out_pt() {
    let mut cb = ClipperBase::new();
    let or_idx = cb.new_out_rec();
    let op_idx = cb.new_out_pt(Point64::new(10, 20), or_idx);
    assert_eq!(cb.outpt_arena[op_idx].pt, Point64::new(10, 20));
    assert_eq!(cb.outpt_arena[op_idx].outrec, or_idx);
    // Should be self-referencing (single node in circular list)
    assert_eq!(cb.outpt_arena[op_idx].next, op_idx);
    assert_eq!(cb.outpt_arena[op_idx].prev, op_idx);
}

#[test]
fn test_clipper_base_duplicate_op() {
    let mut cb = ClipperBase::new();
    let or_idx = cb.new_out_rec();
    let op1 = cb.new_out_pt(Point64::new(10, 20), or_idx);

    let op2 = cb.duplicate_op(op1, true);
    assert_eq!(cb.outpt_arena[op2].pt, Point64::new(10, 20));
    // op1 -> op2 -> op1
    assert_eq!(cb.outpt_arena[op1].next, op2);
    assert_eq!(cb.outpt_arena[op2].next, op1);
    assert_eq!(cb.outpt_arena[op2].prev, op1);
    assert_eq!(cb.outpt_arena[op1].prev, op2);
}

#[test]
fn test_clipper_base_swap_outrecs() {
    let mut cb = ClipperBase::new();
    let or1 = cb.new_out_rec();
    let or2 = cb.new_out_rec();
    let e1 = cb.new_active();
    let e2 = cb.new_active();

    cb.active_arena[e1].outrec = Some(or1);
    cb.active_arena[e2].outrec = Some(or2);
    cb.outrec_list[or1].front_edge = Some(e1);
    cb.outrec_list[or2].front_edge = Some(e2);

    cb.swap_outrecs(e1, e2);
    assert_eq!(cb.active_arena[e1].outrec, Some(or2));
    assert_eq!(cb.active_arena[e2].outrec, Some(or1));
}

// ============================================================================
// Clipper64 tests
// ============================================================================

#[test]
fn test_clipper64_new() {
    let c = Clipper64::new();
    assert_eq!(c.base.cliptype, ClipType::NoClip);
}

#[test]
fn test_clipper64_add_paths() {
    let mut c = Clipper64::new();
    let subjects = vec![vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
        Point64::new(0, 100),
    ]];
    let clips = vec![vec![
        Point64::new(50, 50),
        Point64::new(150, 50),
        Point64::new(150, 150),
        Point64::new(50, 150),
    ]];

    c.add_subject(&subjects);
    c.add_clip(&clips);
    assert_eq!(c.base.vertex_arena.len(), 8);
}

#[test]
fn test_clipper64_preserve_collinear() {
    let mut c = Clipper64::new();
    assert!(c.preserve_collinear());
    c.set_preserve_collinear(false);
    assert!(!c.preserve_collinear());
}

// ============================================================================
// ClipperD tests
// ============================================================================

#[test]
fn test_clipper_d_new() {
    let c = ClipperD::new(2);
    assert!(c.scale() > 0.0);
    assert!(c.inv_scale() > 0.0);
    assert!((c.scale() * c.inv_scale() - 1.0).abs() < 1e-10);
}

#[test]
fn test_clipper_d_add_paths() {
    let mut c = ClipperD::new(2);
    let subjects = vec![vec![
        PointD::new(0.0, 0.0),
        PointD::new(1.0, 0.0),
        PointD::new(1.0, 1.0),
        PointD::new(0.0, 1.0),
    ]];
    c.add_subject(&subjects);
    assert!(!c.base.vertex_arena.is_empty());
}

// ============================================================================
// PolyTree tests
// ============================================================================

#[test]
fn test_polytree64_new() {
    let pt = PolyTree64::new();
    assert_eq!(pt.nodes.len(), 1);
    assert!(pt.root().children().is_empty());
}

#[test]
fn test_polytree64_add_child() {
    let mut pt = PolyTree64::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
    ];
    let child = pt.add_child(0, path.clone());
    assert_eq!(child, 1);
    assert_eq!(pt.nodes[child].polygon(), &path);
    assert_eq!(pt.root().children().len(), 1);
}

#[test]
fn test_polytree64_level() {
    let mut pt = PolyTree64::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
    ];
    let child = pt.add_child(0, path.clone());
    let grandchild = pt.add_child(child, path);

    assert_eq!(pt.level(0), 0);
    assert_eq!(pt.level(child), 1);
    assert_eq!(pt.level(grandchild), 2);
}

#[test]
fn test_polytree64_is_hole() {
    let mut pt = PolyTree64::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
    ];
    let child = pt.add_child(0, path.clone());
    let grandchild = pt.add_child(child, path.clone());
    let great_grandchild = pt.add_child(grandchild, path);

    assert!(!pt.is_hole(0)); // level 0 - not a hole
    assert!(!pt.is_hole(child)); // level 1 - not a hole (odd)
    assert!(pt.is_hole(grandchild)); // level 2 - hole (even > 0)
    assert!(!pt.is_hole(great_grandchild)); // level 3 - not a hole
}

#[test]
fn test_polytree64_clear() {
    let mut pt = PolyTree64::new();
    let path = vec![
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
    ];
    pt.add_child(0, path);
    assert_eq!(pt.nodes.len(), 2);

    pt.clear();
    assert_eq!(pt.nodes.len(), 1);
    assert!(pt.root().children().is_empty());
}

#[test]
fn test_polytreed_new() {
    let pt = PolyTreeD::new();
    assert_eq!(pt.nodes.len(), 1);
    assert!((pt.root().scale() - 1.0).abs() < 1e-10);
}

// ============================================================================
// Area and point count tests
// ============================================================================

#[test]
fn test_point_count_outpt() {
    let mut outpt_arena = Vec::new();
    // Create a triangle in the arena
    let a = OutPt::new(Point64::new(0, 0), 0);
    let b = OutPt::new(Point64::new(10, 0), 0);
    let c = OutPt::new(Point64::new(10, 10), 0);
    outpt_arena.push(a);
    outpt_arena.push(b);
    outpt_arena.push(c);
    outpt_arena[0].next = 1;
    outpt_arena[0].prev = 2;
    outpt_arena[1].next = 2;
    outpt_arena[1].prev = 0;
    outpt_arena[2].next = 0;
    outpt_arena[2].prev = 1;

    assert_eq!(point_count(0, &outpt_arena), 3);
}

#[test]
fn test_area_outpt_triangle() {
    let mut outpt_arena = vec![
        OutPt::new(Point64::new(0, 0), 0),
        OutPt::new(Point64::new(10, 0), 0),
        OutPt::new(Point64::new(10, 10), 0),
    ];
    outpt_arena[0].next = 1;
    outpt_arena[0].prev = 2;
    outpt_arena[1].next = 2;
    outpt_arena[1].prev = 0;
    outpt_arena[2].next = 0;
    outpt_arena[2].prev = 1;

    let a = area_outpt(0, &outpt_arena);
    assert!((a.abs() - 50.0).abs() < 1e-10);
}

#[test]
fn test_area_triangle_fn() {
    let a = area_triangle(
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(10, 10),
    );
    assert!((a.abs() - 100.0).abs() < 1e-10);
}

#[test]
fn test_reverse_out_pts() {
    let mut outpt_arena = vec![
        OutPt::new(Point64::new(0, 0), 0),
        OutPt::new(Point64::new(10, 0), 0),
        OutPt::new(Point64::new(10, 10), 0),
    ];
    outpt_arena[0].next = 1;
    outpt_arena[0].prev = 2;
    outpt_arena[1].next = 2;
    outpt_arena[1].prev = 0;
    outpt_arena[2].next = 0;
    outpt_arena[2].prev = 1;

    reverse_out_pts(0, &mut outpt_arena);

    // After reversal: 0->2->1->0 (reversed from 0->1->2->0)
    assert_eq!(outpt_arena[0].next, 2);
    assert_eq!(outpt_arena[0].prev, 1);
    assert_eq!(outpt_arena[1].next, 0);
    assert_eq!(outpt_arena[1].prev, 2);
    assert_eq!(outpt_arena[2].next, 1);
    assert_eq!(outpt_arena[2].prev, 0);
}

// ============================================================================
// Clipper64 execute integration tests
// ============================================================================

#[test]
fn test_clipper64_local_minima_include_both_subject_and_clip() {
    // Verify that adding subject AND clip paths creates local minima for both
    let subject = vec![
        Point64::new(-100, -100),
        Point64::new(100, -100),
        Point64::new(100, 100),
        Point64::new(-100, 100),
    ];
    let clip = vec![
        Point64::new(-50, -50),
        Point64::new(150, -50),
        Point64::new(150, 150),
        Point64::new(-50, 150),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&vec![subject]);
    c.add_clip(&vec![clip]);

    // After adding both, the base should have local minima for both polygons
    let subject_minima: Vec<_> = c
        .base
        .minima_list
        .iter()
        .filter(|lm| lm.polytype == PathType::Subject)
        .collect();
    let clip_minima: Vec<_> = c
        .base
        .minima_list
        .iter()
        .filter(|lm| lm.polytype == PathType::Clip)
        .collect();

    assert!(
        !subject_minima.is_empty(),
        "Should have Subject local minima"
    );
    assert!(
        !clip_minima.is_empty(),
        "Should have Clip local minima, got 0. Total minima: {}",
        c.base.minima_list.len()
    );
}

#[test]
fn test_clipper64_intersection_two_overlapping_squares() {
    // Two overlapping squares: Subject at (-100,-100)-(100,100), Clip at (-50,-50)-(150,150)
    // Intersection should produce the overlap region: (-50,-50)-(100,100) with area 150*150 = 22500
    let subject = vec![
        Point64::new(-100, -100),
        Point64::new(100, -100),
        Point64::new(100, 100),
        Point64::new(-100, 100),
    ];
    let clip = vec![
        Point64::new(-50, -50),
        Point64::new(150, -50),
        Point64::new(150, 150),
        Point64::new(-50, 150),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&vec![subject]);
    c.add_clip(&vec![clip]);
    let mut result = Paths64::new();
    c.execute(ClipType::Intersection, FillRule::NonZero, &mut result, None);
    assert!(
        !result.is_empty(),
        "Intersection of overlapping squares must not be empty"
    );
    let total_area: f64 = result.iter().map(|p| area(p).abs()).sum();
    assert!(
        (total_area - 22500.0).abs() < 100.0,
        "Intersection area should be ~22500, got {}",
        total_area
    );
}

#[test]
fn test_clipper64_difference_two_overlapping_squares() {
    // Subject at (-100,-100)-(100,100), Clip at (-50,-50)-(150,150)
    // Difference should produce Subject minus overlap, area = 40000 - 22500 = 17500
    let subject = vec![
        Point64::new(-100, -100),
        Point64::new(100, -100),
        Point64::new(100, 100),
        Point64::new(-100, 100),
    ];
    let clip = vec![
        Point64::new(-50, -50),
        Point64::new(150, -50),
        Point64::new(150, 150),
        Point64::new(-50, 150),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&vec![subject]);
    c.add_clip(&vec![clip]);
    let mut result = Paths64::new();
    c.execute(ClipType::Difference, FillRule::NonZero, &mut result, None);
    assert!(
        !result.is_empty(),
        "Difference of overlapping squares must not be empty"
    );
    let total_area: f64 = result.iter().map(|p| area(p).abs()).sum();
    assert!(
        (total_area - 17500.0).abs() < 100.0,
        "Difference area should be ~17500, got {}",
        total_area
    );
}

#[test]
fn test_clipper64_union_two_overlapping_squares() {
    // Union should produce combined area = 40000 + 40000 - 22500 = 57500
    let subject = vec![
        Point64::new(-100, -100),
        Point64::new(100, -100),
        Point64::new(100, 100),
        Point64::new(-100, 100),
    ];
    let clip = vec![
        Point64::new(-50, -50),
        Point64::new(150, -50),
        Point64::new(150, 150),
        Point64::new(-50, 150),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&vec![subject]);
    c.add_clip(&vec![clip]);
    let mut result = Paths64::new();
    c.execute(ClipType::Union, FillRule::NonZero, &mut result, None);
    assert!(
        !result.is_empty(),
        "Union of overlapping squares must not be empty"
    );
    let total_area: f64 = result.iter().map(|p| area(p).abs()).sum();
    assert!(
        (total_area - 57500.0).abs() < 100.0,
        "Union area should be ~57500, got {}",
        total_area
    );
}

#[test]
fn test_clipper64_xor_two_overlapping_squares() {
    // Xor should produce combined minus 2x overlap = 57500 - 2*22500 = 12500? No.
    // Xor area = Subject + Clip - 2*intersection = 40000 + 40000 - 2*22500 = 35000
    let subject = vec![
        Point64::new(-100, -100),
        Point64::new(100, -100),
        Point64::new(100, 100),
        Point64::new(-100, 100),
    ];
    let clip = vec![
        Point64::new(-50, -50),
        Point64::new(150, -50),
        Point64::new(150, 150),
        Point64::new(-50, 150),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&vec![subject]);
    c.add_clip(&vec![clip]);
    let mut result = Paths64::new();
    c.execute(ClipType::Xor, FillRule::NonZero, &mut result, None);
    assert!(
        !result.is_empty(),
        "Xor of overlapping squares must not be empty"
    );
    let total_area: f64 = result.iter().map(|p| area(p).abs()).sum();
    assert!(
        (total_area - 35000.0).abs() < 100.0,
        "Xor area should be ~35000, got {}",
        total_area
    );
}

// ============================================================================
// Polygon test case 37 - reproducer for count=3 bug (expected count=2)
// ============================================================================

#[test]
fn test_polygon_case_37_difference_evenodd() {
    // Test data extracted from CPP/Tests/Polygons.txt, test 37:
    //   CLIPTYPE: DIFFERENCE
    //   FILLRULE: EVENODD
    //   SOL_AREA: 18194
    //   SOL_COUNT: 2
    //   SUBJECTS:
    //     0,0, 102,0, 119,107, 0,101
    //     116,110, 200,114, 200,200, 107,200
    //   CLIPS:
    //     0,108, 117,106, 111,200, 0,200
    //     112,0, 200,0, 200,117, 101,112

    let subjects = vec![
        vec![
            Point64::new(0, 0),
            Point64::new(102, 0),
            Point64::new(119, 107),
            Point64::new(0, 101),
        ],
        vec![
            Point64::new(116, 110),
            Point64::new(200, 114),
            Point64::new(200, 200),
            Point64::new(107, 200),
        ],
    ];
    let clips = vec![
        vec![
            Point64::new(0, 108),
            Point64::new(117, 106),
            Point64::new(111, 200),
            Point64::new(0, 200),
        ],
        vec![
            Point64::new(112, 0),
            Point64::new(200, 0),
            Point64::new(200, 117),
            Point64::new(101, 112),
        ],
    ];

    let mut c = Clipper64::new();
    c.add_subject(&subjects);
    c.add_clip(&clips);
    let mut result = Paths64::new();
    c.execute(ClipType::Difference, FillRule::EvenOdd, &mut result, None);

    let measured_count = result.len();
    assert_eq!(
        measured_count, 2,
        "Polygon test 37: expected 2 output paths, got {}",
        measured_count
    );
    let total_area: f64 = result.iter().map(area).sum();
    assert!(
        (total_area as i64 - 18194).abs() < 200,
        "Polygon test 37: expected area ~18194, got {}",
        total_area as i64
    );
}
