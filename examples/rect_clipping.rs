// Port of CPP/Examples/RectClipping/RectClipping.cpp
// Demonstrates rectangle clipping of random ellipses.

use clipper2::core::{FillRule, Point64, Rect64};
use clipper2::utils::svg::{svg_add_clip_64, svg_add_solution_64, svg_add_subject_64, SvgWriter};
use rand::Rng;

fn main() {
    let display_width = 800i64;
    let display_height = 600i64;
    let margin = 100i64;

    let mut rng = rand::thread_rng();

    // Generate random ellipses as subjects
    let mut subject = Vec::new();
    for _ in 0..30 {
        let cx = rng.gen_range(margin..display_width - margin);
        let cy = rng.gen_range(margin..display_height - margin);
        let rx = rng.gen_range(20..80) as f64;
        let ry = rng.gen_range(20..80) as f64;
        let ellipse = clipper2::ellipse_point64(Point64::new(cx, cy), rx, ry, 0);
        subject.push(ellipse);
    }

    // Clip rectangle
    let rect = Rect64::new(
        display_width / 4,
        display_height / 4,
        display_width * 3 / 4,
        display_height * 3 / 4,
    );

    let solution = clipper2::rect_clip_64(&rect, &subject);

    println!(
        "RectClip: {} input ellipses -> {} clipped paths",
        subject.len(),
        solution.len()
    );

    // Show the clip rectangle as a clip path
    let clip_path = vec![vec![
        Point64::new(rect.left, rect.top),
        Point64::new(rect.right, rect.top),
        Point64::new(rect.right, rect.bottom),
        Point64::new(rect.left, rect.bottom),
    ]];

    let mut svg = SvgWriter::new(2);
    svg_add_subject_64(&mut svg, &subject, FillRule::NonZero);
    svg_add_clip_64(&mut svg, &clip_path, FillRule::NonZero);
    svg_add_solution_64(&mut svg, &solution, FillRule::NonZero, false);
    svg.save_to_file(
        "rect_clipping.svg",
        display_width as i32,
        display_height as i32,
        20,
    );
    println!("Saved rect_clipping.svg");
}
