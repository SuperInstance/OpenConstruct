// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Room — spatial/temporal anchor with snaps and inferences.
//!
//! A room is a fact-space in the signal chain. It contains hard-locked snaps
//! (ground truth) and soft inferences (hypotheses). Query at any dial level
//! to get a different mix of hard facts and soft reasoning.
//!
//! Rooms can also contain child rooms, forming a hierarchy. The [`cascade`](Room::cascade)
//! operation propagates high-confidence inferences down to children as snaps.
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::{Dial, Room};
//!
//! let mut room = Room::new("sonar-array");
//! room.add_snap(serde_json::json!({"depth": 87.2}), 1.0);
//! room.add_inference(serde_json::json!({"possible": "wreckage"}), 0.7);
//!
//! // Hard query: only snaps
//! let hard = room.query(Dial::hard());
//! assert_eq!(hard.len(), 1);
//!
//! // Soft query: snaps + inferences
//! let soft = room.query(Dial::soft());
//! assert_eq!(soft.len(), 2);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Dial, Inference, Snap};
use crate::query::QueryResult;

/// A room is a tile in the signal chain — a fact-space with snaps, inferences,
/// and a dial position.
///
/// # Fields
///
/// - `name`: Room identifier.
/// - `dial_position`: Default dial for this room.
/// - `snaps`: Hard-locked facts (always included in queries).
/// - `inferences`: Soft extrapolations (filtered by dial threshold).
/// - `children`: Nested sub-rooms (for hierarchical fact-spaces).
/// - `metadata`: Arbitrary key-value metadata.
///
/// # Cascade
///
/// The [`cascade`](Room::cascade) method propagates the top inferences (sorted
/// by confidence, descending) into child rooms as snaps. This lets high-level
/// hypotheses flow down to more specific sub-spaces.
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::{Dial, Room};
///
/// let mut room = Room::with_dial("formal-proof", Dial::hard());
/// room.add_absolute(serde_json::json!({"theorem": "proved"}));
///
/// let results = room.query(Dial::hard());
/// assert_eq!(results.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Room name/identifier.
    pub name: String,
    /// Default dial position for this room.
    pub dial_position: Dial,
    /// Hard-locked facts. Always included in query results.
    pub snaps: Vec<Snap>,
    /// Soft extrapolations. Included in query results when confidence >= dial threshold.
    pub inferences: Vec<Inference>,
    /// Child rooms (sub-tiles in the hierarchy).
    #[serde(default)]
    pub children: HashMap<String, Room>,
    /// Additional metadata (arbitrary key-value pairs).
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Room {
    /// Create a new room with the given name and default dial (0.5).
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this room.
    ///
    /// # Returns
    ///
    /// A new [`Room`] with no snaps, no inferences, and dial at 0.5.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Room;
    ///
    /// let room = Room::new("sensor-bay-1");
    /// assert_eq!(room.name, "sensor-bay-1");
    /// assert!(room.snaps.is_empty());
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dial_position: Dial::default(),
            snaps: Vec::new(),
            inferences: Vec::new(),
            children: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new room with a specific dial position.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this room.
    /// - `dial`: The dial position to set for this room.
    ///
    /// # Returns
    ///
    /// A new [`Room`] with the specified dial position.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, Room};
    ///
    /// let room = Room::with_dial("formal-proof", Dial::hard());
    /// assert_eq!(room.dial_position.position, 0.0);
    /// ```
    pub fn with_dial(name: &str, dial: Dial) -> Self {
        Self {
            name: name.to_string(),
            dial_position: dial,
            snaps: Vec::new(),
            inferences: Vec::new(),
            children: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a hard-locked fact to this room.
    ///
    /// Snaps are always included in query results regardless of dial position.
    ///
    /// # Parameters
    ///
    /// - `fact`: The fact data as arbitrary JSON.
    /// - `confidence`: Confidence level (0.0–1.0).
    ///
    /// # Returns
    ///
    /// A mutable reference to the newly created [`Snap`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Room;
    ///
    /// let mut room = Room::new("sonar");
    /// room.add_snap(serde_json::json!({"depth": 42.5}), 1.0);
    /// assert_eq!(room.snaps.len(), 1);
    /// ```
    pub fn add_snap(&mut self, fact: serde_json::Value, confidence: f64) -> &mut Snap {
        let pos = self.dial_position.position;
        self.snaps.push(Snap::new(fact, confidence, pos));
        self.snaps.last_mut().unwrap()
    }

    /// Add an absolute fact (confidence = 1.0).
    ///
    /// Shorthand for `add_snap(fact, 1.0)`.
    ///
    /// # Parameters
    ///
    /// - `fact`: The fact data as arbitrary JSON.
    ///
    /// # Returns
    ///
    /// A mutable reference to the newly created [`Snap`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Room;
    ///
    /// let mut room = Room::new("proofs");
    /// room.add_absolute(serde_json::json!({"theorem": "H1 detects emergence"}));
    /// assert_eq!(room.snaps[0].confidence, 1.0);
    /// ```
    pub fn add_absolute(&mut self, fact: serde_json::Value) -> &mut Snap {
        self.add_snap(fact, 1.0)
    }

    /// Add a soft extrapolation to this room.
    ///
    /// Inferences are included in query results only when their confidence
    /// meets or exceeds the dial threshold.
    ///
    /// # Parameters
    ///
    /// - `hypothesis`: The hypothesis data as arbitrary JSON.
    /// - `confidence`: Confidence level (0.0–1.0).
    ///
    /// # Returns
    ///
    /// A mutable reference to the newly created [`Inference`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Room;
    ///
    /// let mut room = Room::new("predictions");
    /// room.add_inference(serde_json::json!({"next": "rain"}), 0.7);
    /// assert_eq!(room.inferences.len(), 1);
    /// ```
    pub fn add_inference(
        &mut self,
        hypothesis: serde_json::Value,
        confidence: f64,
    ) -> &mut Inference {
        let pos = self.dial_position.position;
        self.inferences
            .push(Inference::new(hypothesis, confidence, pos));
        self.inferences.last_mut().unwrap()
    }

    /// Query this room at a specific dial level.
    ///
    /// Returns all snaps (always included) plus inferences whose confidence
    /// meets or exceeds the dial threshold (`1.0 - dial.position`).
    ///
    /// # Parameters
    ///
    /// - `dial`: The dial level to query at.
    ///
    /// # Returns
    ///
    /// A vector of JSON values — snap facts followed by passing inference hypotheses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, Room};
    ///
    /// let mut room = Room::new("test");
    /// room.add_snap(serde_json::json!({"x": 1}), 1.0);
    /// room.add_inference(serde_json::json!({"y": 2}), 0.7);
    /// room.add_inference(serde_json::json!({"z": 3}), 0.3);
    ///
    /// // At dial 0.5 (threshold 0.5): snap + inference with 0.7
    /// let results = room.query(Dial::new(0.5));
    /// assert_eq!(results.len(), 2);
    /// ```
    pub fn query(&self, dial: Dial) -> Vec<serde_json::Value> {
        let threshold = dial.inference_threshold();

        let mut results: Vec<serde_json::Value> =
            self.snaps.iter().map(|s| s.fact.clone()).collect();

        for inf in &self.inferences {
            if inf.confidence >= threshold {
                results.push(inf.hypothesis.clone());
            }
        }

        results
    }

    /// Query snaps only (hard facts).
    ///
    /// Returns the fact data from all snaps, regardless of confidence or dial.
    ///
    /// # Returns
    ///
    /// A vector of snap fact values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Room;
    ///
    /// let mut room = Room::new("data");
    /// room.add_snap(serde_json::json!({"a": 1}), 1.0);
    /// room.add_snap(serde_json::json!({"b": 2}), 0.8);
    ///
    /// let snaps = room.query_snaps();
    /// assert_eq!(snaps.len(), 2);
    /// ```
    pub fn query_snaps(&self) -> Vec<serde_json::Value> {
        self.snaps.iter().map(|s| s.fact.clone()).collect()
    }

    /// Query this room with tagged results that preserve fact-vs-hypothesis distinction.
    ///
    /// Like [`Room::query`], but returns [`QueryResult`] variants instead of raw JSON,
    /// so callers can distinguish snaps from inferences without additional metadata.
    ///
    /// # Parameters
    ///
    /// - `dial`: The dial level to query at.
    ///
    /// # Returns
    ///
    /// A vector of [`QueryResult`] — snap facts first, then passing inference hypotheses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, Room, QueryResult};
    ///
    /// let mut room = Room::new("test");
    /// room.add_snap(serde_json::json!({"confirmed": 1}), 1.0);
    /// room.add_inference(serde_json::json!({"possible": 2}), 0.7);
    ///
    /// let tagged = room.query_tagged(Dial::new(0.5));
    /// assert!(tagged[0].is_snap());
    /// assert!(tagged[1].is_inference());
    /// ```
    pub fn query_tagged(&self, dial: Dial) -> Vec<QueryResult> {
        let threshold = dial.inference_threshold();

        let mut results: Vec<QueryResult> = self
            .snaps
            .iter()
            .map(|s| QueryResult::Snap(s.fact.clone()))
            .collect();

        for inf in &self.inferences {
            if inf.confidence >= threshold {
                results.push(QueryResult::Inference(inf.hypothesis.clone()));
            }
        }

        results
    }

    /// Query inferences only at given dial level.
    ///
    /// Returns only the inference hypotheses that pass the dial threshold,
    /// without any snap data.
    ///
    /// # Parameters
    ///
    /// - `dial`: The dial level to filter inferences.
    ///
    /// # Returns
    ///
    /// A vector of inference hypothesis values that pass the threshold.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, Room};
    ///
    /// let mut room = Room::new("test");
    /// room.add_inference(serde_json::json!({"high": true}), 0.9);
    /// room.add_inference(serde_json::json!({"low": true}), 0.2);
    ///
    /// let infs = room.query_inferences(Dial::new(0.5)); // threshold 0.5
    /// assert_eq!(infs.len(), 1); // only the high-confidence one
    /// ```
    pub fn query_inferences(&self, dial: Dial) -> Vec<serde_json::Value> {
        let threshold = dial.inference_threshold();
        self.inferences
            .iter()
            .filter(|inf| inf.confidence >= threshold)
            .map(|inf| inf.hypothesis.clone())
            .collect()
    }

    /// Cascade top inferences as snaps into children.
    ///
    /// Sorts inferences by confidence (descending) and propagates the top
    /// ones (confidence > 0.5) into each child room as snaps, with a decay
    /// factor of 0.8×. Recurses to the given depth.
    ///
    /// # Parameters
    ///
    /// - `depth`: How many levels deep to cascade. 0 = no-op.
    ///
    /// # How It Works
    ///
    /// 1. Sort all inferences by confidence, highest first.
    /// 2. Take the top 2 inferences with confidence > 0.5.
    /// 3. Add each as a snap to every child room (with 0.8× confidence decay).
    /// 4. Recursively cascade into each child with depth - 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, Room};
    ///
    /// let mut parent = Room::new("fleet-hq");
    /// parent.add_inference(serde_json::json!({"alert": "anomaly"}), 0.9);
    /// parent.add_inference(serde_json::json!({"low": "noise"}), 0.3);
    /// parent.children.insert("drone-1".to_string(), Room::new("drone-1"));
    ///
    /// parent.cascade(1);
    ///
    /// let child = parent.children.get("drone-1").unwrap();
    /// // Child should have received the high-confidence inference as a snap
    /// assert_eq!(child.snaps.len(), 1);
    /// assert_eq!(child.snaps[0].confidence, 0.9 * 0.8);
    /// ```
    pub fn cascade(&mut self, depth: usize) {
        if depth == 0 {
            return;
        }

        // Sort inferences by confidence descending, take top 2 with confidence > 0.5
        let mut sorted: Vec<&Inference> = self
            .inferences
            .iter()
            .filter(|inf| inf.confidence > 0.5)
            .collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_inferences: Vec<&Inference> = sorted.into_iter().take(2).collect();

        for child in self.children.values_mut() {
            for inf in &top_inferences {
                let mut snap_fact = inf.hypothesis.clone();
                if let Some(obj) = snap_fact.as_object_mut() {
                    obj.insert("_source".to_string(), serde_json::json!(inf.confidence));
                }
                child.add_snap(snap_fact, inf.confidence * 0.8);
            }

            // Continue cascading into children
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

    #[test]
    fn test_cascade_sorts_by_confidence() {
        let mut parent = Room::new("parent");
        // Add inferences in non-sorted order
        parent.add_inference(serde_json::json!({"low": true}), 0.6);
        parent.add_inference(serde_json::json!({"high": true}), 0.95);
        parent.add_inference(serde_json::json!({"mid": true}), 0.7);
        parent.add_inference(serde_json::json!({"noise": true}), 0.3); // below 0.5, should be excluded

        parent
            .children
            .insert("child".to_string(), Room::new("child"));
        parent.cascade(1);

        let child = parent.children.get("child").unwrap();
        assert_eq!(child.snaps.len(), 2); // top 2 by confidence
        // Highest confidence should be first cascaded
        assert_eq!(child.snaps[0].confidence, 0.95 * 0.8);
        assert_eq!(child.snaps[1].confidence, 0.7 * 0.8);
    }

    #[test]
    fn test_cascade_empty_room() {
        let mut parent = Room::new("parent");
        parent
            .children
            .insert("child".to_string(), Room::new("child"));
        parent.cascade(1);

        let child = parent.children.get("child").unwrap();
        assert!(child.snaps.is_empty());
    }

    #[test]
    fn test_cascade_no_children() {
        let mut room = Room::new("orphan");
        room.add_inference(serde_json::json!({"idea": true}), 0.9);
        room.cascade(1); // should not panic
    }

    #[test]
    fn test_dial_boundary_snap_weight() {
        let room = Room::with_dial("test", Dial::new(0.0));
        assert_eq!(room.dial_position.position, 0.0);

        let room = Room::with_dial("test", Dial::new(1.0));
        assert_eq!(room.dial_position.position, 1.0);
    }
}
