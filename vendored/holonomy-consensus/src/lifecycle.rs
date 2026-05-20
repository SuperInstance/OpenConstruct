//! Tile Lifecycle — Trust states and Lamport clocks for causal ordering
//!
//! # The Core Insight
//!
//! A trust verification IS a holonomy check. When the check passes, the tile
//! is Active. When it fails, the tile is Retracted. When a newer verification
//! supersedes an older one, the old tile becomes Superseded.
//!
//! The shapes are identical: holonomy measures whether a loop returns to where
//! it started. Lifecycle measures whether a claim is still active.

use serde::{Deserialize, Serialize};

/// The lifecycle state of a trust tile.
///
/// Transitions:
/// - Active → Superseded (newer verification replaces this one)
/// - Active → Retracted (constraint violation detected)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustState {
    /// Trust tile is current and valid — participates in consensus.
    Active,
    /// A newer verification has replaced this tile — historical only.
    Superseded,
    /// Constraint violation detected — tile is invalidated.
    Retracted,
}

impl std::fmt::Display for TrustState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustState::Active => write!(f, "Active"),
            TrustState::Superseded => write!(f, "Superseded"),
            TrustState::Retracted => write!(f, "Retracted"),
        }
    }
}

/// Lamport logical clock for causal ordering of trust tiles.
///
/// Each agent maintains a Lamport clock. When a trust tile is created,
/// it receives a Lamport timestamp. Tiles can be compared for happens-before
/// relationships: if A.lamport < B.lamport, then A happened before B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportClock(pub u64);

impl LamportClock {
    /// Create a new clock starting at 0.
    pub fn new() -> Self {
        Self(0)
    }

    /// Tick the clock forward (before creating a new tile).
    pub fn tick(&mut self) -> Self {
        self.0 += 1;
        *self
    }

    /// Merge with another clock (after receiving a message).
    /// Takes the maximum and increments.
    pub fn merge(&mut self, other: LamportClock) -> Self {
        self.0 = self.0.max(other.0) + 1;
        *self
    }

    /// Current value without incrementing.
    pub fn now(&self) -> u64 {
        self.0
    }

    /// Compare two timestamps for causal ordering.
    pub fn happened_before(&self, other: &LamportClock) -> bool {
        self.0 < other.0
    }
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A retraction reason — why a trust tile was invalidated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetractionReason {
    /// Holonomy check failed: cycle deviation exceeded tolerance.
    HolonomyViolation { deviation: String },
    /// Constraint check failed: INT8 bounds exceeded.
    ConstraintViolation { error_mask: u32 },
    /// Manual retraction by operator.
    Manual { reason: String },
    /// Tile expired (max cycle age exceeded).
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_state_display() {
        assert_eq!(format!("{}", TrustState::Active), "Active");
        assert_eq!(format!("{}", TrustState::Superseded), "Superseded");
        assert_eq!(format!("{}", TrustState::Retracted), "Retracted");
    }

    #[test]
    fn test_trust_state_equality() {
        assert_eq!(TrustState::Active, TrustState::Active);
        assert_ne!(TrustState::Active, TrustState::Retracted);
        assert_ne!(TrustState::Superseded, TrustState::Retracted);
    }

    #[test]
    fn test_lamport_clock_tick() {
        let mut clock = LamportClock::new();
        assert_eq!(clock.now(), 0);

        let t1 = clock.tick();
        assert_eq!(t1.0, 1);
        assert_eq!(clock.now(), 1);

        let t2 = clock.tick();
        assert_eq!(t2.0, 2);
        assert_eq!(clock.now(), 2);
    }

    #[test]
    fn test_lamport_clock_merge() {
        let mut clock_a = LamportClock::new();
        let mut clock_b = LamportClock::new();

        clock_a.tick(); // a=1
        clock_a.tick(); // a=2
        clock_b.tick(); // b=1

        // a merges b: max(2, 1) + 1 = 3
        let merged = clock_a.merge(clock_b);
        assert_eq!(merged.0, 3);
        assert_eq!(clock_a.now(), 3);
    }

    #[test]
    fn test_lamport_clock_merge_symmetry() {
        let mut clock_a = LamportClock::new();
        let mut clock_b = LamportClock::new();

        // a=1, b=3
        clock_a.tick();
        clock_b.tick();
        clock_b.tick();
        clock_b.tick();

        // a merges b: max(1, 3) + 1 = 4
        let merged_a = clock_a.merge(clock_b);
        assert_eq!(merged_a.0, 4);
    }

    #[test]
    fn test_lamport_clock_ordering() {
        let mut clock = LamportClock::new();
        let t1 = clock.tick();
        let t2 = clock.tick();
        let t3 = clock.tick();

        assert!(t1 < t2);
        assert!(t2 < t3);
        assert!(t1 < t3);
        assert!(!(t3 < t1));
    }

    #[test]
    fn test_lamport_happened_before() {
        let mut clock = LamportClock::new();
        let t1 = clock.tick();
        let t2 = clock.tick();

        assert!(t1.happened_before(&t2));
        assert!(!t2.happened_before(&t1));
        assert!(!t1.happened_before(&t1));
    }

    #[test]
    fn test_lamport_default() {
        let clock = LamportClock::default();
        assert_eq!(clock.now(), 0);
    }
}
