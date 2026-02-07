// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestLines.cpp
//
// Tests line clipping operations from the Lines.txt test data file.

use clipper2::core::*;
use clipper2::engine_public::*;
use clipper2::utils::file_io;

fn test_data_path(name: &str) -> String {
    format!("{}/Tests/data/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_multiple_lines() {
    let path = test_data_path("Lines.txt");
    let mut test_number = 1usize;
    while let Some(data) = file_io::load_test_num(&path, test_number) {
        let mut c = Clipper64::new();
        c.add_subject(&data.subj);
        c.add_open_subject(&data.subj_open);
        c.add_clip(&data.clip);
        let mut solution = Paths64::new();
        let mut solution_open = Paths64::new();
        assert!(
            c.execute(
                data.clip_type,
                data.fill_rule,
                &mut solution,
                Some(&mut solution_open)
            ),
            "Execute failed for test {}",
            test_number
        );

        let count2 = (solution.len() + solution_open.len()) as i64;
        let count_diff = (count2 - data.count).abs();
        let relative_count_diff = if data.count != 0 {
            count_diff as f64 / data.count as f64
        } else {
            0.0
        };

        if test_number == 1 {
            assert_eq!(solution.len(), 1, "Test 1: expected 1 solution path");
            if !solution.is_empty() {
                assert_eq!(
                    solution[0].len(),
                    6,
                    "Test 1: solution[0] should have 6 vertices"
                );
                assert!(
                    is_positive(&solution[0]),
                    "Test 1: solution[0] should be positive"
                );
            }
            assert_eq!(
                solution_open.len(),
                1,
                "Test 1: expected 1 open solution path"
            );
            if !solution_open.is_empty() {
                assert_eq!(
                    solution_open[0].len(),
                    2,
                    "Test 1: solution_open[0] should have 2 vertices"
                );
                if !solution_open[0].is_empty() {
                    // Expect vertex closest to input path's start
                    assert_eq!(
                        solution_open[0][0].y, 6,
                        "Test 1: solution_open[0][0].y should be 6"
                    );
                }
            }
        } else {
            assert!(
                count_diff <= 8,
                "Test {}: count_diff {} > 8",
                test_number,
                count_diff
            );
            assert!(
                relative_count_diff <= 0.1,
                "Test {}: relative_count_diff {} > 0.1",
                test_number,
                relative_count_diff
            );
        }
        test_number += 1;
    }
    assert!(
        test_number >= 17,
        "Expected at least 17 tests, got {}",
        test_number
    );
}
