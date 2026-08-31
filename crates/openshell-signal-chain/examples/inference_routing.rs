// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! inference_routing — demonstrates how InferenceLevel maps to Dial positions
//!
//! This example shows the relationship between desired inference confidence
//! and the dial position that accepts/rejects inferences.
//!
//! Run with: cargo run --example inference_routing -p openshell-signal-chain

use openshell_signal_chain::{Dial, Room};

fn main() {
    println!("=== Inference Routing Examples ===\n");

    // Create a room with inferences at various confidence levels
    let mut room = Room::new("routing-demo");
    room.add_snap(serde_json::json!({"fact": "absolute-truth"}), 1.0);

    // Add inferences at different confidence levels
    room.add_inference(serde_json::json!({"level": "proven", "conf": 1.0}), 1.0);
    room.add_inference(serde_json::json!({"level": "high", "conf": 0.85}), 0.85);
    room.add_inference(
        serde_json::json!({"level": "medium-high", "conf": 0.7}),
        0.7,
    );
    room.add_inference(serde_json::json!({"level": "medium", "conf": 0.5}), 0.5);
    room.add_inference(serde_json::json!({"level": "low", "conf": 0.3}), 0.3);
    room.add_inference(
        serde_json::json!({"level": "speculative", "conf": 0.1}),
        0.1,
    );

    // Table showing dial position -> threshold -> which inferences pass
    println!("--- Dial Position to Inference Routing Table ---");
    println!("{:<8} {:<12} Accepted Inferences", "Dial", "Threshold");
    println!("{}", "-".repeat(60));

    let positions = [0.0, 0.2, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    for pos in positions {
        let dial = Dial::new(pos);
        let threshold = dial.inference_threshold();
        let inferences_vec = room.query_inferences(dial);
        let accepted: Vec<_> = inferences_vec
            .iter()
            .filter_map(|v| v.get("level").and_then(|l| l.as_str()))
            .collect();
        println!("  {:.1}     {:.1}         {:?}", pos, threshold, accepted);
    }

    // Demonstrate use case: finding the right dial for a minimum confidence
    println!("\n--- Finding Dial for Minimum Confidence ---");

    fn dial_for_confidence(target_confidence: f64) -> Dial {
        // We want threshold <= target_confidence
        // threshold = 1.0 - position
        // So: 1.0 - position <= target_confidence
        // position >= 1.0 - target_confidence
        Dial::new(1.0 - target_confidence)
    }

    let targets = [0.9, 0.7, 0.5, 0.3];
    for target in targets {
        let dial = dial_for_confidence(target);
        println!(
            "  To accept inferences with conf >= {:.1}: dial position = {:.1}",
            target, dial.position
        );
        println!(
            "    Query result: {} inferences",
            room.query_inferences(dial).len()
        );
    }

    // Demonstrate routing use case: formal analysis (high threshold)
    println!("\n--- Use Case: Formal Analysis (High Threshold) ---");
    let formal_dial = Dial::new(0.1); // threshold = 0.9
    let formal_results = room.query(formal_dial);
    println!("  Formal analysis (dial=0.1, threshold=0.9):");
    for r in &formal_results {
        println!("    {:?}", r);
    }

    // Demonstrate routing use case: exploratory (low threshold)
    println!("\n--- Use Case: Exploratory Analysis (Low Threshold) ---");
    let explore_dial = Dial::new(0.8); // threshold = 0.2
    let explore_results = room.query(explore_dial);
    println!(
        "  Exploratory (dial=0.8, threshold=0.2): {} results",
        explore_results.len()
    );
    for r in &explore_results {
        println!("    {:?}", r);
    }

    // Demonstrate cascade with routing
    println!("\n--- Cascade with Routing ---");
    let mut high_conf_room = Room::new("high-confidence-source");
    let mut low_conf_room = Room::new("low-confidence-target");

    high_conf_room.add_inference(
        serde_json::json!({"source": "high", "prediction": "confirmed"}),
        0.95,
    );
    high_conf_room.add_inference(
        serde_json::json!({"source": "medium", "prediction": "maybe"}),
        0.55,
    );

    low_conf_room.add_snap(serde_json::json!({"existing": "fact"}), 1.0);

    high_conf_room
        .children
        .insert("low-confidence-target".to_string(), low_conf_room);
    high_conf_room.cascade(1);

    if let Some(child) = high_conf_room.children.get("low-confidence-target") {
        println!("  After cascade: child has {} snaps", child.snaps.len());
        for snap in &child.snaps {
            println!("    {:?} (confidence: {})", snap.fact, snap.confidence);
        }
    }

    // Weight-based selection
    println!("\n--- Weight-Based Result Mixing ---");
    let dial = Dial::new(0.6);
    println!("  Dial position: {:.1}", dial.position);
    println!(
        "  Snap weight: {:.1} (hard results count more)",
        dial.snap_weight()
    );
    println!(
        "  Inference weight: {:.1} (soft results count more)",
        dial.inference_weight()
    );

    let results = room.query(dial);
    let snap_count = room.query_snaps().len();
    let inference_count = results.len() - snap_count;
    println!(
        "  Results: {} snaps + {} inferences = {} total",
        snap_count,
        inference_count,
        results.len()
    );

    println!("\n=== Done ===");
}
