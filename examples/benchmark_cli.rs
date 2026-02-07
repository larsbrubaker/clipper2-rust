// Port of CPP/Examples/Benchmarks/Benchmarks.cpp
// CLI benchmark: times boolean intersection of random polygons with increasing edge counts.

use clipper2::core::{FillRule, Path64, Point64};
use clipper2::engine::ClipType;
use clipper2::utils::svg::{svg_add_clip_64, svg_add_solution_64, svg_add_subject_64, SvgWriter};
use rand::Rng;
use std::time::Instant;

fn make_random_poly(width: i64, height: i64, vert_cnt: usize) -> Path64 {
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(vert_cnt);
    for _ in 0..vert_cnt {
        result.push(Point64::new(
            rng.gen_range(0..width),
            rng.gen_range(0..height),
        ));
    }
    result
}

fn main() {
    let ct = ClipType::Intersection;
    let fr = FillRule::NonZero;
    let width = 800i64;
    let height = 600i64;

    println!("\nComplex Polygons Benchmark:");
    println!("{:>12} {:>12}", "Edge Count", "Time (ms)");
    println!("{}", "-".repeat(26));

    let mut last_subject = Vec::new();
    let mut last_clip = Vec::new();
    let mut last_solution = Vec::new();

    for edge_cnt in (1000..=7000).step_by(1000) {
        let subject = vec![make_random_poly(width, height, edge_cnt)];
        let clip = vec![make_random_poly(width, height, edge_cnt)];

        let start = Instant::now();
        let solution = clipper2::boolean_op_64(ct, fr, &subject, &clip);
        let elapsed = start.elapsed();

        if solution.is_empty() {
            println!("{:>12} FAILED (empty result)", edge_cnt);
            break;
        }

        println!(
            "{:>12} {:>9.2}ms ({} output paths)",
            edge_cnt,
            elapsed.as_secs_f64() * 1000.0,
            solution.len()
        );

        last_subject = subject;
        last_clip = clip;
        last_solution = solution;
    }

    // Save the last result as SVG
    if !last_solution.is_empty() {
        let mut svg = SvgWriter::new(2);
        svg_add_subject_64(&mut svg, &last_subject, fr);
        svg_add_clip_64(&mut svg, &last_clip, fr);
        svg_add_solution_64(&mut svg, &last_solution, fr, false);
        svg.save_to_file("benchmark.svg", width as i32, height as i32, 20);
        println!("\nSaved benchmark.svg");
    }
}
