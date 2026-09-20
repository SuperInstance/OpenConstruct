// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Signal Chain — connecting rooms with dial control.
//!
//! A [`SignalChain`] is a named collection of rooms with a global dial that
//! can be overridden per-room. It provides methods to create rooms, traverse
//! them in order, cascade inferences between rooms, and query all rooms at
//! a given dial level.
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::{Dial, SignalChain, DIAL_ANALYSIS};
//!
//! let mut chain = SignalChain::new("fleet-ops");
//!
//! let sensors = chain.room("sonar");
//! sensors.add_snap(serde_json::json!({"depth": 87.2}), 1.0);
//!
//! let analysis = chain.room_with_dial("analysis", DIAL_ANALYSIS);
//! analysis.add_inference(serde_json::json!({"wreck": true}), 0.7);
//!
//! let all_results = chain.query_all(Dial::new(0.5));
//! assert_eq!(all_results.len(), 2);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Dial, Inference, Room, SignalChainError};

/// A signal chain connects rooms with dial control.
///
/// The global dial sets the default for all rooms; individual rooms can
/// override with their own dial position via [`SignalChain::room_with_dial`].
///
/// # Fields
///
/// - `name`: Chain identifier.
/// - `global_dial`: Default dial for rooms created without an explicit dial.
/// - `rooms`: Map of room name → [`Room`].
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::{Dial, SignalChain};
///
/// let mut chain = SignalChain::with_dial("ops", Dial::new(0.3));
/// let room = chain.room("sensors");
/// room.add_snap(serde_json::json!({"temp": 72}), 1.0);
///
/// assert_eq!(chain.rooms.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalChain {
    /// Chain name/identifier.
    pub name: String,
    /// Global dial position (default for all rooms).
    pub global_dial: Dial,
    /// All rooms in this chain, keyed by name.
    #[serde(default)]
    pub rooms: HashMap<String, Room>,
}

impl SignalChain {
    /// Create a new signal chain with the given name and default dial (0.5).
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this chain.
    ///
    /// # Returns
    ///
    /// A new [`SignalChain`] with no rooms and dial at 0.5.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let chain = SignalChain::new("my-chain");
    /// assert_eq!(chain.name, "my-chain");
    /// assert!(chain.rooms.is_empty());
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            global_dial: Dial::default(),
            rooms: HashMap::new(),
        }
    }

    /// Create a new signal chain with a specific global dial.
    ///
    /// # Parameters
    ///
    /// - `name`: Unique identifier for this chain.
    /// - `global_dial`: The dial position to use as default for all rooms.
    ///
    /// # Returns
    ///
    /// A new [`SignalChain`] with the specified global dial.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, SignalChain};
    ///
    /// let chain = SignalChain::with_dial("formal", Dial::hard());
    /// assert_eq!(chain.global_dial.position, 0.0);
    /// ```
    pub fn with_dial(name: &str, global_dial: Dial) -> Self {
        Self {
            name: name.to_string(),
            global_dial,
            rooms: HashMap::new(),
        }
    }

    /// Get or create a room by name.
    ///
    /// If the room doesn't exist, it's created with the chain's global dial.
    ///
    /// # Parameters
    ///
    /// - `name`: Room name. If it already exists, returns that room.
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`Room`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let mut chain = SignalChain::new("test");
    /// let room = chain.room("sensors");
    /// room.add_snap(serde_json::json!({"val": 42}), 1.0);
    ///
    /// // Calling again returns the same room
    /// let room2 = chain.room("sensors");
    /// assert_eq!(room2.snaps.len(), 1);
    /// ```
    pub fn room(&mut self, name: &str) -> &mut Room {
        if !self.rooms.contains_key(name) {
            self.rooms
                .insert(name.to_string(), Room::with_dial(name, self.global_dial));
        }
        self.rooms.get_mut(name).unwrap()
    }

    /// Get a room by name (immutable).
    ///
    /// # Parameters
    ///
    /// - `name`: Room name to look up.
    ///
    /// # Returns
    ///
    /// `Some(&Room)` if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room("exists");
    ///
    /// assert!(chain.get_room("exists").is_some());
    /// assert!(chain.get_room("nope").is_none());
    /// ```
    pub fn get_room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
    }

    /// Get a mutable reference to a room by name, without auto-creating it.
    ///
    /// Unlike [`SignalChain::room`], this does not create a room if it doesn't exist.
    ///
    /// # Parameters
    ///
    /// - `name`: Room name to look up.
    ///
    /// # Returns
    ///
    /// `Some(&mut Room)` if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room("exists");
    ///
    /// assert!(chain.get_room_mut("exists").is_some());
    /// assert!(chain.get_room_mut("nope").is_none());
    /// ```
    pub fn get_room_mut(&mut self, name: &str) -> Option<&mut Room> {
        self.rooms.get_mut(name)
    }

    /// Create a new room with explicit validation.
    ///
    /// Unlike [`SignalChain::room`], this validates the name and returns an error
    /// if the name is empty. If the room already exists, returns a mutable reference
    /// to it without modifying it.
    ///
    /// # Parameters
    ///
    /// - `name`: Room name. Must be non-empty.
    ///
    /// # Returns
    ///
    /// - `Ok(&mut Room)` on success.
    /// - `Err(`[`SignalChainError::EmptyName`]`)` if the name is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{SignalChain, SignalChainError};
    ///
    /// let mut chain = SignalChain::new("test");
    ///
    /// assert!(chain.create_room("valid").is_ok());
    /// assert!(matches!(chain.create_room(""), Err(SignalChainError::EmptyName)));
    /// ```
    pub fn create_room(&mut self, name: &str) -> Result<&mut Room, SignalChainError> {
        if name.is_empty() {
            return Err(SignalChainError::EmptyName);
        }
        if !self.rooms.contains_key(name) {
            self.rooms
                .insert(name.to_string(), Room::with_dial(name, self.global_dial));
        }
        Ok(self.rooms.get_mut(name).unwrap())
    }

    /// Get or create a room with a specific dial position.
    ///
    /// # Parameters
    ///
    /// - `name`: Room name.
    /// - `dial`: The dial position for this room (overrides global dial).
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`Room`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, SignalChain};
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room_with_dial("hard-room", Dial::hard());
    ///
    /// let room = chain.get_room("hard-room").unwrap();
    /// assert_eq!(room.dial_position.position, 0.0);
    /// ```
    pub fn room_with_dial(&mut self, name: &str, dial: Dial) -> &mut Room {
        if !self.rooms.contains_key(name) {
            self.rooms
                .insert(name.to_string(), Room::with_dial(name, dial));
        }
        self.rooms.get_mut(name).unwrap()
    }

    /// Traverse rooms in the given order.
    ///
    /// Returns references to rooms in the order specified, skipping any
    /// names that don't exist.
    ///
    /// # Parameters
    ///
    /// - `names`: Ordered slice of room names to traverse.
    ///
    /// # Returns
    ///
    /// A vector of room references in traversal order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room("a");
    /// chain.room("b");
    /// chain.room("c");
    ///
    /// let rooms = chain.traverse(&["a", "c", "missing"]);
    /// assert_eq!(rooms.len(), 2); // "missing" skipped
    /// ```
    pub fn traverse(&self, names: &[&str]) -> Vec<&Room> {
        names.iter().filter_map(|n| self.rooms.get(*n)).collect()
    }

    /// Cascade from a room to all sibling rooms in this chain.
    ///
    /// Collects the top-2 inferences from `origin` (sorted by confidence
    /// descending, threshold > 0.5) and injects them as snaps into every
    /// other room in the chain at 0.8× confidence.
    ///
    /// # Parameters
    ///
    /// - `origin`: Name of the room to cascade from.
    /// - `depth`: Reserved for future multi-hop cascades; currently depth 0 is a no-op.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::SignalChain;
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room("parent").add_inference(serde_json::json!({"idea": true}), 0.9);
    /// chain.room("sibling");
    ///
    /// chain.cascade_from("parent", 1);
    ///
    /// let snaps = chain.get_room("sibling").unwrap().query_snaps();
    /// assert_eq!(snaps.len(), 1);
    /// ```
    pub fn cascade_from(&mut self, origin: &str, depth: usize) {
        if depth == 0 {
            return;
        }

        // Extract top-2 inferences from origin, sorted by confidence descending.
        let top: Vec<(serde_json::Value, f64)> = {
            let room = match self.rooms.get(origin) {
                Some(r) => r,
                None => return,
            };
            let mut sorted: Vec<&Inference> = room
                .inferences
                .iter()
                .filter(|inf| inf.confidence > 0.5)
                .collect();
            sorted.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            sorted
                .into_iter()
                .take(2)
                .map(|inf| (inf.hypothesis.clone(), inf.confidence))
                .collect()
        };

        // Inject into every sibling room in the chain.
        for (name, room) in &mut self.rooms {
            if name == origin {
                continue;
            }
            for (hypothesis, confidence) in &top {
                room.add_snap(hypothesis.clone(), confidence * 0.8);
            }
        }
    }

    /// Query all rooms at a given dial level.
    ///
    /// Returns a map from room name to the query results for that room.
    /// See [`Room::query`] for how results are filtered.
    ///
    /// # Parameters
    ///
    /// - `dial`: The dial level to query at.
    ///
    /// # Returns
    ///
    /// A `HashMap<String, Vec<Value>>` mapping room names to their results.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::{Dial, SignalChain};
    ///
    /// let mut chain = SignalChain::new("test");
    /// chain.room("a").add_snap(serde_json::json!({"x": 1}), 1.0);
    /// chain.room("b").add_snap(serde_json::json!({"y": 2}), 1.0);
    ///
    /// let all = chain.query_all(Dial::hard());
    /// assert_eq!(all.len(), 2);
    /// ```
    pub fn query_all(&self, dial: Dial) -> HashMap<String, Vec<serde_json::Value>> {
        self.rooms
            .iter()
            .map(|(name, room)| (name.clone(), room.query(dial)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_room_creation() {
        let mut chain = SignalChain::new("test");
        let room = chain.room("room-1");

        assert_eq!(room.name, "room-1");
    }

    #[test]
    fn test_chain_room_dial_override() {
        let mut chain = SignalChain::new("test");
        chain.room_with_dial("hard-room", Dial::hard());
        chain.room_with_dial("soft-room", Dial::soft());

        assert_eq!(
            chain.rooms.get("hard-room").unwrap().dial_position.position,
            0.0
        );
        assert_eq!(
            chain.rooms.get("soft-room").unwrap().dial_position.position,
            1.0
        );
    }
}
