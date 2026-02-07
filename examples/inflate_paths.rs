// Port of CPP/Examples/Inflate/Inflate.cpp DoSimpleShapes()
// Demonstrates path offsetting with different join types and end types.

use clipper2::core::{FillRule, PointD};
use clipper2::offset::{EndType, JoinType};
use clipper2::utils::svg::{
    svg_add_caption, svg_add_open_subject_d, svg_add_solution_d, svg_save_to_file, SvgWriter,
};

fn main() {
    do_open_paths();
    do_closed_polygon();
}

fn do_open_paths() {
    let fr = FillRule::EvenOdd;
    let mut svg = SvgWriter::new(2);

    // Open path for offsetting
    let base = clipper2::make_path_d(&[
        80.0, 60.0, 20.0, 20.0, 180.0, 20.0, 180.0, 70.0, 25.0, 150.0, 20.0, 180.0, 180.0, 180.0,
    ]);

    // Miter Joins; Square Ends
    let op1 = vec![base.clone()];
    let op2 = clipper2::inflate_paths_d(&op1, 15.0, JoinType::Miter, EndType::Square, 3.0, 2, 0.0);
    svg_add_open_subject_d(&mut svg, &op1, fr, false);
    svg_add_solution_d(&mut svg, &op2, fr, false);
    svg_add_caption(&mut svg, "Miter Joins; Square Ends", 20, 210);

    // Square Joins; Square Ends
    let op1 = clipper2::translate_paths(&op1, 210.0, 0.0);
    let op2 = clipper2::inflate_paths_d(&op1, 15.0, JoinType::Square, EndType::Square, 2.0, 2, 0.0);
    svg_add_open_subject_d(&mut svg, &op1, fr, false);
    svg_add_solution_d(&mut svg, &op2, fr, false);
    svg_add_caption(&mut svg, "Square Joins; Square Ends", 230, 210);

    // Bevel Joins; Butt Ends
    let op1 = clipper2::translate_paths(&op1, 210.0, 0.0);
    let op2 = clipper2::inflate_paths_d(&op1, 15.0, JoinType::Bevel, EndType::Butt, 3.0, 2, 0.0);
    svg_add_open_subject_d(&mut svg, &op1, fr, false);
    svg_add_solution_d(&mut svg, &op2, fr, false);
    svg_add_caption(&mut svg, "Bevel Joins; Butt Ends", 440, 210);

    // Round Joins; Round Ends
    let op1 = clipper2::translate_paths(&op1, 210.0, 0.0);
    let op2 = clipper2::inflate_paths_d(&op1, 15.0, JoinType::Round, EndType::Round, 2.0, 2, 0.0);
    svg_add_open_subject_d(&mut svg, &op1, fr, false);
    svg_add_solution_d(&mut svg, &op2, fr, false);
    svg_add_caption(&mut svg, "Round Joins; Round Ends", 650, 210);

    svg_save_to_file(&mut svg, "open_paths.svg", 800, 600, 20);
    println!("Saved open_paths.svg");
}

fn do_closed_polygon() {
    // Triangle offset with large miter
    let mut solution: Vec<Vec<PointD>> = Vec::new();
    let mut p = vec![clipper2::make_path_d(&[
        30.0, 150.0, 60.0, 350.0, 0.0, 350.0,
    ])];
    solution.extend(p.iter().cloned());
    for _ in 0..5 {
        p = clipper2::inflate_paths_d(&p, 5.0, JoinType::Miter, EndType::Polygon, 10.0, 2, 0.0);
        solution.extend(p.iter().cloned());
    }

    let mut svg = SvgWriter::new(2);
    svg_add_solution_d(&mut svg, &solution, FillRule::EvenOdd, false);
    svg_save_to_file(&mut svg, "polygon_offset.svg", 400, 400, 20);
    println!("Saved polygon_offset.svg");
}
