// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Fleet Operations Example
//!
//! Demonstrates a real workflow using the signal chain:
//! 1. Create a chain for fleet operations
//! 2. Add rooms for different parts of the fleet
//! 3. Add snaps (hard sensor readings)
//! 4. Add inferences (soft predictions)
//! 5. Cascade between rooms
//! 6. Query results at different dial levels

use openshell_signal_chain::{Dial, Room, SignalChain};

fn main() {
    println!("=== OpenShell Signal Chain — Fleet Operations ===\n");

    // 1. Create a chain with a moderate default dial
    let mut chain = SignalChain::with_dial("cocapn-fleet", Dial::new(0.3));

    // 2. Add rooms for different fleet subsystems

    // Sonar array — hard sensor data
    let sonar = chain.room("sonar-array");
    sonar.add_snap(
        serde_json::json!({
            "contact": "solid",
            "bearing": 127.4,
            "range_m": 200,
            "depth_m": 87.2
        }),
        1.0,
    );
    sonar.add_snap(
        serde_json::json!({
            "contact": "weak",
            "bearing": 130.1,
            "range_m": 350,
            "depth_m": 92.0
        }),
        0.7,
    );
    sonar.add_inference(
        serde_json::json!({
            "hypothesis": "large metal object at bearing 127-130",
            "confidence_basis": "two contacts in proximity"
        }),
        0.75,
    );

    // Navigation — position and heading facts
    let nav = chain.room("navigation");
    nav.add_snap(
        serde_json::json!({
            "lat": 45.321,
            "lon": -122.845,
            "heading_deg": 125.0,
            "speed_knots": 4.2
        }),
        1.0,
    );
    nav.add_inference(
        serde_json::json!({
            "hypothesis": "current drift: 0.3 kts east",
            "basis": "GPS vs dead-reckoning delta"
        }),
        0.6,
    );

    // Analysis — human/ML interpretation (softer)
    chain.room_with_dial("analysis", Dial::new(0.6));
    let analysis = chain.room("analysis");
    analysis.add_inference(
        serde_json::json!({
            "classification": "possible shipwreck",
            "era_estimate": "early 1900s",
            "certainty": "moderate"
        }),
        0.65,
    );
    analysis.add_inference(
        serde_json::json!({
            "classification": "natural rock formation",
            "certainty": "low"
        }),
        0.3,
    );

    // Formal proof room — only hard facts
    chain.room_with_dial("formal-proofs", Dial::hard());
    let proofs = chain.room("formal-proofs");
    proofs.add_snap(
        serde_json::json!({
            "theorem": "drift_detected",
            "proof": "sonar_variance > threshold(3σ)"
        }),
        1.0,
    );

    // 3. Set up hierarchy: analysis room has sub-rooms
    let analysis_room = chain.room("analysis");
    analysis_room
        .children
        .insert("classification".to_string(), Room::new("classification"));
    analysis_room.children.insert(
        "historical-context".to_string(),
        Room::new("historical-context"),
    );

    // 4. Cascade: propagate analysis inferences to sub-rooms
    chain.cascade_from("analysis", 2);

    println!("--- Chain Structure ---");
    println!("Chain: {}", chain.name);
    println!("Global dial: {}", chain.global_dial.position);
    println!("Rooms: {}", chain.rooms.len());
    for (name, room) in &chain.rooms {
        println!(
            "  {} (dial={}, snaps={}, inferences={}, children={})",
            name,
            room.dial_position.position,
            room.snaps.len(),
            room.inferences.len(),
            room.children.len(),
        );
        for (child_name, child) in &room.children {
            println!(
                "    └─ {} (snaps={}, inferences={})",
                child_name,
                child.snaps.len(),
                child.inferences.len(),
            );
        }
    }

    // 5. Query at different dial levels
    println!("\n--- Query Results ---");

    println!("\nHard query (dial=0.0, only snaps and absolute inferences):");
    let hard_results = chain.query_all(Dial::hard());
    for (name, results) in &hard_results {
        println!("  {}: {} items", name, results.len());
    }

    println!("\nBalanced query (dial=0.5, threshold=0.5):");
    let mid_results = chain.query_all(Dial::new(0.5));
    for (name, results) in &mid_results {
        println!("  {}: {} items", name, results.len());
    }

    println!("\nSoft query (dial=1.0, everything):");
    let soft_results = chain.query_all(Dial::soft());
    for (name, results) in &soft_results {
        println!("  {}: {} items", name, results.len());
    }

    // 6. Traverse specific rooms in order
    println!("\n--- Traverse: sonar → analysis → formal-proofs ---");
    let traversal = chain.traverse(&["sonar-array", "analysis", "formal-proofs"]);
    for room in traversal {
        println!(
            "  {} (dial={}, snaps={}, inferences={})",
            room.name,
            room.dial_position.position,
            room.snaps.len(),
            room.inferences.len(),
        );
    }

    println!("\n=== Done ===");
}
