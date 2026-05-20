// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! signal_chain_room — demonstrates Room with snaps/inferences and cascade
//!
//! Run with: cargo run --example signal_chain_room -p openshell-signal-chain

use openshell_signal_chain::{Dial, Room, SignalChain, DIAL_BATHY};

fn main() {
    println!("=== Signal Chain Room Examples ===\n");

    // 1. Basic room creation
    println!("--- 1. Basic Room ---");
    let mut room = Room::new("sensor-data");
    room.add_snap(serde_json::json!({"depth": 100.0}), 1.0);
    room.add_snap(serde_json::json!({"heading": 270.0}), 1.0);
    room.add_inference(serde_json::json!({"weather": "storm approaching"}), 0.85);
    room.add_inference(serde_json::json!({"fish": "herring school at 50m"}), 0.6);

    println!("  Room: {}", room.name);
    println!("  Snaps: {:?}", room.query_snaps());
    println!("  Query at hard: {:?}", room.query(Dial::hard()));
    println!("  Query at soft: {:?}", room.query(Dial::soft()));

    // 2. Room with specific dialect
    println!("\n--- 2. Room with Dialect (Bathymetric) ---");
    let mut bathy_room = Room::with_dial("bathy-data", DIAL_BATHY);
    bathy_room.add_absolute(serde_json::json!({"depth": 250.5, "sensor": "multibeam"}));
    bathy_room.add_inference(serde_json::json!({"seabed_type": "sand"}), 0.7);
    println!("  Room dial position: {:.1}", bathy_room.dial_position.position);

    // 3. SignalChain with multiple rooms
    println!("\n--- 3. SignalChain ---");
    let mut chain = SignalChain::new("fishing-expedition");

    // Add rooms
    let nav = chain.room("navigation");
    nav.add_snap(serde_json::json!({"lat": 54.5, "lon": -2.5}), 1.0);
    nav.add_inference(serde_json::json!({"eta": "14:30"}), 0.9);

    let catch = chain.room("catch-log");
    catch.add_snap(serde_json::json!({"species": "cod", "weight": 45.0}), 1.0);
    catch.add_inference(serde_json::json!({"next_bite": "20 min"}), 0.5);

    // Room with custom dial
    let weather = chain.room_with_dial("weather", Dial::new(0.3));
    weather.add_snap(serde_json::json!({"wind_knots": 15}), 1.0);
    weather.add_inference(serde_json::json!({"storm": "warning"}), 0.8);

    // Query all rooms at different dials
    println!("  All rooms at hard (0.0):");
    let all_hard = chain.query_all(Dial::hard());
    for (name, results) in &all_hard {
        println!("    {}: {} results", name, results.len());
    }

    println!("\n  All rooms at soft (1.0):");
    let all_soft = chain.query_all(Dial::soft());
    for (name, results) in &all_soft {
        println!("    {}: {} results", name, results.len());
    }

    // 4. Traverse rooms in order
    println!("\n--- 4. Room Traversal ---");
    let rooms = chain.traverse(&["navigation", "catch-log", "weather"]);
    for room in rooms {
        println!("  Traversed: {} (snaps: {}, inferences: {})",
            room.name, room.snaps.len(), room.inferences.len());
    }

    // 5. Cascade demonstration
    println!("\n--- 5. Cascade ---");
    let mut parent = Room::new("analysis-root");
    let child_a = Room::new("child-a");
    let child_b = Room::new("child-b");

    // Add high-confidence inference to parent
    parent.add_inference(serde_json::json!({"hypothesis": "pattern-detected"}), 0.85);
    parent.add_inference(serde_json::json!({"hypothesis": "anomaly-spotted"}), 0.6);

    // Set up children
    parent.children.insert("child-a".to_string(), child_a);
    parent.children.insert("child-b".to_string(), child_b);

    println!("  Before cascade:");
    println!("    parent inferences: {}", parent.inferences.len());
    if let Some(c) = parent.children.get("child-a") {
        println!("    child-a snaps: {}", c.snaps.len());
    }

    // Cascade depth 1
    parent.cascade(1);

    println!("\n  After cascade (depth=1):");
    if let Some(c) = parent.children.get("child-a") {
        println!("    child-a snaps (from inference): {}", c.snaps.len());
        for snap in &c.snaps {
            println!("      -> {:?}", snap.fact);
        }
    }

    // 6. Room metadata
    println!("\n--- 6. Room Metadata ---");
    let mut meta_room = Room::new("annotated");
    meta_room.metadata.insert("vessel".to_string(), serde_json::json!("MSC-Seeker"));
    meta_room.metadata.insert("captain".to_string(), serde_json::json!("Ng"));
    meta_room.metadata.insert("date".to_string(), serde_json::json!("2026-05-20"));
    meta_room.add_snap(serde_json::json!({"catch": "tuna"}), 1.0);

    println!("  Metadata: {:?}", meta_room.metadata);
    println!("  Room complete: {} with {} snaps",
        meta_room.name, meta_room.snaps.len());

    println!("\n=== Done ===");
}
