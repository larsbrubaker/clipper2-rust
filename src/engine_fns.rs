//! Engine free functions - standalone utility functions for the sweep-line algorithm
//!
//! Direct port from clipper.engine.cpp
//! Copyright (c) Angus Johnson 2010-2025

use crate::core::*;
use crate::engine::*;

// ============================================================================
// Engine Free Functions - Direct port from clipper.engine.cpp
// ============================================================================

/// Check if a value is odd
/// Direct port from clipper.engine.cpp line 61
#[inline]
pub fn is_odd(val: i32) -> bool {
    val & 1 != 0
}

/// Check if an active edge is "hot" (has an output record)
/// Direct port from clipper.engine.cpp line 67
#[inline]
pub fn is_hot_edge(e: &Active) -> bool {
    e.outrec.is_some()
}

/// Check if an active edge is part of an open path
/// Direct port from clipper.engine.cpp line 73
#[inline]
pub fn is_open_active(e: &Active, minima_list: &[LocalMinima]) -> bool {
    minima_list[e.local_min].is_open
}

/// Check if a vertex is an open end
/// Direct port from clipper.engine.cpp line 79
#[inline]
pub fn is_open_end_vertex(v: &Vertex) -> bool {
    (v.flags & (VertexFlags::OPEN_START | VertexFlags::OPEN_END)) != VertexFlags::EMPTY
}

/// Check if an active edge's vertex_top is an open end
/// Direct port from clipper.engine.cpp line 87
#[inline]
pub fn is_open_end_active(e: &Active, vertex_arena: &[Vertex]) -> bool {
    is_open_end_vertex(&vertex_arena[e.vertex_top])
}

/// Calculate slope dx for an edge defined by two points
/// Direct port from clipper.engine.cpp line 116
#[inline]
pub fn get_dx(pt1: Point64, pt2: Point64) -> f64 {
    let dy = (pt2.y - pt1.y) as f64;
    if dy != 0.0 {
        (pt2.x - pt1.x) as f64 / dy
    } else if pt2.x > pt1.x {
        -f64::MAX
    } else {
        f64::MAX
    }
}

/// Get the x coordinate of an active edge at a given y
/// Direct port from clipper.engine.cpp line 127
#[inline]
pub fn top_x(ae: &Active, current_y: i64) -> i64 {
    if current_y == ae.top.y || ae.top.x == ae.bot.x {
        ae.top.x
    } else if current_y == ae.bot.y {
        ae.bot.x
    } else {
        ae.bot.x + (ae.dx * (current_y - ae.bot.y) as f64).round() as i64
    }
}

/// Check if an active edge is horizontal
/// Direct port from clipper.engine.cpp line 137
#[inline]
pub fn is_horizontal_active(e: &Active) -> bool {
    e.top.y == e.bot.y
}

/// Check if a horizontal edge is heading right
/// Direct port from clipper.engine.cpp line 143
#[inline]
pub fn is_heading_right_horz(e: &Active) -> bool {
    e.dx == -f64::MAX
}

/// Check if a horizontal edge is heading left
/// Direct port from clipper.engine.cpp line 149
#[inline]
pub fn is_heading_left_horz(e: &Active) -> bool {
    e.dx == f64::MAX
}

/// Get the polytype of an active edge
/// Direct port from clipper.engine.cpp line 162
#[inline]
pub fn get_poly_type(e: &Active, minima_list: &[LocalMinima]) -> PathType {
    minima_list[e.local_min].polytype
}

/// Check if two active edges are the same polytype
/// Direct port from clipper.engine.cpp line 167
#[inline]
pub fn is_same_poly_type(e1: &Active, e2: &Active, minima_list: &[LocalMinima]) -> bool {
    minima_list[e1.local_min].polytype == minima_list[e2.local_min].polytype
}

/// Set dx for an active edge from its bot/top points
/// Direct port from clipper.engine.cpp line 172
#[inline]
pub fn set_dx(e: &mut Active) {
    e.dx = get_dx(e.bot, e.top);
}

/// Get next vertex for an active edge (depends on winding direction)
/// Direct port from clipper.engine.cpp line 177
#[inline]
pub fn next_vertex(e: &Active, vertex_arena: &[Vertex]) -> usize {
    if e.wind_dx > 0 {
        vertex_arena[e.vertex_top].next
    } else {
        vertex_arena[e.vertex_top].prev
    }
}

/// Get the vertex two steps back (useful for alternate edge top)
/// Direct port from clipper.engine.cpp line 187
#[inline]
pub fn prev_prev_vertex(ae: &Active, vertex_arena: &[Vertex]) -> usize {
    if ae.wind_dx > 0 {
        let prev = vertex_arena[ae.vertex_top].prev;
        vertex_arena[prev].prev
    } else {
        let next = vertex_arena[ae.vertex_top].next;
        vertex_arena[next].next
    }
}

/// Check if a vertex is a local maximum
/// Direct port from clipper.engine.cpp line 215
#[inline]
pub fn is_maxima_vertex(v: &Vertex) -> bool {
    (v.flags & VertexFlags::LOCAL_MAX) != VertexFlags::EMPTY
}

/// Check if an active edge is at a local maximum
/// Direct port from clipper.engine.cpp line 221
#[inline]
pub fn is_maxima_active(e: &Active, vertex_arena: &[Vertex]) -> bool {
    is_maxima_vertex(&vertex_arena[e.vertex_top])
}

/// Count points in an OutPt circular list
/// Direct port from clipper.engine.cpp line 266
pub fn point_count(op_start: usize, outpt_arena: &[OutPt]) -> i32 {
    let mut op2 = op_start;
    let mut cnt = 0;
    loop {
        op2 = outpt_arena[op2].next;
        cnt += 1;
        if op2 == op_start {
            break;
        }
    }
    cnt
}

/// Check if an OutPt path is invalid (single point or None)
/// Direct port from clipper.engine.cpp line 105
#[inline]
pub fn is_invalid_path(op: Option<usize>, outpt_arena: &[OutPt]) -> bool {
    match op {
        None => true,
        Some(idx) => outpt_arena[idx].next == idx,
    }
}

/// Calculate area of an OutPt circular list
/// Direct port from clipper.engine.cpp line 366
pub fn area_outpt(op_start: usize, outpt_arena: &[OutPt]) -> f64 {
    let mut result = 0.0;
    let mut op2 = op_start;
    loop {
        let prev_idx = outpt_arena[op2].prev;
        result += (outpt_arena[prev_idx].pt.y + outpt_arena[op2].pt.y) as f64
            * (outpt_arena[prev_idx].pt.x - outpt_arena[op2].pt.x) as f64;
        op2 = outpt_arena[op2].next;
        if op2 == op_start {
            break;
        }
    }
    result * 0.5
}

/// Calculate area of a triangle formed by three points
/// Direct port from clipper.engine.cpp line 380
#[inline]
pub fn area_triangle(pt1: Point64, pt2: Point64, pt3: Point64) -> f64 {
    (pt3.y + pt1.y) as f64 * (pt3.x - pt1.x) as f64
        + (pt1.y + pt2.y) as f64 * (pt1.x - pt2.x) as f64
        + (pt2.y + pt3.y) as f64 * (pt2.x - pt3.x) as f64
}

/// Reverse the direction of an OutPt circular list
/// Direct port from clipper.engine.cpp line 388
pub fn reverse_out_pts(op_start: usize, outpt_arena: &mut [OutPt]) {
    let mut op1 = op_start;
    loop {
        std::mem::swap(&mut outpt_arena[op1].next, &mut outpt_arena[op1].prev);
        op1 = outpt_arena[op1].prev; // advance to original next (now in prev)
        if op1 == op_start {
            break;
        }
    }
}

/// Sort comparator for IntersectNodes
/// Direct port from clipper.engine.cpp line 322
pub fn intersect_list_sort(a: &IntersectNode, b: &IntersectNode) -> std::cmp::Ordering {
    if a.pt.y == b.pt.y {
        a.pt.x.cmp(&b.pt.x)
    } else {
        b.pt.y.cmp(&a.pt.y)
    }
}

/// Get the maxima pair for an active edge
/// Direct port from clipper.engine.cpp line 254
pub fn get_maxima_pair(e: &Active, active_arena: &[Active]) -> Option<usize> {
    let mut e2_idx = e.next_in_ael;
    while let Some(idx) = e2_idx {
        if active_arena[idx].vertex_top == e.vertex_top {
            return Some(idx);
        }
        e2_idx = active_arena[idx].next_in_ael;
    }
    None
}

/// Get current Y maxima vertex for open paths
/// Direct port from clipper.engine.cpp line 226
pub fn get_curr_y_maxima_vertex_open(e: &Active, vertex_arena: &[Vertex]) -> Option<usize> {
    let mut result = e.vertex_top;
    if e.wind_dx > 0 {
        while vertex_arena[vertex_arena[result].next].pt.y == vertex_arena[result].pt.y
            && (vertex_arena[result].flags & (VertexFlags::OPEN_END | VertexFlags::LOCAL_MAX))
                == VertexFlags::EMPTY
        {
            result = vertex_arena[result].next;
        }
    } else {
        while vertex_arena[vertex_arena[result].prev].pt.y == vertex_arena[result].pt.y
            && (vertex_arena[result].flags & (VertexFlags::OPEN_END | VertexFlags::LOCAL_MAX))
                == VertexFlags::EMPTY
        {
            result = vertex_arena[result].prev;
        }
    }
    if is_maxima_vertex(&vertex_arena[result]) {
        Some(result)
    } else {
        None
    }
}

/// Get current Y maxima vertex for closed paths
/// Direct port from clipper.engine.cpp line 243
pub fn get_curr_y_maxima_vertex(e: &Active, vertex_arena: &[Vertex]) -> Option<usize> {
    let mut result = e.vertex_top;
    if e.wind_dx > 0 {
        while vertex_arena[vertex_arena[result].next].pt.y == vertex_arena[result].pt.y {
            result = vertex_arena[result].next;
        }
    } else {
        while vertex_arena[vertex_arena[result].prev].pt.y == vertex_arena[result].pt.y {
            result = vertex_arena[result].prev;
        }
    }
    if is_maxima_vertex(&vertex_arena[result]) {
        Some(result)
    } else {
        None
    }
}

/// Check if the AEL ordering is valid between two active edges
/// Direct port from clipper.engine.cpp IsValidAelOrder (line 1119)
pub fn is_valid_ael_order(
    resident: &Active,
    newcomer: &Active,
    vertex_arena: &[Vertex],
    minima_list: &[LocalMinima],
) -> bool {
    if newcomer.curr_x != resident.curr_x {
        return newcomer.curr_x > resident.curr_x;
    }

    // get the turning direction: resident.top, newcomer.bot, newcomer.top
    let i = cross_product_sign(
        vertex_arena[resident.vertex_top].pt,
        newcomer.bot,
        vertex_arena[newcomer.vertex_top].pt,
    );
    if i != 0 {
        return i < 0;
    }

    // edges must be collinear to get here
    // for starting open paths, place them according to
    // the direction they're about to turn
    if !is_maxima_active(resident, vertex_arena)
        && vertex_arena[resident.vertex_top].pt.y > vertex_arena[newcomer.vertex_top].pt.y
    {
        let nv = next_vertex(resident, vertex_arena);
        return cross_product_sign(
            newcomer.bot,
            vertex_arena[resident.vertex_top].pt,
            vertex_arena[nv].pt,
        ) <= 0;
    } else if !is_maxima_active(newcomer, vertex_arena)
        && vertex_arena[newcomer.vertex_top].pt.y > vertex_arena[resident.vertex_top].pt.y
    {
        let nv = next_vertex(newcomer, vertex_arena);
        return cross_product_sign(
            newcomer.bot,
            vertex_arena[newcomer.vertex_top].pt,
            vertex_arena[nv].pt,
        ) >= 0;
    }

    let y = newcomer.bot.y;
    let newcomer_is_left = newcomer.is_left_bound;

    if resident.bot.y != y || vertex_arena[minima_list[resident.local_min].vertex].pt.y != y {
        newcomer.is_left_bound
    } else if resident.is_left_bound != newcomer_is_left {
        newcomer_is_left
    } else if is_collinear(
        vertex_arena[prev_prev_vertex(resident, vertex_arena)].pt,
        resident.bot,
        vertex_arena[resident.vertex_top].pt,
    ) {
        true
    } else {
        // compare turning direction of the alternate bound
        let ppv_r = prev_prev_vertex(resident, vertex_arena);
        let ppv_n = prev_prev_vertex(newcomer, vertex_arena);
        (cross_product_sign(vertex_arena[ppv_r].pt, newcomer.bot, vertex_arena[ppv_n].pt) > 0)
            == newcomer_is_left
    }
}

// ============================================================================
// Path building functions
// ============================================================================

/// Build a Path64 from an OutRec
/// Direct port from clipper.engine.cpp GetCleanPath equivalent
pub fn get_clean_path(op_start: usize, outpt_arena: &[OutPt]) -> Path64 {
    let mut result = Path64::new();
    let mut op = op_start;
    loop {
        let prev = outpt_arena[op].prev;
        let next = outpt_arena[op].next;
        if outpt_arena[op].pt == outpt_arena[prev].pt
            || is_collinear(
                outpt_arena[prev].pt,
                outpt_arena[op].pt,
                outpt_arena[next].pt,
            )
        {
            op = next;
            if op == op_start {
                break;
            }
            continue;
        }
        result.push(outpt_arena[op].pt);
        op = next;
        if op == op_start {
            break;
        }
    }
    result
}

// ============================================================================
// Free helper functions for sweep-line algorithm
// Direct port from clipper.engine.cpp
// ============================================================================

/// Get the real (non-disposed) OutRec by following owner chain
/// Direct port from clipper.engine.cpp GetRealOutRec (line 412)
pub fn get_real_outrec(outrec_list: &[OutRec], idx: usize) -> Option<usize> {
    let mut i = Some(idx);
    while let Some(cur) = i {
        if outrec_list[cur].pts.is_some() {
            return Some(cur);
        }
        i = outrec_list[cur].owner;
    }
    None
}

/// Check if an owner is valid (not circular)
/// Direct port from clipper.engine.cpp IsValidOwner (line 418)
pub fn is_valid_owner(outrec_list: &[OutRec], outrec_idx: usize, test_owner_idx: usize) -> bool {
    let mut tmp = Some(test_owner_idx);
    while let Some(idx) = tmp {
        if idx == outrec_idx {
            return false;
        }
        tmp = outrec_list[idx].owner;
    }
    true
}

/// Check if two points are really close (within 2 units)
/// Direct port from clipper.engine.cpp PtsReallyClose (line 436)
#[inline]
pub fn pts_really_close(pt1: Point64, pt2: Point64) -> bool {
    (pt1.x - pt2.x).abs() < 2 && (pt1.y - pt2.y).abs() < 2
}

/// Check if an OutPt triangle is very small
/// Direct port from clipper.engine.cpp IsVerySmallTriangle (line 441)
pub fn is_very_small_triangle(op_idx: usize, outpt_arena: &[OutPt]) -> bool {
    let next = outpt_arena[op_idx].next;
    let prev = outpt_arena[op_idx].prev;
    if outpt_arena[next].next != prev {
        return false;
    }
    pts_really_close(outpt_arena[prev].pt, outpt_arena[next].pt)
        || pts_really_close(outpt_arena[op_idx].pt, outpt_arena[next].pt)
        || pts_really_close(outpt_arena[op_idx].pt, outpt_arena[prev].pt)
}

/// Check if a closed path is valid
/// Direct port from clipper.engine.cpp IsValidClosedPath (line 449)
pub fn is_valid_closed_path(op: Option<usize>, outpt_arena: &[OutPt]) -> bool {
    match op {
        None => false,
        Some(idx) => {
            let next = outpt_arena[idx].next;
            if next == idx {
                return false;
            }
            let prev = outpt_arena[idx].prev;
            if next == prev {
                return false;
            }
            !is_very_small_triangle(idx, outpt_arena)
        }
    }
}

/// Check if the hot edge is ascending (front edge of its outrec)
/// Direct port from clipper.engine.cpp OutrecIsAscending (line 455)
#[inline]
pub fn outrec_is_ascending(
    hot_edge_idx: usize,
    outrec_list: &[OutRec],
    active_arena: &[Active],
) -> bool {
    if let Some(or_idx) = active_arena[hot_edge_idx].outrec {
        outrec_list[or_idx].front_edge == Some(hot_edge_idx)
    } else {
        false
    }
}

/// Swap front and back sides of an OutRec
/// Direct port from clipper.engine.cpp SwapFrontBackSides (line 460)
pub fn swap_front_back_sides(outrec_idx: usize, outrec_list: &mut [OutRec], outpt_arena: &[OutPt]) {
    std::mem::swap(
        &mut outrec_list[outrec_idx].front_edge,
        &mut outrec_list[outrec_idx].back_edge,
    );
    if let Some(pts) = outrec_list[outrec_idx].pts {
        outrec_list[outrec_idx].pts = Some(outpt_arena[pts].next);
    }
}

/// Check if intersect node edges are adjacent in AEL
/// Direct port from clipper.engine.cpp EdgesAdjacentInAEL (line 468)
#[inline]
pub fn edges_adjacent_in_ael(inode: &IntersectNode, active_arena: &[Active]) -> bool {
    active_arena[inode.edge1].next_in_ael == Some(inode.edge2)
        || active_arena[inode.edge1].prev_in_ael == Some(inode.edge2)
}

/// Check if an edge is joined
/// Direct port from clipper.engine.cpp IsJoined (line 473)
#[inline]
pub fn is_joined(e: &Active) -> bool {
    e.join_with != JoinWith::NoJoin
}

/// Set the owner of an OutRec
/// Direct port from clipper.engine.cpp SetOwner (line 478)
pub fn set_owner(outrec_list: &mut [OutRec], outrec_idx: usize, new_owner_idx: usize) {
    // precondition: new_owner_idx is valid
    outrec_list[new_owner_idx].owner = get_real_outrec(
        outrec_list,
        outrec_list[new_owner_idx].owner.unwrap_or(new_owner_idx),
    );
    let mut tmp = Some(new_owner_idx);
    while let Some(t) = tmp {
        if t == outrec_idx {
            outrec_list[new_owner_idx].owner = outrec_list[outrec_idx].owner;
            break;
        }
        tmp = outrec_list[t].owner;
    }
    outrec_list[outrec_idx].owner = Some(new_owner_idx);
}

/// Point in polygon test for OutPt-based polygons
/// Direct port from clipper.engine.cpp PointInOpPolygon (line 488)
pub fn point_in_op_polygon(
    pt: Point64,
    op_start: usize,
    outpt_arena: &[OutPt],
) -> PointInPolygonResult {
    let next = outpt_arena[op_start].next;
    if next == op_start || outpt_arena[op_start].prev == next {
        return PointInPolygonResult::IsOutside;
    }

    let mut op = op_start;
    loop {
        if outpt_arena[op].pt.y != pt.y {
            break;
        }
        op = outpt_arena[op].next;
        if op == op_start {
            break;
        }
    }
    if outpt_arena[op].pt.y == pt.y {
        return PointInPolygonResult::IsOutside;
    }

    let mut is_above = outpt_arena[op].pt.y < pt.y;
    let starting_above = is_above;
    let mut val = 0;
    let mut op2 = outpt_arena[op].next;

    while op2 != op {
        if is_above {
            while op2 != op && outpt_arena[op2].pt.y < pt.y {
                op2 = outpt_arena[op2].next;
            }
        } else {
            while op2 != op && outpt_arena[op2].pt.y > pt.y {
                op2 = outpt_arena[op2].next;
            }
        }
        if op2 == op {
            break;
        }

        if outpt_arena[op2].pt.y == pt.y {
            let prev = outpt_arena[op2].prev;
            if outpt_arena[op2].pt.x == pt.x
                || (outpt_arena[op2].pt.y == outpt_arena[prev].pt.y
                    && (pt.x < outpt_arena[prev].pt.x) != (pt.x < outpt_arena[op2].pt.x))
            {
                return PointInPolygonResult::IsOn;
            }
            op2 = outpt_arena[op2].next;
            if op2 == op {
                break;
            }
            continue;
        }

        let prev = outpt_arena[op2].prev;
        if pt.x < outpt_arena[op2].pt.x && pt.x < outpt_arena[prev].pt.x {
            // do nothing
        } else if pt.x > outpt_arena[prev].pt.x && pt.x > outpt_arena[op2].pt.x {
            val = 1 - val;
        } else {
            let i = cross_product_sign(outpt_arena[prev].pt, outpt_arena[op2].pt, pt);
            if i == 0 {
                return PointInPolygonResult::IsOn;
            }
            if (i < 0) == is_above {
                val = 1 - val;
            }
        }
        is_above = !is_above;
        op2 = outpt_arena[op2].next;
    }

    if is_above != starting_above {
        let prev = outpt_arena[op2].prev;
        let i = cross_product_sign(outpt_arena[prev].pt, outpt_arena[op2].pt, pt);
        if i == 0 {
            return PointInPolygonResult::IsOn;
        }
        if (i < 0) == is_above {
            val = 1 - val;
        }
    }

    if val == 0 {
        PointInPolygonResult::IsOutside
    } else {
        PointInPolygonResult::IsInside
    }
}

/// Check if path1 (as OutPt list) is contained within path2 (as OutPt list)
/// Direct port from clipper.engine.cpp Path2ContainsPath1 (line 576)
pub fn path2_contains_path1_outpt(
    op1_start: usize,
    op2_start: usize,
    outpt_arena: &[OutPt],
) -> bool {
    let mut pip = PointInPolygonResult::IsOn;
    let mut op = op1_start;
    loop {
        match point_in_op_polygon(outpt_arena[op].pt, op2_start, outpt_arena) {
            PointInPolygonResult::IsOutside => {
                if pip == PointInPolygonResult::IsOutside {
                    return false;
                }
                pip = PointInPolygonResult::IsOutside;
            }
            PointInPolygonResult::IsInside => {
                if pip == PointInPolygonResult::IsInside {
                    return true;
                }
                pip = PointInPolygonResult::IsInside;
            }
            _ => {}
        }
        op = outpt_arena[op].next;
        if op == op1_start {
            break;
        }
    }
    // result unclear, try using cleaned paths
    let clean1 = get_clean_path(op1_start, outpt_arena);
    let clean2 = get_clean_path(op2_start, outpt_arena);
    path2_contains_path1(&clean1, &clean2)
}

/// Check if path1 is contained within path2 using Path64 vectors
/// Direct port from clipper.h line 717 - template Path2ContainsPath1<T>
/// precondition: paths must not intersect, except for transient micro intersections
pub fn path2_contains_path1(path1: &Path64, path2: &Path64) -> bool {
    let mut pip = PointInPolygonResult::IsOn;
    for pt in path1 {
        match point_in_polygon(*pt, path2) {
            PointInPolygonResult::IsOutside => {
                if pip == PointInPolygonResult::IsOutside {
                    return false;
                }
                pip = PointInPolygonResult::IsOutside;
            }
            PointInPolygonResult::IsInside => {
                if pip == PointInPolygonResult::IsInside {
                    return true;
                }
                pip = PointInPolygonResult::IsInside;
            }
            _ => {}
        }
    }
    if pip != PointInPolygonResult::IsInside {
        return false;
    }
    // result is likely true but check midpoint
    let mp1 = get_bounds_path(path1).mid_point();
    point_in_polygon(mp1, path2) == PointInPolygonResult::IsInside
}

/// Build a Path64 from OutPt circular list
/// Direct port from clipper.engine.cpp BuildPath64 (line 2891)
pub fn build_path64_from_outpt(
    op_start: usize,
    reverse: bool,
    is_open: bool,
    outpt_arena: &[OutPt],
) -> Option<Path64> {
    let next = outpt_arena[op_start].next;
    if next == op_start || (!is_open && next == outpt_arena[op_start].prev) {
        return None;
    }

    let mut path = Path64::new();
    let (mut last_pt, mut op2);

    if reverse {
        last_pt = outpt_arena[op_start].pt;
        op2 = outpt_arena[op_start].prev;
    } else {
        let op_next = outpt_arena[op_start].next;
        last_pt = outpt_arena[op_next].pt;
        op2 = outpt_arena[op_next].next;
        path.push(last_pt);
        while op2 != outpt_arena[op_start].next {
            if outpt_arena[op2].pt != last_pt {
                last_pt = outpt_arena[op2].pt;
                path.push(last_pt);
            }
            op2 = outpt_arena[op2].next;
        }
        if !is_open
            && path.len() == 3
            && is_very_small_triangle(outpt_arena[op_start].next, outpt_arena)
        {
            return None;
        }
        return if path.len() >= 2 { Some(path) } else { None };
    }

    // reverse case
    path.push(last_pt);
    while op2 != op_start {
        if outpt_arena[op2].pt != last_pt {
            last_pt = outpt_arena[op2].pt;
            path.push(last_pt);
        }
        op2 = outpt_arena[op2].prev;
    }

    if !is_open && path.len() == 3 && is_very_small_triangle(op_start, outpt_arena) {
        return None;
    }

    if path.len() >= 2 {
        Some(path)
    } else {
        None
    }
}

/// Build a PathD from OutPt circular list
/// Direct port from clipper.engine.cpp BuildPathD (line 3055)
pub fn build_path_d_from_outpt(
    op_start: usize,
    reverse: bool,
    is_open: bool,
    outpt_arena: &[OutPt],
    inv_scale: f64,
) -> Option<PathD> {
    let next = outpt_arena[op_start].next;
    if next == op_start || (!is_open && next == outpt_arena[op_start].prev) {
        return None;
    }

    let mut path = PathD::new();
    let (mut last_pt, mut op2);

    if reverse {
        last_pt = outpt_arena[op_start].pt;
        op2 = outpt_arena[op_start].prev;
    } else {
        let op_next = outpt_arena[op_start].next;
        last_pt = outpt_arena[op_next].pt;
        op2 = outpt_arena[op_next].next;
        path.push(PointD::new(
            last_pt.x as f64 * inv_scale,
            last_pt.y as f64 * inv_scale,
        ));
        while op2 != outpt_arena[op_start].next {
            if outpt_arena[op2].pt != last_pt {
                last_pt = outpt_arena[op2].pt;
                path.push(PointD::new(
                    last_pt.x as f64 * inv_scale,
                    last_pt.y as f64 * inv_scale,
                ));
            }
            op2 = outpt_arena[op2].next;
        }
        if path.len() == 3 && is_very_small_triangle(outpt_arena[op_start].next, outpt_arena) {
            return None;
        }
        return if path.len() >= 2 { Some(path) } else { None };
    }

    // reverse case
    path.push(PointD::new(
        last_pt.x as f64 * inv_scale,
        last_pt.y as f64 * inv_scale,
    ));
    while op2 != op_start {
        if outpt_arena[op2].pt != last_pt {
            last_pt = outpt_arena[op2].pt;
            path.push(PointD::new(
                last_pt.x as f64 * inv_scale,
                last_pt.y as f64 * inv_scale,
            ));
        }
        op2 = outpt_arena[op2].prev;
    }

    if path.len() == 3 && is_very_small_triangle(op_start, outpt_arena) {
        return None;
    }

    if path.len() >= 2 {
        Some(path)
    } else {
        None
    }
}

/// Get the last output point for a hot edge
/// Direct port from clipper.engine.cpp GetLastOp (line 2496)
#[inline]
pub fn get_last_op(
    hot_edge_idx: usize,
    active_arena: &[Active],
    outrec_list: &[OutRec],
    outpt_arena: &[OutPt],
) -> Option<usize> {
    if let Some(or_idx) = active_arena[hot_edge_idx].outrec {
        let result = outrec_list[or_idx].pts?;
        if outrec_list[or_idx].front_edge != Some(hot_edge_idx) {
            Some(outpt_arena[result].next)
        } else {
            Some(result)
        }
    } else {
        None
    }
}

/// Fix all OutPt outrec references in an OutRec
/// Direct port from clipper.engine.cpp FixOutRecPts (line 2158)
pub fn fix_outrec_pts(outrec_idx: usize, outrec_list: &[OutRec], outpt_arena: &mut [OutPt]) {
    if let Some(pts) = outrec_list[outrec_idx].pts {
        let mut op = pts;
        loop {
            outpt_arena[op].outrec = outrec_idx;
            op = outpt_arena[op].next;
            if op == pts {
                break;
            }
        }
    }
}

/// Update all OutPt outrec references in an OutRec
/// Direct port from clipper.engine.cpp UpdateOutrecOwner (line 1684)
pub fn update_outrec_owner(outrec_idx: usize, outrec_list: &[OutRec], outpt_arena: &mut [OutPt]) {
    fix_outrec_pts(outrec_idx, outrec_list, outpt_arena);
}

/// Move splits from one OutRec to another
/// Direct port from clipper.engine.cpp MoveSplits (line 2269)
pub fn move_splits(outrec_list: &mut [OutRec], from_or: usize, to_or: usize) {
    let from_splits = outrec_list[from_or].splits.take().unwrap_or_default();
    if outrec_list[to_or].splits.is_none() {
        outrec_list[to_or].splits = Some(Vec::new());
    }
    if let Some(ref mut to_splits) = outrec_list[to_or].splits {
        for s in &from_splits {
            if *s != to_or {
                to_splits.push(*s);
            }
        }
    }
}

/// Uncouple an OutRec from its edges
/// Direct port from clipper.engine.cpp UncoupleOutRec (line 425)
pub fn uncouple_outrec(e_idx: usize, active_arena: &mut [Active], outrec_list: &mut [OutRec]) {
    if let Some(or_idx) = active_arena[e_idx].outrec {
        if let Some(fe) = outrec_list[or_idx].front_edge {
            active_arena[fe].outrec = None;
        }
        if let Some(be) = outrec_list[or_idx].back_edge {
            active_arena[be].outrec = None;
        }
        outrec_list[or_idx].front_edge = None;
        outrec_list[or_idx].back_edge = None;
    }
}

/// Extract an active from the SEL (sorted edge list)
/// Direct port from clipper.engine.cpp ExtractFromSEL
pub fn extract_from_sel(e_idx: usize, active_arena: &mut [Active]) -> Option<usize> {
    let next = active_arena[e_idx].next_in_sel;
    if let Some(next_idx) = next {
        active_arena[next_idx].prev_in_sel = active_arena[e_idx].prev_in_sel;
    }
    if let Some(prev_idx) = active_arena[e_idx].prev_in_sel {
        active_arena[prev_idx].next_in_sel = next;
    }
    active_arena[e_idx].prev_in_sel = None;
    active_arena[e_idx].next_in_sel = None;
    next
}

/// Insert tmp before left in SEL
/// Direct port from clipper.engine.cpp Insert1Before2InSEL
pub fn insert1_before2_in_sel(tmp_idx: usize, left_idx: usize, active_arena: &mut [Active]) {
    let prev = active_arena[left_idx].prev_in_sel;
    active_arena[tmp_idx].prev_in_sel = prev;
    if let Some(prev_idx) = prev {
        active_arena[prev_idx].next_in_sel = Some(tmp_idx);
    }
    active_arena[tmp_idx].next_in_sel = Some(left_idx);
    active_arena[left_idx].prev_in_sel = Some(tmp_idx);
}
