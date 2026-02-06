//! Main polygon clipping engine
//!
//! Direct port from clipper.engine.h and clipper.engine.cpp
//! Copyright (c) Angus Johnson 2010-2025
//! This is the core clipping module implementing the Vatti sweep-line algorithm

use crate::core::*;
use crate::engine_fns::*;
use crate::engine_public::PolyTree64;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

// ============================================================================
// Sentinel value for null indices in arena-based structures
// ============================================================================
pub const NONE: usize = usize::MAX;

// ============================================================================
// Enums - Direct port from clipper.engine.h lines 29-36
// ============================================================================

/// Type of clipping operation
/// Direct port from clipper.engine.h line 29
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum ClipType {
    #[default]
    NoClip,
    Intersection,
    Union,
    Difference,
    Xor,
}

/// Whether a path is Subject or Clip
/// Direct port from clipper.engine.h line 31
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum PathType {
    Subject,
    Clip,
}

/// How an edge joins with another (for horizontal processing)
/// Direct port from clipper.engine.h line 32
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum JoinWith {
    #[default]
    NoJoin,
    Left,
    Right,
}

/// Vertex flags (bitflags)
/// Direct port from clipper.engine.h line 34
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VertexFlags(u32);

impl VertexFlags {
    pub const EMPTY: VertexFlags = VertexFlags(0);
    pub const OPEN_START: VertexFlags = VertexFlags(1);
    pub const OPEN_END: VertexFlags = VertexFlags(2);
    pub const LOCAL_MAX: VertexFlags = VertexFlags(4);
    pub const LOCAL_MIN: VertexFlags = VertexFlags(8);
}

impl std::ops::BitAnd for VertexFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        VertexFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for VertexFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        VertexFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAndAssign for VertexFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for VertexFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// ============================================================================
// Core Data Structures - Arena-indexed
// ============================================================================

/// Input polygon vertex (circular doubly-linked list via arena indices)
/// Direct port from clipper.engine.h line 48
#[derive(Debug, Clone)]
pub struct Vertex {
    pub pt: Point64,
    pub next: usize, // index into vertex arena
    pub prev: usize, // index into vertex arena
    pub flags: VertexFlags,
}

impl Vertex {
    pub fn new(pt: Point64) -> Self {
        Self {
            pt,
            next: NONE,
            prev: NONE,
            flags: VertexFlags::EMPTY,
        }
    }
}

/// Output point in the clipping result (circular doubly-linked list)
/// Direct port from clipper.engine.h line 55
#[derive(Debug, Clone)]
pub struct OutPt {
    pub pt: Point64,
    pub next: usize,         // index into outpt arena
    pub prev: usize,         // index into outpt arena
    pub outrec: usize,       // index into outrec list
    pub horz: Option<usize>, // index into horz_seg_list
}

impl OutPt {
    pub fn new(pt: Point64, outrec_idx: usize) -> Self {
        Self {
            pt,
            next: NONE,
            prev: NONE,
            outrec: outrec_idx,
            horz: None,
        }
    }
}

/// Output polygon record
/// Direct port from clipper.engine.h line 79
#[derive(Debug, Clone)]
pub struct OutRec {
    pub idx: usize,
    pub owner: Option<usize>,           // index into outrec list
    pub front_edge: Option<usize>,      // index into active arena
    pub back_edge: Option<usize>,       // index into active arena
    pub pts: Option<usize>,             // index into outpt arena
    pub polypath: Option<usize>,        // index into polypath list
    pub splits: Option<Vec<usize>>,     // indices of split outrecs
    pub recursive_split: Option<usize>, // index into outrec list
    pub bounds: Rect64,
    pub path: Path64,
    pub is_open: bool,
}

impl OutRec {
    pub fn new(idx: usize) -> Self {
        Self {
            idx,
            owner: None,
            front_edge: None,
            back_edge: None,
            pts: None,
            polypath: None,
            splits: None,
            recursive_split: None,
            bounds: Rect64::new(0, 0, 0, 0),
            path: Path64::new(),
            is_open: false,
        }
    }
}

/// Active edge in the sweep line
/// Direct port from clipper.engine.h line 104
#[derive(Debug, Clone)]
pub struct Active {
    pub bot: Point64,
    pub top: Point64,
    pub curr_x: i64,
    pub dx: f64,
    pub wind_dx: i32, // 1 or -1 depending on winding direction
    pub wind_cnt: i32,
    pub wind_cnt2: i32,        // winding count of the opposite polytype
    pub outrec: Option<usize>, // index into outrec list
    // AEL: active edge list
    pub prev_in_ael: Option<usize>, // index into active arena
    pub next_in_ael: Option<usize>, // index into active arena
    // SEL: sorted edge list
    pub prev_in_sel: Option<usize>, // index into active arena
    pub next_in_sel: Option<usize>, // index into active arena
    pub jump: Option<usize>,        // index into active arena
    pub vertex_top: usize,          // index into vertex arena
    pub local_min: usize,           // index into local_minima list
    pub is_left_bound: bool,
    pub join_with: JoinWith,
}

impl Active {
    pub fn new() -> Self {
        Self {
            bot: Point64::new(0, 0),
            top: Point64::new(0, 0),
            curr_x: 0,
            dx: 0.0,
            wind_dx: 1,
            wind_cnt: 0,
            wind_cnt2: 0,
            outrec: None,
            prev_in_ael: None,
            next_in_ael: None,
            prev_in_sel: None,
            next_in_sel: None,
            jump: None,
            vertex_top: NONE,
            local_min: NONE,
            is_left_bound: false,
            join_with: JoinWith::NoJoin,
        }
    }
}

impl Default for Active {
    fn default() -> Self {
        Self::new()
    }
}

/// Local minimum vertex where ascending and descending bounds meet
/// Direct port from clipper.engine.h line 131
#[derive(Debug, Clone)]
pub struct LocalMinima {
    pub vertex: usize, // index into vertex arena
    pub polytype: PathType,
    pub is_open: bool,
}

impl LocalMinima {
    pub fn new(vertex: usize, polytype: PathType, is_open: bool) -> Self {
        Self {
            vertex,
            polytype,
            is_open,
        }
    }
}

/// Intersection between two active edges
/// Direct port from clipper.engine.h line 139
#[derive(Debug, Clone)]
pub struct IntersectNode {
    pub pt: Point64,
    pub edge1: usize, // index into active arena
    pub edge2: usize, // index into active arena
}

impl IntersectNode {
    pub fn new() -> Self {
        Self {
            pt: Point64::new(0, 0),
            edge1: NONE,
            edge2: NONE,
        }
    }

    pub fn with_edges(edge1: usize, edge2: usize, pt: Point64) -> Self {
        Self { pt, edge1, edge2 }
    }
}

impl Default for IntersectNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Horizontal segment for horizontal edge processing
/// Direct port from clipper.engine.h line 148
#[derive(Debug, Clone)]
pub struct HorzSegment {
    pub left_op: Option<usize>,  // index into outpt arena
    pub right_op: Option<usize>, // index into outpt arena
    pub left_to_right: bool,
}

impl HorzSegment {
    pub fn new() -> Self {
        Self {
            left_op: None,
            right_op: None,
            left_to_right: true,
        }
    }

    pub fn with_op(op_idx: usize) -> Self {
        Self {
            left_op: Some(op_idx),
            right_op: None,
            left_to_right: true,
        }
    }
}

impl Default for HorzSegment {
    fn default() -> Self {
        Self::new()
    }
}

/// Horizontal join between two output points
/// Direct port from clipper.engine.h line 156
#[derive(Debug, Clone)]
pub struct HorzJoin {
    pub op1: Option<usize>, // index into outpt arena
    pub op2: Option<usize>, // index into outpt arena
}

impl HorzJoin {
    pub fn new() -> Self {
        Self {
            op1: None,
            op2: None,
        }
    }

    pub fn with_ops(ltr: usize, rtl: usize) -> Self {
        Self {
            op1: Some(ltr),
            op2: Some(rtl),
        }
    }
}

impl Default for HorzJoin {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ClipperBase - Main clipping engine struct
// Direct port from clipper.engine.h line 192
// ============================================================================

/// Main clipping engine. Manages all arenas and the sweep-line algorithm state.
/// Direct port from clipper.engine.h line 192
pub struct ClipperBase {
    // Configuration
    pub cliptype: ClipType,
    pub fillrule: FillRule,
    pub preserve_collinear: bool,
    pub reverse_solution: bool,
    pub error_code: i32,
    pub has_open_paths: bool,
    pub succeeded: bool,
    using_polytree: bool,

    // Sweep-line state
    bot_y: i64,
    minima_list_sorted: bool,

    // Arenas - all nodes are stored here, referenced by indices
    pub vertex_arena: Vec<Vertex>,
    pub active_arena: Vec<Active>,
    pub outpt_arena: Vec<OutPt>,

    // Lists referencing arena indices
    pub outrec_list: Vec<OutRec>,
    pub minima_list: Vec<LocalMinima>,
    current_locmin_idx: usize,
    vertex_lists: Vec<Vec<usize>>, // groups of vertex indices

    // Active edge list head
    pub actives: Option<usize>, // index into active_arena
    pub sel: Option<usize>,     // index into active_arena

    // Scanline priority queue (min-heap using Reverse)
    scanline_list: BinaryHeap<Reverse<i64>>,

    // Intersection and horizontal processing
    pub intersect_nodes: Vec<IntersectNode>,
    pub horz_seg_list: Vec<HorzSegment>,
    pub horz_join_list: Vec<HorzJoin>,
}

impl ClipperBase {
    pub fn new() -> Self {
        Self {
            cliptype: ClipType::NoClip,
            fillrule: FillRule::EvenOdd,
            preserve_collinear: true,
            reverse_solution: false,
            error_code: 0,
            has_open_paths: false,
            succeeded: true,
            using_polytree: false,
            bot_y: 0,
            minima_list_sorted: false,
            vertex_arena: Vec::new(),
            active_arena: Vec::new(),
            outpt_arena: Vec::new(),
            outrec_list: Vec::new(),
            minima_list: Vec::new(),
            current_locmin_idx: 0,
            vertex_lists: Vec::new(),
            actives: None,
            sel: None,
            scanline_list: BinaryHeap::new(),
            intersect_nodes: Vec::new(),
            horz_seg_list: Vec::new(),
            horz_join_list: Vec::new(),
        }
    }

    /// Clear all data
    /// Direct port from clipper.engine.h line 284
    pub fn clear(&mut self) {
        self.vertex_arena.clear();
        self.active_arena.clear();
        self.outpt_arena.clear();
        self.outrec_list.clear();
        self.minima_list.clear();
        self.current_locmin_idx = 0;
        self.vertex_lists.clear();
        self.actives = None;
        self.sel = None;
        self.scanline_list.clear();
        self.intersect_nodes.clear();
        self.horz_seg_list.clear();
        self.horz_join_list.clear();
        self.minima_list_sorted = false;
        self.has_open_paths = false;
        self.succeeded = true;
    }

    /// Insert a scanline y-value
    /// Direct port from clipper.engine.cpp InsertScanline
    #[inline]
    pub fn insert_scanline(&mut self, y: i64) {
        self.scanline_list.push(Reverse(y));
    }

    /// Pop the next scanline y-value
    /// Direct port from clipper.engine.cpp PopScanline
    #[inline]
    pub fn pop_scanline(&mut self) -> Option<i64> {
        if self.scanline_list.is_empty() {
            return None;
        }
        let y = self.scanline_list.pop().unwrap().0;
        // Skip duplicates
        while !self.scanline_list.is_empty() && self.scanline_list.peek().unwrap().0 == y {
            self.scanline_list.pop();
        }
        Some(y)
    }

    /// Pop a local minimum at a given y-value
    /// Direct port from clipper.engine.cpp PopLocalMinima
    #[inline]
    pub fn pop_local_minima(&mut self, y: i64) -> Option<usize> {
        if self.current_locmin_idx < self.minima_list.len() {
            let vertex_idx = self.minima_list[self.current_locmin_idx].vertex;
            if self.vertex_arena[vertex_idx].pt.y == y {
                let idx = self.current_locmin_idx;
                self.current_locmin_idx += 1;
                return Some(idx);
            }
        }
        None
    }

    /// Create a new OutRec
    /// Direct port from clipper.engine.cpp NewOutRec
    pub fn new_out_rec(&mut self) -> usize {
        let idx = self.outrec_list.len();
        self.outrec_list.push(OutRec::new(idx));
        idx
    }

    /// Create a new OutPt in the arena
    pub fn new_out_pt(&mut self, pt: Point64, outrec_idx: usize) -> usize {
        let idx = self.outpt_arena.len();
        let mut op = OutPt::new(pt, outrec_idx);
        op.next = idx;
        op.prev = idx;
        self.outpt_arena.push(op);
        idx
    }

    /// Create a new Active in the arena
    pub fn new_active(&mut self) -> usize {
        let idx = self.active_arena.len();
        self.active_arena.push(Active::new());
        idx
    }

    /// Add a local minimum
    /// Direct port from clipper.engine.cpp AddLocMin
    pub fn add_loc_min(&mut self, vert_idx: usize, polytype: PathType, is_open: bool) {
        // Check that the vertex is a local minima
        self.vertex_arena[vert_idx].flags |= VertexFlags::LOCAL_MIN;
        self.minima_list
            .push(LocalMinima::new(vert_idx, polytype, is_open));
    }

    /// Add a path for clipping (converts to vertex representation)
    /// Direct port from clipper.engine.cpp AddPath / AddPaths_
    pub fn add_path(&mut self, path: &Path64, polytype: PathType, is_open: bool) {
        let mut path = path.clone();

        // Validate path
        let cnt = path.len();
        if cnt < 2 || (!is_open && cnt < 3) {
            return;
        }

        // Remove consecutive duplicates
        strip_duplicates_path(&mut path, !is_open);

        if path.len() < 2 || (!is_open && path.len() < 3) {
            return;
        }

        if is_open {
            self.has_open_paths = true;
        }

        // Build vertices
        let first_vert_idx = self.vertex_arena.len();
        let vert_count = path.len();

        for pt in &path {
            self.vertex_arena.push(Vertex::new(*pt));
        }

        // Link vertices into circular list
        for i in 0..vert_count {
            let abs_idx = first_vert_idx + i;
            self.vertex_arena[abs_idx].next = first_vert_idx + (i + 1) % vert_count;
            self.vertex_arena[abs_idx].prev = first_vert_idx + (i + vert_count - 1) % vert_count;
        }

        // Record this vertex list
        let vert_indices: Vec<usize> = (first_vert_idx..first_vert_idx + vert_count).collect();
        self.vertex_lists.push(vert_indices);

        if is_open {
            // For open paths, mark start and end
            self.vertex_arena[first_vert_idx].flags |= VertexFlags::OPEN_START;
            self.vertex_arena[first_vert_idx + vert_count - 1].flags |= VertexFlags::OPEN_END;

            // Find local minima for open paths
            let mut i = first_vert_idx;
            let last = first_vert_idx + vert_count - 1;

            // Skip ascending from start
            while i < last && self.vertex_arena[i].pt.y <= self.vertex_arena[i + 1].pt.y {
                i += 1;
            }
            // We need to handle ascending/descending sequences and add local minima
            // The first local min for an open path is the start if it's lower than next,
            // or wherever we first start going up after going down
            self.find_and_add_local_minima_open(first_vert_idx, vert_count, polytype);
        } else {
            self.find_and_add_local_minima_closed(first_vert_idx, vert_count, polytype);
        }
    }

    /// Find and add local minima for a closed path
    fn find_and_add_local_minima_closed(
        &mut self,
        first_vert_idx: usize,
        vert_count: usize,
        polytype: PathType,
    ) {
        // Find all local minima in the closed path
        // A local minimum is where the path changes from descending to ascending
        // (in screen coordinates where Y increases downward)

        // First, find any vertex that is NOT a local minimum to start from
        // We need to find a vertex where prev.y != curr.y to know direction
        let mut start = first_vert_idx;
        let end = first_vert_idx + vert_count;

        // Skip vertices where prev has same y
        let mut found_start = false;
        for i in first_vert_idx..end {
            let prev = self.vertex_arena[i].prev;
            if self.vertex_arena[prev].pt.y != self.vertex_arena[i].pt.y {
                start = i;
                found_start = true;
                break;
            }
        }
        if !found_start {
            return; // All vertices at same y - degenerate
        }

        // Determine initial direction
        let prev = self.vertex_arena[start].prev;
        let mut going_up = self.vertex_arena[prev].pt.y > self.vertex_arena[start].pt.y;

        let mut curr = start;
        loop {
            let next = self.vertex_arena[curr].next;
            let curr_pt = self.vertex_arena[curr].pt;
            let next_pt = self.vertex_arena[next].pt;

            if next_pt.y > curr_pt.y && going_up {
                // Was going up, now going down => local max
                self.vertex_arena[curr].flags |= VertexFlags::LOCAL_MAX;
                going_up = false;
            } else if next_pt.y < curr_pt.y && !going_up {
                // Was going down, now going up => local min
                going_up = true;
                self.add_loc_min(curr, polytype, false);
            }

            // Skip horizontal edges (same y)
            if next_pt.y == curr_pt.y && next != start {
                curr = next;
                continue;
            }

            curr = next;
            if curr == start {
                break;
            }
        }
    }

    /// Find and add local minima for an open path
    fn find_and_add_local_minima_open(
        &mut self,
        first_vert_idx: usize,
        vert_count: usize,
        polytype: PathType,
    ) {
        let last_vert_idx = first_vert_idx + vert_count - 1;

        // For open paths, the first and last vertices are potential local minima
        let first_pt = self.vertex_arena[first_vert_idx].pt;
        let second_pt = self.vertex_arena[first_vert_idx + 1].pt;

        // The start is a local min if it goes up (in screen coords, y decreases)
        // or if horizontal then the next segment goes up
        let mut going_up;
        if first_pt.y > second_pt.y {
            // Going up from first vertex - first is a local min
            self.add_loc_min(first_vert_idx, polytype, true);
            going_up = true;
        } else if first_pt.y < second_pt.y {
            // Going down - first is NOT a local min, it's a local max
            self.vertex_arena[first_vert_idx].flags |= VertexFlags::LOCAL_MAX;
            going_up = false;
        } else {
            // Horizontal at start - need to keep scanning
            let mut i = first_vert_idx + 1;
            while i < last_vert_idx && self.vertex_arena[i].pt.y == first_pt.y {
                i += 1;
            }
            if i >= last_vert_idx {
                return; // degenerate - all same y
            }
            going_up = self.vertex_arena[i].pt.y < first_pt.y;
            if going_up {
                self.add_loc_min(first_vert_idx, polytype, true);
            } else {
                self.vertex_arena[first_vert_idx].flags |= VertexFlags::LOCAL_MAX;
            }
        }

        // Scan interior vertices
        for i in (first_vert_idx + 1)..last_vert_idx {
            let curr_pt = self.vertex_arena[i].pt;
            let next_pt = self.vertex_arena[i + 1].pt;

            if next_pt.y > curr_pt.y && going_up {
                self.vertex_arena[i].flags |= VertexFlags::LOCAL_MAX;
                going_up = false;
            } else if next_pt.y < curr_pt.y && !going_up {
                going_up = true;
                self.add_loc_min(i, polytype, true);
            }
        }

        // Handle last vertex
        let last_pt = self.vertex_arena[last_vert_idx].pt;
        let prev_pt = self.vertex_arena[last_vert_idx - 1].pt;
        if !going_up {
            // Was descending - last vertex is a local min
            self.add_loc_min(last_vert_idx, polytype, true);
        } else if last_pt.y < prev_pt.y || (last_pt.y == prev_pt.y && going_up) {
            self.vertex_arena[last_vert_idx].flags |= VertexFlags::LOCAL_MAX;
        }
    }

    /// Add multiple paths for clipping
    /// Direct port from clipper.engine.h AddPaths
    pub fn add_paths(&mut self, paths: &Paths64, polytype: PathType, is_open: bool) {
        for path in paths {
            self.add_path(path, polytype, is_open);
        }
    }

    /// Sort the local minima list
    pub fn sort_minima_list(&mut self) {
        if !self.minima_list_sorted {
            // Sort by y descending (bottom first), then by x ascending
            let vertex_arena = &self.vertex_arena;
            self.minima_list.sort_by(|a, b| {
                let a_pt = vertex_arena[a.vertex].pt;
                let b_pt = vertex_arena[b.vertex].pt;
                if a_pt.y != b_pt.y {
                    b_pt.y.cmp(&a_pt.y) // descending by y (larger y = lower on screen)
                } else {
                    a_pt.x.cmp(&b_pt.x) // ascending by x
                }
            });
            self.minima_list_sorted = true;
        }
    }

    /// Duplicate an OutPt, inserting after or before
    /// Direct port from clipper.engine.cpp DuplicateOp
    pub fn duplicate_op(&mut self, op_idx: usize, insert_after: bool) -> usize {
        let pt = self.outpt_arena[op_idx].pt;
        let outrec = self.outpt_arena[op_idx].outrec;
        let new_idx = self.outpt_arena.len();
        let mut result = OutPt::new(pt, outrec);

        if insert_after {
            let next = self.outpt_arena[op_idx].next;
            result.next = next;
            result.prev = op_idx;
            self.outpt_arena.push(result);
            self.outpt_arena[next].prev = new_idx;
            self.outpt_arena[op_idx].next = new_idx;
        } else {
            let prev = self.outpt_arena[op_idx].prev;
            result.prev = prev;
            result.next = op_idx;
            self.outpt_arena.push(result);
            self.outpt_arena[prev].next = new_idx;
            self.outpt_arena[op_idx].prev = new_idx;
        }

        new_idx
    }

    /// Dispose (unlink) an OutPt, return the next
    /// Direct port from clipper.engine.cpp DisposeOutPt
    pub fn dispose_out_pt(&mut self, op_idx: usize) -> usize {
        let result = self.outpt_arena[op_idx].next;
        let prev = self.outpt_arena[op_idx].prev;
        self.outpt_arena[prev].next = result;
        self.outpt_arena[result].prev = prev;
        // Note: we don't actually free the arena slot, just unlink
        result
    }

    /// Dispose all OutPts in a circular list, setting outrec.pts to None
    /// Direct port from clipper.engine.cpp DisposeOutPts
    pub fn dispose_out_pts(&mut self, outrec_idx: usize) {
        if let Some(pts_idx) = self.outrec_list[outrec_idx].pts {
            // Unlink the circular list (don't actually free, arena-based)
            let prev = self.outpt_arena[pts_idx].prev;
            self.outpt_arena[prev].next = NONE; // break the circle
                                                // Walk and mark as disposed
            let mut op = Some(pts_idx);
            while let Some(idx) = op {
                if self.outpt_arena[idx].next == NONE {
                    break;
                }
                let next = self.outpt_arena[idx].next;
                if next == NONE || next == idx {
                    break;
                }
                op = Some(next);
            }
        }
        self.outrec_list[outrec_idx].pts = None;
    }

    /// Set the front and back edges of an OutRec
    /// Direct port from clipper.engine.cpp SetSides
    #[inline]
    pub fn set_sides(&mut self, outrec_idx: usize, start_edge: usize, end_edge: usize) {
        self.outrec_list[outrec_idx].front_edge = Some(start_edge);
        self.outrec_list[outrec_idx].back_edge = Some(end_edge);
    }

    /// Swap OutRecs between two active edges
    /// Direct port from clipper.engine.cpp SwapOutrecs
    pub fn swap_outrecs(&mut self, e1_idx: usize, e2_idx: usize) {
        let or1 = self.active_arena[e1_idx].outrec;
        let or2 = self.active_arena[e2_idx].outrec;

        if or1 == or2 {
            if let Some(or_idx) = or1 {
                let fe = self.outrec_list[or_idx].front_edge;
                let be = self.outrec_list[or_idx].back_edge;
                self.outrec_list[or_idx].front_edge = be;
                self.outrec_list[or_idx].back_edge = fe;
            }
            return;
        }

        if let Some(or1_idx) = or1 {
            if self.outrec_list[or1_idx].front_edge == Some(e1_idx) {
                self.outrec_list[or1_idx].front_edge = Some(e2_idx);
            } else {
                self.outrec_list[or1_idx].back_edge = Some(e2_idx);
            }
        }

        if let Some(or2_idx) = or2 {
            if self.outrec_list[or2_idx].front_edge == Some(e2_idx) {
                self.outrec_list[or2_idx].front_edge = Some(e1_idx);
            } else {
                self.outrec_list[or2_idx].back_edge = Some(e1_idx);
            }
        }

        self.active_arena[e1_idx].outrec = or2;
        self.active_arena[e2_idx].outrec = or1;
    }

    /// Check if an active edge is the front edge of its outrec
    /// Direct port from clipper.engine.cpp IsFront
    #[inline]
    pub fn is_front(&self, e_idx: usize) -> bool {
        if let Some(or_idx) = self.active_arena[e_idx].outrec {
            self.outrec_list[or_idx].front_edge == Some(e_idx)
        } else {
            false
        }
    }

    /// Get the previous hot edge in AEL
    /// Direct port from clipper.engine.cpp GetPrevHotEdge
    pub fn get_prev_hot_edge(&self, e_idx: usize) -> Option<usize> {
        let mut prev = self.active_arena[e_idx].prev_in_ael;
        while let Some(p_idx) = prev {
            if is_open_active(&self.active_arena[p_idx], &self.minima_list)
                || !is_hot_edge(&self.active_arena[p_idx])
            {
                prev = self.active_arena[p_idx].prev_in_ael;
            } else {
                return Some(p_idx);
            }
        }
        None
    }

    /// Add a trial horizontal join
    /// Direct port from clipper.engine.cpp AddTrialHorzJoin
    #[inline]
    pub fn add_trial_horz_join(&mut self, op_idx: usize) {
        if self.outpt_arena[op_idx].horz.is_some() {
            return;
        }
        let hs_idx = self.horz_seg_list.len();
        self.horz_seg_list.push(HorzSegment::with_op(op_idx));
        self.outpt_arena[op_idx].horz = Some(hs_idx);
    }

    /// Push a horizontal edge onto the horz stack
    /// Direct port from clipper.engine.cpp PushHorz
    #[inline]
    pub fn push_horz(&mut self, e_idx: usize) {
        self.active_arena[e_idx].next_in_sel = self.sel;
        self.sel = Some(e_idx);
    }

    /// Pop a horizontal edge from the horz stack
    /// Direct port from clipper.engine.cpp PopHorz
    #[inline]
    pub fn pop_horz(&mut self) -> Option<usize> {
        let e = self.sel;
        if let Some(e_idx) = e {
            self.sel = self.active_arena[e_idx].next_in_sel;
        }
        e
    }

    /// Build Path64 output from OutRec
    /// Direct port from clipper.engine.cpp BuildPath64
    pub fn build_path64(&self, outrec: &OutRec) -> Option<Path64> {
        let op_start = outrec.pts?;
        let cnt = point_count(op_start, &self.outpt_arena);
        if cnt < 2 {
            return None;
        }

        let reverse = if outrec.is_open {
            false
        } else {
            let area = area_outpt(op_start, &self.outpt_arena);
            if area == 0.0 {
                return None;
            }
            (area < 0.0) != self.reverse_solution
        };

        let mut result = Path64::with_capacity(cnt as usize);
        if reverse {
            let mut op = op_start;
            loop {
                result.push(self.outpt_arena[op].pt);
                op = self.outpt_arena[op].prev;
                if op == op_start {
                    break;
                }
            }
        } else {
            let op_next = self.outpt_arena[op_start].next;
            let mut op = op_next;
            loop {
                result.push(self.outpt_arena[op].pt);
                op = self.outpt_arena[op].next;
                if op == op_next {
                    break;
                }
            }
        }

        // Remove collinear if not preserving
        if !self.preserve_collinear {
            // Strip collinear points
            let mut i = 0;
            while i < result.len() && result.len() > 2 {
                let prev = if i == 0 { result.len() - 1 } else { i - 1 };
                let next = (i + 1) % result.len();
                if is_collinear(result[prev], result[i], result[next]) {
                    result.remove(i);
                    i = i.saturating_sub(1);
                } else {
                    i += 1;
                }
            }
        }

        if result.len() < 2 {
            None
        } else {
            Some(result)
        }
    }
}

impl Default for ClipperBase {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ClipperBase - Sweep-line algorithm methods
// Direct port from clipper.engine.cpp
// ============================================================================

impl ClipperBase {
    // ---- Helper methods for arena-based access ----

    /// Check if an active edge is part of an open path
    #[inline]
    fn is_open_idx(&self, e_idx: usize) -> bool {
        self.minima_list[self.active_arena[e_idx].local_min].is_open
    }

    /// Check if an active edge is at a local maximum
    #[inline]
    fn is_maxima_idx(&self, e_idx: usize) -> bool {
        is_maxima_vertex(&self.vertex_arena[self.active_arena[e_idx].vertex_top])
    }

    /// Check if an active edge is horizontal
    #[inline]
    fn is_horizontal_idx(&self, e_idx: usize) -> bool {
        self.active_arena[e_idx].top.y == self.active_arena[e_idx].bot.y
    }

    /// Check if an active edge is the open end
    #[inline]
    fn is_open_end_idx(&self, e_idx: usize) -> bool {
        is_open_end_vertex(&self.vertex_arena[self.active_arena[e_idx].vertex_top])
    }

    /// Get polytype for an active edge
    #[inline]
    fn get_poly_type_idx(&self, e_idx: usize) -> PathType {
        self.minima_list[self.active_arena[e_idx].local_min].polytype
    }

    /// Check if two edges have same polytype
    #[inline]
    fn is_same_poly_type_idx(&self, e1: usize, e2: usize) -> bool {
        self.minima_list[self.active_arena[e1].local_min].polytype
            == self.minima_list[self.active_arena[e2].local_min].polytype
    }

    /// Get next vertex index for an active
    #[inline]
    fn next_vertex_idx(&self, e_idx: usize) -> usize {
        if self.active_arena[e_idx].wind_dx > 0 {
            self.vertex_arena[self.active_arena[e_idx].vertex_top].next
        } else {
            self.vertex_arena[self.active_arena[e_idx].vertex_top].prev
        }
    }

    // ---- Reset ----

    /// Reset for a new execution
    /// Direct port from clipper.engine.cpp Reset (line 786)
    pub fn reset(&mut self) {
        self.sort_minima_list();
        // Insert all local minima y-values as scanlines
        for i in (0..self.minima_list.len()).rev() {
            let y = self.vertex_arena[self.minima_list[i].vertex].pt.y;
            self.insert_scanline(y);
        }
        self.current_locmin_idx = 0;
        self.actives = None;
        self.sel = None;
        self.succeeded = true;
    }

    /// Clean up after execution
    pub fn clean_up(&mut self) {
        // In arena-based approach, just clear the runtime state
        self.active_arena.clear();
        self.scanline_list.clear();
        self.intersect_nodes.clear();
        // Dispose all output
        for i in 0..self.outrec_list.len() {
            self.outrec_list[i].pts = None;
        }
        self.outpt_arena.clear();
        self.outrec_list.clear();
        self.horz_seg_list.clear();
        self.horz_join_list.clear();
        self.actives = None;
        self.sel = None;
    }

    // ---- Wind count and contributing checks ----

    /// Check if a closed path edge is contributing to the solution
    /// Direct port from clipper.engine.cpp IsContributingClosed (line 908)
    fn is_contributing_closed(&self, e_idx: usize) -> bool {
        let e = &self.active_arena[e_idx];
        match self.fillrule {
            FillRule::EvenOdd => {}
            FillRule::NonZero => {
                if e.wind_cnt.abs() != 1 {
                    return false;
                }
            }
            FillRule::Positive => {
                if e.wind_cnt != 1 {
                    return false;
                }
            }
            FillRule::Negative => {
                if e.wind_cnt != -1 {
                    return false;
                }
            }
        }

        match self.cliptype {
            ClipType::NoClip => false,
            ClipType::Intersection => match self.fillrule {
                FillRule::Positive => e.wind_cnt2 > 0,
                FillRule::Negative => e.wind_cnt2 < 0,
                _ => e.wind_cnt2 != 0,
            },
            ClipType::Union => match self.fillrule {
                FillRule::Positive => e.wind_cnt2 <= 0,
                FillRule::Negative => e.wind_cnt2 >= 0,
                _ => e.wind_cnt2 == 0,
            },
            ClipType::Difference => {
                let result = match self.fillrule {
                    FillRule::Positive => e.wind_cnt2 <= 0,
                    FillRule::Negative => e.wind_cnt2 >= 0,
                    _ => e.wind_cnt2 == 0,
                };
                if self.get_poly_type_idx(e_idx) == PathType::Subject {
                    result
                } else {
                    !result
                }
            }
            ClipType::Xor => true,
        }
    }

    /// Check if an open path edge is contributing to the solution
    /// Direct port from clipper.engine.cpp IsContributingOpen (line 984)
    fn is_contributing_open(&self, e_idx: usize) -> bool {
        let e = &self.active_arena[e_idx];
        let (is_in_clip, is_in_subj) = match self.fillrule {
            FillRule::Positive => (e.wind_cnt2 > 0, e.wind_cnt > 0),
            FillRule::Negative => (e.wind_cnt2 < 0, e.wind_cnt < 0),
            _ => (e.wind_cnt2 != 0, e.wind_cnt != 0),
        };

        match self.cliptype {
            ClipType::Intersection => is_in_clip,
            ClipType::Union => !is_in_subj && !is_in_clip,
            _ => !is_in_clip,
        }
    }

    /// Set wind count for a closed path edge
    /// Direct port from clipper.engine.cpp SetWindCountForClosedPathEdge (line 1011)
    fn set_wind_count_for_closed_path_edge(&mut self, e_idx: usize) {
        let pt = self.get_poly_type_idx(e_idx);
        let e_wind_dx = self.active_arena[e_idx].wind_dx;

        // Find nearest closed path edge of same PolyType in AEL (heading left)
        let mut e2_opt = self.active_arena[e_idx].prev_in_ael;
        while let Some(e2_idx) = e2_opt {
            if self.get_poly_type_idx(e2_idx) == pt && !self.is_open_idx(e2_idx) {
                break;
            }
            e2_opt = self.active_arena[e2_idx].prev_in_ael;
        }

        let e2_start; // where to start scanning for wind_cnt2
        if let Some(e2_idx) = e2_opt {
            if self.fillrule == FillRule::EvenOdd {
                self.active_arena[e_idx].wind_cnt = e_wind_dx;
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[e2_idx].wind_cnt2;
                e2_start = self.active_arena[e2_idx].next_in_ael;
            } else {
                let e2_wind_cnt = self.active_arena[e2_idx].wind_cnt;
                let e2_wind_dx = self.active_arena[e2_idx].wind_dx;
                // NonZero, Positive, or Negative filling
                if e2_wind_cnt * e2_wind_dx < 0 {
                    // opposite directions: 'e' is outside 'e2'
                    if e2_wind_cnt.abs() > 1 {
                        if e2_wind_dx * e_wind_dx < 0 {
                            self.active_arena[e_idx].wind_cnt = e2_wind_cnt;
                        } else {
                            self.active_arena[e_idx].wind_cnt = e2_wind_cnt + e_wind_dx;
                        }
                    } else {
                        self.active_arena[e_idx].wind_cnt = if self.is_open_idx(e_idx) {
                            1
                        } else {
                            e_wind_dx
                        };
                    }
                } else {
                    // 'e' must be inside 'e2'
                    if e2_wind_dx * e_wind_dx < 0 {
                        self.active_arena[e_idx].wind_cnt = e2_wind_cnt;
                    } else {
                        self.active_arena[e_idx].wind_cnt = e2_wind_cnt + e_wind_dx;
                    }
                }
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[e2_idx].wind_cnt2;
                e2_start = self.active_arena[e2_idx].next_in_ael;
            }
        } else {
            self.active_arena[e_idx].wind_cnt = e_wind_dx;
            e2_start = self.actives;
        }

        // Update wind_cnt2
        let mut e2_cur = e2_start;
        if self.fillrule == FillRule::EvenOdd {
            while e2_cur != Some(e_idx) {
                if let Some(e2i) = e2_cur {
                    if self.get_poly_type_idx(e2i) != pt && !self.is_open_idx(e2i) {
                        let wc2 = self.active_arena[e_idx].wind_cnt2;
                        self.active_arena[e_idx].wind_cnt2 = if wc2 == 0 { 1 } else { 0 };
                    }
                    e2_cur = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        } else {
            while e2_cur != Some(e_idx) {
                if let Some(e2i) = e2_cur {
                    if self.get_poly_type_idx(e2i) != pt && !self.is_open_idx(e2i) {
                        let e2_wd = self.active_arena[e2i].wind_dx;
                        self.active_arena[e_idx].wind_cnt2 += e2_wd;
                    }
                    e2_cur = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        }
    }

    /// Set wind count for an open path edge
    /// Direct port from clipper.engine.cpp SetWindCountForOpenPathEdge (line 1089)
    fn set_wind_count_for_open_path_edge(&mut self, e_idx: usize) {
        let mut e2_opt = self.actives;
        if self.fillrule == FillRule::EvenOdd {
            let mut cnt1 = 0i32;
            let mut cnt2 = 0i32;
            while e2_opt != Some(e_idx) {
                if let Some(e2i) = e2_opt {
                    if self.get_poly_type_idx(e2i) == PathType::Clip {
                        cnt2 += 1;
                    } else if !self.is_open_idx(e2i) {
                        cnt1 += 1;
                    }
                    e2_opt = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
            self.active_arena[e_idx].wind_cnt = if is_odd(cnt1) { 1 } else { 0 };
            self.active_arena[e_idx].wind_cnt2 = if is_odd(cnt2) { 1 } else { 0 };
        } else {
            while e2_opt != Some(e_idx) {
                if let Some(e2i) = e2_opt {
                    if self.get_poly_type_idx(e2i) == PathType::Clip {
                        self.active_arena[e_idx].wind_cnt2 += self.active_arena[e2i].wind_dx;
                    } else if !self.is_open_idx(e2i) {
                        self.active_arena[e_idx].wind_cnt += self.active_arena[e2i].wind_dx;
                    }
                    e2_opt = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        }
    }

    // ---- Edge insertion ----

    /// Insert a left-bound edge into the AEL
    /// Direct port from clipper.engine.cpp InsertLeftEdge (line 1157)
    fn insert_left_edge(&mut self, e_idx: usize) {
        if self.actives.is_none() {
            self.active_arena[e_idx].prev_in_ael = None;
            self.active_arena[e_idx].next_in_ael = None;
            self.actives = Some(e_idx);
        } else {
            let actives_idx = self.actives.unwrap();
            if !is_valid_ael_order(
                &self.active_arena[actives_idx],
                &self.active_arena[e_idx],
                &self.vertex_arena,
                &self.minima_list,
            ) {
                self.active_arena[e_idx].prev_in_ael = None;
                self.active_arena[e_idx].next_in_ael = self.actives;
                self.active_arena[actives_idx].prev_in_ael = Some(e_idx);
                self.actives = Some(e_idx);
            } else {
                let mut e2 = actives_idx;
                while let Some(next) = self.active_arena[e2].next_in_ael {
                    if !is_valid_ael_order(
                        &self.active_arena[next],
                        &self.active_arena[e_idx],
                        &self.vertex_arena,
                        &self.minima_list,
                    ) {
                        break;
                    }
                    e2 = next;
                }
                if self.active_arena[e2].join_with == JoinWith::Right {
                    if let Some(next) = self.active_arena[e2].next_in_ael {
                        e2 = next;
                    } else {
                        return;
                    }
                }
                let next = self.active_arena[e2].next_in_ael;
                self.active_arena[e_idx].next_in_ael = next;
                if let Some(next_idx) = next {
                    self.active_arena[next_idx].prev_in_ael = Some(e_idx);
                }
                self.active_arena[e_idx].prev_in_ael = Some(e2);
                self.active_arena[e2].next_in_ael = Some(e_idx);
            }
        }
    }

    /// Insert a right-bound edge into AEL after its left-bound partner
    /// Direct port from clipper.engine.cpp InsertRightEdge (line 1189)
    fn insert_right_edge(&mut self, e_idx: usize, e2_idx: usize) {
        let next = self.active_arena[e_idx].next_in_ael;
        self.active_arena[e2_idx].next_in_ael = next;
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = Some(e2_idx);
        }
        self.active_arena[e2_idx].prev_in_ael = Some(e_idx);
        self.active_arena[e_idx].next_in_ael = Some(e2_idx);
    }

    // ---- Output management ----

    /// Add an output point for an active edge
    /// Direct port from clipper.engine.cpp AddOutPt (line 1497)
    fn add_out_pt(&mut self, e_idx: usize, pt: Point64) -> usize {
        let or_idx = self.active_arena[e_idx].outrec.unwrap();
        let to_front = self.is_front(e_idx);
        let op_front = self.outrec_list[or_idx].pts.unwrap();
        let op_back = self.outpt_arena[op_front].next;

        if to_front {
            if pt == self.outpt_arena[op_front].pt {
                return op_front;
            }
        } else if pt == self.outpt_arena[op_back].pt {
            return op_back;
        }

        let new_idx = self.outpt_arena.len();
        let mut new_op = OutPt::new(pt, or_idx);
        new_op.prev = op_front;
        new_op.next = op_back;
        self.outpt_arena.push(new_op);
        self.outpt_arena[op_back].prev = new_idx;
        self.outpt_arena[op_front].next = new_idx;

        if to_front {
            self.outrec_list[or_idx].pts = Some(new_idx);
        }
        new_idx
    }

    /// Start an open path output
    /// Direct port from clipper.engine.cpp StartOpenPath (line 1696)
    fn start_open_path(&mut self, e_idx: usize, pt: Point64) -> usize {
        let outrec_idx = self.new_out_rec();
        self.outrec_list[outrec_idx].is_open = true;

        if self.active_arena[e_idx].wind_dx > 0 {
            self.outrec_list[outrec_idx].front_edge = Some(e_idx);
            self.outrec_list[outrec_idx].back_edge = None;
        } else {
            self.outrec_list[outrec_idx].front_edge = None;
            self.outrec_list[outrec_idx].back_edge = Some(e_idx);
        }

        self.active_arena[e_idx].outrec = Some(outrec_idx);

        let op_idx = self.new_out_pt(pt, outrec_idx);
        self.outrec_list[outrec_idx].pts = Some(op_idx);
        op_idx
    }

    /// Add a local minimum polygon
    /// Direct port from clipper.engine.cpp AddLocalMinPoly (line 1332)
    fn add_local_min_poly(
        &mut self,
        e1_idx: usize,
        e2_idx: usize,
        pt: Point64,
        is_new: bool,
    ) -> usize {
        let outrec_idx = self.new_out_rec();
        self.active_arena[e1_idx].outrec = Some(outrec_idx);
        self.active_arena[e2_idx].outrec = Some(outrec_idx);

        if self.is_open_idx(e1_idx) {
            self.outrec_list[outrec_idx].owner = None;
            self.outrec_list[outrec_idx].is_open = true;
            if self.active_arena[e1_idx].wind_dx > 0 {
                self.set_sides(outrec_idx, e1_idx, e2_idx);
            } else {
                self.set_sides(outrec_idx, e2_idx, e1_idx);
            }
        } else {
            let prev_hot = self.get_prev_hot_edge(e1_idx);
            if let Some(prev_hot_idx) = prev_hot {
                if self.using_polytree {
                    if let Some(prev_or) = self.active_arena[prev_hot_idx].outrec {
                        set_owner(&mut self.outrec_list, outrec_idx, prev_or);
                    }
                }
                if outrec_is_ascending(prev_hot_idx, &self.outrec_list, &self.active_arena)
                    == is_new
                {
                    self.set_sides(outrec_idx, e2_idx, e1_idx);
                } else {
                    self.set_sides(outrec_idx, e1_idx, e2_idx);
                }
            } else {
                self.outrec_list[outrec_idx].owner = None;
                if is_new {
                    self.set_sides(outrec_idx, e1_idx, e2_idx);
                } else {
                    self.set_sides(outrec_idx, e2_idx, e1_idx);
                }
            }
        }

        let op_idx = self.new_out_pt(pt, outrec_idx);
        self.outrec_list[outrec_idx].pts = Some(op_idx);
        op_idx
    }

    /// Add a local maximum polygon (close a polygon)
    /// Direct port from clipper.engine.cpp AddLocalMaxPoly (line 1380)
    fn add_local_max_poly(&mut self, e1_idx: usize, e2_idx: usize, pt: Point64) -> Option<usize> {
        if is_joined(&self.active_arena[e1_idx]) {
            self.split(e1_idx, pt);
        }
        if is_joined(&self.active_arena[e2_idx]) {
            self.split(e2_idx, pt);
        }

        if self.is_front(e1_idx) == self.is_front(e2_idx) {
            if self.is_open_end_idx(e1_idx) {
                let or_idx = self.active_arena[e1_idx].outrec.unwrap();
                swap_front_back_sides(or_idx, &mut self.outrec_list, &self.outpt_arena);
            } else if self.is_open_end_idx(e2_idx) {
                let or_idx = self.active_arena[e2_idx].outrec.unwrap();
                swap_front_back_sides(or_idx, &mut self.outrec_list, &self.outpt_arena);
            } else {
                self.succeeded = false;
                return None;
            }
        }

        let result = self.add_out_pt(e1_idx, pt);
        let or1 = self.active_arena[e1_idx].outrec;
        let or2 = self.active_arena[e2_idx].outrec;

        if or1 == or2 {
            let or_idx = or1.unwrap();
            self.outrec_list[or_idx].pts = Some(result);

            if self.using_polytree {
                let prev_hot = self.get_prev_hot_edge(e1_idx);
                if prev_hot.is_none() {
                    self.outrec_list[or_idx].owner = None;
                } else if let Some(prev_hot_idx) = prev_hot {
                    if let Some(prev_or) = self.active_arena[prev_hot_idx].outrec {
                        set_owner(&mut self.outrec_list, or_idx, prev_or);
                    }
                }
            }

            uncouple_outrec(e1_idx, &mut self.active_arena, &mut self.outrec_list);
            let final_result = self.outrec_list[or_idx].pts;
            if let Some(owner) = self.outrec_list[or_idx].owner {
                if self.outrec_list[owner].front_edge.is_none() {
                    self.outrec_list[or_idx].owner = get_real_outrec(&self.outrec_list, owner);
                }
            }
            return final_result;
        } else if self.is_open_idx(e1_idx) {
            if self.active_arena[e1_idx].wind_dx < 0 {
                self.join_outrec_paths(e1_idx, e2_idx);
            } else {
                self.join_outrec_paths(e2_idx, e1_idx);
            }
        } else {
            let or1_idx = or1.unwrap();
            let or2_idx = or2.unwrap();
            if self.outrec_list[or1_idx].idx < self.outrec_list[or2_idx].idx {
                self.join_outrec_paths(e1_idx, e2_idx);
            } else {
                self.join_outrec_paths(e2_idx, e1_idx);
            }
        }
        Some(result)
    }

    /// Join two outrec paths
    /// Direct port from clipper.engine.cpp JoinOutrecPaths (line 1435)
    fn join_outrec_paths(&mut self, e1_idx: usize, e2_idx: usize) {
        let or1 = self.active_arena[e1_idx].outrec.unwrap();
        let or2 = self.active_arena[e2_idx].outrec.unwrap();

        let p1_st = self.outrec_list[or1].pts.unwrap();
        let p2_st = self.outrec_list[or2].pts.unwrap();
        let p1_end = self.outpt_arena[p1_st].next;
        let p2_end = self.outpt_arena[p2_st].next;

        if self.is_front(e1_idx) {
            self.outpt_arena[p2_end].prev = p1_st;
            self.outpt_arena[p1_st].next = p2_end;
            self.outpt_arena[p2_st].next = p1_end;
            self.outpt_arena[p1_end].prev = p2_st;
            self.outrec_list[or1].pts = Some(p2_st);
            self.outrec_list[or1].front_edge = self.outrec_list[or2].front_edge;
            if let Some(fe) = self.outrec_list[or1].front_edge {
                self.active_arena[fe].outrec = Some(or1);
            }
        } else {
            self.outpt_arena[p1_end].prev = p2_st;
            self.outpt_arena[p2_st].next = p1_end;
            self.outpt_arena[p1_st].next = p2_end;
            self.outpt_arena[p2_end].prev = p1_st;
            self.outrec_list[or1].back_edge = self.outrec_list[or2].back_edge;
            if let Some(be) = self.outrec_list[or1].back_edge {
                self.active_arena[be].outrec = Some(or1);
            }
        }

        self.outrec_list[or2].front_edge = None;
        self.outrec_list[or2].back_edge = None;
        self.outrec_list[or2].pts = None;

        if self.is_open_end_idx(e1_idx) {
            self.outrec_list[or2].pts = self.outrec_list[or1].pts;
            self.outrec_list[or1].pts = None;
        } else {
            set_owner(&mut self.outrec_list, or2, or1);
        }

        self.active_arena[e1_idx].outrec = None;
        self.active_arena[e2_idx].outrec = None;
    }

    /// Split joined edges
    /// Direct port from clipper.engine.cpp Split (line 2796)
    fn split(&mut self, e_idx: usize, pt: Point64) {
        if self.active_arena[e_idx].join_with == JoinWith::Right {
            self.active_arena[e_idx].join_with = JoinWith::NoJoin;
            let next = self.active_arena[e_idx].next_in_ael.unwrap();
            self.active_arena[next].join_with = JoinWith::NoJoin;
            self.add_local_min_poly(e_idx, next, pt, true);
        } else {
            self.active_arena[e_idx].join_with = JoinWith::NoJoin;
            let prev = self.active_arena[e_idx].prev_in_ael.unwrap();
            self.active_arena[prev].join_with = JoinWith::NoJoin;
            self.add_local_min_poly(prev, e_idx, pt, true);
        }
    }

    // ---- AEL management ----

    /// Delete an active from the AEL
    /// Direct port from clipper.engine.cpp DeleteFromAEL (line 2099)
    fn delete_from_ael(&mut self, e_idx: usize) {
        let prev = self.active_arena[e_idx].prev_in_ael;
        let next = self.active_arena[e_idx].next_in_ael;
        if prev.is_none() && next.is_none() && self.actives != Some(e_idx) {
            return; // already deleted
        }
        if let Some(prev_idx) = prev {
            self.active_arena[prev_idx].next_in_ael = next;
        } else {
            self.actives = next;
        }
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = prev;
        }
        // Mark as unlinked
        self.active_arena[e_idx].prev_in_ael = None;
        self.active_arena[e_idx].next_in_ael = None;
    }

    /// Swap positions of two adjacent active edges in the AEL
    /// Direct port from clipper.engine.cpp SwapPositionsInAEL (line 2482)
    fn swap_positions_in_ael(&mut self, e1_idx: usize, e2_idx: usize) {
        // precondition: e1 must be immediately to the left of e2
        let next = self.active_arena[e2_idx].next_in_ael;
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = Some(e1_idx);
        }
        let prev = self.active_arena[e1_idx].prev_in_ael;
        if let Some(prev_idx) = prev {
            self.active_arena[prev_idx].next_in_ael = Some(e2_idx);
        }
        self.active_arena[e2_idx].prev_in_ael = prev;
        self.active_arena[e2_idx].next_in_ael = Some(e1_idx);
        self.active_arena[e1_idx].prev_in_ael = Some(e2_idx);
        self.active_arena[e1_idx].next_in_ael = next;
        if self.active_arena[e2_idx].prev_in_ael.is_none() {
            self.actives = Some(e2_idx);
        }
    }

    /// Copy AEL to SEL and update curr_x for top_y
    /// Direct port from clipper.engine.cpp AdjustCurrXAndCopyToSEL (line 2113)
    fn adjust_curr_x_and_copy_to_sel(&mut self, top_y: i64) {
        let mut e_opt = self.actives;
        self.sel = e_opt;
        while let Some(e_idx) = e_opt {
            self.active_arena[e_idx].prev_in_sel = self.active_arena[e_idx].prev_in_ael;
            self.active_arena[e_idx].next_in_sel = self.active_arena[e_idx].next_in_ael;
            self.active_arena[e_idx].jump = self.active_arena[e_idx].next_in_sel;
            self.active_arena[e_idx].curr_x = top_x(&self.active_arena[e_idx], top_y);
            e_opt = self.active_arena[e_idx].next_in_ael;
        }
    }

    // ---- Trim horizontal ----

    /// Trim horizontal edge
    /// Direct port from clipper.engine.cpp TrimHorz (line 1719)
    fn trim_horz(&mut self, e_idx: usize) {
        let mut was_trimmed = false;
        let nv = self.next_vertex_idx(e_idx);
        let mut pt = self.vertex_arena[nv].pt;

        while pt.y == self.active_arena[e_idx].top.y {
            if self.preserve_collinear
                && (pt.x < self.active_arena[e_idx].top.x)
                    != (self.active_arena[e_idx].bot.x < self.active_arena[e_idx].top.x)
            {
                break;
            }

            self.active_arena[e_idx].vertex_top = self.next_vertex_idx(e_idx);
            self.active_arena[e_idx].top = pt;
            was_trimmed = true;
            if self.is_maxima_idx(e_idx) {
                break;
            }
            let nv2 = self.next_vertex_idx(e_idx);
            pt = self.vertex_arena[nv2].pt;
        }

        if was_trimmed {
            set_dx(&mut self.active_arena[e_idx]);
        }
    }

    /// Update edge into AEL (advance to next segment)
    /// Direct port from clipper.engine.cpp UpdateEdgeIntoAEL (line 1742)
    fn update_edge_into_ael(&mut self, e_idx: usize) {
        self.active_arena[e_idx].bot = self.active_arena[e_idx].top;
        self.active_arena[e_idx].vertex_top = self.next_vertex_idx(e_idx);
        self.active_arena[e_idx].top = self.vertex_arena[self.active_arena[e_idx].vertex_top].pt;
        self.active_arena[e_idx].curr_x = self.active_arena[e_idx].bot.x;
        set_dx(&mut self.active_arena[e_idx]);

        if is_joined(&self.active_arena[e_idx]) {
            let bot = self.active_arena[e_idx].bot;
            self.split(e_idx, bot);
        }

        if self.is_horizontal_idx(e_idx) {
            if !self.is_open_idx(e_idx) {
                self.trim_horz(e_idx);
            }
            return;
        }

        let top_y = self.active_arena[e_idx].top.y;
        self.insert_scanline(top_y);
        let bot = self.active_arena[e_idx].bot;
        self.check_join_left(e_idx, bot, false);
        self.check_join_right(e_idx, bot, true);
    }

    /// Find edge with matching local min
    /// Direct port from clipper.engine.cpp FindEdgeWithMatchingLocMin (line 1763)
    fn find_edge_with_matching_loc_min(&self, e_idx: usize) -> Option<usize> {
        let local_min = self.active_arena[e_idx].local_min;

        // Search forward
        let mut result = self.active_arena[e_idx].next_in_ael;
        while let Some(r_idx) = result {
            if self.active_arena[r_idx].local_min == local_min {
                return Some(r_idx);
            }
            if !self.is_horizontal_idx(r_idx)
                && self.active_arena[e_idx].bot != self.active_arena[r_idx].bot
            {
                result = None;
                break;
            }
            result = self.active_arena[r_idx].next_in_ael;
        }

        // Search backward
        if result.is_none() {
            result = self.active_arena[e_idx].prev_in_ael;
            while let Some(r_idx) = result {
                if self.active_arena[r_idx].local_min == local_min {
                    return Some(r_idx);
                }
                if !self.is_horizontal_idx(r_idx)
                    && self.active_arena[e_idx].bot != self.active_arena[r_idx].bot
                {
                    return None;
                }
                result = self.active_arena[r_idx].prev_in_ael;
            }
        }
        result
    }

    // ---- Check Join ----

    /// Check join left
    /// Direct port from clipper.engine.cpp CheckJoinLeft (line 2812)
    fn check_join_left(&mut self, e_idx: usize, pt: Point64, check_curr_x: bool) {
        let prev = self.active_arena[e_idx].prev_in_ael;
        let prev_idx = match prev {
            Some(p) => p,
            None => return,
        };

        if !is_hot_edge(&self.active_arena[e_idx])
            || !is_hot_edge(&self.active_arena[prev_idx])
            || self.is_horizontal_idx(e_idx)
            || self.is_horizontal_idx(prev_idx)
            || self.is_open_idx(e_idx)
            || self.is_open_idx(prev_idx)
        {
            return;
        }

        let e_top = self.active_arena[e_idx].top;
        let p_top = self.active_arena[prev_idx].top;
        if (pt.y < e_top.y + 2 || pt.y < p_top.y + 2)
            && (self.active_arena[e_idx].bot.y > pt.y || self.active_arena[prev_idx].bot.y > pt.y)
        {
            return;
        }

        if check_curr_x {
            if perpendic_dist_from_line_sqrd(
                pt,
                self.active_arena[prev_idx].bot,
                self.active_arena[prev_idx].top,
            ) > 0.25
            {
                return;
            }
        } else if self.active_arena[e_idx].curr_x != self.active_arena[prev_idx].curr_x {
            return;
        }

        if !is_collinear(e_top, pt, p_top) {
            return;
        }

        let e_or = self.active_arena[e_idx].outrec.unwrap();
        let p_or = self.active_arena[prev_idx].outrec.unwrap();

        if self.outrec_list[e_or].idx == self.outrec_list[p_or].idx {
            self.add_local_max_poly(prev_idx, e_idx, pt);
        } else if self.outrec_list[e_or].idx < self.outrec_list[p_or].idx {
            self.join_outrec_paths(e_idx, prev_idx);
        } else {
            self.join_outrec_paths(prev_idx, e_idx);
        }
        self.active_arena[prev_idx].join_with = JoinWith::Right;
        self.active_arena[e_idx].join_with = JoinWith::Left;
    }

    /// Check join right
    /// Direct port from clipper.engine.cpp CheckJoinRight (line 2840)
    fn check_join_right(&mut self, e_idx: usize, pt: Point64, check_curr_x: bool) {
        let next = self.active_arena[e_idx].next_in_ael;
        let next_idx = match next {
            Some(n) => n,
            None => return,
        };

        if !is_hot_edge(&self.active_arena[e_idx])
            || !is_hot_edge(&self.active_arena[next_idx])
            || self.is_horizontal_idx(e_idx)
            || self.is_horizontal_idx(next_idx)
            || self.is_open_idx(e_idx)
            || self.is_open_idx(next_idx)
        {
            return;
        }

        let e_top = self.active_arena[e_idx].top;
        let n_top = self.active_arena[next_idx].top;
        if (pt.y < e_top.y + 2 || pt.y < n_top.y + 2)
            && (self.active_arena[e_idx].bot.y > pt.y || self.active_arena[next_idx].bot.y > pt.y)
        {
            return;
        }

        if check_curr_x {
            if perpendic_dist_from_line_sqrd(
                pt,
                self.active_arena[next_idx].bot,
                self.active_arena[next_idx].top,
            ) > 0.35
            {
                return;
            }
        } else if self.active_arena[e_idx].curr_x != self.active_arena[next_idx].curr_x {
            return;
        }

        if !is_collinear(e_top, pt, n_top) {
            return;
        }

        let e_or = self.active_arena[e_idx].outrec.unwrap();
        let n_or = self.active_arena[next_idx].outrec.unwrap();

        if self.outrec_list[e_or].idx == self.outrec_list[n_or].idx {
            self.add_local_max_poly(e_idx, next_idx, pt);
        } else if self.outrec_list[e_or].idx < self.outrec_list[n_or].idx {
            self.join_outrec_paths(e_idx, next_idx);
        } else {
            self.join_outrec_paths(next_idx, e_idx);
        }

        self.active_arena[e_idx].join_with = JoinWith::Right;
        self.active_arena[next_idx].join_with = JoinWith::Left;
    }

    // ---- Intersect edges ----

    /// Process intersection between two edges
    /// Direct port from clipper.engine.cpp IntersectEdges (line 1783)
    fn intersect_edges(&mut self, e1_idx: usize, e2_idx: usize, pt: Point64) {
        // MANAGE OPEN PATH INTERSECTIONS SEPARATELY
        if self.has_open_paths && (self.is_open_idx(e1_idx) || self.is_open_idx(e2_idx)) {
            if self.is_open_idx(e1_idx) && self.is_open_idx(e2_idx) {
                return;
            }
            let (edge_o, edge_c) = if self.is_open_idx(e1_idx) {
                (e1_idx, e2_idx)
            } else {
                (e2_idx, e1_idx)
            };

            if is_joined(&self.active_arena[edge_c]) {
                self.split(edge_c, pt);
            }

            if self.active_arena[edge_c].wind_cnt.abs() != 1 {
                return;
            }

            match self.cliptype {
                ClipType::Union => {
                    if !is_hot_edge(&self.active_arena[edge_c]) {
                        return;
                    }
                }
                _ => {
                    if self.minima_list[self.active_arena[edge_c].local_min].polytype
                        == PathType::Subject
                    {
                        return;
                    }
                }
            }

            match self.fillrule {
                FillRule::Positive => {
                    if self.active_arena[edge_c].wind_cnt != 1 {
                        return;
                    }
                }
                FillRule::Negative => {
                    if self.active_arena[edge_c].wind_cnt != -1 {
                        return;
                    }
                }
                _ => {
                    if self.active_arena[edge_c].wind_cnt.abs() != 1 {
                        return;
                    }
                }
            }

            // toggle contribution
            if is_hot_edge(&self.active_arena[edge_o]) {
                self.add_out_pt(edge_o, pt);
                if self.is_front(edge_o) {
                    let or = self.active_arena[edge_o].outrec.unwrap();
                    self.outrec_list[or].front_edge = None;
                } else {
                    let or = self.active_arena[edge_o].outrec.unwrap();
                    self.outrec_list[or].back_edge = None;
                }
                self.active_arena[edge_o].outrec = None;
            } else if pt == self.active_arena[edge_o].bot
                && pt
                    == self.vertex_arena
                        [self.minima_list[self.active_arena[edge_o].local_min].vertex]
                        .pt
                && !is_open_end_vertex(
                    &self.vertex_arena
                        [self.minima_list[self.active_arena[edge_o].local_min].vertex],
                )
            {
                let e3 = self.find_edge_with_matching_loc_min(edge_o);
                if let Some(e3_idx) = e3 {
                    if is_hot_edge(&self.active_arena[e3_idx]) {
                        let e3_or = self.active_arena[e3_idx].outrec.unwrap();
                        self.active_arena[edge_o].outrec = Some(e3_or);
                        if self.active_arena[edge_o].wind_dx > 0 {
                            self.set_sides(e3_or, edge_o, e3_idx);
                        } else {
                            self.set_sides(e3_or, e3_idx, edge_o);
                        }
                        return;
                    }
                }
                self.start_open_path(edge_o, pt);
            } else {
                self.start_open_path(edge_o, pt);
            }
            return;
        }

        // MANAGING CLOSED PATHS FROM HERE ON
        if is_joined(&self.active_arena[e1_idx]) {
            self.split(e1_idx, pt);
        }
        if is_joined(&self.active_arena[e2_idx]) {
            self.split(e2_idx, pt);
        }

        // UPDATE WINDING COUNTS
        let old_e1_windcnt;
        let old_e2_windcnt;

        if self.is_same_poly_type_idx(e1_idx, e2_idx) {
            if self.fillrule == FillRule::EvenOdd {
                let tmp = self.active_arena[e1_idx].wind_cnt;
                self.active_arena[e1_idx].wind_cnt = self.active_arena[e2_idx].wind_cnt;
                self.active_arena[e2_idx].wind_cnt = tmp;
            } else {
                let e1_wc = self.active_arena[e1_idx].wind_cnt;
                let e2_wd = self.active_arena[e2_idx].wind_dx;
                let e1_wd = self.active_arena[e1_idx].wind_dx;
                if e1_wc + e2_wd == 0 {
                    self.active_arena[e1_idx].wind_cnt = -e1_wc;
                } else {
                    self.active_arena[e1_idx].wind_cnt = e1_wc + e2_wd;
                }
                let e2_wc = self.active_arena[e2_idx].wind_cnt;
                if e2_wc - e1_wd == 0 {
                    self.active_arena[e2_idx].wind_cnt = -e2_wc;
                } else {
                    self.active_arena[e2_idx].wind_cnt = e2_wc - e1_wd;
                }
            }
        } else if self.fillrule != FillRule::EvenOdd {
            self.active_arena[e1_idx].wind_cnt2 += self.active_arena[e2_idx].wind_dx;
            self.active_arena[e2_idx].wind_cnt2 -= self.active_arena[e1_idx].wind_dx;
        } else {
            let wc2_1 = self.active_arena[e1_idx].wind_cnt2;
            self.active_arena[e1_idx].wind_cnt2 = if wc2_1 == 0 { 1 } else { 0 };
            let wc2_2 = self.active_arena[e2_idx].wind_cnt2;
            self.active_arena[e2_idx].wind_cnt2 = if wc2_2 == 0 { 1 } else { 0 };
        }

        let fillpos = FillRule::Positive;
        match self.fillrule {
            FillRule::EvenOdd | FillRule::NonZero => {
                old_e1_windcnt = self.active_arena[e1_idx].wind_cnt.abs();
                old_e2_windcnt = self.active_arena[e2_idx].wind_cnt.abs();
            }
            _ => {
                if self.fillrule == fillpos {
                    old_e1_windcnt = self.active_arena[e1_idx].wind_cnt;
                    old_e2_windcnt = self.active_arena[e2_idx].wind_cnt;
                } else {
                    old_e1_windcnt = -self.active_arena[e1_idx].wind_cnt;
                    old_e2_windcnt = -self.active_arena[e2_idx].wind_cnt;
                }
            }
        }

        let e1_wc_in_01 = old_e1_windcnt == 0 || old_e1_windcnt == 1;
        let e2_wc_in_01 = old_e2_windcnt == 0 || old_e2_windcnt == 1;

        if (!is_hot_edge(&self.active_arena[e1_idx]) && !e1_wc_in_01)
            || (!is_hot_edge(&self.active_arena[e2_idx]) && !e2_wc_in_01)
        {
            return;
        }

        // NOW PROCESS THE INTERSECTION
        if is_hot_edge(&self.active_arena[e1_idx]) && is_hot_edge(&self.active_arena[e2_idx]) {
            if (old_e1_windcnt != 0 && old_e1_windcnt != 1)
                || (old_e2_windcnt != 0 && old_e2_windcnt != 1)
                || (!self.is_same_poly_type_idx(e1_idx, e2_idx) && self.cliptype != ClipType::Xor)
            {
                self.add_local_max_poly(e1_idx, e2_idx, pt);
            } else if self.is_front(e1_idx)
                || self.active_arena[e1_idx].outrec == self.active_arena[e2_idx].outrec
            {
                self.add_local_max_poly(e1_idx, e2_idx, pt);
                self.add_local_min_poly(e1_idx, e2_idx, pt, false);
            } else {
                self.add_out_pt(e1_idx, pt);
                self.add_out_pt(e2_idx, pt);
                self.swap_outrecs(e1_idx, e2_idx);
            }
        } else if is_hot_edge(&self.active_arena[e1_idx]) {
            self.add_out_pt(e1_idx, pt);
            self.swap_outrecs(e1_idx, e2_idx);
        } else if is_hot_edge(&self.active_arena[e2_idx]) {
            self.add_out_pt(e2_idx, pt);
            self.swap_outrecs(e1_idx, e2_idx);
        } else {
            // neither edge is hot
            let e1wc2;
            let e2wc2;
            match self.fillrule {
                FillRule::EvenOdd | FillRule::NonZero => {
                    e1wc2 = self.active_arena[e1_idx].wind_cnt2.abs() as i64;
                    e2wc2 = self.active_arena[e2_idx].wind_cnt2.abs() as i64;
                }
                _ => {
                    if self.fillrule == fillpos {
                        e1wc2 = self.active_arena[e1_idx].wind_cnt2 as i64;
                        e2wc2 = self.active_arena[e2_idx].wind_cnt2 as i64;
                    } else {
                        e1wc2 = -(self.active_arena[e1_idx].wind_cnt2 as i64);
                        e2wc2 = -(self.active_arena[e2_idx].wind_cnt2 as i64);
                    }
                }
            }

            if !self.is_same_poly_type_idx(e1_idx, e2_idx) {
                self.add_local_min_poly(e1_idx, e2_idx, pt, false);
            } else if old_e1_windcnt == 1 && old_e2_windcnt == 1 {
                match self.cliptype {
                    ClipType::Union => {
                        if e1wc2 <= 0 && e2wc2 <= 0 {
                            self.add_local_min_poly(e1_idx, e2_idx, pt, false);
                        }
                    }
                    ClipType::Difference => {
                        if (self.get_poly_type_idx(e1_idx) == PathType::Clip
                            && e1wc2 > 0
                            && e2wc2 > 0)
                            || (self.get_poly_type_idx(e1_idx) == PathType::Subject
                                && e1wc2 <= 0
                                && e2wc2 <= 0)
                        {
                            self.add_local_min_poly(e1_idx, e2_idx, pt, false);
                        }
                    }
                    ClipType::Xor => {
                        self.add_local_min_poly(e1_idx, e2_idx, pt, false);
                    }
                    _ => {
                        // Intersection
                        if e1wc2 > 0 && e2wc2 > 0 {
                            self.add_local_min_poly(e1_idx, e2_idx, pt, false);
                        }
                    }
                }
            }
        }
    }

    // ---- Insert local minima into AEL ----

    /// Insert local minima into AEL at a given y
    /// Direct port from clipper.engine.cpp InsertLocalMinimaIntoAEL (line 1198)
    fn insert_local_minima_into_ael(&mut self, bot_y: i64) {
        while let Some(loc_min_idx) = self.pop_local_minima(bot_y) {
            let vert_idx = self.minima_list[loc_min_idx].vertex;
            let vert_flags = self.vertex_arena[vert_idx].flags;
            let vert_pt = self.vertex_arena[vert_idx].pt;

            // Create left bound if not open start
            let left_bound_opt = if (vert_flags & VertexFlags::OPEN_START) != VertexFlags::EMPTY {
                None
            } else {
                let lb_idx = self.active_arena.len();
                let mut lb = Active::new();
                lb.bot = vert_pt;
                lb.curr_x = vert_pt.x;
                lb.wind_dx = -1;
                lb.vertex_top = self.vertex_arena[vert_idx].prev; // descending
                lb.top = self.vertex_arena[lb.vertex_top].pt;
                lb.local_min = loc_min_idx;
                set_dx(&mut lb);
                self.active_arena.push(lb);
                Some(lb_idx)
            };

            // Create right bound if not open end
            let right_bound_opt = if (vert_flags & VertexFlags::OPEN_END) != VertexFlags::EMPTY {
                None
            } else {
                let rb_idx = self.active_arena.len();
                let mut rb = Active::new();
                rb.bot = vert_pt;
                rb.curr_x = vert_pt.x;
                rb.wind_dx = 1;
                rb.vertex_top = self.vertex_arena[vert_idx].next; // ascending
                rb.top = self.vertex_arena[rb.vertex_top].pt;
                rb.local_min = loc_min_idx;
                set_dx(&mut rb);
                self.active_arena.push(rb);
                Some(rb_idx)
            };

            // Determine which is actually left and which is right
            let (mut left_bound, mut right_bound) = (left_bound_opt, right_bound_opt);

            if let (Some(lb), Some(rb)) = (left_bound, right_bound) {
                if self.is_horizontal_idx(lb) {
                    if is_heading_right_horz(&self.active_arena[lb]) {
                        std::mem::swap(&mut left_bound, &mut right_bound);
                    }
                } else if self.is_horizontal_idx(rb) {
                    if is_heading_left_horz(&self.active_arena[rb]) {
                        std::mem::swap(&mut left_bound, &mut right_bound);
                    }
                } else if self.active_arena[lb].dx < self.active_arena[rb].dx {
                    std::mem::swap(&mut left_bound, &mut right_bound);
                }
            } else if left_bound.is_none() {
                left_bound = right_bound;
                right_bound = None;
            }

            let lb_idx = left_bound.unwrap();
            self.active_arena[lb_idx].is_left_bound = true;
            self.insert_left_edge(lb_idx);

            let contributing = if self.is_open_idx(lb_idx) {
                self.set_wind_count_for_open_path_edge(lb_idx);
                self.is_contributing_open(lb_idx)
            } else {
                self.set_wind_count_for_closed_path_edge(lb_idx);
                self.is_contributing_closed(lb_idx)
            };

            if let Some(rb_idx) = right_bound {
                self.active_arena[rb_idx].is_left_bound = false;
                self.active_arena[rb_idx].wind_cnt = self.active_arena[lb_idx].wind_cnt;
                self.active_arena[rb_idx].wind_cnt2 = self.active_arena[lb_idx].wind_cnt2;
                self.insert_right_edge(lb_idx, rb_idx);

                if contributing {
                    let bot = self.active_arena[lb_idx].bot;
                    self.add_local_min_poly(lb_idx, rb_idx, bot, true);
                    if !self.is_horizontal_idx(lb_idx) {
                        let bot = self.active_arena[lb_idx].bot;
                        self.check_join_left(lb_idx, bot, false);
                    }
                }

                // Process any right-bound edge intersections
                while let Some(next) = self.active_arena[rb_idx].next_in_ael {
                    if !is_valid_ael_order(
                        &self.active_arena[next],
                        &self.active_arena[rb_idx],
                        &self.vertex_arena,
                        &self.minima_list,
                    ) {
                        break;
                    }
                    let bot = self.active_arena[rb_idx].bot;
                    self.intersect_edges(rb_idx, next, bot);
                    self.swap_positions_in_ael(rb_idx, next);
                }

                if self.is_horizontal_idx(rb_idx) {
                    self.push_horz(rb_idx);
                } else {
                    let bot = self.active_arena[rb_idx].bot;
                    self.check_join_right(rb_idx, bot, false);
                    let top_y = self.active_arena[rb_idx].top.y;
                    self.insert_scanline(top_y);
                }
            } else if contributing {
                let bot = self.active_arena[lb_idx].bot;
                self.start_open_path(lb_idx, bot);
            }

            if self.is_horizontal_idx(lb_idx) {
                self.push_horz(lb_idx);
            } else {
                let top_y = self.active_arena[lb_idx].top.y;
                self.insert_scanline(top_y);
            }
        }
    }

    // ---- Horizontal edge processing ----

    /// Reset horizontal direction for a horizontal edge
    /// Direct port from clipper.engine.cpp ResetHorzDirection (line 2511)
    fn reset_horz_direction(&self, horz_idx: usize, max_vertex: Option<usize>) -> (i64, i64, bool) {
        let horz = &self.active_arena[horz_idx];
        if horz.bot.x == horz.top.x {
            let horz_left = horz.curr_x;
            let horz_right = horz.curr_x;
            let mut e = horz.next_in_ael;
            while let Some(e_idx) = e {
                if Some(self.active_arena[e_idx].vertex_top) == max_vertex {
                    return (horz_left, horz_right, true);
                }
                e = self.active_arena[e_idx].next_in_ael;
            }
            (horz_left, horz_right, false)
        } else if horz.curr_x < horz.top.x {
            (horz.curr_x, horz.top.x, true)
        } else {
            (horz.top.x, horz.curr_x, false)
        }
    }

    /// Process a horizontal edge
    /// Direct port from clipper.engine.cpp DoHorizontal (line 2537)
    fn do_horizontal(&mut self, horz_idx: usize) {
        let horz_is_open = self.is_open_idx(horz_idx);
        let y = self.active_arena[horz_idx].bot.y;

        let vertex_max = if horz_is_open {
            get_curr_y_maxima_vertex_open(&self.active_arena[horz_idx], &self.vertex_arena)
        } else {
            get_curr_y_maxima_vertex(&self.active_arena[horz_idx], &self.vertex_arena)
        };

        let (mut horz_left, mut horz_right, mut is_left_to_right) =
            self.reset_horz_direction(horz_idx, vertex_max);

        if is_hot_edge(&self.active_arena[horz_idx]) {
            let curr_x = self.active_arena[horz_idx].curr_x;
            let op = self.add_out_pt(horz_idx, Point64::new(curr_x, y));
            let or = self.outpt_arena[op].outrec;
            if !self.outrec_list[or].is_open {
                self.add_trial_horz_join(op);
            }
        }

        loop {
            let e_start = if is_left_to_right {
                self.active_arena[horz_idx].next_in_ael
            } else {
                self.active_arena[horz_idx].prev_in_ael
            };

            let mut e_opt = e_start;
            while let Some(e_idx) = e_opt {
                // Check if we've reached the maxima vertex pair
                if Some(self.active_arena[e_idx].vertex_top) == vertex_max {
                    if is_hot_edge(&self.active_arena[horz_idx])
                        && is_joined(&self.active_arena[e_idx])
                    {
                        let top = self.active_arena[e_idx].top;
                        self.split(e_idx, top);
                    }

                    if is_hot_edge(&self.active_arena[horz_idx]) {
                        while Some(self.active_arena[horz_idx].vertex_top) != vertex_max {
                            let top = self.active_arena[horz_idx].top;
                            self.add_out_pt(horz_idx, top);
                            self.update_edge_into_ael(horz_idx);
                        }
                        if is_left_to_right {
                            let top = self.active_arena[horz_idx].top;
                            self.add_local_max_poly(horz_idx, e_idx, top);
                        } else {
                            let top = self.active_arena[horz_idx].top;
                            self.add_local_max_poly(e_idx, horz_idx, top);
                        }
                    }
                    self.delete_from_ael(e_idx);
                    self.delete_from_ael(horz_idx);
                    return;
                }

                // Check break conditions for non-maxima
                if vertex_max != Some(self.active_arena[horz_idx].vertex_top)
                    || self.is_open_end_idx(horz_idx)
                {
                    if (is_left_to_right && self.active_arena[e_idx].curr_x > horz_right)
                        || (!is_left_to_right && self.active_arena[e_idx].curr_x < horz_left)
                    {
                        break;
                    }

                    if self.active_arena[e_idx].curr_x == self.active_arena[horz_idx].top.x
                        && !self.is_horizontal_idx(e_idx)
                    {
                        let nv_idx = self.next_vertex_idx(horz_idx);
                        let nv_pt = self.vertex_arena[nv_idx].pt;

                        if is_left_to_right {
                            if self.is_open_idx(e_idx)
                                && !self.is_same_poly_type_idx(e_idx, horz_idx)
                                && !is_hot_edge(&self.active_arena[e_idx])
                            {
                                if top_x(&self.active_arena[e_idx], nv_pt.y) > nv_pt.x {
                                    break;
                                }
                            } else if top_x(&self.active_arena[e_idx], nv_pt.y) >= nv_pt.x {
                                break;
                            }
                        } else if self.is_open_idx(e_idx)
                            && !self.is_same_poly_type_idx(e_idx, horz_idx)
                            && !is_hot_edge(&self.active_arena[e_idx])
                        {
                            if top_x(&self.active_arena[e_idx], nv_pt.y) < nv_pt.x {
                                break;
                            }
                        } else if top_x(&self.active_arena[e_idx], nv_pt.y) <= nv_pt.x {
                            break;
                        }
                    }
                }

                let pt = Point64::new(
                    self.active_arena[e_idx].curr_x,
                    self.active_arena[horz_idx].bot.y,
                );
                if is_left_to_right {
                    self.intersect_edges(horz_idx, e_idx, pt);
                    self.swap_positions_in_ael(horz_idx, e_idx);
                    let pt2 = pt;
                    self.check_join_left(e_idx, pt2, false);
                    self.active_arena[horz_idx].curr_x = self.active_arena[e_idx].curr_x;
                    e_opt = self.active_arena[horz_idx].next_in_ael;
                } else {
                    self.intersect_edges(e_idx, horz_idx, pt);
                    self.swap_positions_in_ael(e_idx, horz_idx);
                    let pt2 = pt;
                    self.check_join_right(e_idx, pt2, false);
                    self.active_arena[horz_idx].curr_x = self.active_arena[e_idx].curr_x;
                    e_opt = self.active_arena[horz_idx].prev_in_ael;
                }

                if self.active_arena[horz_idx].outrec.is_some() {
                    if let Some(last_op) = get_last_op(
                        horz_idx,
                        &self.active_arena,
                        &self.outrec_list,
                        &self.outpt_arena,
                    ) {
                        self.add_trial_horz_join(last_op);
                    }
                }
            }

            // Check if finished with consecutive horizontals
            if horz_is_open && self.is_open_end_idx(horz_idx) {
                if is_hot_edge(&self.active_arena[horz_idx]) {
                    let top = self.active_arena[horz_idx].top;
                    self.add_out_pt(horz_idx, top);
                    if self.is_front(horz_idx) {
                        let or = self.active_arena[horz_idx].outrec.unwrap();
                        self.outrec_list[or].front_edge = None;
                    } else {
                        let or = self.active_arena[horz_idx].outrec.unwrap();
                        self.outrec_list[or].back_edge = None;
                    }
                    self.active_arena[horz_idx].outrec = None;
                }
                self.delete_from_ael(horz_idx);
                return;
            }

            let nv_idx = self.next_vertex_idx(horz_idx);
            if self.vertex_arena[nv_idx].pt.y != self.active_arena[horz_idx].top.y {
                break;
            }

            // Still more horizontals in bound
            if is_hot_edge(&self.active_arena[horz_idx]) {
                let top = self.active_arena[horz_idx].top;
                self.add_out_pt(horz_idx, top);
            }
            self.update_edge_into_ael(horz_idx);

            let result = self.reset_horz_direction(horz_idx, vertex_max);
            horz_left = result.0;
            horz_right = result.1;
            is_left_to_right = result.2;
        }

        if is_hot_edge(&self.active_arena[horz_idx]) {
            let top = self.active_arena[horz_idx].top;
            let op = self.add_out_pt(horz_idx, top);
            self.add_trial_horz_join(op);
        }

        self.update_edge_into_ael(horz_idx);
    }

    // ---- Intersection list processing ----

    /// Add a new intersect node
    /// Direct port from clipper.engine.cpp AddNewIntersectNode (line 2356)
    fn add_new_intersect_node(&mut self, e1_idx: usize, e2_idx: usize, top_y: i64) {
        let e1 = &self.active_arena[e1_idx];
        let e2 = &self.active_arena[e2_idx];
        let mut ip = Point64::new(e1.curr_x, top_y);
        get_segment_intersect_pt(e1.bot, e1.top, e2.bot, e2.top, &mut ip);

        if ip.y > self.bot_y || ip.y < top_y {
            let abs_dx1 = e1.dx.abs();
            let abs_dx2 = e2.dx.abs();
            if abs_dx1 > 100.0 && abs_dx2 > 100.0 {
                if abs_dx1 > abs_dx2 {
                    ip = get_closest_point_on_segment(ip, e1.bot, e1.top);
                } else {
                    ip = get_closest_point_on_segment(ip, e2.bot, e2.top);
                }
            } else if abs_dx1 > 100.0 {
                ip = get_closest_point_on_segment(ip, e1.bot, e1.top);
            } else if abs_dx2 > 100.0 {
                ip = get_closest_point_on_segment(ip, e2.bot, e2.top);
            } else {
                if ip.y < top_y {
                    ip.y = top_y;
                } else {
                    ip.y = self.bot_y;
                }
                if abs_dx1 < abs_dx2 {
                    ip.x = top_x(&self.active_arena[e1_idx], ip.y);
                } else {
                    ip.x = top_x(&self.active_arena[e2_idx], ip.y);
                }
            }
        }
        self.intersect_nodes
            .push(IntersectNode::with_edges(e1_idx, e2_idx, ip));
    }

    /// Build the intersection list
    /// Direct port from clipper.engine.cpp BuildIntersectList (line 2390)
    fn build_intersect_list(&mut self, top_y: i64) -> bool {
        if self.actives.is_none() {
            return false;
        }
        let actives_idx = self.actives.unwrap();
        if self.active_arena[actives_idx].next_in_ael.is_none() {
            return false;
        }

        self.adjust_curr_x_and_copy_to_sel(top_y);

        let mut left_opt = self.sel;
        // Check if we have a jump
        if left_opt.is_none() || self.active_arena[left_opt.unwrap()].jump.is_none() {
            return !self.intersect_nodes.is_empty();
        }

        while let Some(left_idx) = left_opt {
            if self.active_arena[left_idx].jump.is_none() {
                break;
            }

            let mut prev_base: Option<usize> = None;
            let mut left_inner = left_opt;

            while let Some(li) = left_inner {
                if self.active_arena[li].jump.is_none() {
                    break;
                }

                let curr_base = li;
                let right_idx = self.active_arena[li].jump.unwrap();
                let mut l_end = right_idx;
                let r_end = self.active_arena[right_idx].jump;
                self.active_arena[li].jump = r_end;

                let mut left_scan = li;
                let mut right_scan = right_idx;

                while left_scan != l_end && right_scan != r_end.unwrap_or(NONE) {
                    if self.active_arena[right_scan].curr_x < self.active_arena[left_scan].curr_x {
                        let mut tmp = self.active_arena[right_scan].prev_in_sel;
                        loop {
                            let tmp_idx = match tmp {
                                Some(idx) => idx,
                                None => break, // Safety: SEL list ended before finding left_scan
                            };
                            self.add_new_intersect_node(tmp_idx, right_scan, top_y);
                            if tmp_idx == left_scan {
                                break;
                            }
                            tmp = self.active_arena[tmp_idx].prev_in_sel;
                        }

                        let tmp_idx = right_scan;
                        right_scan =
                            extract_from_sel(tmp_idx, &mut self.active_arena).unwrap_or(NONE);
                        l_end = right_scan;

                        insert1_before2_in_sel(tmp_idx, left_scan, &mut self.active_arena);
                        if left_scan == curr_base {
                            let new_curr_base = tmp_idx;
                            self.active_arena[new_curr_base].jump = r_end;
                            if let Some(pb) = prev_base {
                                self.active_arena[pb].jump = Some(new_curr_base);
                            } else {
                                self.sel = Some(new_curr_base);
                            }
                            // update left_inner tracking: curr_base changed
                        }
                    } else {
                        left_scan = self.active_arena[left_scan].next_in_sel.unwrap_or(NONE);
                    }
                }

                prev_base = Some(curr_base);
                left_inner = r_end;
            }

            left_opt = self.sel;
        }

        !self.intersect_nodes.is_empty()
    }

    /// Do intersections
    /// Direct port from clipper.engine.cpp DoIntersections (line 2347)
    fn do_intersections(&mut self, top_y: i64) {
        if self.build_intersect_list(top_y) {
            self.process_intersect_list();
            self.intersect_nodes.clear();
        }
    }

    /// Process the intersection list
    /// Direct port from clipper.engine.cpp ProcessIntersectList (line 2448)
    fn process_intersect_list(&mut self) {
        // Sort by y descending (bottom up), then x ascending
        self.intersect_nodes.sort_by(intersect_list_sort);

        let len = self.intersect_nodes.len();
        for i in 0..len {
            // Ensure edges are adjacent
            if !edges_adjacent_in_ael(&self.intersect_nodes[i], &self.active_arena) {
                let mut j = i + 1;
                while j < len
                    && !edges_adjacent_in_ael(&self.intersect_nodes[j], &self.active_arena)
                {
                    j += 1;
                }
                if j < len {
                    self.intersect_nodes.swap(i, j);
                }
            }

            let e1 = self.intersect_nodes[i].edge1;
            let e2 = self.intersect_nodes[i].edge2;
            let pt = self.intersect_nodes[i].pt;

            self.intersect_edges(e1, e2, pt);
            self.swap_positions_in_ael(e1, e2);

            self.active_arena[e1].curr_x = pt.x;
            self.active_arena[e2].curr_x = pt.x;
            self.check_join_left(e2, pt, true);
            self.check_join_right(e1, pt, true);
        }
    }

    // ---- Top of scanbeam ----

    /// Process the top of a scanbeam
    /// Direct port from clipper.engine.cpp DoTopOfScanbeam (line 2708)
    fn do_top_of_scanbeam(&mut self, y: i64) {
        self.sel = None;
        let mut e_opt = self.actives;
        while let Some(e_idx) = e_opt {
            if self.active_arena[e_idx].top.y == y {
                self.active_arena[e_idx].curr_x = self.active_arena[e_idx].top.x;
                if self.is_maxima_idx(e_idx) {
                    e_opt = self.do_maxima(e_idx);
                    continue;
                } else {
                    if is_hot_edge(&self.active_arena[e_idx]) {
                        let top = self.active_arena[e_idx].top;
                        self.add_out_pt(e_idx, top);
                    }
                    self.update_edge_into_ael(e_idx);
                    if self.is_horizontal_idx(e_idx) {
                        self.push_horz(e_idx);
                    }
                }
            } else {
                self.active_arena[e_idx].curr_x = top_x(&self.active_arena[e_idx], y);
            }
            e_opt = self.active_arena[e_idx].next_in_ael;
        }
    }

    /// Process a maxima edge
    /// Direct port from clipper.engine.cpp DoMaxima (line 2740)
    fn do_maxima(&mut self, e_idx: usize) -> Option<usize> {
        let prev_e = self.active_arena[e_idx].prev_in_ael;
        let mut next_e = self.active_arena[e_idx].next_in_ael;

        if self.is_open_end_idx(e_idx) {
            if is_hot_edge(&self.active_arena[e_idx]) {
                let top = self.active_arena[e_idx].top;
                self.add_out_pt(e_idx, top);
            }
            if !self.is_horizontal_idx(e_idx) {
                if is_hot_edge(&self.active_arena[e_idx]) {
                    if self.is_front(e_idx) {
                        let or = self.active_arena[e_idx].outrec.unwrap();
                        self.outrec_list[or].front_edge = None;
                    } else {
                        let or = self.active_arena[e_idx].outrec.unwrap();
                        self.outrec_list[or].back_edge = None;
                    }
                    self.active_arena[e_idx].outrec = None;
                }
                self.delete_from_ael(e_idx);
            }
            return next_e;
        }

        let max_pair = get_maxima_pair(&self.active_arena[e_idx], &self.active_arena);
        if max_pair.is_none() {
            return next_e;
        }
        let max_pair_idx = max_pair.unwrap();

        if is_joined(&self.active_arena[e_idx]) {
            let top = self.active_arena[e_idx].top;
            self.split(e_idx, top);
        }
        if is_joined(&self.active_arena[max_pair_idx]) {
            let top = self.active_arena[max_pair_idx].top;
            self.split(max_pair_idx, top);
        }

        // Process edges between maxima pair
        while next_e != Some(max_pair_idx) {
            if let Some(ne_idx) = next_e {
                let top = self.active_arena[e_idx].top;
                self.intersect_edges(e_idx, ne_idx, top);
                self.swap_positions_in_ael(e_idx, ne_idx);
                next_e = self.active_arena[e_idx].next_in_ael;
            } else {
                break;
            }
        }

        if self.is_open_idx(e_idx) {
            if is_hot_edge(&self.active_arena[e_idx]) {
                let top = self.active_arena[e_idx].top;
                self.add_local_max_poly(e_idx, max_pair_idx, top);
            }
            self.delete_from_ael(max_pair_idx);
            self.delete_from_ael(e_idx);
            return if let Some(pe) = prev_e {
                self.active_arena[pe].next_in_ael
            } else {
                self.actives
            };
        }

        // non-open maxima
        if is_hot_edge(&self.active_arena[e_idx]) {
            let top = self.active_arena[e_idx].top;
            self.add_local_max_poly(e_idx, max_pair_idx, top);
        }

        self.delete_from_ael(e_idx);
        self.delete_from_ael(max_pair_idx);
        if let Some(pe) = prev_e {
            self.active_arena[pe].next_in_ael
        } else {
            self.actives
        }
    }

    // ---- Horizontal join processing ----

    /// Set horizontal segment heading forward
    #[allow(dead_code)]
    fn set_horz_seg_heading_forward(&mut self, hs_idx: usize, op_p: usize, op_n: usize) -> bool {
        if self.outpt_arena[op_p].pt.x == self.outpt_arena[op_n].pt.x {
            return false;
        }
        if self.outpt_arena[op_p].pt.x < self.outpt_arena[op_n].pt.x {
            self.horz_seg_list[hs_idx].left_op = Some(op_p);
            self.horz_seg_list[hs_idx].right_op = Some(op_n);
            self.horz_seg_list[hs_idx].left_to_right = true;
        } else {
            self.horz_seg_list[hs_idx].left_op = Some(op_n);
            self.horz_seg_list[hs_idx].right_op = Some(op_p);
            self.horz_seg_list[hs_idx].left_to_right = false;
        }
        true
    }

    /// Convert horizontal segments to joins
    /// Direct port from clipper.engine.cpp ConvertHorzSegsToJoins (line 2218)
    fn convert_horz_segs_to_joins(&mut self) {
        // Update horizontal segments
        let mut valid_count = 0;
        for i in 0..self.horz_seg_list.len() {
            let op = match self.horz_seg_list[i].left_op {
                Some(op) => op,
                None => continue,
            };
            let or_idx = self.outpt_arena[op].outrec;
            let real_or = get_real_outrec(&self.outrec_list, or_idx);
            if real_or.is_none() {
                continue;
            }
            let real_or_idx = real_or.unwrap();
            let has_edges = self.outrec_list[real_or_idx].front_edge.is_some();
            let curr_y = self.outpt_arena[op].pt.y;

            let mut op_p = op;
            let mut op_n = op;

            if has_edges {
                let op_a = self.outrec_list[real_or_idx].pts.unwrap();
                let op_z = self.outpt_arena[op_a].next;
                while op_p != op_z && self.outpt_arena[self.outpt_arena[op_p].prev].pt.y == curr_y {
                    op_p = self.outpt_arena[op_p].prev;
                }
                while op_n != op_a && self.outpt_arena[self.outpt_arena[op_n].next].pt.y == curr_y {
                    op_n = self.outpt_arena[op_n].next;
                }
            } else {
                while self.outpt_arena[op_p].prev != op_n
                    && self.outpt_arena[self.outpt_arena[op_p].prev].pt.y == curr_y
                {
                    op_p = self.outpt_arena[op_p].prev;
                }
                while self.outpt_arena[op_n].next != op_p
                    && self.outpt_arena[self.outpt_arena[op_n].next].pt.y == curr_y
                {
                    op_n = self.outpt_arena[op_n].next;
                }
            }

            if self.outpt_arena[op_p].pt.x == self.outpt_arena[op_n].pt.x {
                self.horz_seg_list[i].right_op = None;
                continue;
            }

            // Set heading
            if self.outpt_arena[op_p].pt.x < self.outpt_arena[op_n].pt.x {
                self.horz_seg_list[i].left_op = Some(op_p);
                self.horz_seg_list[i].right_op = Some(op_n);
                self.horz_seg_list[i].left_to_right = true;
            } else {
                self.horz_seg_list[i].left_op = Some(op_n);
                self.horz_seg_list[i].right_op = Some(op_p);
                self.horz_seg_list[i].left_to_right = false;
            }

            // Check if left_op already has a horz
            let left_op = self.horz_seg_list[i].left_op.unwrap();
            if self.outpt_arena[left_op].horz.is_some() {
                self.horz_seg_list[i].right_op = None;
                continue;
            }

            self.outpt_arena[left_op].horz = Some(i);
            valid_count += 1;
        }

        if valid_count < 2 {
            return;
        }

        // Sort by left_op x coordinate
        self.horz_seg_list.sort_by(|a, b| {
            let a_valid = a.right_op.is_some();
            let b_valid = b.right_op.is_some();
            if a_valid != b_valid {
                return if a_valid {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            if !a_valid {
                return std::cmp::Ordering::Equal;
            }
            let a_x = self.outpt_arena[a.left_op.unwrap()].pt.x;
            let b_x = self.outpt_arena[b.left_op.unwrap()].pt.x;
            b_x.cmp(&a_x)
        });

        // Find pairs and create joins
        for i in 0..self.horz_seg_list.len() {
            if self.horz_seg_list[i].right_op.is_none() {
                break;
            }
            for j in (i + 1)..self.horz_seg_list.len() {
                if self.horz_seg_list[j].right_op.is_none() {
                    break;
                }

                let hs1_left_x = self.outpt_arena[self.horz_seg_list[i].left_op.unwrap()]
                    .pt
                    .x;
                let hs1_right_x = self.outpt_arena[self.horz_seg_list[i].right_op.unwrap()]
                    .pt
                    .x;
                let hs2_left_x = self.outpt_arena[self.horz_seg_list[j].left_op.unwrap()]
                    .pt
                    .x;
                let hs2_right_x = self.outpt_arena[self.horz_seg_list[j].right_op.unwrap()]
                    .pt
                    .x;

                if hs2_left_x >= hs1_right_x
                    || self.horz_seg_list[j].left_to_right == self.horz_seg_list[i].left_to_right
                    || hs2_right_x <= hs1_left_x
                {
                    continue;
                }

                let curr_y = self.outpt_arena[self.horz_seg_list[i].left_op.unwrap()]
                    .pt
                    .y;
                let hs1_ltr = self.horz_seg_list[i].left_to_right;

                if hs1_ltr {
                    let mut lo1 = self.horz_seg_list[i].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo1].next].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo1].next].pt.x <= hs2_left_x
                    {
                        lo1 = self.outpt_arena[lo1].next;
                    }
                    let mut lo2 = self.horz_seg_list[j].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo2].prev].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo2].prev].pt.x
                            <= self.outpt_arena[lo1].pt.x
                    {
                        lo2 = self.outpt_arena[lo2].prev;
                    }
                    let dup1 = self.duplicate_op(lo1, true);
                    let dup2 = self.duplicate_op(lo2, false);
                    self.horz_join_list.push(HorzJoin::with_ops(dup1, dup2));
                } else {
                    let mut lo1 = self.horz_seg_list[i].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo1].prev].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo1].prev].pt.x <= hs2_left_x
                    {
                        lo1 = self.outpt_arena[lo1].prev;
                    }
                    let mut lo2 = self.horz_seg_list[j].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo2].next].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo2].next].pt.x
                            <= self.outpt_arena[lo1].pt.x
                    {
                        lo2 = self.outpt_arena[lo2].next;
                    }
                    let dup1 = self.duplicate_op(lo2, true);
                    let dup2 = self.duplicate_op(lo1, false);
                    self.horz_join_list.push(HorzJoin::with_ops(dup1, dup2));
                }
            }
        }
    }

    /// Process horizontal joins
    /// Direct port from clipper.engine.cpp ProcessHorzJoins (line 2279)
    fn process_horz_joins(&mut self) {
        for idx in 0..self.horz_join_list.len() {
            let op1 = match self.horz_join_list[idx].op1 {
                Some(op) => op,
                None => continue,
            };
            let op2 = match self.horz_join_list[idx].op2 {
                Some(op) => op,
                None => continue,
            };

            let or1 = get_real_outrec(&self.outrec_list, self.outpt_arena[op1].outrec);
            let or2 = get_real_outrec(&self.outrec_list, self.outpt_arena[op2].outrec);
            if or1.is_none() || or2.is_none() {
                continue;
            }
            let or1 = or1.unwrap();
            let or2_orig = or2.unwrap();

            let op1b = self.outpt_arena[op1].next;
            let op2b = self.outpt_arena[op2].prev;
            self.outpt_arena[op1].next = op2;
            self.outpt_arena[op2].prev = op1;
            self.outpt_arena[op1b].prev = op2b;
            self.outpt_arena[op2b].next = op1b;

            if or1 == or2_orig {
                // 'join' is really a split
                let or2_new = self.new_out_rec();
                self.outrec_list[or2_new].pts = Some(op1b);
                fix_outrec_pts(or2_new, &self.outrec_list, &mut self.outpt_arena);

                if self.outrec_list[or1]
                    .pts
                    .map(|p| self.outpt_arena[p].outrec)
                    == Some(or2_new)
                {
                    self.outrec_list[or1].pts = Some(op1);
                    self.outpt_arena[op1].outrec = or1;
                }

                if self.using_polytree {
                    if path2_contains_path1_outpt(
                        self.outrec_list[or1].pts.unwrap(),
                        self.outrec_list[or2_new].pts.unwrap(),
                        &self.outpt_arena,
                    ) {
                        let or1_pts = self.outrec_list[or1].pts;
                        let or2_pts = self.outrec_list[or2_new].pts;
                        self.outrec_list[or1].pts = or2_pts;
                        self.outrec_list[or2_new].pts = or1_pts;
                        fix_outrec_pts(or1, &self.outrec_list, &mut self.outpt_arena);
                        fix_outrec_pts(or2_new, &self.outrec_list, &mut self.outpt_arena);
                        self.outrec_list[or2_new].owner = Some(or1);
                    } else if path2_contains_path1_outpt(
                        self.outrec_list[or2_new].pts.unwrap(),
                        self.outrec_list[or1].pts.unwrap(),
                        &self.outpt_arena,
                    ) {
                        self.outrec_list[or2_new].owner = Some(or1);
                    } else {
                        self.outrec_list[or2_new].owner = self.outrec_list[or1].owner;
                    }

                    if self.outrec_list[or1].splits.is_none() {
                        self.outrec_list[or1].splits = Some(Vec::new());
                    }
                    self.outrec_list[or1].splits.as_mut().unwrap().push(or2_new);
                } else {
                    self.outrec_list[or2_new].owner = Some(or1);
                }
            } else {
                // joining, not splitting
                self.outrec_list[or2_orig].pts = None;
                if self.using_polytree {
                    set_owner(&mut self.outrec_list, or2_orig, or1);
                    if self.outrec_list[or2_orig].splits.is_some() {
                        move_splits(&mut self.outrec_list, or2_orig, or1);
                    }
                } else {
                    self.outrec_list[or2_orig].owner = Some(or1);
                }
            }
        }
    }

    // ---- Clean collinear and self-intersection fixing ----

    /// Clean collinear points from an OutRec
    /// Direct port from clipper.engine.cpp CleanCollinear (line 1525)
    pub fn clean_collinear(&mut self, outrec_idx: usize) {
        let real_or = get_real_outrec(&self.outrec_list, outrec_idx);
        let or_idx = match real_or {
            Some(idx) => idx,
            None => return,
        };
        if self.outrec_list[or_idx].is_open {
            return;
        }
        if !is_valid_closed_path(self.outrec_list[or_idx].pts, &self.outpt_arena) {
            self.dispose_out_pts(or_idx);
            return;
        }

        let start_op = self.outrec_list[or_idx].pts.unwrap();
        let mut op2 = start_op;
        loop {
            let prev = self.outpt_arena[op2].prev;
            let next = self.outpt_arena[op2].next;
            let prev_pt = self.outpt_arena[prev].pt;
            let op2_pt = self.outpt_arena[op2].pt;
            let next_pt = self.outpt_arena[next].pt;

            if is_collinear(prev_pt, op2_pt, next_pt)
                && (op2_pt == prev_pt
                    || op2_pt == next_pt
                    || !self.preserve_collinear
                    || dot_product_three_points(prev_pt, op2_pt, next_pt) < 0.0)
            {
                if Some(op2) == self.outrec_list[or_idx].pts {
                    self.outrec_list[or_idx].pts = Some(prev);
                }
                op2 = self.dispose_out_pt(op2);
                if !is_valid_closed_path(Some(op2), &self.outpt_arena) {
                    self.dispose_out_pts(or_idx);
                    return;
                }
                // Reset start
                continue;
            }
            op2 = next;
            if op2 == start_op
                || (self.outrec_list[or_idx].pts.is_some()
                    && op2 == self.outrec_list[or_idx].pts.unwrap())
            {
                break;
            }
        }
        self.fix_self_intersects(or_idx);
    }

    /// Fix self-intersections in an OutRec
    /// Direct port from clipper.engine.cpp FixSelfIntersects (line 1646)
    fn fix_self_intersects(&mut self, outrec_idx: usize) {
        let start_op = match self.outrec_list[outrec_idx].pts {
            Some(op) => op,
            None => return,
        };

        let prev = self.outpt_arena[start_op].prev;
        let next = self.outpt_arena[start_op].next;
        if prev == self.outpt_arena[next].next {
            return; // triangles can't self-intersect
        }

        let mut op2 = start_op;
        loop {
            let prev = self.outpt_arena[op2].prev;
            let next = self.outpt_arena[op2].next;
            let next_next = self.outpt_arena[next].next;

            if segments_intersect(
                self.outpt_arena[prev].pt,
                self.outpt_arena[op2].pt,
                self.outpt_arena[next].pt,
                self.outpt_arena[next_next].pt,
                false,
            ) {
                let next_next_next = self.outpt_arena[next_next].next;
                if segments_intersect(
                    self.outpt_arena[prev].pt,
                    self.outpt_arena[op2].pt,
                    self.outpt_arena[next_next].pt,
                    self.outpt_arena[next_next_next].pt,
                    false,
                ) {
                    // Adjacent intersections (micro self-intersection)
                    op2 = self.duplicate_op(op2, false);
                    let nnext = self.outpt_arena[self.outpt_arena[op2].next].next;
                    let nnn = self.outpt_arena[nnext].next;
                    self.outpt_arena[op2].pt = self.outpt_arena[nnn].pt;
                    op2 = self.outpt_arena[op2].next;
                } else {
                    if Some(op2) == self.outrec_list[outrec_idx].pts
                        || Some(next) == self.outrec_list[outrec_idx].pts
                    {
                        let pts = self.outrec_list[outrec_idx].pts.unwrap();
                        self.outrec_list[outrec_idx].pts = Some(self.outpt_arena[pts].prev);
                    }
                    self.do_split_op(outrec_idx, op2);
                    if self.outrec_list[outrec_idx].pts.is_none() {
                        break;
                    }
                    op2 = self.outrec_list[outrec_idx].pts.unwrap();
                    let prev = self.outpt_arena[op2].prev;
                    let next = self.outpt_arena[op2].next;
                    if prev == self.outpt_arena[next].next {
                        break;
                    }
                    continue;
                }
            } else {
                op2 = self.outpt_arena[op2].next;
            }
            if op2 == start_op
                || (self.outrec_list[outrec_idx].pts.is_some()
                    && op2 == self.outrec_list[outrec_idx].pts.unwrap())
            {
                break;
            }
        }
    }

    /// Do split operation on self-intersecting polygon
    /// Direct port from clipper.engine.cpp DoSplitOp (line 1562)
    fn do_split_op(&mut self, outrec_idx: usize, split_op: usize) {
        let prev_op = self.outpt_arena[split_op].prev;
        let next_op = self.outpt_arena[split_op].next;
        let next_next_op = self.outpt_arena[next_op].next;
        self.outrec_list[outrec_idx].pts = Some(prev_op);

        let mut ip = self.outpt_arena[prev_op].pt;
        get_segment_intersect_pt(
            self.outpt_arena[prev_op].pt,
            self.outpt_arena[split_op].pt,
            self.outpt_arena[next_op].pt,
            self.outpt_arena[next_next_op].pt,
            &mut ip,
        );

        let area1 = area_outpt(self.outrec_list[outrec_idx].pts.unwrap(), &self.outpt_arena);
        let abs_area1 = area1.abs();
        if abs_area1 < 2.0 {
            self.dispose_out_pts(outrec_idx);
            return;
        }

        let area2 = area_triangle(
            ip,
            self.outpt_arena[split_op].pt,
            self.outpt_arena[next_op].pt,
        );
        let abs_area2 = area2.abs();

        // De-link split_op and next_op, inserting intersection point
        if ip == self.outpt_arena[prev_op].pt || ip == self.outpt_arena[next_next_op].pt {
            self.outpt_arena[next_next_op].prev = prev_op;
            self.outpt_arena[prev_op].next = next_next_op;
        } else {
            let new_op2 = self.new_out_pt(ip, self.outpt_arena[prev_op].outrec);
            self.outpt_arena[new_op2].prev = prev_op;
            self.outpt_arena[new_op2].next = next_next_op;
            self.outpt_arena[next_next_op].prev = new_op2;
            self.outpt_arena[prev_op].next = new_op2;
        }

        if abs_area2 >= 1.0 && (abs_area2 > abs_area1 || (area2 > 0.0) == (area1 > 0.0)) {
            let new_or = self.new_out_rec();
            self.outrec_list[new_or].owner = self.outrec_list[outrec_idx].owner;

            self.outpt_arena[split_op].outrec = new_or;
            self.outpt_arena[next_op].outrec = new_or;

            let new_op = self.new_out_pt(ip, new_or);
            self.outpt_arena[new_op].prev = next_op;
            self.outpt_arena[new_op].next = split_op;
            self.outrec_list[new_or].pts = Some(new_op);
            self.outpt_arena[split_op].prev = new_op;
            self.outpt_arena[next_op].next = new_op;

            if self.using_polytree {
                if path2_contains_path1_outpt(prev_op, new_op, &self.outpt_arena) {
                    self.outrec_list[new_or].splits = Some(vec![outrec_idx]);
                } else {
                    if self.outrec_list[outrec_idx].splits.is_none() {
                        self.outrec_list[outrec_idx].splits = Some(Vec::new());
                    }
                    self.outrec_list[outrec_idx]
                        .splits
                        .as_mut()
                        .unwrap()
                        .push(new_or);
                }
            }
        }
        // Otherwise the split triangle is too small, just discard split_op and next_op
    }

    // ---- Check bounds and recursive owner checks (for polytree) ----

    /// Check bounds of an outrec
    /// Direct port from clipper.engine.cpp CheckBounds (line 2929)
    pub fn check_bounds(&mut self, outrec_idx: usize) -> bool {
        if self.outrec_list[outrec_idx].pts.is_none() {
            return false;
        }
        if !self.outrec_list[outrec_idx].bounds.is_empty() {
            return true;
        }
        self.clean_collinear(outrec_idx);
        if self.outrec_list[outrec_idx].pts.is_none() {
            return false;
        }

        let op_start = self.outrec_list[outrec_idx].pts.unwrap();
        let path =
            build_path64_from_outpt(op_start, self.reverse_solution, false, &self.outpt_arena);
        match path {
            None => {
                self.outrec_list[outrec_idx].path = Path64::new();
                false
            }
            Some(p) => {
                self.outrec_list[outrec_idx].bounds = get_bounds(&p);
                self.outrec_list[outrec_idx].path = p;
                true
            }
        }
    }

    /// Check if a split outrec should own this outrec
    /// Direct port from clipper.engine.cpp CheckSplitOwner (line 2941)
    fn check_split_owner(&mut self, outrec_idx: usize, splits: &[usize]) -> bool {
        for &split_idx in splits {
            if self.outrec_list[split_idx].pts.is_none() {
                if let Some(ref sub_splits) = self.outrec_list[split_idx].splits.clone() {
                    if self.check_split_owner(outrec_idx, sub_splits) {
                        return true;
                    }
                }
            }
            let real_split = get_real_outrec(&self.outrec_list, split_idx);
            let split = match real_split {
                Some(s) if s != outrec_idx => s,
                _ => continue,
            };

            if self.outrec_list[split].recursive_split == Some(outrec_idx) {
                continue;
            }
            self.outrec_list[split].recursive_split = Some(outrec_idx);

            if let Some(ref sub_splits) = self.outrec_list[split].splits.clone() {
                if self.check_split_owner(outrec_idx, sub_splits) {
                    return true;
                }
            }

            if !self.check_bounds(split) {
                continue;
            }
            let or_bounds = self.outrec_list[outrec_idx].bounds;
            if !self.outrec_list[split].bounds.contains_rect(&or_bounds) {
                continue;
            }

            let or_pts = self.outrec_list[outrec_idx].pts.unwrap();
            let split_pts = self.outrec_list[split].pts.unwrap();
            if !path2_contains_path1_outpt(or_pts, split_pts, &self.outpt_arena) {
                continue;
            }

            if !is_valid_owner(&self.outrec_list, outrec_idx, split) {
                self.outrec_list[split].owner = self.outrec_list[outrec_idx].owner;
            }

            self.outrec_list[outrec_idx].owner = Some(split);
            return true;
        }
        false
    }

    /// Recursively check and set owners for polytree building
    /// Direct port from clipper.engine.cpp RecursiveCheckOwners (line 2967)
    pub fn recursive_check_owners(&mut self, outrec_idx: usize, polytree: &mut PolyTree64) {
        if self.outrec_list[outrec_idx].polypath.is_some()
            || self.outrec_list[outrec_idx].bounds.is_empty()
        {
            return;
        }

        while let Some(owner) = self.outrec_list[outrec_idx].owner {
            if let Some(ref splits) = self.outrec_list[owner].splits.clone() {
                if self.check_split_owner(outrec_idx, splits) {
                    break;
                }
            }
            if self.outrec_list[owner].pts.is_some() && self.check_bounds(owner) {
                let or_bounds = self.outrec_list[outrec_idx].bounds;
                if self.outrec_list[owner].bounds.contains_rect(&or_bounds) {
                    let or_pts = self.outrec_list[outrec_idx].pts.unwrap();
                    let owner_pts = self.outrec_list[owner].pts.unwrap();
                    if path2_contains_path1_outpt(or_pts, owner_pts, &self.outpt_arena) {
                        break;
                    }
                }
            }
            self.outrec_list[outrec_idx].owner = self.outrec_list[owner].owner;
        }

        let path = self.outrec_list[outrec_idx].path.clone();
        if let Some(owner) = self.outrec_list[outrec_idx].owner {
            if self.outrec_list[owner].polypath.is_none() {
                self.recursive_check_owners(owner, polytree);
            }
            let parent_pp = self.outrec_list[owner].polypath.unwrap_or(0);
            let pp = polytree.add_child(parent_pp, path);
            self.outrec_list[outrec_idx].polypath = Some(pp);
        } else {
            let pp = polytree.add_child(0, path);
            self.outrec_list[outrec_idx].polypath = Some(pp);
        }
    }

    // ---- ExecuteInternal ----

    /// Main execution loop of the sweep-line algorithm
    /// Direct port from clipper.engine.cpp ExecuteInternal (line 2129)
    pub fn execute_internal(
        &mut self,
        ct: ClipType,
        fillrule: FillRule,
        use_polytrees: bool,
    ) -> bool {
        self.cliptype = ct;
        self.fillrule = fillrule;
        self.using_polytree = use_polytrees;
        self.reset();

        if ct == ClipType::NoClip {
            return true;
        }

        let y = match self.pop_scanline() {
            Some(y) => y,
            None => return true,
        };

        let mut y = y;
        while self.succeeded {
            self.insert_local_minima_into_ael(y);

            while let Some(e) = self.pop_horz() {
                self.do_horizontal(e);
            }

            if !self.horz_seg_list.is_empty() {
                self.convert_horz_segs_to_joins();
                self.horz_seg_list.clear();
            }

            self.bot_y = y;

            match self.pop_scanline() {
                Some(new_y) => y = new_y,
                None => break,
            }

            self.do_intersections(y);
            self.do_top_of_scanbeam(y);

            while let Some(e) = self.pop_horz() {
                self.do_horizontal(e);
            }
        }

        if self.succeeded {
            self.process_horz_joins();
        }

        self.succeeded
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
