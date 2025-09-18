use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_placeholder(c: &mut Criterion) {
    // Benchmarks will be added as functions are implemented
    c.bench_function("version_access", |b| {
        b.iter(|| {
            clipper2::CLIPPER2_VERSION
        })
    });
}

criterion_group!(benches, benchmark_placeholder);
criterion_main!(benches);