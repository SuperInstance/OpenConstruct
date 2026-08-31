// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! vm_integration — Signal chain × spline constraints × holonomy rooms
//!
//! Run with: cargo run --example vm_integration -p openshell-signal-chain

use openshell_signal_chain::{Dial, HolonomyRoom, SignalChain, maritime_spline};

fn main() {
    println!("=== Signal Chain × Spline × Holonomy Integration ===\n");

    // ── Part 1: Maritime spline constraints ──────────────────────────────
    println!("─── 1. Maritime Spline Constraints ───\n");

    let constraints = maritime_spline();
    println!("  {} constraints:", constraints.len());
    for c in &constraints {
        println!(
            "    {:25} range=[{:5}, {:5}] neutral={:.1}",
            c.name, c.lo, c.hi, c.neutral
        );
    }

    let readings = vec![
        ("wave_height_dm", 35_i32),
        ("sea_temp_celsius", 12_i32),
        ("wind_speed_knots", 22_i32),
        ("pressure_deviation_hpa", -8_i32),
    ];

    println!("\n  Readings → curvature distances:");
    for (name, value) in &readings {
        if let Some(c) = constraints.iter().find(|c| c.name == *name) {
            let d = c.curvature_distance(*value);
            let status = if d > 1.0 {
                "⚠ violation"
            } else if d > 0.5 {
                "~ boundary"
            } else {
                "✓ safe"
            };
            println!("    {:25} = {:5}  → {:.3}  {}", name, value, d, status);
        }
    }

    // ── Part 2: Signal chain rooms ───────────────────────────────────────
    println!("\n─── 2. Signal Chain Rooms ───\n");

    let mut chain = SignalChain::new("cocapn-fleet");

    let nav = chain.room("navigation");
    nav.add_snap(
        serde_json::json!({"lat": 54.432, "lon": -2.891, "heading": 127.4}),
        1.0,
    );
    nav.add_inference(serde_json::json!({"eta": "14:30"}), 0.85);

    let sonar = chain.room("sonar-array");
    sonar.add_snap(
        serde_json::json!({"contact": "solid", "bearing": 127.4, "range_m": 200}),
        1.0,
    );
    sonar.add_inference(serde_json::json!({"hypothesis": "metal wreck"}), 0.75);

    let weather = chain.room_with_dial("weather", Dial::new(0.6));
    weather.add_snap(
        serde_json::json!({"wind_knots": 22, "pressure_hpa": 1008}),
        1.0,
    );
    weather.add_inference(serde_json::json!({"storm": "gale force 8 in 6hrs"}), 0.7);

    println!("  Rooms at different dials:");
    for (label, dial) in [
        ("hard (0.0)", Dial::hard()),
        ("analysis (0.5)", Dial::new(0.5)),
        ("soft (1.0)", Dial::soft()),
    ] {
        let results = chain.query_all(dial);
        let total: usize = results.values().map(|v| v.len()).sum();
        print!("    {:18}: {} results  [", label, total);
        for (name, items) in &results {
            print!(" {}={}", name, items.len());
        }
        println!(" ]");
    }

    // ── Part 3: Holonomy rooms (Betti numbers) ───────────────────────────
    println!("\n─── 3. Holonomy Rooms — Betti Analysis ───\n");

    let scenarios = vec![
        ("scouting", 3, 3),
        ("transit", 5, 7),
        ("dockside", 4, 10),
        ("storm-ops", 6, 15),
    ];

    let mut rigid = 0;
    let mut emergent = 0;

    for (name, v, e) in &scenarios {
        let mut room = HolonomyRoom::new(name, Dial::hard());
        for _ in 0..*v {
            room.add_snap();
        }
        for _ in 0..*e {
            room.add_edge();
        }

        let b = room.betti();
        let thr = if *v >= 2 { *v - 2 } else { 0 };

        let label = if b.is_rigid {
            rigid += 1;
            "RIGID ✓"
        } else if b.has_emergence {
            emergent += 1;
            "EMERGENT"
        } else {
            "loose"
        };

        println!(
            "  {:12}  V={:2} E={:3}  β₁={:2}  (V-2={:2})  {}",
            name, v, e, b.beta, thr, label
        );
    }

    println!("\n  Fleet: {} rigid, {} emergent", rigid, emergent);

    // ── Part 4: Dial × constraint interaction ─────────────────────────────
    println!("\n─── 4. Dial × Constraint Interaction ───\n");

    let wave = constraints
        .iter()
        .find(|c| c.name == "wave_height_dm")
        .unwrap();
    let wave_val = 40_i32;
    let dist = wave.curvature_distance(wave_val);
    println!("  Wave height: {} dm → curvature={:.3}", wave_val, dist);
    println!(
        "  At hard (0.0): {} violation",
        if dist > 1.0 { "⚠" } else { "no" }
    );
    println!("  At soft (1.0): advisory, not blocking");

    println!("\n=== Integration complete ===");
    println!("Rooms hold snaps (verified) and inferences (propagating).");
    println!("SplineConstraints model physical limits (wave, temp, wind, pressure).");
    println!("HolonomyRooms detect emergence via β₁ = E - V + C.");
    println!("The dial controls what surfaces — hard=snaps only, soft=everything.");
}
