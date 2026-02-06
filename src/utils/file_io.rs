// Copyright 2025 - Clipper2 Rust port
// Direct port of ClipFileLoad.h / ClipFileLoad.cpp / ClipFileSave.h / ClipFileSave.cpp
// by Angus Johnson
// License: https://www.boost.org/LICENSE_1_0.txt
//
// Purpose: Test data file loading and saving for clipper operations

use crate::core::{Path64, Paths64, Point64};
use crate::engine::ClipType;
use crate::FillRule;
use std::fs;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;

// ============================================================================
// Helper: file existence check
// ============================================================================

/// Check if a file exists.
/// Direct port from C++ `FileExists()`.
pub fn file_exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

// ============================================================================
// Internal parsing helpers
// ============================================================================

/// Parse an i64 integer from a string iterator position.
/// Direct port from C++ `GetInt()`.
///
/// Skips leading whitespace, parses an optional sign and digits,
/// then skips trailing whitespace and an optional comma.
fn get_int(s: &str, pos: &mut usize) -> Option<i64> {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Skip leading whitespace
    while *pos < len && bytes[*pos] == b' ' {
        *pos += 1;
    }
    if *pos >= len {
        return None;
    }

    let is_neg = bytes[*pos] == b'-';
    if is_neg {
        *pos += 1;
    }

    let start = *pos;
    let mut value: i64 = 0;

    while *pos < len && bytes[*pos] >= b'0' && bytes[*pos] <= b'9' {
        value = value * 10 + (bytes[*pos] - b'0') as i64;
        *pos += 1;
    }

    if *pos == start {
        return None; // no digits found
    }

    // Trim trailing whitespace
    while *pos < len && bytes[*pos] == b' ' {
        *pos += 1;
    }
    // Skip a comma if present
    if *pos < len && bytes[*pos] == b',' {
        *pos += 1;
    }

    if is_neg {
        value = -value;
    }
    Some(value)
}

/// Parse a line of integer coordinate pairs into a Path64.
/// Direct port from C++ `GetPath()`.
fn get_path(line: &str) -> Option<Path64> {
    let mut path = Path64::new();
    let mut pos = 0;

    while let Some(x) = get_int(line, &mut pos) {
        if let Some(y) = get_int(line, &mut pos) {
            path.push(Point64::new(x, y));
        } else {
            break;
        }
    }

    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Read consecutive path lines from a buffered reader until a non-path line.
/// Direct port from C++ `GetPaths()`.
fn get_paths(reader: &mut BufReader<fs::File>, paths: &mut Paths64) -> io::Result<Option<String>> {
    let mut line = String::new();
    loop {
        let pos_before = reader.stream_position()?;
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let trimmed = line.trim();
        if let Some(path) = get_path(trimmed) {
            paths.push(path);
        } else {
            // Not a path line - seek back so the caller can re-read this line
            reader.seek(SeekFrom::Start(pos_before))?;
            return Ok(None);
        }
    }
}

// ============================================================================
// Test result struct
// ============================================================================

/// Result of loading a test from a test data file.
///
/// Contains the subject, open subject, and clip paths along with
/// expected results and operation parameters.
#[derive(Debug, Clone)]
pub struct ClipTestData {
    pub subj: Paths64,
    pub subj_open: Paths64,
    pub clip: Paths64,
    pub area: i64,
    pub count: i64,
    pub clip_type: ClipType,
    pub fill_rule: FillRule,
}

impl Default for ClipTestData {
    fn default() -> Self {
        Self {
            subj: Paths64::new(),
            subj_open: Paths64::new(),
            clip: Paths64::new(),
            area: 0,
            count: 0,
            clip_type: ClipType::Intersection,
            fill_rule: FillRule::EvenOdd,
        }
    }
}

// ============================================================================
// Loading functions
// ============================================================================

/// Load a specific test number from a test data file.
///
/// Direct port from C++ `LoadTestNum()`.
///
/// # Arguments
/// * `filename` - Path to the test data file
/// * `test_num` - 1-based test number to load
///
/// # Returns
/// `Some(ClipTestData)` if the test was found, `None` otherwise.
pub fn load_test_num(filename: &str, test_num: usize) -> Option<ClipTestData> {
    let file = fs::File::open(filename).ok()?;
    let mut reader = BufReader::new(file);
    load_test_num_from_reader(&mut reader, test_num)
}

/// Load the first test from a test data file.
///
/// Direct port from C++ `LoadTest()`.
pub fn load_test(filename: &str) -> Option<ClipTestData> {
    load_test_num(filename, 1)
}

/// Internal: Load a test from an already-opened reader.
/// Direct port from C++ `LoadTestNum()`.
///
/// The C++ logic:
///   while (getline(source, line)) {
///     if (test_num) {          // still searching for the right CAPTION
///       if (line.find("CAPTION:") != npos) --test_num;
///       continue;              // skip everything until test_num == 0
///     }
///     if (line.find("CAPTION:") != npos) break;  // next test, stop
///     // ... parse data lines ...
///   }
///   return !test_num;          // true if we found the right CAPTION
fn load_test_num_from_reader(
    reader: &mut BufReader<fs::File>,
    test_num: usize,
) -> Option<ClipTestData> {
    let mut test_num = test_num.max(1) as i64;
    reader.seek(SeekFrom::Start(0)).ok()?;

    let mut data = ClipTestData::default();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).ok()?;
        if bytes == 0 {
            break; // EOF
        }

        // Phase 1: Skip to the correct CAPTION (matching C++ `if (test_num)` block)
        if test_num > 0 {
            if line.contains("CAPTION:") {
                test_num -= 1;
            }
            continue;
        }

        // Phase 2: Parse test data
        let trimmed = line.trim();

        if trimmed.contains("CAPTION:") {
            break; // next test - stop
        } else if trimmed.contains("INTERSECTION") {
            data.clip_type = ClipType::Intersection;
        } else if trimmed.contains("UNION") {
            data.clip_type = ClipType::Union;
        } else if trimmed.contains("DIFFERENCE") {
            data.clip_type = ClipType::Difference;
        } else if trimmed.contains("XOR") {
            data.clip_type = ClipType::Xor;
        } else if trimmed.contains("EVENODD") {
            data.fill_rule = FillRule::EvenOdd;
        } else if trimmed.contains("NONZERO") {
            data.fill_rule = FillRule::NonZero;
        } else if trimmed.contains("POSITIVE") {
            data.fill_rule = FillRule::Positive;
        } else if trimmed.contains("NEGATIVE") {
            data.fill_rule = FillRule::Negative;
        } else if trimmed.contains("SOL_AREA") {
            if let Some(colon_pos) = trimmed.find(':') {
                let val_str = trimmed[colon_pos + 1..].trim();
                let mut pos = 0;
                if let Some(val) = get_int(val_str, &mut pos) {
                    data.area = val;
                }
            }
        } else if trimmed.contains("SOL_COUNT") {
            if let Some(colon_pos) = trimmed.find(':') {
                let val_str = trimmed[colon_pos + 1..].trim();
                let mut pos = 0;
                if let Some(val) = get_int(val_str, &mut pos) {
                    data.count = val;
                }
            }
        } else if trimmed.contains("SUBJECTS_OPEN") {
            let _ = get_paths(reader, &mut data.subj_open);
        } else if trimmed.contains("SUBJECTS") {
            let _ = get_paths(reader, &mut data.subj);
        } else if trimmed.contains("CLIPS") {
            let _ = get_paths(reader, &mut data.clip);
        }
    }

    // C++ returns !test_num (true if we found and consumed the target CAPTION)
    if test_num > 0 {
        None
    } else {
        Some(data)
    }
}

// ============================================================================
// Saving functions
// ============================================================================

/// Write paths as coordinate text to a writer.
/// Direct port from C++ `PathsToStream()`.
fn paths_to_stream(paths: &Paths64, writer: &mut dyn Write) -> io::Result<()> {
    for path in paths {
        if path.is_empty() {
            continue;
        }
        let last_idx = path.len() - 1;
        for (i, pt) in path.iter().enumerate() {
            if i < last_idx {
                write!(writer, "{},{}, ", pt.x, pt.y)?;
            } else {
                writeln!(writer, "{},{}", pt.x, pt.y)?;
            }
        }
    }
    Ok(())
}

/// Save a test to a file.
///
/// Direct port from C++ `SaveTest()`.
///
/// # Arguments
/// * `filename` - Path to the output file
/// * `append` - If true, append to existing file and auto-number tests
/// * `subj` - Optional subject paths
/// * `subj_open` - Optional open subject paths
/// * `clip` - Optional clip paths
/// * `area` - Expected solution area
/// * `count` - Expected solution count
/// * `ct` - Clip type
/// * `fr` - Fill rule
#[allow(clippy::too_many_arguments)]
pub fn save_test(
    filename: &str,
    append: bool,
    subj: Option<&Paths64>,
    subj_open: Option<&Paths64>,
    clip: Option<&Paths64>,
    area: i64,
    count: i64,
    ct: ClipType,
    fr: FillRule,
) -> bool {
    let mut last_test_no: i64 = 0;

    if append && file_exists(filename) {
        // Find the last CAPTION number
        if let Ok(content) = fs::read_to_string(filename) {
            for line in content.lines().rev() {
                if let Some(cap_pos) = line.find("CAPTION:") {
                    let after = line[cap_pos + 8..].trim();
                    // Parse the test number (strip trailing period/dot)
                    let num_str = after.trim_end_matches('.');
                    if let Ok(n) = num_str.trim().parse::<i64>() {
                        last_test_no = n;
                    }
                    break;
                }
            }
        }
    } else if file_exists(filename) {
        let _ = fs::remove_file(filename);
    }

    last_test_no += 1;

    let file = if append && file_exists(filename) {
        fs::OpenOptions::new().append(true).open(filename)
    } else {
        fs::File::create(filename)
    };

    let mut file = match file {
        Ok(f) => f,
        Err(_) => return false,
    };

    let cliptype_string = match ct {
        ClipType::NoClip => "NOCLIP",
        ClipType::Intersection => "INTERSECTION",
        ClipType::Union => "UNION",
        ClipType::Difference => "DIFFERENCE",
        ClipType::Xor => "XOR",
    };

    let fillrule_string = match fr {
        FillRule::EvenOdd => "EVENODD",
        FillRule::NonZero => "NONZERO",
        FillRule::Positive => "POSITIVE",
        FillRule::Negative => "NEGATIVE",
    };

    let header = format!(
        "CAPTION: {}.\nCLIPTYPE: {}\nFILLRULE: {}\nSOL_AREA: {}\nSOL_COUNT: {}\n",
        last_test_no, cliptype_string, fillrule_string, area, count
    );

    if write!(file, "{}", header).is_err() {
        return false;
    }

    if let Some(subj) = subj {
        if writeln!(file, "SUBJECTS").is_err() {
            return false;
        }
        if paths_to_stream(subj, &mut file).is_err() {
            return false;
        }
    }

    if let Some(subj_open) = subj_open {
        if writeln!(file, "SUBJECTS_OPEN").is_err() {
            return false;
        }
        if paths_to_stream(subj_open, &mut file).is_err() {
            return false;
        }
    }

    if let Some(clip) = clip {
        if !clip.is_empty() {
            if writeln!(file, "CLIPS").is_err() {
                return false;
            }
            if paths_to_stream(clip, &mut file).is_err() {
                return false;
            }
        }
    }

    if writeln!(file).is_err() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        // A file that definitely doesn't exist
        assert!(!file_exists("__nonexistent_test_file_xyz__.txt"));
        // A file we know exists (Cargo.toml in project root)
        assert!(file_exists("Cargo.toml"));
    }

    #[test]
    fn test_get_int_basic() {
        let s = "123, -456, 789";
        let mut pos = 0;
        assert_eq!(get_int(s, &mut pos), Some(123));
        assert_eq!(get_int(s, &mut pos), Some(-456));
        assert_eq!(get_int(s, &mut pos), Some(789));
        assert_eq!(get_int(s, &mut pos), None);
    }

    #[test]
    fn test_get_int_with_whitespace() {
        let s = "  42  ";
        let mut pos = 0;
        assert_eq!(get_int(s, &mut pos), Some(42));
    }

    #[test]
    fn test_get_int_empty() {
        let s = "   ";
        let mut pos = 0;
        assert_eq!(get_int(s, &mut pos), None);
    }

    #[test]
    fn test_get_path_basic() {
        let line = "10,20, 30,40, 50,60";
        let path = get_path(line).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Point64::new(10, 20));
        assert_eq!(path[1], Point64::new(30, 40));
        assert_eq!(path[2], Point64::new(50, 60));
    }

    #[test]
    fn test_get_path_negative() {
        let line = "-10,-20, 30,-40";
        let path = get_path(line).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], Point64::new(-10, -20));
        assert_eq!(path[1], Point64::new(30, -40));
    }

    #[test]
    fn test_get_path_empty() {
        let line = "SUBJECTS";
        assert!(get_path(line).is_none());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_fileio.txt");
        let filename = tmp_file.to_str().unwrap();

        let subj = vec![vec![
            Point64::new(0, 0),
            Point64::new(100, 0),
            Point64::new(100, 100),
            Point64::new(0, 100),
        ]];
        let clip = vec![vec![
            Point64::new(50, 50),
            Point64::new(150, 50),
            Point64::new(150, 150),
            Point64::new(50, 150),
        ]];

        // Save
        let result = save_test(
            filename,
            false,
            Some(&subj),
            None,
            Some(&clip),
            2500,
            1,
            ClipType::Intersection,
            FillRule::EvenOdd,
        );
        assert!(result);
        assert!(file_exists(filename));

        // Load
        let data = load_test(filename).unwrap();
        assert_eq!(data.clip_type, ClipType::Intersection);
        assert_eq!(data.fill_rule, FillRule::EvenOdd);
        assert_eq!(data.area, 2500);
        assert_eq!(data.count, 1);
        assert_eq!(data.subj.len(), 1);
        assert_eq!(data.subj[0].len(), 4);
        assert_eq!(data.clip.len(), 1);
        assert_eq!(data.clip[0].len(), 4);

        // Verify coordinates
        assert_eq!(data.subj[0][0], Point64::new(0, 0));
        assert_eq!(data.subj[0][1], Point64::new(100, 0));
        assert_eq!(data.clip[0][0], Point64::new(50, 50));

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_save_append_increments_test_number() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_append.txt");
        let filename = tmp_file.to_str().unwrap();

        // Remove if exists from previous run
        let _ = fs::remove_file(&tmp_file);

        let subj = vec![vec![
            Point64::new(0, 0),
            Point64::new(10, 0),
            Point64::new(10, 10),
        ]];

        // First save (test 1)
        assert!(save_test(
            filename,
            false,
            Some(&subj),
            None,
            None,
            100,
            1,
            ClipType::Union,
            FillRule::NonZero,
        ));

        // Second save (append, should be test 2)
        assert!(save_test(
            filename,
            true,
            Some(&subj),
            None,
            None,
            200,
            2,
            ClipType::Difference,
            FillRule::EvenOdd,
        ));

        // Verify file contains both captions
        let content = fs::read_to_string(&tmp_file).unwrap();
        assert!(content.contains("CAPTION: 1."));
        assert!(content.contains("CAPTION: 2."));

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_save_all_clip_types() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_clip_types.txt");
        let filename = tmp_file.to_str().unwrap();

        for (ct, expected) in [
            (ClipType::Intersection, "INTERSECTION"),
            (ClipType::Union, "UNION"),
            (ClipType::Difference, "DIFFERENCE"),
            (ClipType::Xor, "XOR"),
            (ClipType::NoClip, "NOCLIP"),
        ] {
            assert!(save_test(
                filename,
                false,
                None,
                None,
                None,
                0,
                0,
                ct,
                FillRule::EvenOdd,
            ));
            let content = fs::read_to_string(&tmp_file).unwrap();
            assert!(content.contains(expected), "Missing {}", expected);
        }

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_save_all_fill_rules() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_fill_rules.txt");
        let filename = tmp_file.to_str().unwrap();

        for (fr, expected) in [
            (FillRule::EvenOdd, "EVENODD"),
            (FillRule::NonZero, "NONZERO"),
            (FillRule::Positive, "POSITIVE"),
            (FillRule::Negative, "NEGATIVE"),
        ] {
            assert!(save_test(
                filename,
                false,
                None,
                None,
                None,
                0,
                0,
                ClipType::Union,
                fr,
            ));
            let content = fs::read_to_string(&tmp_file).unwrap();
            assert!(content.contains(expected), "Missing {}", expected);
        }

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_save_with_open_subjects() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_open_subj.txt");
        let filename = tmp_file.to_str().unwrap();

        let subj_open = vec![vec![Point64::new(0, 0), Point64::new(100, 100)]];

        assert!(save_test(
            filename,
            false,
            None,
            Some(&subj_open),
            None,
            0,
            0,
            ClipType::Union,
            FillRule::NonZero,
        ));

        let content = fs::read_to_string(&tmp_file).unwrap();
        assert!(content.contains("SUBJECTS_OPEN"));
        assert!(content.contains("0,0"));
        assert!(content.contains("100,100"));

        let _ = fs::remove_file(&tmp_file);
    }

    #[test]
    fn test_load_nonexistent_file() {
        assert!(load_test("__nonexistent_file_xyz__.txt").is_none());
    }

    #[test]
    fn test_load_test_num_out_of_range() {
        let tmp_file = std::env::temp_dir().join("clipper2_test_range.txt");
        let filename = tmp_file.to_str().unwrap();

        let subj = vec![vec![
            Point64::new(0, 0),
            Point64::new(10, 0),
            Point64::new(10, 10),
        ]];

        assert!(save_test(
            filename,
            false,
            Some(&subj),
            None,
            None,
            50,
            1,
            ClipType::Union,
            FillRule::NonZero,
        ));

        // Test 1 should exist
        assert!(load_test_num(filename, 1).is_some());
        // Test 2 should not exist
        assert!(load_test_num(filename, 2).is_none());

        let _ = fs::remove_file(&tmp_file);
    }
}
