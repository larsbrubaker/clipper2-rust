// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestPolytreeIntersection.cpp, TestPolytreeUnion.cpp,
// TestPolytreeHoles.cpp
//
// These are integration tests matching the original GoogleTest suite.

use clipper2_rust::core::*;
use clipper2_rust::engine::ClipType;
use clipper2_rust::engine_public::*;
use clipper2_rust::utils::file_io;

fn test_data_path(name: &str) -> String {
    format!("{}/Tests/data/{}", env!("CARGO_MANIFEST_DIR"), name)
}

/// Helper: recursively check if a point is contained in a polytree
/// Returns a counter: +1 for each outer containing the point, -1 for each hole
fn poly_path_contains_point(tree: &PolyTree64, node_idx: usize, pt: Point64, counter: &mut i32) {
    let node = &tree.nodes[node_idx];
    if !node.polygon().is_empty()
        && point_in_polygon(pt, node.polygon()) != PointInPolygonResult::IsOutside
    {
        if tree.is_hole(node_idx) {
            *counter -= 1;
        } else {
            *counter += 1;
        }
    }
    for &child_idx in node.children() {
        poly_path_contains_point(tree, child_idx, pt, counter);
    }
}

fn polytree_contains_point(tree: &PolyTree64, pt: Point64) -> bool {
    let mut counter = 0i32;
    let root = &tree.nodes[0];
    for &child_idx in root.children() {
        poly_path_contains_point(tree, child_idx, pt, &mut counter);
    }
    assert!(counter >= 0, "Point inside more holes than outers");
    counter != 0
}

/// Get total area of all polygons in the polytree
fn get_polytree_area(tree: &PolyTree64) -> f64 {
    fn get_poly_path_area(tree: &PolyTree64, node_idx: usize, result: &mut f64) {
        let node = &tree.nodes[node_idx];
        *result += area(node.polygon());
        for &child_idx in node.children() {
            get_poly_path_area(tree, child_idx, result);
        }
    }
    let mut result = 0.0;
    let root = &tree.nodes[0];
    for &child_idx in root.children() {
        get_poly_path_area(tree, child_idx, &mut result);
    }
    result
}

/// Helper: get children count of PolyTree64 root (equivalent to C++ `solution.Count()`)
fn root_count(tree: &PolyTree64) -> usize {
    tree.root().count()
}

/// Helper: get child node of root by index (equivalent to C++ `solution[idx]`)
fn root_child(tree: &PolyTree64, idx: usize) -> &PolyPath64 {
    let child_idx = tree.root().children()[idx];
    &tree.nodes[child_idx]
}

/// Helper: get child node of a node by index (equivalent to C++ `node[idx]`)
fn node_child<'a>(tree: &'a PolyTree64, node: &PolyPath64, idx: usize) -> &'a PolyPath64 {
    let child_idx = node.children()[idx];
    &tree.nodes[child_idx]
}

// ==========================================================================
// From TestPolytreeIntersection.cpp
// ==========================================================================

#[test]
fn test_polytree_intersection() {
    let mut clipper = Clipper64::new();
    let subject = vec![clipper2_rust::make_path64(&[0, 0, 0, 5, 5, 5, 5, 0])];
    clipper.add_subject(&subject);
    let clip = vec![clipper2_rust::make_path64(&[1, 1, 1, 6, 6, 6, 6, 1])];
    clipper.add_clip(&clip);
    let mut solution = PolyTree64::new();
    let mut open_paths = Paths64::new();
    let fr = if is_positive(&subject[0]) {
        FillRule::Positive
    } else {
        FillRule::Negative
    };
    clipper.execute_tree(ClipType::Intersection, fr, &mut solution, &mut open_paths);
    assert_eq!(open_paths.len(), 0);
    assert_eq!(root_count(&solution), 1);
    assert_eq!(root_child(&solution, 0).polygon().len(), 4);
}

// ==========================================================================
// From TestPolytreeUnion.cpp
// ==========================================================================

#[test]
fn test_polytree_union() {
    let subject = vec![
        clipper2_rust::make_path64(&[0, 0, 0, 5, 5, 5, 5, 0]),
        clipper2_rust::make_path64(&[1, 1, 1, 6, 6, 6, 6, 1]),
    ];
    let mut clipper = Clipper64::new();
    clipper.add_subject(&subject);
    let mut solution = PolyTree64::new();
    let mut open_paths = Paths64::new();
    if is_positive(&subject[0]) {
        clipper.execute_tree(
            ClipType::Union,
            FillRule::Positive,
            &mut solution,
            &mut open_paths,
        );
    } else {
        clipper.set_reverse_solution(true);
        clipper.execute_tree(
            ClipType::Union,
            FillRule::Negative,
            &mut solution,
            &mut open_paths,
        );
    }
    assert_eq!(open_paths.len(), 0);
    assert_eq!(root_count(&solution), 1);
    let child = root_child(&solution, 0);
    assert_eq!(child.polygon().len(), 8);
    assert_eq!(is_positive(&subject[0]), is_positive(child.polygon()));
}

#[test]
fn test_polytree_union2_issue_987() {
    let subject = vec![
        clipper2_rust::make_path64(&[534, 1024, 534, -800, 1026, -800, 1026, 1024]),
        clipper2_rust::make_path64(&[1, 1024, 8721, 1024, 8721, 1920, 1, 1920]),
        clipper2_rust::make_path64(&[30, 1024, 30, -800, 70, -800, 70, 1024]),
        clipper2_rust::make_path64(&[1, 1024, 1, -1024, 3841, -1024, 3841, 1024]),
        clipper2_rust::make_path64(&[3900, -1024, 6145, -1024, 6145, 1024, 3900, 1024]),
        clipper2_rust::make_path64(&[5884, 1024, 5662, 1024, 5662, -1024, 5884, -1024]),
        clipper2_rust::make_path64(&[534, 1024, 200, 1024, 200, -800, 534, -800]),
        clipper2_rust::make_path64(&[200, -800, 200, 1024, 70, 1024, 70, -800]),
        clipper2_rust::make_path64(&[1200, 1920, 1313, 1920, 1313, -800, 1200, -800]),
        clipper2_rust::make_path64(&[6045, -800, 6045, 1024, 5884, 1024, 5884, -800]),
    ];
    let mut clipper = Clipper64::new();
    clipper.add_subject(&subject);
    let mut solution = PolyTree64::new();
    let mut open_paths = Paths64::new();
    clipper.execute_tree(
        ClipType::Union,
        FillRule::EvenOdd,
        &mut solution,
        &mut open_paths,
    );
    assert_eq!(root_count(&solution), 1);
    assert_eq!(root_child(&solution, 0).count(), 1);
}

#[test]
fn test_polytree_union3() {
    let subject = vec![clipper2_rust::make_path64(&[
        -120927680, 590077597, -120919386, 590077307, -120919432, 590077309, -120919451, 590077309,
        -120919455, 590077310, -120099297, 590048669, -120928004, 590077608, -120902794, 590076728,
        -120919444, 590077309, -120919450, 590077309, -120919842, 590077323, -120922852, 590077428,
        -120902452, 590076716, -120902455, 590076716, -120912590, 590077070, 11914491, 249689797,
    ])];
    let mut clipper = Clipper64::new();
    clipper.add_subject(&subject);
    let mut solution = PolyTree64::new();
    // This should not crash
    clipper.execute_tree(
        ClipType::Union,
        FillRule::EvenOdd,
        &mut solution,
        &mut Paths64::new(),
    );
}

// ==========================================================================
// From TestPolytreeHoles.cpp - tests not requiring data files
// ==========================================================================

#[test]
fn test_polytree_holes3() {
    let subject = vec![clipper2_rust::make_path64(&[
        1072, 501, 1072, 501, 1072, 539, 1072, 539, 1072, 539, 870, 539, 870, 539, 870, 539, 870,
        520, 894, 520, 898, 524, 911, 524, 915, 520, 915, 520, 936, 520, 940, 524, 953, 524, 957,
        520, 957, 520, 978, 520, 983, 524, 995, 524, 1000, 520, 1021, 520, 1025, 524, 1038, 524,
        1042, 520, 1038, 516, 1025, 516, 1021, 520, 1000, 520, 995, 516, 983, 516, 978, 520, 957,
        520, 953, 516, 940, 516, 936, 520, 915, 520, 911, 516, 898, 516, 894, 520, 870, 520, 870,
        516, 870, 501, 870, 501, 870, 501, 1072, 501,
    ])];
    let clip = vec![clipper2_rust::make_path64(&[
        870, 501, 971, 501, 971, 539, 870, 539,
    ])];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.add_clip(&clip);
    let mut solution = PolyTree64::new();
    c.execute_tree(
        ClipType::Intersection,
        FillRule::NonZero,
        &mut solution,
        &mut Paths64::new(),
    );
    assert_eq!(root_count(&solution), 1);
    assert_eq!(root_child(&solution, 0).count(), 2);
}

#[test]
fn test_polytree_holes4_issue_618() {
    let subject = vec![
        clipper2_rust::make_path64(&[
            50, 500, 50, 300, 100, 300, 100, 350, 150, 350, 150, 250, 200, 250, 200, 450, 350, 450,
            350, 200, 400, 200, 400, 225, 450, 225, 450, 175, 400, 175, 400, 200, 350, 200, 350,
            175, 200, 175, 200, 250, 150, 250, 150, 200, 100, 200, 100, 300, 50, 300, 50, 125, 500,
            125, 500, 500,
        ]),
        clipper2_rust::make_path64(&[250, 425, 250, 375, 300, 375, 300, 425]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    let mut solution = PolyTree64::new();
    c.execute_tree(
        ClipType::Union,
        FillRule::NonZero,
        &mut solution,
        &mut Paths64::new(),
    );
    // Polytree root -> 1 polygon with 3 holes
    assert_eq!(root_count(&solution), 1);
    assert_eq!(root_child(&solution, 0).count(), 3);
}

#[test]
fn test_polytree_holes5() {
    let subject = vec![clipper2_rust::make_path64(&[
        0, 30, 400, 30, 400, 100, 0, 100,
    ])];
    let clip = vec![
        clipper2_rust::make_path64(&[20, 30, 30, 30, 30, 150, 20, 150]),
        clipper2_rust::make_path64(&[
            200, 0, 300, 0, 300, 30, 280, 30, 280, 20, 220, 20, 220, 30, 200, 30,
        ]),
        clipper2_rust::make_path64(&[200, 50, 300, 50, 300, 80, 200, 80]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.add_clip(&clip);
    let mut tree = PolyTree64::new();
    c.execute_tree(
        ClipType::Xor,
        FillRule::NonZero,
        &mut tree,
        &mut Paths64::new(),
    );
    // Polytree with 3 polygons, 3rd one has 2 holes
    assert_eq!(root_count(&tree), 3);
    assert_eq!(root_child(&tree, 2).count(), 2);
}

#[test]
fn test_polytree_holes6_issue_618() {
    let subject = vec![
        clipper2_rust::make_path64(&[150, 50, 200, 50, 200, 100, 150, 100]),
        clipper2_rust::make_path64(&[125, 100, 150, 100, 150, 150, 125, 150]),
        clipper2_rust::make_path64(&[225, 50, 300, 50, 300, 80, 225, 80]),
        clipper2_rust::make_path64(&[
            225, 100, 300, 100, 300, 150, 275, 150, 275, 175, 260, 175, 260, 250, 235, 250, 235,
            300, 275, 300, 275, 275, 300, 275, 300, 350, 225, 350,
        ]),
        clipper2_rust::make_path64(&[300, 150, 350, 150, 350, 175, 300, 175]),
    ];
    let clip = vec![
        clipper2_rust::make_path64(&[0, 0, 400, 0, 400, 50, 0, 50]),
        clipper2_rust::make_path64(&[0, 100, 400, 100, 400, 150, 0, 150]),
        clipper2_rust::make_path64(&[260, 175, 325, 175, 325, 275, 260, 275]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    c.add_clip(&clip);
    let mut tree = PolyTree64::new();
    c.execute_tree(
        ClipType::Xor,
        FillRule::NonZero,
        &mut tree,
        &mut Paths64::new(),
    );
    // Polytree with 3 polygons, 3rd has 1 hole
    assert_eq!(root_count(&tree), 3);
    assert_eq!(root_child(&tree, 2).count(), 1);
}

#[test]
fn test_polytree_holes7_issue_618() {
    let subject = vec![
        clipper2_rust::make_path64(&[
            0, 0, 100000, 0, 100000, 100000, 200000, 100000, 200000, 0, 300000, 0, 300000, 200000,
            0, 200000,
        ]),
        clipper2_rust::make_path64(&[0, 0, 0, -100000, 250000, -100000, 250000, 0]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    let mut polytree = PolyTree64::new();
    c.execute_tree(
        ClipType::Union,
        FillRule::NonZero,
        &mut polytree,
        &mut Paths64::new(),
    );
    assert_eq!(root_count(&polytree), 1);
    assert_eq!(root_child(&polytree, 0).count(), 1);
}

#[test]
fn test_polytree_holes8_issue_942() {
    let subject = vec![
        clipper2_rust::make_path64(&[1588700, -8717600, 1616200, -8474800, 1588700, -8474800]),
        clipper2_rust::make_path64(&[
            13583800, -15601600, 13582800, -15508500, 13555300, -15508500, 13555500, -15182200,
            13010900, -15185400,
        ]),
        clipper2_rust::make_path64(&[956700, -3092300, 1152600, 3147400, 25600, 3151700]),
        clipper2_rust::make_path64(&[
            22575900, -16604000, 31286800, -12171900, 31110200, 4882800, 30996200, 4826300,
            30414400, 5447400, 30260000, 5391500, 29662200, 5805400, 28844500, 5337900, 28435000,
            5789300, 27721400, 5026400, 22876300, 5034300, 21977700, 4414900, 21148000, 4654700,
            20917600, 4653400, 19334300, 12411000, -2591700, 12177200, 53200, 3151100, -2564300,
            12149800, 7819400, 4692400, 10116000, 5228600, 6975500, 3120100, 7379700, 3124700,
            11037900, 596200, 12257000, 2587800, 12257000, 596200, 15227300, 2352700, 18444400,
            1112100, 19961100, 5549400, 20173200, 5078600, 20330000, 5079300, 20970200, 4544300,
            20989600, 4563700, 19465500, 1112100, 21611600, 4182100, 22925100, 1112200, 22952700,
            1637200, 23059000, 1112200, 24908100, 4181200, 27070100, 3800600, 27238000, 3800700,
            28582200, 520300, 29367800, 1050100, 29291400, 179400, 29133700, 360700, 29056700,
            312600, 29121900, 332500, 29269900, 162300, 28941400, 213100, 27491300, -3041500,
            27588700, -2997800, 22104900, -16142800, 13010900, -15603000, 13555500, -15182200,
            13555300, -15508500, 13582800, -15508500, 13583100, -15154700, 1588700, -8822800,
            1588700, -8379900, 1588700, -8474800, 1616200, -8474800, 1003900, -630100, 1253300,
            -12284500, 12983400, -16239900,
        ]),
        clipper2_rust::make_path64(&[198200, 12149800, 1010600, 12149800, 1011500, 11859600]),
        clipper2_rust::make_path64(&[21996700, -7432000, 22096700, -7432000, 22096700, -7332000]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    let mut solution = PolyTree64::new();
    c.execute_tree(
        ClipType::Union,
        FillRule::NonZero,
        &mut solution,
        &mut Paths64::new(),
    );
    let child0 = root_child(&solution, 0);
    assert_eq!(root_count(&solution), 1);
    assert_eq!(child0.count(), 2);
    let child0_child1 = node_child(&solution, child0, 1);
    assert_eq!(child0_child1.count(), 1);
}

#[test]
fn test_polytree_holes9_issue_957() {
    let subject = vec![
        clipper2_rust::make_path64(&[
            77910, 46865, 78720, 46865, 78720, 48000, 77910, 48000, 77910, 46865,
        ]),
        clipper2_rust::make_path64(&[
            82780, 53015, 93600, 53015, 93600, 54335, 82780, 54335, 82780, 53015,
        ]),
        clipper2_rust::make_path64(&[
            82780, 48975, 84080, 48975, 84080, 53015, 82780, 53015, 82780, 48975,
        ]),
        clipper2_rust::make_path64(&[
            77910, 48000, 84080, 48000, 84080, 48975, 77910, 48975, 77910, 48000,
        ]),
        clipper2_rust::make_path64(&[
            89880, 40615, 90700, 40615, 90700, 46865, 89880, 46865, 89880, 40615,
        ]),
        clipper2_rust::make_path64(&[
            92700, 54335, 93600, 54335, 93600, 61420, 92700, 61420, 92700, 54335,
        ]),
        clipper2_rust::make_path64(&[
            78950, 47425, 84080, 47425, 84080, 47770, 78950, 47770, 78950, 47425,
        ]),
        clipper2_rust::make_path64(&[
            82780, 61420, 93600, 61420, 93600, 62435, 82780, 62435, 82780, 61420,
        ]),
        clipper2_rust::make_path64(&[
            101680, 63085, 100675, 63085, 100675, 47770, 100680, 47770, 100680, 40615, 101680,
            40615, 101680, 63085,
        ]),
        clipper2_rust::make_path64(&[
            76195, 39880, 89880, 39880, 89880, 41045, 76195, 41045, 76195, 39880,
        ]),
        clipper2_rust::make_path64(&[
            85490, 56145, 90520, 56145, 90520, 59235, 85490, 59235, 85490, 56145,
        ]),
        clipper2_rust::make_path64(&[
            89880, 39880, 101680, 39880, 101680, 40615, 89880, 40615, 89880, 39880,
        ]),
        clipper2_rust::make_path64(&[
            89880, 46865, 100680, 46865, 100680, 47770, 89880, 47770, 89880, 46865,
        ]),
        clipper2_rust::make_path64(&[
            82780, 54335, 83280, 54335, 83280, 61420, 82780, 61420, 82780, 54335,
        ]),
        clipper2_rust::make_path64(&[
            76195, 41045, 76855, 41045, 76855, 62665, 76195, 62665, 76195, 41045,
        ]),
        clipper2_rust::make_path64(&[
            76195, 62665, 100675, 62665, 100675, 63085, 76195, 63085, 76195, 62665,
        ]),
        clipper2_rust::make_path64(&[
            82780, 41045, 84080, 41045, 84080, 47425, 82780, 47425, 82780, 41045,
        ]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    let mut solution = PolyTree64::new();
    c.execute_tree(
        ClipType::Union,
        FillRule::NonZero,
        &mut solution,
        &mut Paths64::new(),
    );
    // 1 polygon with 2 holes, first hole has 1 nested polygon with 1 hole, that has 1 nested polygon
    assert_eq!(root_count(&solution), 1);
    let child0 = root_child(&solution, 0);
    assert_eq!(child0.count(), 2);
    let child0_child0 = node_child(&solution, child0, 0);
    assert_eq!(child0_child0.count(), 1);
}

#[test]
fn test_polytree_holes10_issue_973() {
    let subject = vec![
        clipper2_rust::make_path64(&[0, 0, 79530, 0, 79530, 940, 0, 940, 0, 0]),
        clipper2_rust::make_path64(&[0, 33360, 79530, 33360, 79530, 34300, 0, 34300, 0, 33360]),
        clipper2_rust::make_path64(&[
            78470, 940, 79530, 940, 79530, 33360, 78470, 33360, 78470, 940,
        ]),
        clipper2_rust::make_path64(&[0, 940, 940, 940, 940, 33360, 0, 33360, 0, 940]),
        clipper2_rust::make_path64(&[
            29290, 940, 30350, 940, 30350, 33360, 29290, 33360, 29290, 940,
        ]),
    ];
    let mut c = Clipper64::new();
    c.add_subject(&subject);
    let mut solution = PolyTree64::new();
    c.execute_tree(
        ClipType::Union,
        FillRule::NonZero,
        &mut solution,
        &mut Paths64::new(),
    );
    // 1 polygon with 2 holes
    assert_eq!(root_count(&solution), 1);
    assert_eq!(root_child(&solution, 0).count(), 2);
}

// ==========================================================================
// TestPolytreeHoles1 - loads from PolytreeHoleOwner.txt
// ==========================================================================

#[test]
fn test_polytree_holes1() {
    let path = test_data_path("PolytreeHoleOwner.txt");
    let data = file_io::load_test_num(&path, 1).expect("Failed to load PolytreeHoleOwner.txt");

    let mut c = Clipper64::new();
    c.add_subject(&data.subj);
    c.add_open_subject(&data.subj_open);
    c.add_clip(&data.clip);
    let mut solution = PolyTree64::new();
    let mut solution_open = Paths64::new();
    c.execute_tree(
        data.clip_type,
        data.fill_rule,
        &mut solution,
        &mut solution_open,
    );
    assert!(
        clipper2_rust::check_polytree_fully_contains_children(&solution),
        "TestPolytreeHoles1: children not fully contained by parents"
    );
}

// ==========================================================================
// TestPolytreeHoles2 - loads from PolytreeHoleOwner2.txt, tests with Negative fill rule
// ==========================================================================

#[test]
fn test_polytree_holes2() {
    let path = test_data_path("PolytreeHoleOwner2.txt");
    let data = file_io::load_test_num(&path, 1).expect("Failed to load PolytreeHoleOwner2.txt");

    let points_of_interest_outside = vec![
        Point64::new(21887, 10420),
        Point64::new(21726, 10825),
        Point64::new(21662, 10845),
        Point64::new(21617, 10890),
    ];
    // Confirm each outside point is not inside any subject polygon
    for poi in &points_of_interest_outside {
        let mut count = 0;
        for path in &data.subj {
            if point_in_polygon(*poi, path) != PointInPolygonResult::IsOutside {
                count += 1;
            }
        }
        assert_eq!(count, 0, "Point {:?} should be outside all subjects", poi);
    }

    let points_of_interest_inside = vec![
        Point64::new(21887, 10430),
        Point64::new(21843, 10520),
        Point64::new(21810, 10686),
        Point64::new(21900, 10461),
    ];
    // Confirm each inside point is inside exactly one subject polygon
    for poi in &points_of_interest_inside {
        let mut count = 0;
        for path in &data.subj {
            if point_in_polygon(*poi, path) != PointInPolygonResult::IsOutside {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "Point {:?} should be inside exactly 1 subject",
            poi
        );
    }

    let mut c = Clipper64::new();
    c.add_subject(&data.subj);
    c.add_open_subject(&data.subj_open);
    c.add_clip(&data.clip);
    let mut solution_tree = PolyTree64::new();
    let mut solution_open = Paths64::new();
    c.execute_tree(
        data.clip_type,
        FillRule::Negative,
        &mut solution_tree,
        &mut solution_open,
    );
    let solution_paths = clipper2_rust::poly_tree_to_paths64(&solution_tree);
    assert!(
        !solution_paths.is_empty(),
        "TestPolytreeHoles2: solution should not be empty"
    );
    let subject_area = -area_paths(&data.subj); // negate (see fillrule)
    let solution_tree_area = get_polytree_area(&solution_tree);
    let solution_paths_area = area_paths(&solution_paths);

    // 1a. solution_paths_area should be smaller than subject_area
    assert!(
        solution_paths_area < subject_area,
        "Solution area ({}) should be < subject area ({})",
        solution_paths_area,
        subject_area
    );
    // 1b. but not too much smaller
    assert!(
        solution_paths_area > subject_area * 0.92,
        "Solution area ({}) should be > 92% of subject area ({})",
        solution_paths_area,
        subject_area * 0.92
    );
    // 2. solution_tree area matches solution_paths area
    assert!(
        (solution_tree_area - solution_paths_area).abs() < 0.0001,
        "Tree area ({}) should match paths area ({})",
        solution_tree_area,
        solution_paths_area
    );
    // 3. children fully inside parents
    assert!(
        clipper2_rust::check_polytree_fully_contains_children(&solution_tree),
        "TestPolytreeHoles2: children not fully contained by parents"
    );
    // 4. outside points are outside polytree
    for poi in &points_of_interest_outside {
        assert!(
            !polytree_contains_point(&solution_tree, *poi),
            "Point {:?} should be outside solution polytree",
            poi
        );
    }
    // 5. inside points are inside polytree
    for poi in &points_of_interest_inside {
        assert!(
            polytree_contains_point(&solution_tree, *poi),
            "Point {:?} should be inside solution polytree",
            poi
        );
    }
}
