// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestRectClip.cpp
//
// These are integration tests matching the original GoogleTest suite.

use clipper2::core::*;
use clipper2::rectclip::*;

#[test]
fn test_rectclip_basic() {
    let rect = Rect64::new(100, 100, 700, 500);
    let clp = vec![rect.as_path()];

    // Subject == rect -> solution area == subject area
    let sub = vec![clipper2::make_path64(&[
        100, 100, 700, 100, 700, 500, 100, 500,
    ])];
    let sol = clipper2::rect_clip_64(&rect, &sub);
    assert!((area_paths(&sol) - area_paths(&sub)).abs() < 1.0);

    // Subject slightly inside -> solution area == subject area
    let sub = vec![clipper2::make_path64(&[
        110, 110, 700, 100, 700, 500, 100, 500,
    ])];
    let sol = clipper2::rect_clip_64(&rect, &sub);
    assert!((area_paths(&sol) - area_paths(&sub)).abs() < 1.0);

    // Subject extends outside -> solution area == clip area
    let sub = vec![clipper2::make_path64(&[
        90, 90, 700, 100, 700, 500, 100, 500,
    ])];
    let sol = clipper2::rect_clip_64(&rect, &sub);
    assert!((area_paths(&sol) - area_paths(&clp)).abs() < 1.0);

    // Subject fully inside rect
    let sub = vec![clipper2::make_path64(&[
        110, 110, 690, 110, 690, 490, 110, 490,
    ])];
    let sol = clipper2::rect_clip_64(&rect, &sub);
    assert!((area_paths(&sol) - area_paths(&sub)).abs() < 1.0);

    // Subject touching rect edge (outside) -> empty
    let rect2 = Rect64::new(390, 290, 410, 310);
    let sub = vec![clipper2::make_path64(&[
        410, 290, 500, 290, 500, 310, 410, 310,
    ])];
    let sol = clipper2::rect_clip_64(&rect2, &sub);
    assert!(sol.is_empty());

    // Triangle outside rect
    let sub = vec![clipper2::make_path64(&[430, 290, 470, 330, 390, 330])];
    let sol = clipper2::rect_clip_64(&rect2, &sub);
    assert!(sol.is_empty());

    // Another triangle outside rect
    let sub = vec![clipper2::make_path64(&[450, 290, 480, 330, 450, 330])];
    let sol = clipper2::rect_clip_64(&rect2, &sub);
    assert!(sol.is_empty());

    // Complex polygon intersecting rect
    let sub = vec![clipper2::make_path64(&[
        208, 66, 366, 112, 402, 303, 234, 332, 233, 262, 243, 140, 215, 126, 40, 172,
    ])];
    let rect3 = Rect64::new(237, 164, 322, 248);
    let sol = clipper2::rect_clip_64(&rect3, &sub);
    let sol_bounds = get_bounds_paths(&sol);
    assert_eq!(sol_bounds.width(), rect3.width());
    assert_eq!(sol_bounds.height(), rect3.height());
}

#[test]
fn test_rectclip2_issue_597() {
    let rect = Rect64::new(54690, 0, 65628, 6000);
    let subject = vec![vec![
        Point64::new(700000, 6000),
        Point64::new(0, 6000),
        Point64::new(0, 5925),
        Point64::new(700000, 5925),
    ]];
    let solution = clipper2::rect_clip_64(&rect, &subject);
    assert_eq!(solution.len(), 1);
    assert_eq!(solution[0].len(), 4);
}

#[test]
fn test_rectclip3_issue_637() {
    let r = Rect64::new(-1800000000, -137573171, -1741475021, 3355443);
    let subject = vec![clipper2::make_path64(&[
        -1800000000,
        10005000,
        -1800000000,
        -5000,
        -1789994999,
        -5000,
        -1789994999,
        10005000,
    ])];
    let solution = clipper2::rect_clip_64(&r, &subject);
    assert_eq!(solution.len(), 1);
}

#[test]
fn test_rectclip_orientation_issue_864() {
    let rect = Rect64::new(1222, 1323, 3247, 3348);
    let subject = clipper2::make_path64(&[375, 1680, 1915, 4716, 5943, 586, 3987, 152]);
    let mut clip = RectClip64::new(rect);
    let paths = vec![subject.clone()];
    let solution = clip.execute(&paths);
    assert_eq!(solution.len(), 1);
    assert_eq!(is_positive(&subject), is_positive(&solution[0]));
}
