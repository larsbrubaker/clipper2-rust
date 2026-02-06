// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestOffsets.cpp, TestOffsetOrientation.cpp
//
// These are integration tests matching the original GoogleTest suite.

use clipper2::core::*;
use clipper2::offset::*;

// ==========================================================================
// From TestOffsetOrientation.cpp
// ==========================================================================

#[test]
fn test_offsetting_orientation1() {
    let subject = vec![clipper2::make_path64(&[0, 0, 0, 5, 5, 5, 5, 0])];
    let solution = clipper2::inflate_paths_64(
        &subject, 1.0, JoinType::Round, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);
    // Output orientation should match input
    assert_eq!(is_positive(&subject[0]), is_positive(&solution[0]));
}

#[test]
fn test_offsetting_orientation2() {
    let subject = vec![
        clipper2::make_path64(&[20, 220, 280, 220, 280, 280, 20, 280]),
        clipper2::make_path64(&[0, 200, 0, 300, 300, 300, 300, 200]),
    ];
    let mut co = ClipperOffset::new(2.0, 0.0, false, true); // reverse_solution=true
    co.add_paths(&subject, JoinType::Round, EndType::Polygon);
    let mut solution = Paths64::new();
    co.execute(5.0, &mut solution);
    assert_eq!(solution.len(), 2);
    // ReverseSolution=true reverses output orientation
    assert_ne!(is_positive(&subject[1]), is_positive(&solution[0]));
}

// ==========================================================================
// From TestOffsets.cpp - TestOffsets2 (#448 & #456)
// ==========================================================================

#[test]
fn test_offsets2_issue_448_456() {
    let scale = 10.0;
    let delta = 10.0 * scale;
    let arc_tol = 0.25 * scale;
    let subject_raw = vec![clipper2::make_path64(&[50, 50, 100, 50, 100, 150, 50, 150, 0, 100])];
    let mut err = 0;
    let subject: Paths64 = scale_paths(&subject_raw, scale, scale, &mut err);
    assert_eq!(err, 0);
    let mut c = ClipperOffset::new(2.0, arc_tol, false, false);
    c.add_paths(&subject, JoinType::Round, EndType::Polygon);
    let mut solution = Paths64::new();
    c.execute(delta, &mut solution);
    assert!(!solution.is_empty());
    assert!(solution[0].len() <= 21);
}

// ==========================================================================
// TestOffsets4 - see #482
// ==========================================================================

#[test]
fn test_offsets4_issue_482() {
    let paths = vec![vec![
        Point64::new(0, 0), Point64::new(20000, 200), Point64::new(40000, 0),
        Point64::new(40000, 50000), Point64::new(0, 50000), Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths, -5000.0, JoinType::Square, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution[0].len(), 5);

    let paths = vec![vec![
        Point64::new(0, 0), Point64::new(20000, 400), Point64::new(40000, 0),
        Point64::new(40000, 50000), Point64::new(0, 50000), Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths, -5000.0, JoinType::Square, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution[0].len(), 5);

    let paths = vec![vec![
        Point64::new(0, 0), Point64::new(20000, 400), Point64::new(40000, 0),
        Point64::new(40000, 50000), Point64::new(0, 50000), Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths, -5000.0, JoinType::Round, EndType::Polygon, 2.0, 100.0,
    );
    assert!(solution[0].len() > 5);

    let paths = vec![vec![
        Point64::new(0, 0), Point64::new(20000, 1500), Point64::new(40000, 0),
        Point64::new(40000, 50000), Point64::new(0, 50000), Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths, -5000.0, JoinType::Round, EndType::Polygon, 2.0, 100.0,
    );
    assert!(solution[0].len() > 5);
}

// ==========================================================================
// TestOffsets7 - #593 & #715
// ==========================================================================

#[test]
fn test_offsets7_issue_593_715() {
    // Shrink 100x100 square by 50 -> should disappear
    let mut subject = vec![clipper2::make_path64(&[0, 0, 100, 0, 100, 100, 0, 100])];
    let solution = clipper2::inflate_paths_64(
        &subject, -50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 0);

    // Square with hole, inflated by 10 -> should merge to 1 path
    subject.push(clipper2::make_path64(&[40, 60, 60, 60, 60, 40, 40, 40]));
    let solution = clipper2::inflate_paths_64(
        &subject, 10.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);

    // Reverse both paths, inflate by 10 -> should still be 1
    let mut reversed_subject = subject.clone();
    reversed_subject[0].reverse();
    reversed_subject[1].reverse();
    let solution = clipper2::inflate_paths_64(
        &reversed_subject, 10.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);

    // Just the reversed outer, shrink by 50 -> should disappear
    let single_reversed = vec![reversed_subject[0].clone()];
    let solution = clipper2::inflate_paths_64(
        &single_reversed, -50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 0);
}

// ==========================================================================
// TestOffsets9 - #733 - Orientation matching
// ==========================================================================

#[test]
fn test_offsets9_issue_733() {
    // Positive orientation subject
    let subject = vec![clipper2::make_path64(&[100, 100, 200, 100, 200, 400, 100, 400])];
    let solution = clipper2::inflate_paths_64(
        &subject, 50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);
    assert!(is_positive(&solution[0]));

    // Reverse subject -> solution should also be reversed, but area still larger
    let mut rev_subject = subject.clone();
    rev_subject[0].reverse();
    let solution = clipper2::inflate_paths_64(
        &rev_subject, 50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);
    assert!(area(&solution[0]).abs() > area(&rev_subject[0]).abs());
    assert!(!is_positive(&solution[0]));

    // With reverse_solution = true
    let mut co = ClipperOffset::new(2.0, 0.0, false, true);
    co.add_paths(&rev_subject, JoinType::Miter, EndType::Polygon);
    let mut solution = Paths64::new();
    co.execute(50.0, &mut solution);
    assert_eq!(solution.len(), 1);
    assert!(area(&solution[0]).abs() > area(&rev_subject[0]).abs());
    assert!(is_positive(&solution[0]));

    // Add a hole (reverse orientation to outer)
    let mut subject_with_hole = rev_subject.clone();
    subject_with_hole.push(clipper2::make_path64(&[130, 130, 170, 130, 170, 370, 130, 370]));
    let solution = clipper2::inflate_paths_64(
        &subject_with_hole, 30.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 1);
    assert!(!is_positive(&solution[0]));

    // With reverse_solution on the holed subject
    co.clear();
    co.add_paths(&subject_with_hole, JoinType::Miter, EndType::Polygon);
    let mut solution = Paths64::new();
    co.execute(30.0, &mut solution);
    assert_eq!(solution.len(), 1);
    assert!(is_positive(&solution[0]));

    // Shrink holed subject by 15 -> should disappear
    let solution = clipper2::inflate_paths_64(
        &subject_with_hole, -15.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert_eq!(solution.len(), 0);
}

// ==========================================================================
// TestOffsets11 - see #405
// ==========================================================================

#[test]
fn test_offsets11_issue_405() {
    let subject: Vec<Vec<PointD>> = vec![clipper2::make_path_d(&[
        -1.0, -1.0, -1.0, 11.0, 11.0, 11.0, 11.0, -1.0,
    ])];
    let solution = clipper2::inflate_paths_d(
        &subject, -50.0, JoinType::Miter, EndType::Polygon, 2.0, 2, 0.0,
    );
    assert!(solution.is_empty());
}

// ==========================================================================
// TestOffsets12 - see #873
// ==========================================================================

#[test]
fn test_offsets12_issue_873() {
    let subject = vec![clipper2::make_path64(&[
        667680768, -36382704, 737202688, -87034880, 742581888, -86055680, 747603968, -84684800,
    ])];
    let solution = clipper2::inflate_paths_64(
        &subject, -249561088.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    assert!(solution.is_empty());
}

// ==========================================================================
// TestOffsets13 - see #965
// ==========================================================================

#[test]
fn test_offsets13_issue_965() {
    let subject1 = vec![vec![
        Point64::new(0, 0),
        Point64::new(0, 10),
        Point64::new(10, 0),
    ]];
    let delta = 2.0;
    let solution1 = clipper2::inflate_paths_64(
        &subject1, delta, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    let area1 = area_paths(&solution1).abs();
    assert_eq!(area1, 122.0);

    // Adding a single-point path should not change the solution
    let subject2 = vec![
        vec![
            Point64::new(0, 0),
            Point64::new(0, 10),
            Point64::new(10, 0),
        ],
        vec![Point64::new(0, 20)],
    ];
    let solution2 = clipper2::inflate_paths_64(
        &subject2, delta, JoinType::Miter, EndType::Polygon, 2.0, 0.0,
    );
    let area2 = area_paths(&solution2).abs();
    assert_eq!(area2, 122.0);
}
