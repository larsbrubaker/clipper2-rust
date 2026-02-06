//! Public polygon clipper API types
//!
//! Contains PolyPath/PolyTree types, Clipper64, and ClipperD.
//! Direct port from clipper.engine.h
//! Copyright (c) Angus Johnson 2010-2025

use crate::core::*;
use crate::engine::*;
use crate::engine_fns::*;

// ============================================================================
// PolyPath - Tree structure for polygon results
// Direct port from clipper.engine.h line 298
// ============================================================================

/// PolyPath64 - Integer coordinate polytree node
/// Direct port from clipper.engine.h line 334
#[derive(Debug, Clone)]
pub struct PolyPath64 {
    parent: Option<usize>, // index into polytree arena
    children: Vec<usize>,  // indices into polytree arena
    pub(crate) polygon: Path64,
}

impl PolyPath64 {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            polygon: Path64::new(),
        }
    }

    pub fn with_parent(parent_idx: usize) -> Self {
        Self {
            parent: Some(parent_idx),
            children: Vec::new(),
            polygon: Path64::new(),
        }
    }

    pub fn with_path(parent_idx: usize, path: Path64) -> Self {
        Self {
            parent: Some(parent_idx),
            children: Vec::new(),
            polygon: path,
        }
    }

    pub fn polygon(&self) -> &Path64 {
        &self.polygon
    }

    pub fn count(&self) -> usize {
        self.children.len()
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn children(&self) -> &[usize] {
        &self.children
    }
}

impl Default for PolyPath64 {
    fn default() -> Self {
        Self::new()
    }
}

/// PolyTree64 arena - stores all PolyPath64 nodes
/// Direct port from clipper.engine.h PolyTree64
pub struct PolyTree64 {
    pub nodes: Vec<PolyPath64>,
}

impl PolyTree64 {
    pub fn new() -> Self {
        // Root node is always at index 0
        let root = PolyPath64::new();
        Self { nodes: vec![root] }
    }

    /// Add a child path to a parent node
    pub fn add_child(&mut self, parent_idx: usize, path: Path64) -> usize {
        let child_idx = self.nodes.len();
        self.nodes.push(PolyPath64::with_path(parent_idx, path));
        self.nodes[parent_idx].children.push(child_idx);
        child_idx
    }

    /// Get the root node
    pub fn root(&self) -> &PolyPath64 {
        &self.nodes[0]
    }

    /// Clear the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(PolyPath64::new());
    }

    /// Get level of a node
    pub fn level(&self, node_idx: usize) -> u32 {
        let mut result = 0u32;
        let mut p = self.nodes[node_idx].parent;
        while let Some(parent_idx) = p {
            result += 1;
            p = self.nodes[parent_idx].parent;
        }
        result
    }

    /// Check if a node is a hole
    pub fn is_hole(&self, node_idx: usize) -> bool {
        let lvl = self.level(node_idx);
        lvl > 0 && (lvl & 1) == 0
    }

    /// Get total area including children
    pub fn area_of(&self, node_idx: usize) -> f64 {
        let mut result = area(&self.nodes[node_idx].polygon);
        for &child_idx in &self.nodes[node_idx].children {
            result += self.area_of(child_idx);
        }
        result
    }
}

impl Default for PolyTree64 {
    fn default() -> Self {
        Self::new()
    }
}

/// PolyPathD - Double coordinate polytree node
/// Direct port from clipper.engine.h line 385
#[derive(Debug, Clone)]
pub struct PolyPathD {
    parent: Option<usize>,
    children: Vec<usize>,
    polygon: PathD,
    pub(crate) scale: f64,
}

impl PolyPathD {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            polygon: PathD::new(),
            scale: 1.0,
        }
    }

    pub fn with_parent(parent_idx: usize, scale: f64) -> Self {
        Self {
            parent: Some(parent_idx),
            children: Vec::new(),
            polygon: PathD::new(),
            scale,
        }
    }

    pub fn polygon(&self) -> &PathD {
        &self.polygon
    }

    pub fn set_polygon(&mut self, polygon: PathD) {
        self.polygon = polygon;
    }

    pub fn count(&self) -> usize {
        self.children.len()
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn children(&self) -> &[usize] {
        &self.children
    }
}

impl Default for PolyPathD {
    fn default() -> Self {
        Self::new()
    }
}

/// PolyTreeD arena
pub struct PolyTreeD {
    pub nodes: Vec<PolyPathD>,
}

impl PolyTreeD {
    pub fn new() -> Self {
        Self {
            nodes: vec![PolyPathD::new()],
        }
    }

    pub fn add_child(&mut self, parent_idx: usize, path: PathD) -> usize {
        let scale = self.nodes[parent_idx].scale;
        let child_idx = self.nodes.len();
        let mut node = PolyPathD::with_parent(parent_idx, scale);
        node.polygon = path;
        self.nodes.push(node);
        self.nodes[parent_idx].children.push(child_idx);
        child_idx
    }

    pub fn add_child_from_path64(&mut self, parent_idx: usize, path: &Path64) -> usize {
        let scale = self.nodes[parent_idx].scale;
        let child_idx = self.nodes.len();
        let mut node = PolyPathD::with_parent(parent_idx, scale);
        let mut error_code = 0;
        node.polygon = scale_path(path, scale, scale, &mut error_code);
        self.nodes.push(node);
        self.nodes[parent_idx].children.push(child_idx);
        child_idx
    }

    pub fn root(&self) -> &PolyPathD {
        &self.nodes[0]
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        let mut root = PolyPathD::new();
        root.scale = 1.0;
        self.nodes.push(root);
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.nodes[0].scale = scale;
    }

    pub fn level(&self, node_idx: usize) -> u32 {
        let mut result = 0u32;
        let mut p = self.nodes[node_idx].parent;
        while let Some(parent_idx) = p {
            result += 1;
            p = self.nodes[parent_idx].parent;
        }
        result
    }

    pub fn is_hole(&self, node_idx: usize) -> bool {
        let lvl = self.level(node_idx);
        lvl > 0 && (lvl & 1) == 0
    }
}

impl Default for PolyTreeD {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Clipper64 - Public int64 clipper
// Direct port from clipper.engine.h line 459
// ============================================================================

/// Main integer-coordinate polygon clipper
/// Direct port from clipper.engine.h line 459
pub struct Clipper64 {
    pub base: ClipperBase,
}

impl Clipper64 {
    pub fn new() -> Self {
        Self {
            base: ClipperBase::new(),
        }
    }

    /// Add subject paths
    pub fn add_subject(&mut self, subjects: &Paths64) {
        self.base.add_paths(subjects, PathType::Subject, false);
    }

    /// Add open subject paths
    pub fn add_open_subject(&mut self, open_subjects: &Paths64) {
        self.base.add_paths(open_subjects, PathType::Subject, true);
    }

    /// Add clip paths
    pub fn add_clip(&mut self, clips: &Paths64) {
        self.base.add_paths(clips, PathType::Clip, false);
    }

    /// Get error code
    pub fn error_code(&self) -> i32 {
        self.base.error_code
    }

    /// Set/get preserve collinear
    pub fn set_preserve_collinear(&mut self, val: bool) {
        self.base.preserve_collinear = val;
    }

    pub fn preserve_collinear(&self) -> bool {
        self.base.preserve_collinear
    }

    /// Set/get reverse solution
    pub fn set_reverse_solution(&mut self, val: bool) {
        self.base.reverse_solution = val;
    }

    pub fn reverse_solution(&self) -> bool {
        self.base.reverse_solution
    }

    /// Clear the clipper
    pub fn clear(&mut self) {
        self.base.clear();
    }

    /// Execute a clipping operation, returning closed and open paths
    /// Direct port from clipper.engine.h Clipper64::Execute
    pub fn execute(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        solution_closed: &mut Paths64,
        mut solution_open: Option<&mut Paths64>,
    ) -> bool {
        solution_closed.clear();
        if let Some(ref mut open) = solution_open {
            open.clear();
        }

        if self.base.execute_internal(clip_type, fill_rule, false) {
            self.build_paths64(solution_closed, solution_open);
        }
        self.base.clean_up();
        self.base.succeeded
    }

    /// Execute a clipping operation, returning a polytree and open paths
    /// Direct port from clipper.engine.h Clipper64::Execute (polytree version)
    pub fn execute_tree(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        polytree: &mut PolyTree64,
        open_paths: &mut Paths64,
    ) -> bool {
        polytree.clear();
        open_paths.clear();

        if self.base.execute_internal(clip_type, fill_rule, true) {
            self.build_tree64(polytree, open_paths);
        }
        self.base.clean_up();
        self.base.succeeded
    }

    /// Build output paths from outrec list
    /// Direct port from clipper.engine.cpp Clipper64::BuildPaths64 (line 2992)
    fn build_paths64(
        &mut self,
        solution_closed: &mut Paths64,
        solution_open: Option<&mut Paths64>,
    ) {
        solution_closed.clear();
        solution_closed.reserve(self.base.outrec_list.len());

        let mut open_paths = Vec::new();

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path64_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                ) {
                    open_paths.push(path);
                }
            } else {
                self.base.clean_collinear(i);
                if self.base.outrec_list[i].pts.is_some() {
                    let op = self.base.outrec_list[i].pts.unwrap();
                    if let Some(path) = build_path64_from_outpt(
                        op,
                        self.base.reverse_solution,
                        false,
                        &self.base.outpt_arena,
                    ) {
                        solution_closed.push(path);
                    }
                }
            }
            i += 1;
        }

        if let Some(open) = solution_open {
            *open = open_paths;
        }
    }

    /// Build polytree output
    /// Direct port from clipper.engine.cpp Clipper64::BuildTree64 (line 3027)
    fn build_tree64(&mut self, polytree: &mut PolyTree64, open_paths: &mut Paths64) {
        polytree.clear();
        open_paths.clear();
        if self.base.has_open_paths {
            open_paths.reserve(self.base.outrec_list.len());
        }

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path64_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                ) {
                    open_paths.push(path);
                }
                i += 1;
                continue;
            }

            if self.base.check_bounds(i) {
                self.base.recursive_check_owners(i, polytree);
            }
            i += 1;
        }
    }
}

impl Default for Clipper64 {
    fn default() -> Self {
        Self::new()
    }
}

/// Double-precision polygon clipper that scales to int64 internally
/// Direct port from clipper.engine.h line 520
pub struct ClipperD {
    pub base: ClipperBase,
    scale: f64,
    inv_scale: f64,
}

impl ClipperD {
    pub fn new(precision: i32) -> Self {
        let mut prec = precision;
        let mut error_code = 0;
        check_precision_range(&mut prec, &mut error_code);

        // Set the scale to a power of double's radix (2)
        let scale = 2.0f64.powi(((10.0f64.powi(prec)).log2().floor() as i32) + 1);
        let inv_scale = 1.0 / scale;

        let mut base = ClipperBase::new();
        base.error_code = error_code;

        Self {
            base,
            scale,
            inv_scale,
        }
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn inv_scale(&self) -> f64 {
        self.inv_scale
    }

    /// Add subject paths (double precision)
    pub fn add_subject(&mut self, subjects: &PathsD) {
        let scaled: Paths64 =
            scale_paths(subjects, self.scale, self.scale, &mut self.base.error_code);
        self.base.add_paths(&scaled, PathType::Subject, false);
    }

    /// Add open subject paths (double precision)
    pub fn add_open_subject(&mut self, open_subjects: &PathsD) {
        let scaled: Paths64 = scale_paths(
            open_subjects,
            self.scale,
            self.scale,
            &mut self.base.error_code,
        );
        self.base.add_paths(&scaled, PathType::Subject, true);
    }

    /// Add clip paths (double precision)
    pub fn add_clip(&mut self, clips: &PathsD) {
        let scaled: Paths64 = scale_paths(clips, self.scale, self.scale, &mut self.base.error_code);
        self.base.add_paths(&scaled, PathType::Clip, false);
    }

    pub fn error_code(&self) -> i32 {
        self.base.error_code
    }

    pub fn set_preserve_collinear(&mut self, val: bool) {
        self.base.preserve_collinear = val;
    }

    pub fn preserve_collinear(&self) -> bool {
        self.base.preserve_collinear
    }

    pub fn set_reverse_solution(&mut self, val: bool) {
        self.base.reverse_solution = val;
    }

    pub fn reverse_solution(&self) -> bool {
        self.base.reverse_solution
    }

    pub fn clear(&mut self) {
        self.base.clear();
    }

    /// Execute a clipping operation with double-precision paths
    /// Direct port from clipper.engine.h ClipperD::Execute
    pub fn execute(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        solution_closed: &mut PathsD,
        mut solution_open: Option<&mut PathsD>,
    ) -> bool {
        solution_closed.clear();
        if let Some(ref mut open) = solution_open {
            open.clear();
        }

        if self.base.execute_internal(clip_type, fill_rule, false) {
            self.build_paths_d(solution_closed, solution_open);
        }
        self.base.clean_up();
        self.base.succeeded
    }

    /// Execute returning polytree with double-precision
    pub fn execute_tree(
        &mut self,
        clip_type: ClipType,
        fill_rule: FillRule,
        polytree: &mut PolyTreeD,
        open_paths: &mut PathsD,
    ) -> bool {
        polytree.clear();
        open_paths.clear();

        if self.base.execute_internal(clip_type, fill_rule, true) {
            self.build_tree_d(polytree, open_paths);
        }
        self.base.clean_up();
        self.base.succeeded
    }

    /// Build output paths for double-precision
    /// Direct port from clipper.engine.cpp ClipperD::BuildPathsD (line 3101)
    fn build_paths_d(&mut self, solution_closed: &mut PathsD, solution_open: Option<&mut PathsD>) {
        solution_closed.clear();
        solution_closed.reserve(self.base.outrec_list.len());

        let mut open_paths = Vec::new();

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path_d_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                    self.inv_scale,
                ) {
                    open_paths.push(path);
                }
            } else {
                self.base.clean_collinear(i);
                if self.base.outrec_list[i].pts.is_some() {
                    let op = self.base.outrec_list[i].pts.unwrap();
                    if let Some(path) = build_path_d_from_outpt(
                        op,
                        self.base.reverse_solution,
                        false,
                        &self.base.outpt_arena,
                        self.inv_scale,
                    ) {
                        solution_closed.push(path);
                    }
                }
            }
            i += 1;
        }

        if let Some(open) = solution_open {
            *open = open_paths;
        }
    }

    /// Build polytree for double-precision
    /// Direct port from clipper.engine.cpp ClipperD::BuildTreeD (line 3135)
    fn build_tree_d(&mut self, polytree: &mut PolyTreeD, open_paths: &mut PathsD) {
        polytree.clear();
        polytree.set_scale(self.inv_scale);
        open_paths.clear();
        if self.base.has_open_paths {
            open_paths.reserve(self.base.outrec_list.len());
        }

        // Build a PolyTree64 internally, then convert to PolyTreeD
        // This matches the C++ approach where RecursiveCheckOwners works with Path64
        let mut polytree64 = PolyTree64::new();

        let mut i = 0;
        while i < self.base.outrec_list.len() {
            if self.base.outrec_list[i].pts.is_none() {
                i += 1;
                continue;
            }

            if self.base.outrec_list[i].is_open {
                let op = self.base.outrec_list[i].pts.unwrap();
                if let Some(path) = build_path_d_from_outpt(
                    op,
                    self.base.reverse_solution,
                    true,
                    &self.base.outpt_arena,
                    self.inv_scale,
                ) {
                    open_paths.push(path);
                }
                i += 1;
                continue;
            }

            if self.base.check_bounds(i) {
                self.base.recursive_check_owners(i, &mut polytree64);
            }
            i += 1;
        }

        // Convert PolyTree64 to PolyTreeD by scaling all paths
        self.convert_polytree64_to_d(&polytree64, polytree);
    }

    /// Convert a PolyTree64 to PolyTreeD by scaling all paths
    fn convert_polytree64_to_d(&self, src: &PolyTree64, dst: &mut PolyTreeD) {
        dst.clear();
        dst.set_scale(self.inv_scale);
        // Recursively convert nodes, skipping the root (index 0)
        let root_children = src.nodes[0].children.clone();
        for &child_idx in &root_children {
            Self::convert_polypath64_node(src, child_idx, 0, dst);
        }
    }

    fn convert_polypath64_node(
        src: &PolyTree64,
        src_idx: usize,
        dst_parent: usize,
        dst: &mut PolyTreeD,
    ) {
        let src_node = &src.nodes[src_idx];
        let dst_idx = dst.add_child_from_path64(dst_parent, &src_node.polygon);
        let children = src_node.children.clone();
        for &child_idx in &children {
            Self::convert_polypath64_node(src, child_idx, dst_idx, dst);
        }
    }
}
