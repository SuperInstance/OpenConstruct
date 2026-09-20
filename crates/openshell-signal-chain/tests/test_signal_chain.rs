// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for signal-chain.

use openshell_signal_chain::{
    Dial, Room, SignalChain, SignalChainError, DIAL_ANALYSIS, DIAL_FORMAL,
};

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
    drone.add_snap(
        serde_json::json!({"lat": 45.3, "lon": -122.8, "depth": 87.2}),
        1.0,
    );
    drone.add_inference(
        serde_json::json!({"hypothesis": "possible anchor at 45.5, -123.0"}),
        0.6,
    );

    // Formal analysis room
    chain.room_with_dial("formal-proof", Dial::hard());
    let formal = chain.room("formal-proof");
    formal.add_snap(
        serde_json::json!({"theorem": "H1_cohomology_detects_emergence"}),
        1.0,
    );

    // Query at different dials
    let all = chain.query_all(DIAL_ANALYSIS);
    assert_eq!(all.len(), 2);
}

#[test]
fn test_cascade() {
    // Two sibling rooms in a SignalChain (not parent→child nested hierarchy).
    let mut chain = SignalChain::new("test");
    chain
        .room("parent")
        .add_inference(serde_json::json!({"idea": "from_parent"}), 0.8);

    // Child starts empty — cascade_from must prove propagation across siblings.
    chain.room("child");
    assert_eq!(chain.get_room("child").unwrap().snaps.len(), 0);

    chain.cascade_from("parent", 1);

    let child = chain.get_room("child").unwrap();
    assert_eq!(
        child.snaps.len(),
        1,
        "cascade_from should inject one snap into sibling"
    );
    let expected = 0.8_f64 * 0.8;
    assert!(
        (child.snaps[0].confidence - expected).abs() < 1e-9,
        "snap confidence should be 0.8 × 0.8 = {expected}, got {}",
        child.snaps[0].confidence,
    );
}

#[test]
fn test_room_child_hierarchy() {
    let mut room = Room::new("parent");
    room.add_inference(serde_json::json!({"level": "parent_inference"}), 0.7);

    // Add child
    room.children
        .insert("child".to_string(), Room::new("child"));

    room.cascade(1);

    let child = room.children.get("child").unwrap();
    assert_eq!(child.snaps.len(), 1); // cascaded from parent
}

// --- Edge case tests ---

#[test]
fn test_empty_room_cascade() {
    // Room with no inferences cascading into children
    let mut parent = Room::new("empty-parent");
    parent
        .children
        .insert("child".to_string(), Room::new("child"));

    parent.cascade(1);

    let child = parent.children.get("child").unwrap();
    assert!(child.snaps.is_empty());
    assert!(child.inferences.is_empty());
}

#[test]
fn test_dial_at_exact_boundaries() {
    let zero = Dial::new(0.0);
    assert_eq!(zero.position, 0.0);
    assert_eq!(zero.snap_weight(), 1.0);
    assert_eq!(zero.inference_weight(), 0.0);

    let one = Dial::new(1.0);
    assert_eq!(one.position, 1.0);
    assert_eq!(one.snap_weight(), 0.0);
    assert_eq!(one.inference_weight(), 1.0);

    // try_new should accept exact boundaries
    assert!(Dial::try_new(0.0).is_ok());
    assert!(Dial::try_new(1.0).is_ok());
}

#[test]
fn test_room_many_snaps() {
    let mut room = Room::new("high-volume");

    // Add 150 snaps
    for i in 0..150 {
        room.add_snap(serde_json::json!({"index": i}), 1.0);
    }

    assert_eq!(room.snaps.len(), 150);

    let results = room.query(Dial::hard());
    assert_eq!(results.len(), 150);
}

#[test]
fn test_confidence_zero() {
    let mut room = Room::new("test");
    room.add_inference(serde_json::json!({"wild": "guess"}), 0.0);

    // At dial 1.0 (threshold 0.0), confidence 0.0 passes
    let soft_results = room.query(Dial::soft());
    assert_eq!(soft_results.len(), 1);

    // At dial 0.0 (threshold 1.0), confidence 0.0 fails
    let hard_results = room.query(Dial::hard());
    assert_eq!(hard_results.len(), 0);
}

#[test]
fn test_query_empty_chain() {
    let chain = SignalChain::new("empty");
    let results = chain.query_all(Dial::new(0.5));
    assert!(results.is_empty());
}

#[test]
fn test_cascade_sorts_by_confidence() {
    let mut parent = Room::new("parent");
    // Add inferences in reverse confidence order
    parent.add_inference(serde_json::json!({"c": 3}), 0.6);
    parent.add_inference(serde_json::json!({"a": 1}), 0.95);
    parent.add_inference(serde_json::json!({"b": 2}), 0.8);
    parent.add_inference(serde_json::json!({"noise": 0}), 0.2); // below 0.5, excluded

    parent
        .children
        .insert("child".to_string(), Room::new("child"));
    parent.cascade(1);

    let child = parent.children.get("child").unwrap();
    assert_eq!(child.snaps.len(), 2); // top 2 only

    // Should be sorted: highest first → 0.95, then 0.8
    assert_eq!(child.snaps[0].confidence, 0.95 * 0.8);
    assert_eq!(child.snaps[1].confidence, 0.8 * 0.8);
}

#[test]
fn test_try_new_rejects_invalid() {
    assert!(matches!(
        Dial::try_new(-0.1),
        Err(SignalChainError::InvalidDial(_))
    ));
    assert!(matches!(
        Dial::try_new(1.1),
        Err(SignalChainError::InvalidDial(_))
    ));
    assert!(matches!(
        Dial::try_new(f64::NAN),
        Err(SignalChainError::InvalidDial(_))
    ));
    assert!(matches!(
        Dial::try_new(f64::INFINITY),
        Err(SignalChainError::InvalidDial(_))
    ));
}

#[test]
fn test_traverse_skips_missing() {
    let mut chain = SignalChain::new("test");
    chain.room("a");
    chain.room("c");

    let rooms = chain.traverse(&["a", "b", "c"]);
    assert_eq!(rooms.len(), 2);
    assert_eq!(rooms[0].name, "a");
    assert_eq!(rooms[1].name, "c");
}

#[test]
fn test_snap_absolute() {
    let mut room = Room::new("test");
    room.add_absolute(serde_json::json!({"proven": true}));

    assert_eq!(room.snaps.len(), 1);
    assert_eq!(room.snaps[0].confidence, 1.0);
}
