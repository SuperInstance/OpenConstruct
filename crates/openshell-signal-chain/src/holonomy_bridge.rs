// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Holonomy Bridge — zero-holonomy consensus inside signal chain rooms.
//!
//! The bridge: a signal chain room at hard dial (0.0) operates like
//! zero-holonomy consensus — geometric constraint satisfaction, no voting.
//! A room at soft dial (1.0) operates like exploratory consensus —
//! hypotheses propagate until the cycle closes with identity.
//!
//! # The Integration
//!
//! | Signal Chain | Zero Holonomy Consensus |
//! |-------------|--------------------------|
//! | Snap (confidence=1.0) | Tile with zero holonomy (verified) |
//! | Inference (confidence<1.0) | Tile with non-trivial holonomy (hypothesis) |
//! | Room dial | Consensus strictness threshold |
//! | Cascade | Cycle traversal → inference propagation |
//! | β₁ (Betti number) | E - V + C emergence detector |
//!
//! # Emergence Detection
//!
//! When β₁ > V - 2, the room has more edges than Laman's theorem allows.
//! This is emergence — the room is learning something that can't be reduced
//! to pairwise trust.
//!
//! - **β₁ < V - 2**: Under-constrained — room needs more tiles
//! - **β₁ = V - 2**: Exactly rigid (Laman's theorem) — optimal
//! - **β₁ > V - 2**: Over-constrained — emergence detected

use serde::{Deserialize, Serialize};
use crate::{Dial, SignalChainError};

/// Holonomy room status — mirrors ConsensusResult from holonomy-consensus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HolonomyStatus {
    /// Room is verified — all tiles have zero holonomy
    Verified,
    /// Room has hypotheses pending (non-trivial holonomy)
    Pending(Vec<usize>),
    /// Room has inconsistent tiles (holonomy != identity)
    Inconsistent(Vec<usize>),
    /// Room is emergent — β₁ > V - 2, new structure forming
    Emergent { beta: isize, threshold: isize },
}

/// Betti number result — computed from room edge/vertex counts.
#[derive(Debug, Clone)]
pub struct BettiResult {
    /// β₁ = E - V + C
    pub beta: isize,
    /// Number of edges in the room graph
    pub edge_count: usize,
    /// Number of vertices (tiles) in the room
    pub vertex_count: usize,
    /// Number of connected components
    pub component_count: usize,
    /// Whether the room is rigid (β₁ = V - 2)
    pub is_rigid: bool,
    /// Whether emergence is detected (β₁ > V - 2)
    pub has_emergence: bool,
}

impl BettiResult {
    /// Compute Betti number from edge count, vertex count, component count.
    ///
    /// Formula: β₁ = E - V + C
    /// where E = edges, V = vertices, C = components
    pub fn compute(edge_count: usize, vertex_count: usize, component_count: usize) -> Self {
        let beta = (edge_count as isize) - (vertex_count as isize) + (component_count as isize);
        let threshold = (vertex_count as isize) - 2;
        let is_rigid = beta == threshold;
        let has_emergence = beta > threshold;
        BettiResult {
            beta,
            edge_count,
            vertex_count,
            component_count,
            is_rigid,
            has_emergence,
        }
    }

    /// Laman's theorem check: E = 2V - 3.
    ///
    /// If this holds, the graph is exactly rigid (no under/over constraint).
    pub fn laman_check(&self) -> (usize, usize, bool) {
        let expected = if self.vertex_count >= 2 {
            2 * self.vertex_count - 3
        } else {
            0
        };
        ((expected, self.edge_count, expected == self.edge_count))
    }
}

/// Holonomy room — a signal chain room with holonomy consensus semantics.
///
/// A holonomy room tracks edges (connections between tiles) and vertices
/// (tiles) to compute the Betti number β₁ = E - V + C. This tells you
/// whether the room is rigid, under-constrained, or emergent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolonomyRoom {
    /// Room identifier
    pub name: String,
    /// Room dial position
    pub dial: Dial,
    /// Edge count (connections between tiles)
    edge_count: usize,
    /// Vertex count (tiles in room)
    vertex_count: usize,
    /// Snap count (verified tiles with zero holonomy)
    snap_count: usize,
    /// Inference count (unverified tiles with non-trivial holonomy)
    inference_count: usize,
}

impl HolonomyRoom {
    /// Create a new holonomy room.
    pub fn new(name: &str, dial: Dial) -> Self {
        HolonomyRoom {
            name: name.to_string(),
            dial,
            edge_count: 0,
            vertex_count: 0,
            snap_count: 0,
            inference_count: 0,
        }
    }

    /// Add a snap — verified tile, zero holonomy.
    pub fn add_snap(&mut self) {
        self.vertex_count += 1;
        self.snap_count += 1;
    }

    /// Add an inference — unverified tile, non-trivial holonomy.
    pub fn add_inference(&mut self) {
        self.vertex_count += 1;
        self.inference_count += 1;
    }

    /// Add an edge (connection between two tiles).
    pub fn add_edge(&mut self) {
        self.edge_count += 1;
    }

    /// Add a connection between two tiles (adds one edge).
    pub fn connect(&mut self, tile_a: usize, tile_b: usize) {
        self.add_edge();
    }

    /// Compute the Betti number for this room.
    pub fn betti(&self) -> BettiResult {
        BettiResult::compute(self.edge_count, self.vertex_count, 1)
    }

    /// Get holonomy status based on current room state and dial.
    ///
    /// At hard dial (0.0): only snaps pass, inferences are Pending.
    /// At soft dial (1.0): snaps and inferences both pass.
    pub fn status(&self) -> HolonomyStatus {
        let betti = self.betti();
        if betti.has_emergence {
            HolonomyStatus::Emergent {
                beta: betti.beta,
                threshold: (self.vertex_count as isize) - 2,
            }
        } else if self.inference_count > 0 && self.dial.position < 0.25 {
            HolonomyStatus::Pending((0..self.inference_count).collect())
        } else if self.snap_count == 0 && self.inference_count == 0 {
            HolonomyStatus::Verified // empty room = trivially verified
        } else {
            HolonomyStatus::Verified
        }
    }

    /// Check if this room passes the hard threshold.
    pub fn passes_hard(&self) -> bool {
        self.inference_count == 0 && self.snap_count > 0
    }

    /// Room vertex count.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Room edge count.
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
}

/// Holonomy chain — a signal chain where every room computes Betti numbers.
#[derive(Debug, Clone)]
pub struct HolonomyChain {
    name: String,
    rooms: Vec<HolonomyRoom>,
}

impl HolonomyChain {
    pub fn new(name: &str) -> Self {
        HolonomyChain {
            name: name.to_string(),
            rooms: Vec::new(),
        }
    }

    /// Add a holonomy room to the chain.
    pub fn add_room(&mut self, room: HolonomyRoom) {
        self.rooms.push(room);
    }

    /// Get all emergent rooms (β₁ > V - 2).
    pub fn emergent_rooms(&self) -> Vec<(String, isize)> {
        self.rooms
            .iter()
            .filter(|r| r.betti().has_emergence)
            .map(|r| (r.name.clone(), r.betti().beta))
            .collect()
    }

    /// Get all rigid rooms (β₁ = V - 2).
    pub fn rigid_rooms(&self) -> Vec<String> {
        self.rooms
            .iter()
            .filter(|r| r.betti().is_rigid)
            .map(|r| r.name.clone())
            .collect()
    }

    /// Total edges across all rooms.
    pub fn total_edges(&self) -> usize {
        self.rooms.iter().map(|r| r.edge_count()).sum()
    }

    /// Total vertices across all rooms.
    pub fn total_vertices(&self) -> usize {
        self.rooms.iter().map(|r| r.vertex_count()).sum()
    }

    /// Chain-wide Betti number (sum of room Betti numbers).
    pub fn chain_betti(&self) -> isize {
        self.rooms.iter().map(|r| r.betti().beta).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laman_rigid() {
        // V=5 → E=7 (2*5-3=7) → β₁ = 7-5+1 = 3 = V-2 ✓
        let mut room = HolonomyRoom::new("test", Dial::hard());
        room.add_snap(); // v=1
        room.add_snap(); // v=2
        room.add_snap(); // v=3
        room.add_snap(); // v=4
        room.add_snap(); // v=5
        for _ in 0..7 {
            room.add_edge(); // e=7
        }
        let b = room.betti();
        assert!(b.is_rigid);
        assert!(!b.has_emergence);
        assert_eq!(b.beta, 3);
    }

    #[test]
    fn test_emergence() {
        // V=5 but E=10 → β₁ = 10-5+1 = 6 > V-2=3 → emergent
        let mut room = HolonomyRoom::new("test", Dial::hard());
        for _ in 0..5 {
            room.add_snap();
        }
        for _ in 0..10 {
            room.add_edge();
        }
        let b = room.betti();
        assert!(b.has_emergence);
        assert!(!b.is_rigid);
    }

    #[test]
    fn test_hard_passes() {
        let mut room = HolonomyRoom::new("test", Dial::hard());
        room.add_snap();
        room.add_snap();
        assert!(room.passes_hard());

        room.add_inference(); // adds non-trivial holonomy
        assert!(!room.passes_hard());
    }

    #[test]
    fn test_holonomy_chain() {
        let mut chain = HolonomyChain::new("fleet");
        
        let mut room1 = HolonomyRoom::new("room1", Dial::hard());
        for _ in 0..5 { room1.add_snap(); }
        for _ in 0..7 { room1.add_edge(); }
        
        let mut room2 = HolonomyRoom::new("room2", Dial::soft());
        for _ in 0..3 { room2.add_snap(); }
        for _ in 0..4 { room2.add_edge(); }
        
        chain.add_room(room1);
        chain.add_room(room2);
        
        assert_eq!(chain.rigid_rooms(), vec!["room1".to_string()]);
        assert_eq!(chain.total_vertices(), 8);
        assert_eq!(chain.total_edges(), 11);
    }
}
