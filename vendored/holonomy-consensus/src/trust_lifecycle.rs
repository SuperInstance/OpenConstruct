//! Trust Lifecycle — Trust tiles with lifecycle management
//!
//! # The Core Insight
//!
//! Holonomy is about whether a loop returns to where it started.
//! Tile lifecycle is about whether a claim is still active.
//! A trust verification IS a holonomy check — when the check fails,
//! the trust tile should be retracted. The shapes are identical.

use crate::constraints::{ConstraintResult, HolonomyBounds};
use crate::consensus::{ConsensusResult, HolonomyConsensus};
use crate::lifecycle::{LamportClock, RetractionReason, TrustState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A trust tile — the result of a holonomy verification cycle.
///
/// Each tile records:
/// - Which agents participated in the verification cycle
/// - The consensus result (pass/fail, deviation)
/// - The current lifecycle state (Active/Superseded/Retracted)
/// - A Lamport timestamp for causal ordering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustTile {
    /// Unique identifier for this trust cycle.
    pub cycle_id: u64,
    /// Agents that participated in this verification cycle.
    pub agents: Vec<u64>,
    /// The consensus result from the holonomy check.
    pub result: TrustCycleResult,
    /// Current lifecycle state.
    pub state: TrustState,
    /// Lamport timestamp for causal ordering.
    pub lamport: LamportClock,
    /// Wall-clock timestamp (epoch seconds).
    pub timestamp: u64,
}

/// The outcome of a trust verification cycle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustCycleResult {
    /// Whether the cycle passed holonomy verification.
    pub passed: bool,
    /// Holonomy deviation (0 = perfect).
    pub deviation: f64,
    /// If failed, which tile was faulty.
    pub faulty_tile: Option<u64>,
    /// If failed, the constraint check result.
    pub constraint: Option<ConstraintCheckResult>,
}

/// Simplified constraint check result for serialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstraintCheckResult {
    pub pass: bool,
    pub error_mask: u32,
    pub deviation: i32,
}

impl TrustTile {
    /// Create a new trust tile from a consensus result.
    pub fn new(
        cycle_id: u64,
        agents: Vec<u64>,
        consensus_result: &ConsensusResult,
        lamport: LamportClock,
    ) -> Self {
        let passed = consensus_result.is_consistent;
        let deviation = consensus_result.deviation;
        let faulty_tile = consensus_result.faulty_tile;

        let result = TrustCycleResult {
            passed,
            deviation,
            faulty_tile,
            constraint: None,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        TrustTile {
            cycle_id,
            agents,
            result,
            state: TrustState::Active,
            lamport,
            timestamp,
        }
    }

    /// Create a trust tile with an explicit constraint check.
    pub fn with_constraint(
        cycle_id: u64,
        agents: Vec<u64>,
        consensus_result: &ConsensusResult,
        constraint_result: &ConstraintResult,
        lamport: LamportClock,
    ) -> Self {
        let mut tile = Self::new(cycle_id, agents, consensus_result, lamport);
        tile.result.constraint = Some(ConstraintCheckResult {
            pass: constraint_result.pass,
            error_mask: constraint_result.error_mask,
            deviation: constraint_result.deviation,
        });
        tile
    }

    /// Check if this tile is still active.
    pub fn is_active(&self) -> bool {
        self.state == TrustState::Active
    }
}

/// A pool of trust tiles with lifecycle management.
///
/// The TrustPool:
/// - Stores all trust tiles (Active, Superseded, Retracted)
/// - Filters Active tiles for consensus participation
/// - Manages supersession and retraction transitions
/// - Integrates with holonomy checking for automatic retraction
pub struct TrustPool {
    tiles: HashMap<u64, TrustTile>,
    next_cycle_id: u64,
    lamport: LamportClock,
}

impl TrustPool {
    /// Create a new empty trust pool.
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            next_cycle_id: 1,
            lamport: LamportClock::new(),
        }
    }

    /// Add a trust tile to the pool. Returns the cycle_id.
    pub fn add(&mut self, tile: TrustTile) -> u64 {
        let id = tile.cycle_id;
        if id >= self.next_cycle_id {
            self.next_cycle_id = id + 1;
        }
        self.tiles.insert(id, tile);
        id
    }

    /// Create and add a trust tile from a consensus check.
    pub fn verify_cycle(
        &mut self,
        agents: Vec<u64>,
        consensus: &HolonomyConsensus,
        bounds: &HolonomyBounds,
    ) -> u64 {
        let cycle_id = self.next_cycle_id;
        self.lamport.tick();

        let consensus_result = consensus.check_consensus();
        let deviation = consensus_result.deviation;
        let constraint_result = ConstraintResult::check(deviation, bounds);

        let tile = TrustTile::with_constraint(
            cycle_id,
            agents,
            &consensus_result,
            &constraint_result,
            self.lamport,
        );

        // If constraint violated, immediately retract
        let tile = if !constraint_result.pass {
            let mut retracted = tile;
            retracted.state = TrustState::Retracted;
            retracted
        } else {
            tile
        };

        self.add(tile)
    }

    /// Get a trust tile by cycle_id.
    pub fn get(&self, cycle_id: u64) -> Option<&TrustTile> {
        self.tiles.get(&cycle_id)
    }

    /// Get all Active trust tiles.
    pub fn active_trust(&self) -> Vec<&TrustTile> {
        self.tiles.values().filter(|t| t.is_active()).collect()
    }

    /// Get trust tiles sorted by Lamport timestamp.
    pub fn sorted_by_lamport(&self) -> Vec<&TrustTile> {
        let mut tiles: Vec<&TrustTile> = self.tiles.values().collect();
        tiles.sort_by_key(|t| t.lamport);
        tiles
    }

    /// Total number of tiles (all states).
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Number of active tiles.
    pub fn active_count(&self) -> usize {
        self.tiles.values().filter(|t| t.is_active()).count()
    }

    /// Supersede an old cycle with a new one.
    ///
    /// The old cycle transitions Active → Superseded.
    /// The new cycle is added as Active.
    /// Returns the new cycle_id.
    pub fn supersede_cycle(
        &mut self,
        old_id: u64,
        agents: Vec<u64>,
        consensus: &HolonomyConsensus,
        bounds: &HolonomyBounds,
    ) -> Result<u64, LifecycleError> {
        // Mark old as superseded
        if let Some(old_tile) = self.tiles.get_mut(&old_id) {
            if old_tile.state != TrustState::Active {
                return Err(LifecycleError::InvalidTransition {
                    from: old_tile.state,
                    to: TrustState::Superseded,
                    reason: "can only supersede Active tiles".into(),
                });
            }
            old_tile.state = TrustState::Superseded;
        }
        // If old_id doesn't exist, that's fine — just add the new one

        // Create new verification
        let new_id = self.verify_cycle(agents, consensus, bounds);
        Ok(new_id)
    }

    /// Retract a cycle — Active → Retracted.
    pub fn retract_cycle(
        &mut self,
        cycle_id: u64,
        reason: RetractionReason,
    ) -> Result<(), LifecycleError> {
        if let Some(tile) = self.tiles.get_mut(&cycle_id) {
            if tile.state != TrustState::Active {
                return Err(LifecycleError::InvalidTransition {
                    from: tile.state,
                    to: TrustState::Retracted,
                    reason: "can only retract Active tiles".into(),
                });
            }
            tile.state = TrustState::Retracted;

            // Update result based on reason
            match &reason {
                RetractionReason::HolonomyViolation { deviation } => {
                    tile.result.passed = false;
                    // deviation string stored in reason, no numeric update needed
                    let _ = deviation;
                }
                RetractionReason::ConstraintViolation { error_mask } => {
                    if let Some(ref mut constraint) = tile.result.constraint {
                        constraint.pass = false;
                        constraint.error_mask |= *error_mask;
                    }
                }
                _ => {}
            }

            Ok(())
        } else {
            Err(LifecycleError::TileNotFound { cycle_id })
        }
    }

    /// Merge a Lamport clock from an external source.
    pub fn merge_clock(&mut self, external: LamportClock) -> LamportClock {
        self.lamport.merge(external)
    }

    /// Current Lamport clock value.
    pub fn clock(&self) -> LamportClock {
        self.lamport
    }
}

impl Default for TrustPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors for lifecycle transitions.
#[derive(Clone, Debug, PartialEq)]
pub enum LifecycleError {
    /// Tile not found in pool.
    TileNotFound { cycle_id: u64 },
    /// Invalid state transition.
    InvalidTransition {
        from: TrustState,
        to: TrustState,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::{ConsensusTile, HolonomyMatrix};

    fn make_consensus(n_tiles: u64, deviation: f64) -> HolonomyConsensus {
        let mut consensus = HolonomyConsensus::new(0.01);
        for i in 0..n_tiles {
            let neighbors: Vec<u64> = if i == 0 {
                vec![n_tiles - 1, 1]
            } else if i == n_tiles - 1 {
                vec![i - 1, 0]
            } else {
                vec![i - 1, i + 1]
            };
            consensus.add_tile(ConsensusTile {
                id: i,
                holonomy: HolonomyMatrix::from_rotation([0.0, 0.0, 1.0], deviation),
                neighbors,
                cycle_id: None,
            });
        }
        consensus
    }

    #[test]
    fn test_active_to_superseded() {
        let consensus = make_consensus(4, 0.0001); // Small deviation → passes
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        // Create first verification
        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let tile1 = pool.get(id1).unwrap();
        assert_eq!(tile1.state, TrustState::Active);

        // Supersede with new verification
        let id2 = pool
            .supersede_cycle(id1, vec![0, 1, 2, 3], &consensus, &bounds)
            .unwrap();

        // Old should be superseded
        let old = pool.get(id1).unwrap();
        assert_eq!(old.state, TrustState::Superseded);

        // New should be active
        let new = pool.get(id2).unwrap();
        assert_eq!(new.state, TrustState::Active);
    }

    #[test]
    fn test_active_to_retracted_on_constraint_violation() {
        // Large deviation → fails constraint check → immediate retraction
        let consensus = make_consensus(4, 0.5); // Large deviation
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let tile = pool.get(id).unwrap();

        // Should be immediately retracted due to constraint violation
        assert_eq!(tile.state, TrustState::Retracted);
    }

    #[test]
    fn test_manual_retraction() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        assert_eq!(pool.get(id).unwrap().state, TrustState::Active);

        pool.retract_cycle(id, RetractionReason::Manual { reason: "test".into() })
            .unwrap();

        assert_eq!(pool.get(id).unwrap().state, TrustState::Retracted);
    }

    #[test]
    fn test_cannot_retract_superseded() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        pool.supersede_cycle(id1, vec![0, 1, 2, 3], &consensus, &bounds)
            .unwrap();

        // Trying to retract a superseded tile should fail
        let result = pool.retract_cycle(id1, RetractionReason::Expired);
        assert!(result.is_err());
    }

    #[test]
    fn test_active_trust_filters() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id2 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id3 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);

        // Retract id2
        pool.retract_cycle(id2, RetractionReason::Expired).unwrap();

        // Only id1 and id3 should be active
        let active = pool.active_trust();
        assert_eq!(active.len(), 2);
        let active_ids: Vec<u64> = active.iter().map(|t| t.cycle_id).collect();
        assert!(active_ids.contains(&id1));
        assert!(active_ids.contains(&id3));
        assert!(!active_ids.contains(&id2));
    }

    #[test]
    fn test_lamport_ordering_across_tiles() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id2 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id3 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);

        let t1 = pool.get(id1).unwrap().lamport;
        let t2 = pool.get(id2).unwrap().lamport;
        let t3 = pool.get(id3).unwrap().lamport;

        assert!(t1.happened_before(&t2));
        assert!(t2.happened_before(&t3));
        assert!(t1.happened_before(&t3));
    }

    #[test]
    fn test_sorted_by_lamport() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id2 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id3 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);

        let sorted = pool.sorted_by_lamport();
        assert_eq!(sorted[0].cycle_id, id1);
        assert_eq!(sorted[1].cycle_id, id2);
        assert_eq!(sorted[2].cycle_id, id3);
    }

    #[test]
    fn test_tile_not_found_error() {
        let mut pool = TrustPool::new();
        let result = pool.retract_cycle(999, RetractionReason::Expired);
        assert!(matches!(result, Err(LifecycleError::TileNotFound { cycle_id: 999 })));
    }

    #[test]
    fn test_pool_counts() {
        let consensus = make_consensus(4, 0.0001);
        let bounds = HolonomyBounds::default();
        let mut pool = TrustPool::new();

        let id1 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);
        let id2 = pool.verify_cycle(vec![0, 1, 2, 3], &consensus, &bounds);

        assert_eq!(pool.len(), 2);
        assert_eq!(pool.active_count(), 2);

        pool.retract_cycle(id1, RetractionReason::Expired).unwrap();
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_trust_tile_is_active() {
        let mut clock = LamportClock::new();
        clock.tick();
        let result = crate::consensus::ConsensusResult {
            is_consistent: true,
            deviation: 0.0,
            faulty_tile: None,
            information: f64::INFINITY,
        };
        let tile = TrustTile::new(1, vec![0, 1], &result, clock);
        assert!(tile.is_active());
    }

    #[test]
    fn test_merge_external_clock() {
        let mut pool = TrustPool::new();
        let mut external = LamportClock::new();
        external.tick();
        external.tick();
        external.tick(); // external = 3

        let merged = pool.merge_clock(external);
        assert_eq!(merged.0, 4); // max(0, 3) + 1 = 4
    }
}
