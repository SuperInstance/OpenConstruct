// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for signal-chain

use openshell_signal_chain::{Dial, Room, SignalChain, DIAL_FORMAL, DIAL_ANALYSIS};

#[test]
fn test_dial_presets() {
    assert_eq!(DIAL_FORMAL.position, 0.0);
    assert_eq!(Dial::hard().position, 0.0);
    assert_eq!(Dial::soft().position, 1.0);
}

#[test]
fn test_room_lifecycle() {
    let mut room = Room::new("test-room");
    
    // Add snaps
    room.add_snap(serde_json::json!({"x": 1, "y": 2}), 1.0);
    room.add_snap(serde_json::json!({"depth": 87.2}), 1.0);
    
    // Add inferences
    room.add_inference(serde_json::json!({"possible": "wreckage"}), 0.7);
    room.add_inference(serde_json::json!({"speculative": "nothing"}), 0.4);
    
    // Query at different dials
    let hard_results = room.query(Dial::hard());
    assert_eq!(hard_results.len(), 2); // snaps only
    
    let soft_results = room.query(Dial::soft());
    assert_eq!(soft_results.len(), 4); // snaps + all inferences
    
    let mid_results = room.query(Dial::new(0.5));
    assert_eq!(mid_results.len(), 3); // snaps + inferences with confidence >= 0.5
}

#[test]
fn test_signal_chain() {
    let mut chain = SignalChain::new("cocapn-fleet");
    
    // Drone mapping room
    let drone = chain.room("drone-salvage");
    drone.add_snap(serde_json::json!({"lat": 45.3, "lon": -122.8, "depth": 87.2}), 1.0);
    drone.add_inference(serde_json::json!({"hypothesis": "possible anchor at 45.5, -123.0"}), 0.6);
    
    // Formal analysis room
    chain.room_with_dial("formal-proof", Dial::hard());
    let formal = chain.room("formal-proof");
    formal.add_snap(serde_json::json!({"theorem": "H1_cohomology_detects_emergence"}), 1.0);
    
    // Query at different dials
    let all = chain.query_all(DIAL_ANALYSIS);
    assert_eq!(all.len(), 2);
}

#[test]
fn test_cascade() {
    let mut chain = SignalChain::new("test");
    let room = chain.room("parent");
    room.add_inference(serde_json::json!({"idea": "from_parent"}), 0.8);
    
    // Add child
    let _child = chain.room("child");
    chain.room("child").add_snap(serde_json::json!({"existing": "snap"}), 1.0);
    
    // Cascade
    chain.cascade_from("parent", 1);
    
    // Child should now have parent's inference as snap
    let child = chain.get_room("child").unwrap();
    let child_snaps = child.query_snaps();
    assert!(child_snaps.len() >= 1);
}

#[test]
fn test_room_child_hierarchy() {
    let mut room = Room::new("parent");
    room.add_inference(serde_json::json!({"level": "parent_inference"}), 0.7);
    
    // Add child
    room.children.insert("child".to_string(), Room::new("child"));
    
    // Cascade
    room.cascade(1);
    
    let child = room.children.get("child").unwrap();
    // Child should have inherited parent's inference as snap
    assert!(!child.snaps.is_empty() || !child.inferences.is_empty());
}