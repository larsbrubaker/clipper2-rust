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
    let solution =
        clipper2::inflate_paths_64(&subject, 1.0, JoinType::Round, EndType::Polygon, 2.0, 0.0);
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
    let subject_raw = vec![clipper2::make_path64(&[
        50, 50, 100, 50, 100, 150, 50, 150, 0, 100,
    ])];
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
        Point64::new(0, 0),
        Point64::new(20000, 200),
        Point64::new(40000, 0),
        Point64::new(40000, 50000),
        Point64::new(0, 50000),
        Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths,
        -5000.0,
        JoinType::Square,
        EndType::Polygon,
        2.0,
        0.0,
    );
    assert_eq!(solution[0].len(), 5);

    let paths = vec![vec![
        Point64::new(0, 0),
        Point64::new(20000, 400),
        Point64::new(40000, 0),
        Point64::new(40000, 50000),
        Point64::new(0, 50000),
        Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths,
        -5000.0,
        JoinType::Square,
        EndType::Polygon,
        2.0,
        0.0,
    );
    assert_eq!(solution[0].len(), 5);

    let paths = vec![vec![
        Point64::new(0, 0),
        Point64::new(20000, 400),
        Point64::new(40000, 0),
        Point64::new(40000, 50000),
        Point64::new(0, 50000),
        Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths,
        -5000.0,
        JoinType::Round,
        EndType::Polygon,
        2.0,
        100.0,
    );
    assert!(solution[0].len() > 5);

    let paths = vec![vec![
        Point64::new(0, 0),
        Point64::new(20000, 1500),
        Point64::new(40000, 0),
        Point64::new(40000, 50000),
        Point64::new(0, 50000),
        Point64::new(0, 0),
    ]];
    let solution = clipper2::inflate_paths_64(
        &paths,
        -5000.0,
        JoinType::Round,
        EndType::Polygon,
        2.0,
        100.0,
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
    let solution =
        clipper2::inflate_paths_64(&subject, -50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0);
    assert_eq!(solution.len(), 0);

    // Square with hole, inflated by 10 -> should merge to 1 path
    subject.push(clipper2::make_path64(&[40, 60, 60, 60, 60, 40, 40, 40]));
    let solution =
        clipper2::inflate_paths_64(&subject, 10.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0);
    assert_eq!(solution.len(), 1);

    // Reverse both paths, inflate by 10 -> should still be 1
    let mut reversed_subject = subject.clone();
    reversed_subject[0].reverse();
    reversed_subject[1].reverse();
    let solution = clipper2::inflate_paths_64(
        &reversed_subject,
        10.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
    );
    assert_eq!(solution.len(), 1);

    // Just the reversed outer, shrink by 50 -> should disappear
    let single_reversed = vec![reversed_subject[0].clone()];
    let solution = clipper2::inflate_paths_64(
        &single_reversed,
        -50.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
    );
    assert_eq!(solution.len(), 0);
}

// ==========================================================================
// TestOffsets9 - #733 - Orientation matching
// ==========================================================================

#[test]
fn test_offsets9_issue_733() {
    // Positive orientation subject
    let subject = vec![clipper2::make_path64(&[
        100, 100, 200, 100, 200, 400, 100, 400,
    ])];
    let solution =
        clipper2::inflate_paths_64(&subject, 50.0, JoinType::Miter, EndType::Polygon, 2.0, 0.0);
    assert_eq!(solution.len(), 1);
    assert!(is_positive(&solution[0]));

    // Reverse subject -> solution should also be reversed, but area still larger
    let mut rev_subject = subject.clone();
    rev_subject[0].reverse();
    let solution = clipper2::inflate_paths_64(
        &rev_subject,
        50.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
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
    subject_with_hole.push(clipper2::make_path64(&[
        130, 130, 170, 130, 170, 370, 130, 370,
    ]));
    let solution = clipper2::inflate_paths_64(
        &subject_with_hole,
        30.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
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
        &subject_with_hole,
        -15.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
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
        &subject,
        -50.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        2,
        0.0,
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
        &subject,
        -249561088.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
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
        &subject1,
        delta,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
    );
    let area1 = area_paths(&solution1).abs();
    assert_eq!(area1, 122.0);

    // Adding a single-point path should not change the solution
    let subject2 = vec![
        vec![Point64::new(0, 0), Point64::new(0, 10), Point64::new(10, 0)],
        vec![Point64::new(0, 20)],
    ];
    let solution2 = clipper2::inflate_paths_64(
        &subject2,
        delta,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
    );
    let area2 = area_paths(&solution2).abs();
    assert_eq!(area2, 122.0);
}

// ==========================================================================
// TestOffsets (basic) - loads from Offsets.txt
// ==========================================================================

#[test]
fn test_offsets_basic_from_file() {
    let path = format!("{}/Tests/data/Offsets.txt", env!("CARGO_MANIFEST_DIR"));
    for test_number in 1..=2 {
        let data = clipper2::utils::file_io::load_test_num(&path, test_number);
        let data = match data {
            Some(d) => d,
            None => panic!("Failed to load Offsets.txt test {}", test_number),
        };
        let mut co = ClipperOffset::new(2.0, 0.0, false, false);
        co.add_paths(&data.subj, JoinType::Round, EndType::Polygon);
        let mut outputs = Paths64::new();
        co.execute(1.0, &mut outputs);
        let outer_is_positive = area_paths(&outputs) > 0.0;
        let is_positive_count = outputs.iter().filter(|p| is_positive(p)).count();
        let is_negative_count = outputs.len() - is_positive_count;
        if outer_is_positive {
            assert_eq!(
                is_positive_count, 1,
                "Offsets test {}: expected 1 positive path, got {}",
                test_number, is_positive_count
            );
        } else {
            assert_eq!(
                is_negative_count, 1,
                "Offsets test {}: expected 1 negative path, got {}",
                test_number, is_negative_count
            );
        }
    }
}

// ==========================================================================
// TestOffsets3 - see #424 (large path Miter offset)
// ==========================================================================

#[test]
fn test_offsets3_issue_424() {
    let subjects = vec![vec![
        Point64::new(1525311078, 1352369439),
        Point64::new(1526632284, 1366692987),
        Point64::new(1519397110, 1367437476),
        Point64::new(1520246456, 1380177674),
        Point64::new(1520613458, 1385913385),
        Point64::new(1517383844, 1386238444),
        Point64::new(1517771817, 1392099983),
        Point64::new(1518233190, 1398758441),
        Point64::new(1518421934, 1401883197),
        Point64::new(1518694564, 1406612275),
        Point64::new(1520267428, 1430289121),
        Point64::new(1520770744, 1438027612),
        Point64::new(1521148232, 1443438264),
        Point64::new(1521441833, 1448964260),
        Point64::new(1521683005, 1452518932),
        Point64::new(1521819320, 1454374912),
        Point64::new(1527943004, 1454154711),
        Point64::new(1527649403, 1448523858),
        Point64::new(1535901696, 1447989084),
        Point64::new(1535524209, 1442788147),
        Point64::new(1538953052, 1442463089),
        Point64::new(1541553521, 1442242888),
        Point64::new(1541459149, 1438855987),
        Point64::new(1538764308, 1439076188),
        Point64::new(1538575565, 1436832236),
        Point64::new(1538764308, 1436832236),
        Point64::new(1536509870, 1405374956),
        Point64::new(1550497874, 1404347351),
        Point64::new(1550214758, 1402428457),
        Point64::new(1543818445, 1402868859),
        Point64::new(1543734559, 1402124370),
        Point64::new(1540672717, 1402344571),
        Point64::new(1540473487, 1399995761),
        Point64::new(1524996506, 1400981422),
        Point64::new(1524807762, 1398223667),
        Point64::new(1530092585, 1397898609),
        Point64::new(1531675935, 1397783265),
        Point64::new(1531392819, 1394920653),
        Point64::new(1529809469, 1395025510),
        Point64::new(1529348096, 1388880855),
        Point64::new(1531099218, 1388660654),
        Point64::new(1530826588, 1385158410),
        Point64::new(1532955197, 1384938209),
        Point64::new(1532661596, 1379003269),
        Point64::new(1532472852, 1376235028),
        Point64::new(1531277476, 1376350372),
        Point64::new(1530050642, 1361806623),
        Point64::new(1599487345, 1352704983),
        Point64::new(1602758902, 1378489467),
        Point64::new(1618990858, 1376350372),
        Point64::new(1615058698, 1344085688),
        Point64::new(1603230761, 1345700495),
        Point64::new(1598648484, 1346329641),
        Point64::new(1598931599, 1348667965),
        Point64::new(1596698132, 1348993024),
        Point64::new(1595775386, 1342722540),
    ]];
    let solution = clipper2::inflate_paths_64(
        &subjects,
        -209715.0,
        JoinType::Miter,
        EndType::Polygon,
        2.0,
        0.0,
    );
    assert!(
        !solution.is_empty(),
        "TestOffsets3: solution should not be empty"
    );
    let diff = (solution[0].len() as i64 - subjects[0].len() as i64).unsigned_abs() as usize;
    assert!(
        diff <= 1,
        "TestOffsets3: solution vertex count ({}) should be within 1 of subject ({})",
        solution[0].len(),
        subjects[0].len()
    );
}

// ==========================================================================
// TestOffsets6 - also from #593 (tests rounded ends)
// ==========================================================================

#[test]
fn test_offsets6_issue_593_rounded_ends() {
    let subjects = vec![
        vec![
            Point64::new(620, 620),
            Point64::new(-620, 620),
            Point64::new(-620, -620),
            Point64::new(620, -620),
        ],
        vec![
            Point64::new(20, -277),
            Point64::new(42, -275),
            Point64::new(59, -272),
            Point64::new(80, -266),
            Point64::new(97, -261),
            Point64::new(114, -254),
            Point64::new(135, -243),
            Point64::new(149, -235),
            Point64::new(167, -222),
            Point64::new(182, -211),
            Point64::new(197, -197),
            Point64::new(212, -181),
            Point64::new(223, -167),
            Point64::new(234, -150),
            Point64::new(244, -133),
            Point64::new(253, -116),
            Point64::new(260, -99),
            Point64::new(267, -78),
            Point64::new(272, -61),
            Point64::new(275, -40),
            Point64::new(278, -18),
            Point64::new(276, -39),
            Point64::new(272, -61),
            Point64::new(267, -79),
            Point64::new(260, -99),
            Point64::new(253, -116),
            Point64::new(245, -133),
            Point64::new(235, -150),
            Point64::new(223, -167),
            Point64::new(212, -181),
            Point64::new(197, -197),
            Point64::new(182, -211),
            Point64::new(168, -222),
            Point64::new(152, -233),
            Point64::new(135, -243),
            Point64::new(114, -254),
            Point64::new(97, -261),
            Point64::new(80, -267),
            Point64::new(59, -272),
            Point64::new(42, -275),
            Point64::new(20, -278),
        ],
    ];
    let offset = -50.0;
    let mut co = ClipperOffset::new(2.0, 0.0, false, false);
    co.add_paths(&subjects, JoinType::Round, EndType::Polygon);
    let mut solution = Paths64::new();
    co.execute(offset, &mut solution);
    assert_eq!(solution.len(), 2, "TestOffsets6: expected 2 paths");
    let area1 = area(&solution[1]);
    assert!(
        area1 < -47500.0,
        "TestOffsets6: inner path area ({}) should be < -47500",
        area1
    );
}

// ==========================================================================
// TestOffsets10 - see #715
// ==========================================================================

#[test]
fn test_offsets10_issue_715() {
    let subjects = vec![
        vec![
            Point64::new(508685336, -435806096),
            Point64::new(509492982, -434729201),
            Point64::new(509615525, -434003092),
            Point64::new(509615525, 493372891),
            Point64::new(509206033, 494655198),
            Point64::new(508129138, 495462844),
            Point64::new(507403029, 495585387),
            Point64::new(-545800889, 495585387),
            Point64::new(-547083196, 495175895),
            Point64::new(-547890842, 494099000),
            Point64::new(-548013385, 493372891),
            Point64::new(-548013385, -434003092),
            Point64::new(-547603893, -435285399),
            Point64::new(-546526998, -436093045),
            Point64::new(-545800889, -436215588),
            Point64::new(507403029, -436215588),
        ],
        vec![
            Point64::new(106954765, -62914568),
            Point64::new(106795129, -63717113),
            Point64::new(106340524, -64397478),
            Point64::new(105660159, -64852084),
            Point64::new(104857613, -65011720),
            Point64::new(104055068, -64852084),
            Point64::new(103374703, -64397478),
            Point64::new(102920097, -63717113),
            Point64::new(102760461, -62914568),
            Point64::new(102920097, -62112022),
            Point64::new(103374703, -61431657),
            Point64::new(104055068, -60977052),
            Point64::new(104857613, -60817416),
            Point64::new(105660159, -60977052),
            Point64::new(106340524, -61431657),
            Point64::new(106795129, -62112022),
        ],
    ];
    let mut co = ClipperOffset::new(2.0, 104_857.613_187_5, false, false);
    let mut solution = Paths64::new();
    co.add_paths(&subjects, JoinType::Round, EndType::Polygon);
    co.execute(-2_212_495.638_256_25, &mut solution);
    assert_eq!(
        solution.len(),
        2,
        "TestOffsets10: expected 2 paths, got {}",
        solution.len()
    );
}
