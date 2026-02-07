// Port of CPP/Examples/SimpleClipping/SimpleClipping.cpp
// Demonstrates basic boolean intersection of two star polygons with SVG output.

use clipper2_rust::core::FillRule;
use clipper2_rust::utils::svg::{svg_add_clip_64, svg_add_solution_64, svg_add_subject_64, SvgWriter};

fn main() {
    // Intersect a star and another modestly rotated star
    let subject = vec![clipper2_rust::make_path64(&[
        200, 100, 20, 158, 130, 4, 130, 196, 20, 42,
    ])];
    let clip = vec![clipper2_rust::make_path64(&[
        196, 126, 8, 136, 154, 16, 104, 200, 38, 24,
    ])];

    let solution = clipper2_rust::intersect_64(&subject, &clip, FillRule::NonZero);

    println!(
        "Intersect: {} subject paths, {} clip paths -> {} solution paths",
        subject.len(),
        clip.len(),
        solution.len()
    );

    // Save as SVG
    let mut svg = SvgWriter::new(2);
    svg_add_subject_64(&mut svg, &subject, FillRule::NonZero);
    svg_add_clip_64(&mut svg, &clip, FillRule::NonZero);
    svg_add_solution_64(&mut svg, &solution, FillRule::NonZero, false);
    svg.save_to_file("simple_clipping.svg", 400, 400, 10);
    println!("Saved simple_clipping.svg");
}
