// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! basic_dial — demonstrates Dial creation and querying at different levels
//!
//! Run with: cargo run --example basic_dial -p openshell-signal-chain

use openshell_signal_chain::{Dial, Room};

fn main() {
    println!("=== Signal Chain Dial Examples ===\n");

    // 1. Create a room and add snaps (hard facts)
    let mut room = Room::new("sonar-readings");
    room.add_snap(
        serde_json::json!({
            "type": "depth",
            "value": 42.5,
            "unit": "meters"
        }),
        1.0,
    );
    room.add_snap(
        serde_json::json!({
            "type": "temperature",
            "value": 12.3,
            "unit": "celsius"
        }),
        1.0,
    );

    // 2. Add inferences (soft extrapolations)
    room.add_inference(
        serde_json::json!({
            "type": "prediction",
            "hypothesis": "depth will decrease",
            "delta": -2.1
        }),
        0.75,
    );
    room.add_inference(
        serde_json::json!({
            "type": "speculation",
            "hypothesis": "fish school detected nearby",
            "confidence_signal": 0.3
        }),
        0.3,
    );

    // 3. Query at different dial levels
    println!("--- Dial: HARD (0.0) ---");
    println!("  Threshold: {:.1}", Dial::hard().inference_threshold());
    let hard_results = room.query(Dial::hard());
    println!("  Results ({} items):", hard_results.len());
    for r in &hard_results {
        println!("    {:?}", r);
    }

    println!("\n--- Dial: ANALYSIS (0.4) ---");
    let analysis_dial = Dial::new(0.4);
    println!("  Threshold: {:.1}", analysis_dial.inference_threshold());
    let analysis_results = room.query(analysis_dial);
    println!("  Results ({} items):", analysis_results.len());
    for r in &analysis_results {
        println!("    {:?}", r);
    }

    println!("\n--- Dial: CREATIVE (0.9) ---");
    let creative_dial = Dial::new(0.9);
    println!("  Threshold: {:.1}", creative_dial.inference_threshold());
    let creative_results = room.query(creative_dial);
    println!("  Results ({} items):", creative_results.len());
    for r in &creative_results {
        println!("    {:?}", r);
    }

    // 4. Use preset dials
    println!("\n--- Using Preset Dials ---");
    println!(
        "  DIAL_FORMAL (formal reasoning): position = {:.1}",
        openshell_signal_chain::DIAL_FORMAL.position
    );
    println!(
        "  DIAL_BATHY (bathymetric data):  position = {:.1}",
        openshell_signal_chain::DIAL_BATHY.position
    );
    println!(
        "  DIAL_COMMIT (git history):      position = {:.1}",
        openshell_signal_chain::DIAL_COMMIT.position
    );
    println!(
        "  DIAL_REVIEW (balanced):         position = {:.1}",
        openshell_signal_chain::DIAL_REVIEW.position
    );
    println!(
        "  DIAL_CREATIVE (generative):     position = {:.1}",
        openshell_signal_chain::DIAL_CREATIVE.position
    );

    // 5. Query snaps and inferences separately
    println!("\n--- Separate Queries ---");
    let snaps = room.query_snaps();
    println!("  Snaps only ({}): {:?}", snaps.len(), snaps);

    let inferences = room.query_inferences(Dial::new(0.5));
    println!(
        "  Inferences at 0.5 ({}): {:?}",
        inferences.len(),
        inferences
    );

    // 6. Demonstrate threshold checking
    println!("\n--- Inference Threshold Demo ---");
    let dial = Dial::new(0.6); // threshold = 0.4
    println!("  Dial position: {:.1}", dial.position);
    println!("  Inference threshold: {:.1}", dial.inference_threshold());
    println!("  Accepts 0.8 confidence? {}", dial.accepts_inference(0.8));
    println!("  Accepts 0.3 confidence? {}", dial.accepts_inference(0.3));
    println!("  Snap weight: {:.1}", dial.snap_weight());
    println!("  Inference weight: {:.1}", dial.inference_weight());

    println!("\n=== Done ===");
}
