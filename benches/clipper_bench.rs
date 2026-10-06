use criterion::{criterion_group, criterion_main, Criterion};

use clipper2_rust::core::{FillRule, Path64, Point64};
use clipper2_rust::engine::ClipType;
use clipper2_rust::offset::{EndType, JoinType};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn make_random_poly(rng: &mut StdRng, width: i64, height: i64, vert_cnt: usize) -> Path64 {
    let mut result = Vec::with_capacity(vert_cnt);
    for _ in 0..vert_cnt {
        result.push(Point64::new(
            rng.gen_range(0..width),
            rng.gen_range(0..height),
        ));
    }
    result
}

fn bench_boolean_intersection(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let subject = vec![make_random_poly(&mut rng, 800, 600, 1000)];
    let clip = vec![make_random_poly(&mut rng, 800, 600, 1000)];

    c.bench_function("boolean_intersection_1000", |b| {
        b.iter(|| {
            clipper2_rust::boolean_op_64(ClipType::Intersection, FillRule::NonZero, &subject, &clip)
        })
    });
}

fn bench_boolean_union(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let subject = vec![make_random_poly(&mut rng, 800, 600, 1000)];
    let clip = vec![make_random_poly(&mut rng, 800, 600, 1000)];

    c.bench_function("boolean_union_1000", |b| {
        b.iter(|| clipper2_rust::boolean_op_64(ClipType::Union, FillRule::NonZero, &subject, &clip))
    });
}

fn bench_union_disjoint_rings(c: &mut Criterion) {
    // A grid of glyph-like shapes that don't touch, like a page of text:
    // an outer ring per cell and a hole in about half of them
    let mut rng = StdRng::seed_from_u64(7);
    let mut subject = Vec::new();
    for row in 0..40 {
        for col in 0..50 {
            let center = Point64::new(col * 1_000_000, row * 1_600_000);
            subject.push(clipper2_rust::ellipse_point64(
                center,
                rng.gen_range(400_000.0..450_000.0),
                rng.gen_range(700_000.0..750_000.0),
                rng.gen_range(8..48),
            ));
            if rng.gen_bool(0.5) {
                let mut hole = clipper2_rust::ellipse_point64(
                    center,
                    rng.gen_range(150_000.0..200_000.0),
                    rng.gen_range(300_000.0..350_000.0),
                    rng.gen_range(8..24),
                );
                hole.reverse();
                subject.push(hole);
            }
        }
    }

    c.bench_function("union_disjoint_rings_2000", |b| {
        b.iter(|| clipper2_rust::union_subjects_64(&subject, FillRule::NonZero))
    });
}

fn bench_inflate_round(c: &mut Criterion) {
    let paths = vec![clipper2_rust::make_path64(&[
        0, 0, 100, 0, 100, 100, 200, 100, 200, 0, 300, 0, 300, 200, 0, 200,
    ])];

    c.bench_function("inflate_round_join", |b| {
        b.iter(|| {
            clipper2_rust::inflate_paths_64(
                &paths,
                10.0,
                JoinType::Round,
                EndType::Polygon,
                2.0,
                0.0,
            )
        })
    });
}

fn bench_inflate_miter(c: &mut Criterion) {
    let paths = vec![clipper2_rust::make_path64(&[
        0, 0, 100, 0, 100, 100, 200, 100, 200, 0, 300, 0, 300, 200, 0, 200,
    ])];

    c.bench_function("inflate_miter_join", |b| {
        b.iter(|| {
            clipper2_rust::inflate_paths_64(
                &paths,
                10.0,
                JoinType::Miter,
                EndType::Polygon,
                2.0,
                0.0,
            )
        })
    });
}

fn bench_rect_clip(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(99);
    let mut subjects = Vec::new();
    for _ in 0..20 {
        let cx = rng.gen_range(100..700);
        let cy = rng.gen_range(100..500);
        subjects.push(clipper2_rust::ellipse_point64(
            Point64::new(cx, cy),
            50.0,
            50.0,
            0,
        ));
    }
    let rect = clipper2_rust::core::Rect64::new(200, 150, 600, 450);

    c.bench_function("rect_clip_20_ellipses", |b| {
        b.iter(|| clipper2_rust::rect_clip_64(&rect, &subjects))
    });
}

fn bench_simplify(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(123);
    let paths = vec![make_random_poly(&mut rng, 800, 600, 500)];

    c.bench_function("simplify_500_points", |b| {
        b.iter(|| clipper2_rust::simplify_paths(&paths, 2.0, true))
    });
}

criterion_group!(
    benches,
    bench_boolean_intersection,
    bench_boolean_union,
    bench_union_disjoint_rings,
    bench_inflate_round,
    bench_inflate_miter,
    bench_rect_clip,
    bench_simplify,
);
criterion_main!(benches);
