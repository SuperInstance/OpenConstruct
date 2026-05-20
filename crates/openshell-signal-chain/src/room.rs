// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Room — spatial/temporal anchor with snaps and inferences
//! 
//! A room is a fact-space. It contains hard-locked snaps and soft inferences.
//! Query at any dial level to get facts + extrapolations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{Dial, Snap, Inference};

/// A room is a tile in the signal chain — a fact-space with:
/// - snaps: hard-locked facts (ground truth anchors)
/// - inferences: soft extrapolations (hypotheses, predictions)
/// - dialect: current dial position for this room
/// 
/// Rooms can be queried at any inference level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Room name/identifier
    pub name: String,
    /// Default dial position for this room
    pub dialect: Dial,
    /// Hard-locked facts
    pub snaps: Vec<Snap>,
    /// Soft extrapolations
    pub inferences: Vec<Inference>,
    /// Child rooms (sub-tiles)
    #[serde(default)]
    pub children: HashMap<String, Room>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Room {
    /// Create a new room with given name and optional dial
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dialect: Dial::default(),
            snaps: Vec::new(),
            inferences: Vec::new(),
            children: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new room with specific dial position
    pub fn with_dial(name: &str, dialect: Dial) -> Self {
        Self {
            name: name.to_string(),
            dialect,
            snaps: Vec::new(),
            inferences: Vec::new(),
            children: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a hard-locked fact
    pub fn add_snap(&mut self, fact: serde_json::Value, confidence: f64) -> &mut Snap {
        let dialect_pos = self.dialect.position;
        self.snaps.push(Snap::new(fact, confidence, dialect_pos));
        self.snaps.last_mut().unwrap()
    }

    /// Add an absolute fact (confidence = 1.0)
    pub fn add_absolute(&mut self, fact: serde_json::Value) -> &mut Snap {
        self.add_snap(fact, 1.0)
    }

    /// Add a soft extrapolation
    pub fn add_inference(&mut self, hypothesis: serde_json::Value, confidence: f64) -> &mut Inference {
        let dialect_pos = self.dialect.position;
        self.inferences.push(Inference::new(hypothesis, confidence, dialect_pos));
        self.inferences.last_mut().unwrap()
    }

    /// Query this room at a specific dial level
    /// 
    /// Returns snaps always + inferences that pass threshold
    pub fn query(&self, dial: Dial) -> Vec<serde_json::Value> {
        let threshold = dial.inference_threshold();
        
        let mut results: Vec<serde_json::Value> = self.snaps
            .iter()
            .map(|s| s.fact.clone())
            .collect();
        
        for inf in &self.inferences {
            if inf.confidence >= threshold {
                results.push(inf.hypothesis.clone());
            }
        }
        
        results
    }

    /// Query snaps only (hard facts)
    pub fn query_snaps(&self) -> Vec<serde_json::Value> {
        self.snaps.iter().map(|s| s.fact.clone()).collect()
    }

    /// Query inferences only at given dial
    pub fn query_inferences(&self, dial: Dial) -> Vec<serde_json::Value> {
        let threshold = dial.inference_threshold();
        self.inferences
            .iter()
            .filter(|inf| inf.confidence >= threshold)
            .map(|inf| inf.hypothesis.clone())
            .collect()
    }

    /// Cascade top inferences as snaps into children
    pub fn cascade(&mut self, depth: usize) {
        if depth == 0 { return; }
        
        for (name, child) in &mut self.children {
            // Snap top inferences from this room into child
            for inf in self.inferences.iter().take(2) {
                if inf.confidence > 0.5 {
                    let mut snap_fact = inf.hypothesis.clone();
                    if let Some(obj) = snap_fact.as_object_mut() {
                        obj.insert("_source".to_string(), serde_json::json!(inf.confidence));
                    }
                    child.add_snap(snap_fact, inf.confidence * 0.8);
                }
            }
            
            // Continue cascading
            child.cascade(depth - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_snap_and_inference() {
        let mut room = Room::new("test");
        room.add_snap(serde_json::json!({"x": 1}), 1.0);
        room.add_inference(serde_json::json!({"y": 2}), 0.7);
        
        assert_eq!(room.snaps.len(), 1);
        assert_eq!(room.inferences.len(), 1);
    }

    #[test]
    fn test_query_at_dial_zero() {
        let mut room = Room::new("test");
        room.add_snap(serde_json::json!({"snap": 1}), 1.0);
        room.add_inference(serde_json::json!({"inf": 1}), 0.8);
        
        let results = room.query(Dial::hard());
        assert_eq!(results.len(), 1); // only snaps
    }

    #[test]
    fn test_query_at_high_dial() {
        let mut room = Room::new("test");
        room.add_snap(serde_json::json!({"snap": 1}), 1.0);
        room.add_inference(serde_json::json!({"inf": 1}), 0.35);
        room.add_inference(serde_json::json!({"inf": 2}), 0.8);
        
        let results = room.query(Dial { position: 0.7 }); // threshold = 0.3
        assert_eq!(results.len(), 3); // snap + 2 inferences
    }
}