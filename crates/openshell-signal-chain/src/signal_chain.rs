// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Signal Chain — connecting rooms with dial control
//! 
//! A chain of rooms with a dial that can be tuned per-room or globally.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{Dial, Room};

/// A signal chain connects rooms with dial control.
/// 
/// Global dial sets default for all rooms; rooms can override with their own dialect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalChain {
    /// Chain name/identifier
    pub name: String,
    /// Global dial position (default for all rooms)
    pub global_dial: Dial,
    /// All rooms in this chain
    #[serde(default)]
    pub rooms: HashMap<String, Room>,
}

impl SignalChain {
    /// Create a new signal chain with name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            global_dial: Dial::default(),
            rooms: HashMap::new(),
        }
    }

    /// Create a new signal chain with specific global dial
    pub fn with_dial(name: &str, global_dial: Dial) -> Self {
        Self {
            name: name.to_string(),
            global_dial,
            rooms: HashMap::new(),
        }
    }

    /// Get or create a room by name
    pub fn room(&mut self, name: &str) -> &mut Room {
        if !self.rooms.contains_key(name) {
            self.rooms.insert(name.to_string(), Room::with_dial(name, self.global_dial));
        }
        self.rooms.get_mut(name).unwrap()
    }

    /// Get a room by name (immutable)
    pub fn get_room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
    }

    /// Create a room with specific dial
    pub fn room_with_dial(&mut self, name: &str, dialect: Dial) -> &mut Room {
        if !self.rooms.contains_key(name) {
            self.rooms.insert(name.to_string(), Room::with_dial(name, dialect));
        }
        self.rooms.get_mut(name).unwrap()
    }

    /// Traverse rooms in order
    pub fn traverse(&self, names: &[&str]) -> Vec<&Room> {
        names.iter().filter_map(|n| self.rooms.get(*n)).collect()
    }

    /// Cascade from a room outward
    pub fn cascade_from(&mut self, origin: &str, depth: usize) {
        if let Some(room) = self.rooms.get_mut(origin) {
            room.cascade(depth);
        }
    }

    /// Query all rooms at a given dial level
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
        
        assert_eq!(chain.rooms.get("hard-room").unwrap().dialect.position, 0.0);
        assert_eq!(chain.rooms.get("soft-room").unwrap().dialect.position, 1.0);
    }
}