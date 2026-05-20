// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for openshell-signal-chain
//! Run with: cargo bench -p openshell-signal-chain

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use openshell_signal_chain::{Dial, Room, SignalChain, SplineConstraint, HolonomyRoom, BettiResult};

fn bench_dial(c: &mut Criterion) {
    let mut g = c.benchmark_group("Dial");
    g.bench_function("new_hard", |b| b.iter(|| { let d = Dial::hard(); black_box(d.position); }));
    g.bench_function("new_soft", |b| b.iter(|| { let d = Dial::soft(); black_box(d.position); }));
    for pos in [0.0, 0.25, 0.5, 0.75, 1.0] {
        g.bench_with_input(BenchmarkId::from_parameter(pos), &pos, |b, &p| {
            b.iter(|| { let d = Dial::new(p); black_box(d.snap_weight()); black_box(d.inference_threshold()); });
        });
    }
    g.finish();
}

fn bench_spline(c: &mut Criterion) {
    let mut g = c.benchmark_group("SplineConstraint");
    let wave = SplineConstraint::new("wave_height_dm", 0, 127, "lo_curv", 1.5, 1.5, 3.0);
    for val in [10_i32, 35, 60, 90, 120] {
        g.bench_with_input(BenchmarkId::from_parameter(val), &val, |b, &v| {
            b.iter(|| { let d = wave.curvature_distance(*v); black_box(d); });
        });
    }
    g.bench_function("maritime_preset", |b| b.iter(|| {
        use openshell_signal_chain::maritime_spline;
        black_box(maritime_spline())
    }));
    g.finish();
}

fn bench_room(c: &mut Criterion) {
    let mut g = c.benchmark_group("Room");
    g.bench_function("new", |b| b.iter(|| { let r = Room::new("t"); black_box(r); }));

    let room = {
        let mut r = Room::new("t");
        for i in 0..100 { r.add_snap(serde_json::json!({"i":i}), 1.0); }
        for i in 0..50 { r.add_inference(serde_json::json!({"i":i}), 0.7); }
        r
    };
    for d in [0.0, 0.3, 0.6, 1.0] {
        g.bench_with_input(BenchmarkId::from_parameter(d), &d, |b, &p| {
            b.iter(|| { let res = room.query(Dial::new(*p)); black_box(res); });
        });
    }
    g.finish();
}

fn bench_holonomy(c: &mut Criterion) {
    let mut g = c.benchmark_group("HolonomyRoom");
    let cases = vec![(3,3),(5,7),(10,17),(20,37),(4,10),(10,30)];
    for (v,e) in &cases {
        g.bench_with_input(BenchmarkId::new("betti", format!("V{}E{}",v,e)), &(*v,*e), |b,&(v,e)| {
            b.iter(|| {
                let mut r = HolonomyRoom::new("b", Dial::hard());
                for _ in 0..v { r.add_snap(); }
                for _ in 0..e { r.add_edge(); }
                black_box(r.betti());
            });
        });
    }
    g.bench_function("betti_pure", |b| b.iter(|| {
        black_box(BettiResult::compute(17, 10, 1));
    }));
    g.finish();
}

fn bench_chain(c: &mut Criterion) {
    let mut g = c.benchmark_group("SignalChain");
    g.bench_function("new", |b| b.iter(|| { black_box(SignalChain::new("f")); }));

    let chain = {
        let mut ch = SignalChain::new("f");
        for n in ["nav","sonar","weather","analysis"] {
            ch.room(n);
        }
        ch
    };
    for d in [0.0, 0.5, 1.0] {
        g.bench_with_input(BenchmarkId::from_parameter(d), &d, |b,&p| {
            b.iter(|| { black_box(chain.query_all(Dial::new(*p))); });
        });
    }
    g.bench_function("traverse", |b| b.iter(|| { black_box(chain.traverse(&["nav","sonar","weather","analysis"])); }));
    g.finish();
}

fn bench_tput(c: &mut Criterion) {
    let mut g = c.benchmark_group("Throughput");
    g.bench_function("add_snap_10k", |b| b.iter(|| {
        let mut r = Room::new("t");
        for _ in 0..10_000 { r.add_snap(serde_json::json!({"x":1}), 1.0); }
        black_box(r)
    }));
    g.bench_function("add_inference_10k", |b| b.iter(|| {
        let mut r = Room::new("t");
        for _ in 0..10_000 { r.add_inference(serde_json::json!({"x":1}), 0.75); }
        black_box(r)
    }));
    g.finish();
}

criterion_group!(benches, bench_dial, bench_spline, bench_room, bench_holonomy, bench_chain, bench_tput);
criterion_main!(benches);
