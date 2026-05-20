// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! cascade — Benchmark cascade propagation at different depths and tree sizes.
//!
//! Run with: cargo bench -p openshell-signal-chain --bench cascade

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use openshell_signal_chain::{Room, SignalChain};

/// Build a room tree of given depth and branching factor, with n inferences at the root.
fn build_tree(depth: usize, branching: usize, inference_count: usize) -> Room {
    let mut root = Room::new("root");

    for i in 0..inference_count {
        let confidence = 0.55 + (i as f64 * 0.05).min(0.44);
        root.add_inference(
            serde_json::json!({"id": i, "hypothesis": format!("h-{i}")}),
            confidence,
        );
    }

    fn add_children(room: &mut Room, depth: usize, branching: usize, label: &str) {
        if depth == 0 {
            return;
        }
        for i in 0..branching {
            let child_name = format!("{label}-{i}");
            let mut child = Room::new(&child_name);
            add_children(&mut child, depth - 1, branching, &child_name);
            room.children.insert(child_name, child);
        }
    }

    add_children(&mut root, depth, branching, "node");
    root
}

fn bench_cascade_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("cascade_depth");

    for depth in [1usize, 2, 3].iter() {
        let mut root = build_tree(*depth, 3, 5);
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let mut r = root.clone();
                r.cascade(black_box(depth));
                black_box(r)
            });
        });
    }

    group.finish();
}

fn bench_cascade_inference_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("cascade_inference_count");

    for n_inferences in [2usize, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_inferences),
            n_inferences,
            |b, &n| {
                let root = build_tree(2, 3, n);
                b.iter(|| {
                    let mut r = root.clone();
                    r.cascade(black_box(2));
                    black_box(r)
                });
            },
        );
    }

    group.finish();
}

fn bench_cascade_from_chain(c: &mut Criterion) {
    c.benchmark_group("cascade_from_chain")
        .bench_function("10_rooms_depth_1", |b| {
            b.iter(|| {
                let mut chain = SignalChain::new("bench");
                let root = chain.room("root");
                for i in 0..5 {
                    root.add_inference(
                        serde_json::json!({"i": i}),
                        0.6 + i as f64 * 0.06,
                    );
                }
                // Add children directly to root's Room.children
                let root_room = chain.rooms.get_mut("root").unwrap();
                for i in 0..10 {
                    root_room
                        .children
                        .insert(format!("child-{i}"), Room::new(&format!("child-{i}")));
                }
                chain.cascade_from(black_box("root"), black_box(1));
                black_box(chain.rooms.len())
            });
        });
}

criterion_group!(
    benches,
    bench_cascade_depth,
    bench_cascade_inference_count,
    bench_cascade_from_chain,
);
criterion_main!(benches);
