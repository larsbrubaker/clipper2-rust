// Copyright 2025 - Clipper2 Rust port
// Direct port of clipper.svg.h / clipper.svg.cpp by Angus Johnson
// Original Copyright: Angus Johnson 2010-2024
// License: https://www.boost.org/LICENSE_1_0.txt
//
// Purpose: SVG writer and reader for path visualization

use crate::core::{
    scale_path, scale_paths, transform_paths, Path64, PathD, Paths64, PathsD, PointD, RectD,
};
use crate::FillRule;
use std::fs;
use std::io::Write;

// ============================================================================
// SVG constants
// ============================================================================

/// Default colors for SVG visualization (from clipper.svg.utils.h)
pub const SUBJ_BRUSH_CLR: u32 = 0x1800009C;
pub const SUBJ_STROKE_CLR: u32 = 0xFFB3B3DA;
pub const CLIP_BRUSH_CLR: u32 = 0x129C0000;
pub const CLIP_STROKE_CLR: u32 = 0xCCFFA07A;
pub const SOLUTION_BRUSH_CLR: u32 = 0x4466FF66;

// ============================================================================
// Helper functions
// ============================================================================

/// Convert a u32 ARGB color to an HTML hex string (#RRGGBB).
/// Direct port from C++ `ColorToHtml()`.
pub fn color_to_html(clr: u32) -> String {
    format!("#{:06x}", clr & 0xFFFFFF)
}

/// Extract the alpha channel as a fraction (0.0 to 1.0).
/// Direct port from C++ `GetAlphaAsFrac()`.
pub fn get_alpha_as_frac(clr: u32) -> f32 {
    (clr >> 24) as f32 / 255.0
}

// ============================================================================
// PathInfo - stores path data with rendering attributes
// ============================================================================

/// Stores a set of paths with their rendering attributes.
/// Direct port from C++ `PathInfo` class.
#[derive(Debug, Clone)]
pub struct PathInfo {
    pub paths: PathsD,
    pub is_open_path: bool,
    pub fillrule: FillRule,
    pub brush_color: u32,
    pub pen_color: u32,
    pub pen_width: f64,
    pub show_coords: bool,
}

impl PathInfo {
    pub fn new(
        paths: PathsD,
        is_open: bool,
        fillrule: FillRule,
        brush_color: u32,
        pen_color: u32,
        pen_width: f64,
        show_coords: bool,
    ) -> Self {
        Self {
            paths,
            is_open_path: is_open,
            fillrule,
            brush_color,
            pen_color,
            pen_width,
            show_coords,
        }
    }
}

// ============================================================================
// TextInfo - stores text label data
// ============================================================================

/// Stores a text label with its rendering attributes.
/// Direct port from C++ `SvgWriter::TextInfo` class.
#[derive(Debug, Clone)]
pub struct TextInfo {
    pub text: String,
    pub font_name: String,
    pub font_color: u32,
    pub font_weight: u32,
    pub font_size: u32,
    pub x: f64,
    pub y: f64,
}

impl TextInfo {
    pub fn new(
        text: &str,
        font_name: &str,
        font_color: u32,
        font_weight: u32,
        font_size: u32,
        x: f64,
        y: f64,
    ) -> Self {
        Self {
            text: text.to_string(),
            font_name: font_name.to_string(),
            font_color,
            font_weight,
            font_size,
            x,
            y,
        }
    }
}

// ============================================================================
// CoordsStyle - styling for coordinate display
// ============================================================================

#[derive(Debug, Clone)]
struct CoordsStyle {
    font_name: String,
    font_color: u32,
    font_size: u32,
}

impl Default for CoordsStyle {
    fn default() -> Self {
        Self {
            font_name: "Verdana".to_string(),
            font_color: 0xFF000000,
            font_size: 11,
        }
    }
}

// ============================================================================
// SvgWriter
// ============================================================================

/// SVG file writer for visualizing clipper paths.
///
/// Direct port from C++ `SvgWriter` class.
/// Add paths and text, then save to an SVG file.
///
/// # Examples
///
/// ```
/// use clipper2_rust::utils::svg::SvgWriter;
/// use clipper2_rust::core::FillRule;
///
/// let mut svg = SvgWriter::new(0);
/// // svg.add_paths_64(&paths, false, FillRule::NonZero, 0x1800009C, 0xFFB3B3DA, 0.8, false);
/// // svg.save_to_file("output.svg", 800, 600, 20);
/// ```
pub struct SvgWriter {
    scale: f64,
    fill_rule: FillRule,
    coords_style: CoordsStyle,
    text_infos: Vec<TextInfo>,
    path_infos: Vec<PathInfo>,
}

impl SvgWriter {
    /// Create a new SvgWriter with the given precision (decimal places).
    /// Direct port from C++ `SvgWriter(int precision)`.
    pub fn new(precision: i32) -> Self {
        Self {
            scale: 10.0_f64.powi(precision),
            fill_rule: FillRule::NonZero,
            coords_style: CoordsStyle::default(),
            text_infos: Vec::new(),
            path_infos: Vec::new(),
        }
    }

    /// Clear all stored paths and text.
    pub fn clear(&mut self) {
        self.path_infos.clear();
        self.text_infos.clear();
    }

    /// Get the current fill rule.
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    /// Set the coordinate display style.
    /// Direct port from C++ `SetCoordsStyle()`.
    pub fn set_coords_style(&mut self, font_name: &str, font_color: u32, font_size: u32) {
        self.coords_style.font_name = font_name.to_string();
        self.coords_style.font_color = font_color;
        self.coords_style.font_size = font_size;
    }

    /// Add a text label at the given position.
    /// Direct port from C++ `AddText()`.
    pub fn add_text(&mut self, text: &str, font_color: u32, font_size: u32, x: f64, y: f64) {
        self.text_infos
            .push(TextInfo::new(text, "", font_color, 600, font_size, x, y));
    }

    /// Add a single Path64, scaling by the writer's precision.
    /// Direct port from C++ `AddPath(const Path64&, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_path_64(
        &mut self,
        path: &Path64,
        is_open: bool,
        fillrule: FillRule,
        brush_color: u32,
        pen_color: u32,
        pen_width: f64,
        show_coords: bool,
    ) {
        if path.is_empty() {
            return;
        }
        let mut error_code = 0;
        let scaled: PathD = scale_path::<f64, i64>(path, self.scale, self.scale, &mut error_code);
        if error_code != 0 {
            return;
        }
        self.path_infos.push(PathInfo::new(
            vec![scaled],
            is_open,
            fillrule,
            brush_color,
            pen_color,
            pen_width,
            show_coords,
        ));
    }

    /// Add a single PathD.
    /// Direct port from C++ `AddPath(const PathD&, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_path_d(
        &mut self,
        path: &PathD,
        is_open: bool,
        fillrule: FillRule,
        brush_color: u32,
        pen_color: u32,
        pen_width: f64,
        show_coords: bool,
    ) {
        if path.is_empty() {
            return;
        }
        self.path_infos.push(PathInfo::new(
            vec![path.clone()],
            is_open,
            fillrule,
            brush_color,
            pen_color,
            pen_width,
            show_coords,
        ));
    }

    /// Add multiple Paths64, scaling by the writer's precision.
    /// Direct port from C++ `AddPaths(const Paths64&, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_paths_64(
        &mut self,
        paths: &Paths64,
        is_open: bool,
        fillrule: FillRule,
        brush_color: u32,
        pen_color: u32,
        pen_width: f64,
        show_coords: bool,
    ) {
        if paths.is_empty() {
            return;
        }
        let mut error_code = 0;
        let scaled: PathsD =
            scale_paths::<f64, i64>(paths, self.scale, self.scale, &mut error_code);
        if error_code != 0 {
            return;
        }
        self.path_infos.push(PathInfo::new(
            scaled,
            is_open,
            fillrule,
            brush_color,
            pen_color,
            pen_width,
            show_coords,
        ));
    }

    /// Add multiple PathsD.
    /// Direct port from C++ `AddPaths(const PathsD&, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_paths_d(
        &mut self,
        paths: &PathsD,
        is_open: bool,
        fillrule: FillRule,
        brush_color: u32,
        pen_color: u32,
        pen_width: f64,
        show_coords: bool,
    ) {
        if paths.is_empty() {
            return;
        }
        self.path_infos.push(PathInfo::new(
            paths.clone(),
            is_open,
            fillrule,
            brush_color,
            pen_color,
            pen_width,
            show_coords,
        ));
    }

    /// Save all stored paths and text to an SVG file.
    /// Direct port from C++ `SaveToFile()`.
    ///
    /// Returns true on success, false on failure.
    pub fn save_to_file(
        &self,
        filename: &str,
        max_width: i32,
        max_height: i32,
        margin: i32,
    ) -> bool {
        // Compute bounding rect of all paths
        let mut rec = RectD {
            left: f64::MAX,
            top: f64::MAX,
            right: f64::MIN,
            bottom: f64::MIN,
        };
        for pi in &self.path_infos {
            for path in &pi.paths {
                for pt in path {
                    if pt.x < rec.left {
                        rec.left = pt.x;
                    }
                    if pt.x > rec.right {
                        rec.right = pt.x;
                    }
                    if pt.y < rec.top {
                        rec.top = pt.y;
                    }
                    if pt.y > rec.bottom {
                        rec.bottom = pt.y;
                    }
                }
            }
        }
        if rec.left >= rec.right || rec.top >= rec.bottom {
            return false;
        }

        let margin = margin.max(20);
        let max_width = max_width.max(100);
        let max_height = max_height.max(100);

        let rec_width = rec.right - rec.left;
        let rec_height = rec.bottom - rec.top;
        let scale = ((max_width - margin * 2) as f64 / rec_width)
            .min((max_height - margin * 2) as f64 / rec_height);

        rec.left *= scale;
        rec.top *= scale;
        rec.right *= scale;
        rec.bottom *= scale;
        let offset_x = margin as f64 - rec.left;
        let offset_y = margin as f64 - rec.top;

        let file = fs::File::create(filename);
        let mut file = match file {
            Ok(f) => f,
            Err(_) => return false,
        };

        // SVG header
        let header = format!(
            "<?xml version=\"1.0\" standalone=\"no\"?>\n\
             <!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\"\n\
             \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n\n\
             <svg width=\"{}px\" height=\"{}px\" viewBox=\"0 0 {} {}\" \
             version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n\n",
            max_width, max_height, max_width, max_height
        );
        if write!(file, "{}", header).is_err() {
            return false;
        }

        // First pass: render Positive/Negative fill rule paths with simulated fill
        // (Skipped in Rust port as it requires calling Union which is complex and
        // SVG only supports EvenOdd/NonZero natively. Paths with Positive/Negative
        // fill rules will be rendered normally.)

        // Main path rendering
        for pi in &self.path_infos {
            let brush_color = pi.brush_color;

            let _ = write!(file, "  <path d=\"");
            for path in &pi.paths {
                if path.len() < 2 || (path.len() == 2 && !pi.is_open_path) {
                    continue;
                }
                let _ = write!(
                    file,
                    " M {:.2} {:.2}",
                    path[0].x * scale + offset_x,
                    path[0].y * scale + offset_y
                );
                for pt in path {
                    let _ = write!(
                        file,
                        " L {:.2} {:.2}",
                        pt.x * scale + offset_x,
                        pt.y * scale + offset_y
                    );
                }
                if !pi.is_open_path {
                    let _ = write!(file, " z");
                }
            }

            let fill_rule_str = if pi.fillrule == FillRule::NonZero {
                "nonzero"
            } else {
                "evenodd"
            };

            let _ = write!(
                file,
                "\"\n    style=\"fill:{}; fill-opacity:{:.2}; fill-rule:{}; \
                 stroke:{}; stroke-opacity:{:.2}; stroke-width:{:.1};\"/>\n",
                color_to_html(brush_color),
                get_alpha_as_frac(brush_color),
                fill_rule_str,
                color_to_html(pi.pen_color),
                get_alpha_as_frac(pi.pen_color),
                pi.pen_width
            );

            // Coordinate display
            if pi.show_coords {
                let _ = writeln!(
                    file,
                    "  <g font-family=\"{}\" font-size=\"{}\" fill=\"{}\" fill-opacity=\"{:.2}\">",
                    self.coords_style.font_name,
                    self.coords_style.font_size,
                    color_to_html(self.coords_style.font_color),
                    get_alpha_as_frac(self.coords_style.font_color)
                );
                for path in &pi.paths {
                    if path.len() < 2 || (path.len() == 2 && !pi.is_open_path) {
                        continue;
                    }
                    for pt in path {
                        let _ = writeln!(
                            file,
                            "    <text x=\"{}\" y=\"{}\">{:.0},{:.0}</text>",
                            (pt.x * scale + offset_x) as i64,
                            (pt.y * scale + offset_y) as i64,
                            pt.x,
                            pt.y
                        );
                    }
                }
                let _ = writeln!(file, "  </g>\n");
            }
        }

        // Text labels
        for ti in &self.text_infos {
            let _ = writeln!(
                file,
                "  <g font-family=\"{}\" font-size=\"{}\" fill=\"{}\" fill-opacity=\"{:.2}\">",
                if ti.font_name.is_empty() {
                    "Verdana"
                } else {
                    &ti.font_name
                },
                ti.font_size,
                color_to_html(ti.font_color),
                get_alpha_as_frac(ti.font_color)
            );
            let _ = writeln!(
                file,
                "    <text x=\"{}\" y=\"{}\">{}</text>\n  </g>\n",
                (ti.x * scale + offset_x) as i64,
                (ti.y * scale + offset_y) as i64,
                ti.text
            );
        }

        let _ = writeln!(file, "</svg>");
        true
    }
}

// ============================================================================
// SvgReader
// ============================================================================

/// SVG file reader that extracts path data from SVG files.
///
/// Direct port from C++ `SvgReader` class.
/// Parses SVG `<path>` elements and extracts their coordinates.
pub struct SvgReader {
    pub xml: String,
    path_infos: Vec<PathInfo>,
}

impl SvgReader {
    pub fn new() -> Self {
        Self {
            xml: String::new(),
            path_infos: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.path_infos.clear();
    }

    /// Load and parse an SVG file. Returns true if paths were found.
    /// Direct port from C++ `LoadFromFile()`.
    pub fn load_from_file(&mut self, filename: &str) -> bool {
        self.clear();
        let content = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.xml = content;
        self.parse_paths();
        !self.path_infos.is_empty()
    }

    /// Parse all `<path>` elements from the loaded XML.
    fn parse_paths(&mut self) {
        let xml = self.xml.clone();
        let mut pos = 0;
        while let Some(path_start) = xml[pos..].find("<path") {
            let abs_start = pos + path_start + 5;
            if let Some(path_end) = xml[abs_start..].find("/>") {
                let abs_end = abs_start + path_end;
                let element = &xml[abs_start..abs_end];
                self.load_path(element);
                pos = abs_end + 2;
            } else {
                break;
            }
        }
    }

    /// Parse a single path element's `d` attribute.
    /// Direct port from C++ `LoadPath()`.
    fn load_path(&mut self, element: &str) -> bool {
        let d_attr = match element.find("d=\"") {
            Some(pos) => &element[pos + 3..],
            None => return false,
        };

        let d_end = match d_attr.find('"') {
            Some(pos) => pos,
            None => return false,
        };
        let d_value = &d_attr[..d_end];

        let mut paths: PathsD = Vec::new();
        let mut current_path: PathD = Vec::new();
        let mut x: f64;
        let mut y: f64;
        let mut command;
        let mut is_relative;

        let chars: Vec<char> = d_value.chars().collect();
        let mut i = 0;

        // Skip leading whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // Expect 'M' or 'm' as first command
        if i >= chars.len() {
            return false;
        }
        if chars[i] == 'M' {
            is_relative = false;
            i += 1;
        } else if chars[i] == 'm' {
            is_relative = true;
            i += 1;
        } else {
            return false;
        }
        command = 'M';

        // Read initial x,y
        if let Some((val, next)) = parse_number(&chars, i) {
            x = val;
            i = next;
        } else {
            return false;
        }
        if let Some((val, next)) = parse_number(&chars, i) {
            y = val;
            i = next;
        } else {
            return false;
        }
        current_path.push(PointD::new(x, y));

        // Process remaining path data
        while i < chars.len() {
            // Skip whitespace
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= chars.len() {
                break;
            }

            // Check for command letter
            if chars[i].is_ascii_alphabetic() {
                let ch = chars[i];
                match ch.to_ascii_uppercase() {
                    'L' | 'M' => {
                        command = ch.to_ascii_uppercase();
                        is_relative = ch.is_ascii_lowercase();
                        i += 1;
                    }
                    'H' => {
                        command = 'H';
                        is_relative = ch.is_ascii_lowercase();
                        i += 1;
                    }
                    'V' => {
                        command = 'V';
                        is_relative = ch.is_ascii_lowercase();
                        i += 1;
                    }
                    'Z' => {
                        if current_path.len() > 2 {
                            paths.push(current_path.clone());
                        }
                        current_path.clear();
                        i += 1;
                        continue;
                    }
                    _ => break, // Unsupported command
                }
            }

            // Parse values based on current command
            match command {
                'H' => {
                    if let Some((val, next)) = parse_number(&chars, i) {
                        x = if is_relative { x + val } else { val };
                        current_path.push(PointD::new(x, y));
                        i = next;
                    } else {
                        break;
                    }
                }
                'V' => {
                    if let Some((val, next)) = parse_number(&chars, i) {
                        y = if is_relative { y + val } else { val };
                        current_path.push(PointD::new(x, y));
                        i = next;
                    } else {
                        break;
                    }
                }
                'L' | 'M' => {
                    if let Some((vx, next1)) = parse_number(&chars, i) {
                        if let Some((vy, next2)) = parse_number(&chars, next1) {
                            x = if is_relative { x + vx } else { vx };
                            y = if is_relative { y + vy } else { vy };
                            current_path.push(PointD::new(x, y));
                            i = next2;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        // Push final path if it has enough points
        if current_path.len() > 3 {
            paths.push(current_path);
        }

        if paths.is_empty() {
            return false;
        }

        self.path_infos.push(PathInfo::new(
            paths,
            false,
            FillRule::EvenOdd,
            0,
            0xFF000000,
            1.0,
            false,
        ));
        true
    }

    /// Extract all paths from the loaded SVG.
    /// Direct port from C++ `GetPaths()`.
    pub fn get_paths(&self) -> PathsD {
        let mut result = PathsD::new();
        for pi in &self.path_infos {
            for path in &pi.paths {
                result.push(path.clone());
            }
        }
        result
    }
}

impl Default for SvgReader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SVG utility functions (from clipper.svg.utils.h)
// ============================================================================

/// Add a caption text to the SVG.
pub fn svg_add_caption(svg: &mut SvgWriter, caption: &str, x: i32, y: i32) {
    svg.add_text(caption, 0xFF000000, 14, x as f64, y as f64);
}

/// Add subject paths (Paths64) to the SVG.
pub fn svg_add_subject_64(svg: &mut SvgWriter, paths: &Paths64, fillrule: FillRule) {
    let tmp: PathsD = transform_paths(paths);
    svg.add_paths_d(
        &tmp,
        false,
        fillrule,
        SUBJ_BRUSH_CLR,
        SUBJ_STROKE_CLR,
        0.8,
        false,
    );
}

/// Add subject paths (PathsD) to the SVG.
pub fn svg_add_subject_d(svg: &mut SvgWriter, paths: &PathsD, fillrule: FillRule) {
    svg.add_paths_d(
        paths,
        false,
        fillrule,
        SUBJ_BRUSH_CLR,
        SUBJ_STROKE_CLR,
        0.8,
        false,
    );
}

/// Add open subject paths (Paths64) to the SVG.
pub fn svg_add_open_subject_64(svg: &mut SvgWriter, paths: &Paths64, fillrule: FillRule) {
    let tmp: PathsD = transform_paths(paths);
    svg.add_paths_d(&tmp, true, fillrule, 0x0, 0xCCB3B3DA, 1.3, false);
}

/// Add open subject paths (PathsD) to the SVG.
pub fn svg_add_open_subject_d(
    svg: &mut SvgWriter,
    paths: &PathsD,
    fillrule: FillRule,
    is_joined: bool,
) {
    if is_joined {
        svg.add_paths_d(
            paths,
            false,
            fillrule,
            SUBJ_BRUSH_CLR,
            SUBJ_STROKE_CLR,
            1.3,
            false,
        );
    } else {
        svg.add_paths_d(paths, true, fillrule, 0x0, SUBJ_STROKE_CLR, 1.3, false);
    }
}

/// Add clip paths (Paths64) to the SVG.
pub fn svg_add_clip_64(svg: &mut SvgWriter, paths: &Paths64, fillrule: FillRule) {
    let tmp: PathsD = transform_paths(paths);
    svg.add_paths_d(
        &tmp,
        false,
        fillrule,
        CLIP_BRUSH_CLR,
        CLIP_STROKE_CLR,
        0.8,
        false,
    );
}

/// Add clip paths (PathsD) to the SVG.
pub fn svg_add_clip_d(svg: &mut SvgWriter, paths: &PathsD, fillrule: FillRule) {
    svg.add_paths_d(
        paths,
        false,
        fillrule,
        CLIP_BRUSH_CLR,
        CLIP_STROKE_CLR,
        0.8,
        false,
    );
}

/// Add solution paths (Paths64) to the SVG.
pub fn svg_add_solution_64(
    svg: &mut SvgWriter,
    paths: &Paths64,
    fillrule: FillRule,
    show_coords: bool,
) {
    let tmp: PathsD = transform_paths(paths);
    svg.add_paths_d(
        &tmp,
        false,
        fillrule,
        SOLUTION_BRUSH_CLR,
        0xFF003300,
        1.0,
        show_coords,
    );
}

/// Add solution paths (PathsD) to the SVG.
pub fn svg_add_solution_d(
    svg: &mut SvgWriter,
    paths: &PathsD,
    fillrule: FillRule,
    show_coords: bool,
) {
    svg.add_paths_d(
        paths,
        false,
        fillrule,
        SOLUTION_BRUSH_CLR,
        0xFF003300,
        1.2,
        show_coords,
    );
}

/// Add open solution paths (Paths64) to the SVG.
pub fn svg_add_open_solution_64(
    svg: &mut SvgWriter,
    paths: &Paths64,
    fillrule: FillRule,
    show_coords: bool,
    is_joined: bool,
) {
    let tmp: PathsD = transform_paths(paths);
    svg.add_paths_d(
        &tmp,
        !is_joined,
        fillrule,
        0x0,
        0xFF006600,
        1.8,
        show_coords,
    );
}

/// Add open solution paths (PathsD) to the SVG.
pub fn svg_add_open_solution_d(
    svg: &mut SvgWriter,
    paths: &PathsD,
    fillrule: FillRule,
    show_coords: bool,
    is_joined: bool,
) {
    svg.add_paths_d(
        paths,
        !is_joined,
        fillrule,
        0x0,
        0xFF006600,
        1.8,
        show_coords,
    );
}

/// Save SVG to file with sensible defaults and coordinate styling.
pub fn svg_save_to_file(
    svg: &mut SvgWriter,
    filename: &str,
    max_width: i32,
    max_height: i32,
    margin: i32,
) {
    svg.set_coords_style("Verdana", 0xFF0000AA, 7);
    svg.save_to_file(filename, max_width, max_height, margin);
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Parse a number from a character slice starting at position `start`.
/// Returns the parsed value and the new position after the number.
fn parse_number(chars: &[char], start: usize) -> Option<(f64, usize)> {
    let mut i = start;
    // Skip whitespace and commas
    while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ',') {
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }

    let start_pos = i;
    let is_neg = chars[i] == '-';
    if is_neg {
        i += 1;
    }
    if chars.get(i) == Some(&'+') {
        i += 1;
    }

    let mut has_digits = false;
    let mut has_dot = false;

    while i < chars.len() {
        if chars[i] == '.' {
            if has_dot {
                break;
            }
            has_dot = true;
        } else if chars[i].is_ascii_digit() {
            has_digits = true;
        } else {
            break;
        }
        i += 1;
    }

    if !has_digits {
        return None;
    }

    let num_str: String = chars[start_pos..i].iter().collect();
    match num_str.parse::<f64>() {
        Ok(val) => Some((val, i)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Point64;

    #[test]
    fn test_color_to_html() {
        assert_eq!(color_to_html(0xFF123456), "#123456");
        assert_eq!(color_to_html(0x00000000), "#000000");
        assert_eq!(color_to_html(0xFFFFFFFF), "#ffffff");
    }

    #[test]
    fn test_get_alpha_as_frac() {
        assert!((get_alpha_as_frac(0xFF000000) - 1.0).abs() < 0.01);
        assert!((get_alpha_as_frac(0x80000000) - 0.502).abs() < 0.01);
        assert!((get_alpha_as_frac(0x00000000) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_svg_writer_new() {
        let svg = SvgWriter::new(0);
        assert_eq!(svg.fill_rule(), FillRule::NonZero);
    }

    #[test]
    fn test_svg_writer_add_text() {
        let mut svg = SvgWriter::new(0);
        svg.add_text("Hello", 0xFF000000, 12, 10.0, 20.0);
        assert_eq!(svg.text_infos.len(), 1);
        assert_eq!(svg.text_infos[0].text, "Hello");
    }

    #[test]
    fn test_svg_writer_add_paths() {
        let mut svg = SvgWriter::new(0);
        let paths = vec![vec![
            Point64::new(0, 0),
            Point64::new(100, 0),
            Point64::new(100, 100),
            Point64::new(0, 100),
        ]];
        svg.add_paths_64(
            &paths,
            false,
            FillRule::NonZero,
            0xFF0000FF,
            0xFF000000,
            1.0,
            false,
        );
        assert_eq!(svg.path_infos.len(), 1);
    }

    #[test]
    fn test_svg_writer_add_empty_paths() {
        let mut svg = SvgWriter::new(0);
        let paths: Paths64 = vec![];
        svg.add_paths_64(&paths, false, FillRule::NonZero, 0, 0, 1.0, false);
        assert_eq!(svg.path_infos.len(), 0);
    }

    #[test]
    fn test_svg_writer_clear() {
        let mut svg = SvgWriter::new(0);
        svg.add_text("Test", 0xFF000000, 12, 0.0, 0.0);
        let paths = vec![vec![
            PointD::new(0.0, 0.0),
            PointD::new(1.0, 1.0),
            PointD::new(2.0, 0.0),
        ]];
        svg.add_paths_d(&paths, false, FillRule::NonZero, 0, 0, 1.0, false);
        svg.clear();
        assert!(svg.path_infos.is_empty());
        assert!(svg.text_infos.is_empty());
    }

    #[test]
    fn test_svg_writer_save_to_file() {
        let mut svg = SvgWriter::new(0);
        let paths = vec![vec![
            PointD::new(0.0, 0.0),
            PointD::new(100.0, 0.0),
            PointD::new(100.0, 100.0),
            PointD::new(0.0, 100.0),
        ]];
        svg.add_paths_d(
            &paths,
            false,
            FillRule::NonZero,
            0x800000FF,
            0xFF000000,
            1.0,
            false,
        );
        svg.add_text("Test SVG", 0xFF000000, 14, 10.0, 10.0);

        let tmp_file = std::env::temp_dir().join("clipper2_test_output.svg");
        let result = svg.save_to_file(tmp_file.to_str().unwrap(), 800, 600, 20);
        assert!(result);

        let content = fs::read_to_string(&tmp_file).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("</svg>"));
        assert!(content.contains("<path"));
        assert!(content.contains("Test SVG"));

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_svg_reader_new() {
        let reader = SvgReader::new();
        assert!(reader.xml.is_empty());
        assert!(reader.get_paths().is_empty());
    }

    #[test]
    fn test_svg_reader_roundtrip() {
        // Write an SVG and read it back
        let mut writer = SvgWriter::new(0);
        let paths = vec![vec![
            PointD::new(10.0, 10.0),
            PointD::new(90.0, 10.0),
            PointD::new(90.0, 90.0),
            PointD::new(10.0, 90.0),
        ]];
        writer.add_paths_d(
            &paths,
            false,
            FillRule::NonZero,
            0x800000FF,
            0xFF000000,
            1.0,
            false,
        );

        let tmp_file = std::env::temp_dir().join("clipper2_test_roundtrip.svg");
        let filename = tmp_file.to_str().unwrap();
        assert!(writer.save_to_file(filename, 400, 400, 20));

        let mut reader = SvgReader::new();
        assert!(reader.load_from_file(filename));
        let read_paths = reader.get_paths();
        assert!(!read_paths.is_empty());

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_parse_number() {
        let chars: Vec<char> = "123.45, -67.8".chars().collect();
        let (val, next) = parse_number(&chars, 0).unwrap();
        assert!((val - 123.45).abs() < 0.001);
        let (val2, _) = parse_number(&chars, next).unwrap();
        assert!((val2 - (-67.8)).abs() < 0.001);
    }

    #[test]
    fn test_parse_number_whitespace() {
        let chars: Vec<char> = "  42  ".chars().collect();
        let (val, _) = parse_number(&chars, 0).unwrap();
        assert!((val - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_number_empty() {
        let chars: Vec<char> = "   ".chars().collect();
        assert!(parse_number(&chars, 0).is_none());
    }
}
