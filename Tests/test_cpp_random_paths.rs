// Copyright 2025 - Clipper2 Rust port
// Ported from C++ tests: TestRandomPaths.cpp
//
// Tests that Paths64-based solutions match PolyTree-based solutions
// for randomly generated paths with deterministic seeding.

use clipper2::core::*;
use clipper2::engine::ClipType;
use clipper2::engine_public::*;

/// Simple PRNG based on minstd_rand0 (matching C++ std::default_random_engine on MSVC).
/// Uses a linear congruential generator for deterministic results.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate next pseudo-random value and return uniform int in [min, max] (inclusive)
    fn next_int(&mut self, min: i32, max: i32) -> i32 {
        if min == max {
            return min;
        }
        // LCG: minstd_rand0 parameters (a=16807, m=2^31-1)
        self.state = self.state.wrapping_mul(16807) % 2147483647;
        let range = (max - min + 1) as u64;
        let val = self.state % range;
        min + val as i32
    }
}

fn generate_random_paths(rng: &mut Rng, min_path_count: i32, max_complexity: i32) -> Paths64 {
    let path_count = rng.next_int(min_path_count, max_complexity);
    let mut result = Vec::with_capacity(path_count.max(0) as usize);
    for _ in 0..path_count {
        let path_length = rng.next_int(0, max_complexity.max(0));
        let mut path = Vec::with_capacity(path_length.max(0) as usize);
        for _ in 0..path_length {
            if path.is_empty() {
                let x = rng.next_int(-max_complexity, max_complexity * 2);
                let y = rng.next_int(-max_complexity, max_complexity * 2);
                path.push(Point64::new(x as i64, y as i64));
            } else {
                let prev = *path.last().unwrap();
                let dx = rng.next_int(-5, 5);
                let dy = rng.next_int(-5, 5);
                path.push(Point64::new(prev.x + dx as i64, prev.y + dy as i64));
            }
        }
        result.push(path);
    }
    result
}

#[test]
fn test_random_paths() {
    let mut rng = Rng::new(42);

    // In release: 750 iterations, in debug: fewer for speed
    let iterations = if cfg!(debug_assertions) { 10 } else { 750 };

    for i in 0..iterations {
        let max_complexity = (i / 10).max(1);
        let subject = generate_random_paths(&mut rng, 1, max_complexity);
        let subject_open = generate_random_paths(&mut rng, 0, max_complexity);
        let clip = generate_random_paths(&mut rng, 0, max_complexity);
        let ct_val = rng.next_int(0, 4);
        let fr_val = rng.next_int(0, 3);
        let ct = match ct_val {
            0 => ClipType::NoClip,
            1 => ClipType::Intersection,
            2 => ClipType::Union,
            3 => ClipType::Difference,
            4 => ClipType::Xor,
            _ => ClipType::NoClip,
        };
        let fr = match fr_val {
            0 => FillRule::EvenOdd,
            1 => FillRule::NonZero,
            2 => FillRule::Positive,
            3 => FillRule::Negative,
            _ => FillRule::EvenOdd,
        };

        // Paths-based solution
        let mut solution = Paths64::new();
        let mut solution_open = Paths64::new();
        {
            let mut c = Clipper64::new();
            c.add_subject(&subject);
            c.add_open_subject(&subject_open);
            c.add_clip(&clip);
            c.execute(ct, fr, &mut solution, Some(&mut solution_open));
        }
        let area_p = area_paths(&solution) as i64;

        // PolyTree-based solution
        let mut solution_polytree = PolyTree64::new();
        let mut solution_polytree_open = Paths64::new();
        {
            let mut c = Clipper64::new();
            c.add_subject(&subject);
            c.add_open_subject(&subject_open);
            c.add_clip(&clip);
            c.execute_tree(ct, fr, &mut solution_polytree, &mut solution_polytree_open);
        }
        let solution_polytree_paths = clipper2::poly_tree_to_paths64(&solution_polytree);
        let area_pt = area_paths(&solution_polytree_paths) as i64;

        assert_eq!(
            area_p, area_pt,
            "Area mismatch at iteration {} (ct={:?}, fr={:?})",
            i, ct, fr
        );
    }
}
