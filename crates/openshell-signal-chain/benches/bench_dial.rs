// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! bench_dial — Benchmark query at different dial levels
//!
//! Run with: cargo bench -p openshell-signal-chain bench_dial
//!
//! Note: Run `cargo bench` from the crate directory, or use:
//!   cargo bench -p openshell-signal-chain

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use openshell_signal_chain::{Dial, Room};

/// Pre-populate a room with snaps and inferences for benchmarking
fn setup_room(item_count: usize, inference_ratio: f64) -> Room {
    let mut room = Room::new("benchmark");

    let snap_count = ((item_count as f64) * (1.0 - inference_ratio)) as usize;
    let inference_count = ((item_count as f64) * inference_ratio) as usize;

    for i in 0..snap_count {
        room.add_snap(
            serde_json::json!({"id": i, "type": "snap", "data": format!("snap-{}", i)}),
            1.0,
        );
    }

    // Add inferences at varying confidence levels
    for i in 0..inference_count {
        let confidence = (i as f64) / (inference_count as f64);
        room.add_inference(
            serde_json::json!({"id": i, "type": "inference", "data": format!("inf-{}", i)}),
            confidence.max(0.1),
        );
    }

    room
}

fn bench_dial_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("dial_query");

    // Vary dial position
    for pos in [0.0, 0.25, 0.5, 0.75, 1.0].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(pos), pos, |b, &pos| {
            let room = setup_room(100, 0.5);
            let dial = Dial::new(pos);
            b.iter(|| black_box(room.query(black_box(dial))));
        });
    }

    group.finish();
}

fn bench_dial_weights(c: &mut Criterion) {
    c.benchmark_group("dial_weights")
        .bench_function("snap_inference_weights", |b| {
            let dial = Dial::new(0.6);
            b.iter(|| {
                black_box(dial.snap_weight());
                black_box(dial.inference_weight());
                black_box(dial.inference_threshold());
            });
        });

    c.benchmark_group("dial_accepts_inference")
        .bench_function("accepts_high_confidence", |b| {
            let dial = Dial::new(0.3);
            b.iter(|| black_box(dial.accepts_inference(black_box(0.9))));
        });
}

fn bench_room_scales(c: &mut Criterion) {
    let mut group = c.benchmark_group("room_scale");

    // Vary room size
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let room = setup_room(size, 0.5);
            let dial = Dial::new(0.5);
            b.iter(|| black_box(room.query(black_box(dial))));
        });
    }

    group.finish();
}

fn bench_query_snaps_vs_inferences(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_types");

    let room = setup_room(100, 0.5);

    group.bench_function("query_snaps_only", |b| {
        b.iter(|| black_box(room.query_snaps()));
    });

    group.bench_function("query_all_hard", |b| {
        b.iter(|| black_box(room.query(black_box(Dial::hard()))));
    });

    group.bench_function("query_all_soft", |b| {
        b.iter(|| black_box(room.query(black_box(Dial::soft()))));
    });

    group.finish();
}

fn bench_dial_creation(c: &mut Criterion) {
    c.benchmark_group("dial_creation")
        .bench_function("new_dial", |b| {
            b.iter(|| black_box(Dial::new(black_box(0.5))));
        });

    c.benchmark_group("dial_presets")
        .bench_function("hard_soft_default", |b| {
            b.iter(|| {
                black_box(Dial::hard());
                black_box(Dial::soft());
                black_box(Dial::default());
            });
        });
}

fn bench_threshold_calculation(c: &mut Criterion) {
    c.benchmark_group("threshold_calc")
        .bench_function("threshold_from_position", |b| {
            let dial = Dial::new(0.5);
            b.iter(|| black_box(dial.inference_threshold()));
        });

    c.benchmark_group("threshold_via_accepts")
        .bench_function("accepts_loop", |b| {
            let dial = Dial::new(0.5);
            let confidences = [0.1, 0.3, 0.5, 0.7, 0.9];
            b.iter(|| {
                for &conf in &confidences {
                    black_box(dial.accepts_inference(conf));
                }
            });
        });
}

criterion_group!(
    benches,
    bench_dial_query,
    bench_dial_weights,
    bench_room_scales,
    bench_query_snaps_vs_inferences,
    bench_dial_creation,
    bench_threshold_calculation,
);
criterion_main!(benches);
