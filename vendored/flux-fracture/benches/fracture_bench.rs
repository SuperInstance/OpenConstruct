use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use flux_fracture::*;

fn build_identity(n: usize) -> DependencyGraph {
    DependencyGraph::identity(n)
}

fn build_block_diagonal(n: usize) -> DependencyGraph {
    let mut graph = DependencyGraph::identity(n);
    let half = n / 2;
    graph.fill_block(0, half, 0, half);
    graph.fill_block(half, n, half, n);
    graph
}

fn bench_fracture_8(c: &mut Criterion) {
    let graph = build_identity(8);
    let fracturer = Fracturer::new();
    c.bench_function("fracture_8_identity", |b| {
        b.iter(|| fracturer.fracture(black_box(&graph)))
    });
}

fn bench_fracture_64(c: &mut Criterion) {
    let graph = build_identity(64);
    let fracturer = Fracturer::new();
    c.bench_function("fracture_64_identity", |b| {
        b.iter(|| fracturer.fracture(black_box(&graph)))
    });
}

fn bench_fracture_256(c: &mut Criterion) {
    let graph = build_identity(256);
    let fracturer = Fracturer::new();
    c.bench_function("fracture_256_identity", |b| {
        b.iter(|| fracturer.fracture(black_box(&graph)))
    });
}

fn bench_fracture_block_diagonal(c: &mut Criterion) {
    let mut group = c.benchmark_group("fracture_block_diagonal");
    for size in [8usize, 64, 256] {
        let graph = build_block_diagonal(size);
        let fracturer = Fracturer::new();
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| fracturer.fracture(black_box(&graph)))
        });
    }
    group.finish();
}

fn bench_coalesce(c: &mut Criterion) {
    let coalescer = Coalescer::new();
    let masks: Vec<u64> = (0..8).map(|i| 1u64 << i).collect();
    c.bench_function("coalesce_8_masks", |b| {
        b.iter(|| coalescer.coalesce_masks(black_box(&masks)))
    });
}

criterion_group!(benches, bench_fracture_8, bench_fracture_64, bench_fracture_256, bench_fracture_block_diagonal, bench_coalesce);
criterion_main!(benches);
