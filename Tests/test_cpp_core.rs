// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestIsCollinear.cpp, TestRect.cpp,
// TestSimplifyPath.cpp, TestTrimCollinear.cpp, TestOrientation.cpp
//
// These are integration tests matching the original GoogleTest suite.

use clipper2_rust::core::*;
use clipper2_rust::engine::ClipType;
use clipper2_rust::engine_public::*;

// ==========================================================================
// From TestIsCollinear.cpp - TesthiCalculation
// ==========================================================================

#[test]
fn test_hi_calculation() {
    assert_eq!(
        multiply_u64(0x51eaed81157de061, 0x3a271fb2745b6fe9).hi,
        0x129bbebdfae0464e
    );
    assert_eq!(
        multiply_u64(0x3a271fb2745b6fe9, 0x51eaed81157de061).hi,
        0x129bbebdfae0464e
    );
    assert_eq!(
        multiply_u64(0xc2055706a62883fa, 0x26c78bc79c2322cc).hi,
        0x1d640701d192519b
    );
    assert_eq!(
        multiply_u64(0x26c78bc79c2322cc, 0xc2055706a62883fa).hi,
        0x1d640701d192519b
    );
    assert_eq!(
        multiply_u64(0x874ddae32094b0de, 0x9b1559a06fdf83e0).hi,
        0x51f76c49563e5bfe
    );
    assert_eq!(
        multiply_u64(0x9b1559a06fdf83e0, 0x874ddae32094b0de).hi,
        0x51f76c49563e5bfe
    );
    assert_eq!(
        multiply_u64(0x81fb3ad3636ca900, 0x239c000a982a8da4).hi,
        0x12148e28207b83a3
    );
    assert_eq!(
        multiply_u64(0x239c000a982a8da4, 0x81fb3ad3636ca900).hi,
        0x12148e28207b83a3
    );
    assert_eq!(
        multiply_u64(0x4be0b4c5d2725c44, 0x990cd6db34a04c30).hi,
        0x2d5d1a4183fd6165
    );
    assert_eq!(
        multiply_u64(0x990cd6db34a04c30, 0x4be0b4c5d2725c44).hi,
        0x2d5d1a4183fd6165
    );
    assert_eq!(
        multiply_u64(0x978ec0c0433c01f6, 0x2df03d097966b536).hi,
        0x1b3251d91fe272a5
    );
    assert_eq!(
        multiply_u64(0x2df03d097966b536, 0x978ec0c0433c01f6).hi,
        0x1b3251d91fe272a5
    );
    assert_eq!(
        multiply_u64(0x49c5cbbcfd716344, 0xc489e3b34b007ad3).hi,
        0x38a32c74c8c191a4
    );
    assert_eq!(
        multiply_u64(0xc489e3b34b007ad3, 0x49c5cbbcfd716344).hi,
        0x38a32c74c8c191a4
    );
    assert_eq!(
        multiply_u64(0xd3361cdbeed655d5, 0x1240da41e324953a).hi,
        0x0f0f4fa11e7e8f2a
    );
    assert_eq!(
        multiply_u64(0x1240da41e324953a, 0xd3361cdbeed655d5).hi,
        0x0f0f4fa11e7e8f2a
    );
    assert_eq!(
        multiply_u64(0x51b854f8e71b0ae0, 0x6f8d438aae530af5).hi,
        0x239c04ee3c8cc248
    );
    assert_eq!(
        multiply_u64(0x6f8d438aae530af5, 0x51b854f8e71b0ae0).hi,
        0x239c04ee3c8cc248
    );
    assert_eq!(
        multiply_u64(0xbbecf7dbc6147480, 0xbb0f73d0f82e2236).hi,
        0x895170f4e9a216a7
    );
    assert_eq!(
        multiply_u64(0xbb0f73d0f82e2236, 0xbbecf7dbc6147480).hi,
        0x895170f4e9a216a7
    );
}

// ==========================================================================
// From TestIsCollinear.cpp - TestIsCollinear
// ==========================================================================

#[test]
fn test_is_collinear_large_integers() {
    let i: i64 = 9007199254740993;
    let pt1 = Point64::new(0, 0);
    let shared_pt = Point64::new(i, i * 10);
    let pt2 = Point64::new(i * 10, i * 100);
    assert!(is_collinear(pt1, shared_pt, pt2));
}

#[test]
fn test_is_collinear2_issue_831() {
    let i: i64 = 0x4000000000000;
    let subject = vec![vec![
        Point64::new(-i, -i),
        Point64::new(i, -i),
        Point64::new(-i, i),
        Point64::new(i, i),
    ]];
    let mut clipper = Clipper64::new();
    clipper.add_subject(&subject);
    let mut solution = Paths64::new();
    let mut solution_open = Paths64::new();
    clipper.execute(
        ClipType::Union,
        FillRule::EvenOdd,
        &mut solution,
        Some(&mut solution_open),
    );
    assert_eq!(solution.len(), 2);
}

// ==========================================================================
// From TestRect.cpp - TestRectOpPlus
// ==========================================================================

#[test]
fn test_rect_op_plus() {
    // Invalid + valid = valid
    {
        let lhs = Rect64::invalid();
        let rhs = Rect64::new(-1, -1, 10, 10);
        let sum = lhs + rhs;
        assert_eq!(sum, rhs);
        let sum2 = rhs + lhs;
        assert_eq!(sum2, rhs);
    }
    // Invalid + positive valid = valid
    {
        let lhs = Rect64::invalid();
        let rhs = Rect64::new(1, 1, 10, 10);
        let sum = lhs + rhs;
        assert_eq!(sum, rhs);
        let sum2 = rhs + lhs;
        assert_eq!(sum2, rhs);
    }
    // Adjacent rects
    {
        let lhs = Rect64::new(0, 0, 1, 1);
        let rhs = Rect64::new(-1, -1, 0, 0);
        let expected = Rect64::new(-1, -1, 1, 1);
        let sum = lhs + rhs;
        assert_eq!(sum, expected);
        let sum2 = rhs + lhs;
        assert_eq!(sum2, expected);
    }
    // Separated rects
    {
        let lhs = Rect64::new(-10, -10, -1, -1);
        let rhs = Rect64::new(1, 1, 10, 10);
        let expected = Rect64::new(-10, -10, 10, 10);
        let sum = lhs + rhs;
        assert_eq!(sum, expected);
        let sum2 = rhs + lhs;
        assert_eq!(sum2, expected);
    }
}

// ==========================================================================
// From TestSimplifyPath.cpp
// ==========================================================================

#[test]
fn test_simplify_path_cpp() {
    let input = clipper2_rust::make_path64(&[
        0, 0, 1, 1, 0, 20, 0, 21, 1, 40, 0, 41, 0, 60, 0, 61, 0, 80, 1, 81, 0, 100,
    ]);
    let output = clipper2_rust::simplify_path(&input, 2.0, false);
    let len = clipper2_rust::path_length(&output, false);
    assert!((len - 100.0).abs() < 1.0, "Length was {}", len);
    assert_eq!(output.len(), 2);
}

// ==========================================================================
// From TestTrimCollinear.cpp
// ==========================================================================

#[test]
fn test_trim_collinear_cpp() {
    let input1 = clipper2_rust::make_path64(&[
        10, 10, 10, 10, 50, 10, 100, 10, 100, 100, 10, 100, 10, 10, 20, 10,
    ]);
    let output1 = clipper2_rust::trim_collinear_64(&input1, false);
    assert_eq!(output1.len(), 4);

    let input2 =
        clipper2_rust::make_path64(&[10, 10, 10, 10, 100, 10, 100, 100, 10, 100, 10, 10, 10, 10]);
    let output2 = clipper2_rust::trim_collinear_64(&input2, true);
    assert_eq!(output2.len(), 5);

    let input3 = clipper2_rust::make_path64(&[
        10, 10, 10, 50, 10, 10, 50, 10, 50, 50, 50, 10, 70, 10, 70, 50, 70, 10, 50, 10, 100, 10,
        100, 50, 100, 10,
    ]);
    let output3 = clipper2_rust::trim_collinear_64(&input3, false);
    assert_eq!(output3.len(), 0);

    let input4 = clipper2_rust::make_path64(&[
        2, 3, 3, 4, 4, 4, 4, 5, 7, 5, 8, 4, 8, 3, 9, 3, 8, 3, 7, 3, 6, 3, 5, 3, 4, 3, 3, 3, 2, 3,
    ]);
    let output4a = clipper2_rust::trim_collinear_64(&input4, false);
    let output4b = clipper2_rust::trim_collinear_64(&output4a, false);
    let area4a = area(&output4a) as i32;
    let area4b = area(&output4b) as i32;
    assert_eq!(output4a.len(), 7);
    assert_eq!(area4a, -9);
    assert_eq!(output4a.len(), output4b.len());
    assert_eq!(area4a, area4b);
}

// ==========================================================================
// From TestOrientation.cpp - TestNegativeOrientation
// ==========================================================================

#[test]
fn test_negative_orientation() {
    let subjects = vec![
        clipper2_rust::make_path64(&[0, 0, 0, 100, 100, 100, 100, 0]),
        clipper2_rust::make_path64(&[10, 10, 10, 110, 110, 110, 110, 10]),
    ];
    assert!(!is_positive(&subjects[0]));
    assert!(!is_positive(&subjects[1]));

    let clips = vec![clipper2_rust::make_path64(&[
        50, 50, 50, 150, 150, 150, 150, 50,
    ])];
    assert!(!is_positive(&clips[0]));

    let solution = clipper2_rust::union_64(&subjects, &clips, FillRule::Negative);
    assert_eq!(solution.len(), 1);
    assert_eq!(solution[0].len(), 12);
}
