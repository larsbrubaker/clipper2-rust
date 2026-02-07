use super::*;

// ============================================================================
// RectClip64 tests
// ============================================================================

#[test]
fn test_rectclip64_new() {
    let rect = Rect64::new(10, 10, 100, 100);
    let rc = RectClip64::new(rect);
    assert_eq!(rc.rect, rect);
    assert_eq!(rc.rect_as_path.len(), 4);
    assert_eq!(rc.rect_mp, Point64::new(55, 55));
}

#[test]
fn test_rectclip64_empty_rect() {
    let rect = Rect64::new(0, 0, 0, 0);
    let mut rc = RectClip64::new(rect);
    let paths = vec![vec![
        Point64::new(10, 10),
        Point64::new(20, 10),
        Point64::new(20, 20),
    ]];
    let result = rc.execute(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_rectclip64_path_fully_inside() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rc = RectClip64::new(rect);
    let paths = vec![vec![
        Point64::new(10, 10),
        Point64::new(20, 10),
        Point64::new(20, 20),
        Point64::new(10, 20),
    ]];
    let result = rc.execute(&paths);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], paths[0]); // path returned unchanged
}

#[test]
fn test_rectclip64_path_fully_outside() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rc = RectClip64::new(rect);
    let paths = vec![vec![
        Point64::new(200, 200),
        Point64::new(300, 200),
        Point64::new(300, 300),
        Point64::new(200, 300),
    ]];
    let result = rc.execute(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_rectclip64_path_partially_inside() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rc = RectClip64::new(rect);
    // Triangle that extends beyond the right side
    let paths = vec![vec![
        Point64::new(50, 10),
        Point64::new(150, 50),
        Point64::new(50, 90),
    ]];
    let result = rc.execute(&paths);
    assert!(!result.is_empty());
    // All result points should be within or on the rect boundary
    for path in &result {
        for pt in path {
            assert!(
                pt.x >= 0 && pt.x <= 100 && pt.y >= 0 && pt.y <= 100,
                "Point {:?} is outside rect",
                pt
            );
        }
    }
}

#[test]
fn test_rectclip64_path_containing_rect() {
    let rect = Rect64::new(20, 20, 80, 80);
    let mut rc = RectClip64::new(rect);
    // Large square that fully contains the rect
    let paths = vec![vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
        Point64::new(0, 100),
    ]];
    let result = rc.execute(&paths);
    assert!(!result.is_empty());
    // The result should be approximately the rect itself
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].len(), 4);
}

#[test]
fn test_rectclip64_skip_small_paths() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rc = RectClip64::new(rect);
    // Path with less than 3 points should be skipped
    let paths = vec![vec![Point64::new(10, 10), Point64::new(20, 20)]];
    let result = rc.execute(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_rectclip64_multiple_paths() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rc = RectClip64::new(rect);
    let paths = vec![
        // Fully inside
        vec![
            Point64::new(10, 10),
            Point64::new(20, 10),
            Point64::new(20, 20),
            Point64::new(10, 20),
        ],
        // Fully outside
        vec![
            Point64::new(200, 200),
            Point64::new(300, 200),
            Point64::new(300, 300),
        ],
        // Partially inside
        vec![
            Point64::new(50, 50),
            Point64::new(150, 50),
            Point64::new(150, 150),
            Point64::new(50, 150),
        ],
    ];
    let result = rc.execute(&paths);
    assert!(result.len() >= 2); // at least the fully-inside and partially-inside paths
}

// ============================================================================
// RectClipLines64 tests
// ============================================================================

#[test]
fn test_rectcliplines64_new() {
    let rect = Rect64::new(10, 10, 100, 100);
    let rcl = RectClipLines64::new(rect);
    assert_eq!(rcl.rect, rect);
}

#[test]
fn test_rectcliplines64_empty_rect() {
    let rect = Rect64::new(0, 0, 0, 0);
    let mut rcl = RectClipLines64::new(rect);
    let paths = vec![vec![Point64::new(10, 10), Point64::new(20, 20)]];
    let result = rcl.execute(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_rectcliplines64_line_fully_inside() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rcl = RectClipLines64::new(rect);
    let paths = vec![vec![
        Point64::new(10, 10),
        Point64::new(20, 20),
        Point64::new(30, 10),
    ]];
    let result = rcl.execute(&paths);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].len(), 3);
}

#[test]
fn test_rectcliplines64_line_fully_outside() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rcl = RectClipLines64::new(rect);
    let paths = vec![vec![Point64::new(200, 200), Point64::new(300, 300)]];
    let result = rcl.execute(&paths);
    assert!(result.is_empty());
}

#[test]
fn test_rectcliplines64_line_crossing() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rcl = RectClipLines64::new(rect);
    // Line that enters and exits the rect
    let paths = vec![vec![Point64::new(-50, 50), Point64::new(150, 50)]];
    let result = rcl.execute(&paths);
    assert!(!result.is_empty());
    // The clipped line should start at x=0 and end at x=100
    for path in &result {
        for pt in path {
            assert!(
                pt.x >= 0 && pt.x <= 100 && pt.y >= 0 && pt.y <= 100,
                "Point {:?} is outside rect",
                pt
            );
        }
    }
}

#[test]
fn test_rectcliplines64_line_exiting() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rcl = RectClipLines64::new(rect);
    // Line starting inside, exiting
    let paths = vec![vec![Point64::new(50, 50), Point64::new(150, 50)]];
    let result = rcl.execute(&paths);
    assert!(!result.is_empty());
}

#[test]
fn test_rectcliplines64_line_entering() {
    let rect = Rect64::new(0, 0, 100, 100);
    let mut rcl = RectClipLines64::new(rect);
    // Line starting outside, entering
    let paths = vec![vec![Point64::new(-50, 50), Point64::new(50, 50)]];
    let result = rcl.execute(&paths);
    assert!(!result.is_empty());
}

// ============================================================================
// Helper function tests
// ============================================================================

#[test]
fn test_heading_clockwise() {
    assert!(heading_clockwise(Location::Left, Location::Top));
    assert!(heading_clockwise(Location::Top, Location::Right));
    assert!(heading_clockwise(Location::Right, Location::Bottom));
    assert!(heading_clockwise(Location::Bottom, Location::Left));
    assert!(!heading_clockwise(Location::Left, Location::Bottom));
    assert!(!heading_clockwise(Location::Top, Location::Left));
}

#[test]
fn test_are_opposites() {
    assert!(are_opposites(Location::Left, Location::Right));
    assert!(are_opposites(Location::Right, Location::Left));
    assert!(are_opposites(Location::Top, Location::Bottom));
    assert!(are_opposites(Location::Bottom, Location::Top));
    assert!(!are_opposites(Location::Left, Location::Top));
    assert!(!are_opposites(Location::Left, Location::Bottom));
}

#[test]
fn test_get_adjacent_location() {
    assert_eq!(get_adjacent_location(Location::Left, true), Location::Top);
    assert_eq!(get_adjacent_location(Location::Top, true), Location::Right);
    assert_eq!(
        get_adjacent_location(Location::Right, true),
        Location::Bottom
    );
    assert_eq!(
        get_adjacent_location(Location::Bottom, true),
        Location::Left
    );

    assert_eq!(
        get_adjacent_location(Location::Left, false),
        Location::Bottom
    );
    assert_eq!(get_adjacent_location(Location::Top, false), Location::Left);
    assert_eq!(get_adjacent_location(Location::Right, false), Location::Top);
    assert_eq!(
        get_adjacent_location(Location::Bottom, false),
        Location::Right
    );
}

#[test]
fn test_get_edges_for_pt() {
    let rect = Rect64::new(10, 10, 100, 100);
    assert_eq!(get_edges_for_pt(&Point64::new(10, 50), &rect), 1); // left
    assert_eq!(get_edges_for_pt(&Point64::new(100, 50), &rect), 4); // right
    assert_eq!(get_edges_for_pt(&Point64::new(50, 10), &rect), 2); // top
    assert_eq!(get_edges_for_pt(&Point64::new(50, 100), &rect), 8); // bottom
    assert_eq!(get_edges_for_pt(&Point64::new(10, 10), &rect), 3); // left+top
    assert_eq!(get_edges_for_pt(&Point64::new(100, 100), &rect), 12); // right+bottom
    assert_eq!(get_edges_for_pt(&Point64::new(50, 50), &rect), 0); // inside
}

#[test]
fn test_start_locs_are_clockwise() {
    // Clockwise: Left -> Top -> Right -> Bottom
    let locs = vec![
        Location::Left,
        Location::Top,
        Location::Right,
        Location::Bottom,
    ];
    assert!(start_locs_are_clockwise(&locs));

    // Counter-clockwise: Left -> Bottom -> Right -> Top
    let locs = vec![
        Location::Left,
        Location::Bottom,
        Location::Right,
        Location::Top,
    ];
    assert!(!start_locs_are_clockwise(&locs));
}

#[test]
fn test_path1_contains_path2() {
    let outer = vec![
        Point64::new(0, 0),
        Point64::new(100, 0),
        Point64::new(100, 100),
        Point64::new(0, 100),
    ];
    let inner = vec![
        Point64::new(20, 20),
        Point64::new(80, 20),
        Point64::new(80, 80),
        Point64::new(20, 80),
    ];
    assert!(path1_contains_path2(&outer, &inner));
    assert!(!path1_contains_path2(&inner, &outer));
}

#[test]
fn test_get_segment_intersection_crossing() {
    let mut ip = Point64::new(0, 0);
    // Two crossing segments
    let result = get_segment_intersection(
        Point64::new(0, 0),
        Point64::new(10, 10),
        Point64::new(0, 10),
        Point64::new(10, 0),
        &mut ip,
    );
    assert!(result);
    assert_eq!(ip, Point64::new(5, 5));
}

#[test]
fn test_get_segment_intersection_collinear() {
    let mut ip = Point64::new(0, 0);
    // Collinear segments
    let result = get_segment_intersection(
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(5, 0),
        Point64::new(15, 0),
        &mut ip,
    );
    // Collinear with res1==0 and res2==0 should return false
    assert!(!result);
}

#[test]
fn test_get_segment_intersection_parallel() {
    let mut ip = Point64::new(0, 0);
    // Parallel but not touching
    let result = get_segment_intersection(
        Point64::new(0, 0),
        Point64::new(10, 0),
        Point64::new(0, 5),
        Point64::new(10, 5),
        &mut ip,
    );
    assert!(!result);
}

#[test]
fn test_rectclip64_triangle_touching_corner_should_be_empty() {
    // Triangle only touches the rect at corner point (410,310) which is bottom-right
    // rect = (390, 290, 410, 310)
    // triangle = (430,290), (470,330), (390,330)
    // C++ returns empty (0 paths) because the triangle is outside the rect,
    // only touching at the corner point (410,310).
    let rect = Rect64::new(390, 290, 410, 310);
    let mut rc = RectClip64::new(rect);
    let triangle = vec![
        Point64::new(430, 290),
        Point64::new(470, 330),
        Point64::new(390, 330),
    ];
    let result = rc.execute(&vec![triangle]);
    assert!(
        result.is_empty(),
        "Expected empty result for triangle that only touches rect at a corner, got {} paths: {:?}",
        result.len(),
        result
    );
}
