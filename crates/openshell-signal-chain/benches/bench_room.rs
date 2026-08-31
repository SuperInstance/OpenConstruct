// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! bench_room — Benchmark adding snaps/inferences and querying large rooms
//!
//! Run with: cargo bench -p openshell-signal-chain bench_room
//!
//! Note: Run `cargo bench` from the crate directory, or use:
//!   cargo bench -p openshell-signal-chain

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use openshell_signal_chain::{Dial, Room, SignalChain};

/// Pre-populate a room with snaps and inferences for benchmarking
fn setup_room(item_count: usize, inference_ratio: f64) -> Room {
    let mut room = Room::new("benchmark");

    let snap_count = ((item_count as f64) * (1.0 - inference_ratio)) as usize;
    let inference_count = ((item_count as f64) * inference_ratio) as usize;

    for i in 0..snap_count {
        room.add_snap(
            serde_json::json!({"id": i, "type": "snap", "value": i as f64 * 1.5}),
            1.0,
        );
    }

    for i in 0..inference_count {
        let confidence = (i as f64) / (inference_count as f64);
        room.add_inference(
            serde_json::json!({"id": i, "type": "inference", "hypothesis": format!("h-{}", i)}),
            confidence.max(0.1),
        );
    }

    room
}

/// Pre-populate a SignalChain with rooms
fn setup_chain(room_count: usize, items_per_room: usize) -> SignalChain {
    let mut chain = SignalChain::new("benchmark");

    for r in 0..room_count {
        let room = chain.room(&format!("room-{}", r));
        for i in 0..items_per_room {
            room.add_snap(serde_json::json!({"room": r, "id": i}), 1.0);
            if i % 2 == 0 {
                room.add_inference(serde_json::json!({"room": r, "id": i, "inf": true}), 0.7);
            }
        }
    }

    chain
}

fn bench_room_add_snap(c: &mut Criterion) {
    c.benchmark_group("room_add_snap")
        .bench_function("add_100_snaps", |b| {
            b.iter(|| {
                let mut room = Room::new("test");
                for i in 0..100 {
                    room.add_snap(
                        black_box(serde_json::json!({"id": i, "data": format!("value-{}", i)})),
                        1.0,
                    );
                }
            });
        });

    c.benchmark_group("room_add_inference")
        .bench_function("add_100_inferences", |b| {
            b.iter(|| {
                let mut room = Room::new("test");
                for i in 0..100 {
                    room.add_inference(
                        black_box(serde_json::json!({"id": i, "hyp": format!("h-{}", i)})),
                        0.5 + (i as f64) * 0.005,
                    );
                }
            });
        });
}

fn bench_room_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("room_query");

    for size in [10, 50, 100, 500].iter() {
        let room = setup_room(*size, 0.5);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            let dial = Dial::new(0.5);
            b.iter(|| black_box(room.query(black_box(dial))));
        });
    }

    group.finish();
}

fn bench_room_query_inferences_separate(c: &mut Criterion) {
    let room = setup_room(100, 0.5);

    c.benchmark_group("room_query_inferences")
        .bench_function("query_at_various_dials", |b| {
            let dials = [0.0, 0.25, 0.5, 0.75, 1.0];
            b.iter(|| {
                for &pos in &dials {
                    black_box(room.query_inferences(black_box(Dial::new(pos))));
                }
            });
        });
}

fn bench_room_cascade(c: &mut Criterion) {
    c.benchmark_group("room_cascade")
        .bench_function("cascade_depth_3", |b| {
            b.iter(|| {
                let mut root = Room::new("root");
                let mut child1 = Room::new("child1");
                let mut child2 = Room::new("child2");
                let mut child3 = Room::new("child3");

                // Build tree
                child1
                    .children
                    .insert("child1a".to_string(), Room::new("child1a"));
                child1
                    .children
                    .insert("child1b".to_string(), Room::new("child1b"));
                child2
                    .children
                    .insert("child2a".to_string(), Room::new("child2a"));
                child3
                    .children
                    .insert("child3a".to_string(), Room::new("child3a"));

                root.children.insert("child1".to_string(), child1);
                root.children.insert("child2".to_string(), child2);
                root.children.insert("child3".to_string(), child3);

                // Add inferences
                for i in 0..10 {
                    root.add_inference(
                        serde_json::json!({"hyp": format!("h-{}", i)}),
                        0.6 + (i as f64) * 0.04,
                    );
                }

                root.cascade(black_box(3));
                black_box(());
            });
        });
}

fn bench_signal_chain_operations(c: &mut Criterion) {
    c.benchmark_group("signal_chain_create")
        .bench_function("create_with_10_rooms", |b| {
            b.iter(|| {
                black_box(setup_chain(10, 10));
            });
        });

    c.benchmark_group("signal_chain_query_all")
        .bench_function("query_all_rooms_soft", |b| {
            let chain = setup_chain(10, 100);
            b.iter(|| {
                black_box(chain.query_all(black_box(Dial::soft())));
            });
        });

    c.benchmark_group("signal_chain_traverse")
        .bench_function("traverse_5_rooms", |b| {
            let chain = setup_chain(10, 10);
            let names = ["room-0", "room-2", "room-4", "room-6", "room-8"];
            b.iter(|| {
                black_box(chain.traverse(black_box(&names)));
            });
        });
}

fn bench_room_children(c: &mut Criterion) {
    c.benchmark_group("room_children")
        .bench_function("insert_and_query_children", |b| {
            b.iter(|| {
                let mut room = Room::new("parent");
                for i in 0..20 {
                    room.children
                        .insert(format!("child-{}", i), Room::new(&format!("child-{}", i)));
                }
                // Query all children
                for child in room.children.values() {
                    black_box(child.query_snaps());
                }
            });
        });
}

fn bench_room_serialization(c: &mut Criterion) {
    let room = setup_room(100, 0.5);
    let chain = setup_chain(10, 10);

    c.benchmark_group("serialization")
        .bench_function("room_to_json", |b| {
            b.iter(|| {
                black_box(serde_json::to_string(&room).unwrap());
            });
        });

    c.benchmark_group("serialization")
        .bench_function("room_from_json", |b| {
            let json = serde_json::to_string(&room).unwrap();
            b.iter(|| {
                black_box(serde_json::from_str::<Room>(&json).unwrap());
            });
        });

    c.benchmark_group("serialization")
        .bench_function("chain_to_json", |b| {
            b.iter(|| {
                black_box(serde_json::to_string(&chain).unwrap());
            });
        });

    c.benchmark_group("serialization")
        .bench_function("chain_from_json", |b| {
            let json = serde_json::to_string(&chain).unwrap();
            b.iter(|| {
                black_box(serde_json::from_str::<SignalChain>(&json).unwrap());
            });
        });
}

criterion_group!(
    benches,
    bench_room_add_snap,
    bench_room_query,
    bench_room_query_inferences_separate,
    bench_room_cascade,
    bench_signal_chain_operations,
    bench_room_children,
    bench_room_serialization,
);
criterion_main!(benches);
