// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The room mask (RFC 0004, *The Room Grows a Mask*, §1).
//!
//! The mask is **not** a permission system. `OpenShell` already has real
//! permission systems (Landlock, seccomp, policy). The mask is ontological:
//! it declares what dimensions of the world *exist* for this room. A room
//! whose mask grew facing `Journal` does not see the network — not because
//! it is forbidden, but because the network is not *in its world*.
//!
//! The mask locks at room creation, like a crystal's lattice when it leaves
//! the bath. It is derivable deterministically from the seed, introspectable
//! by the room's own agent ("here is your lattice — this is what you are"),
//! and a room can be re-seeded (new growth) but never un-masked.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Domain-separation prefix for the mask derivation (versioned: bump to
/// change the lattice rule for new growth).
const MASK_DOMAIN: &[u8] = b"openshell-construct/mask/v1";

/// A dimension of the world a room can read.
///
/// Ordinal order (`Yard` < `Journal` < `Road` < `Wall` < `Self_` < `Fleet`)
/// is the canonical derivation order and the `BTreeSet` iteration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MaskChannel {
    /// Telemetry of the shared yard.
    #[serde(rename = "yard")]
    Yard,
    /// The room's own journal — writes against need.
    #[serde(rename = "journal")]
    Journal,
    /// The road network — arrivals and link quality.
    #[serde(rename = "road")]
    Road,
    /// The gallery wall — what other rooms have left on display.
    #[serde(rename = "wall")]
    Wall,
    /// The room's own self-inspection.
    #[serde(rename = "self")]
    Self_,
    /// The fleet channel — other rooms, other keepers.
    #[serde(rename = "fleet")]
    Fleet,
}

/// All channels, in canonical (derivation) order.
pub const ALL_CHANNELS: [MaskChannel; 6] = [
    MaskChannel::Yard,
    MaskChannel::Journal,
    MaskChannel::Road,
    MaskChannel::Wall,
    MaskChannel::Self_,
    MaskChannel::Fleet,
];

/// The locked lattice of what this room can read of the world.
///
/// Locked at room creation; derived, never configured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomMask {
    /// The channels that exist for this room.
    pub channels: BTreeSet<MaskChannel>,
}

impl RoomMask {
    /// Does this room's world include `channel`?
    pub fn contains(&self, channel: MaskChannel) -> bool {
        self.channels.contains(&channel)
    }

    /// Number of channels in the lattice.
    pub fn len(&self) -> usize {
        self.channels.len()
    }

    /// A mask is never empty: a grown room always faces at least one
    /// channel (the never-grown-blind fallback guarantees one).
    pub fn is_empty(&self) -> bool {
        false
    }

    /// True for a grown room (a mask is never blind).
    pub fn is_grown(&self) -> bool {
        !self.channels.is_empty()
    }
}

/// Derive the room's mask from its seed, charter, and creation tick.
///
/// # Derivation rule (verbatim)
///
/// 1. `mask_digest = SHA-256( "openshell-construct/mask/v1" || 0x00 ||
///        u64_le(seed.len()) || seed ||
///        u64_le(charter.len()) || charter.as_bytes() ||
///        u64_le(creation_tick) )`
///
///    The length prefixes make the framing injective in
///    `(seed, charter, creation_tick)`: no concatenation ambiguity is
///    possible, and every input is bound into the lattice.
///
/// 2. The channel with ordinal `i` (in canonical order
///    `Yard=0, Journal=1, Road=2, Wall=3, Self_=4, Fleet=5`) is present
///    iff bit `i` of `mask_digest[i]` is `1`.
///
/// 3. A room is never grown blind: if step 2 yields the empty set, the
///    mask falls back to `{Wall}` — the room faces at least the wall.
///
/// The same `(seed, charter, creation_tick)` always grows the same mask;
/// different growth inputs grow different lattices.
pub fn derive_mask(seed: &[u8], charter: &str, creation_tick: u64) -> RoomMask {
    let mut hasher = Sha256::new();
    hasher.update(MASK_DOMAIN);
    hasher.update([0x00]);
    hasher.update((seed.len() as u64).to_le_bytes());
    hasher.update(seed);
    hasher.update((charter.len() as u64).to_le_bytes());
    hasher.update(charter.as_bytes());
    hasher.update(creation_tick.to_le_bytes());
    let digest: [u8; 32] = hasher.finalize().into();

    let mut channels = BTreeSet::new();
    for (i, channel) in ALL_CHANNELS.iter().enumerate() {
        if digest[i] & (1_u8 << i) != 0 {
            channels.insert(*channel);
        }
    }
    // A room is never grown blind (step 3).
    if channels.is_empty() {
        channels.insert(MaskChannel::Wall);
    }

    RoomMask { channels }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED_A: &[u8] = b"seed-of-the-first-room";
    const CHARTER_A: &str = "hold the commission; read the water";

    #[test]
    fn mask_is_deterministic() {
        let a = derive_mask(SEED_A, CHARTER_A, 1_000);
        let b = derive_mask(SEED_A, CHARTER_A, 1_000);
        assert_eq!(a, b, "same growth inputs must grow the same lattice");
        assert!(a.is_grown());
    }

    #[test]
    fn different_growth_inputs_grow_different_lattices() {
        let base = derive_mask(SEED_A, CHARTER_A, 1_000);
        let other_seed = derive_mask(b"seed-of-the-second-room", CHARTER_A, 1_000);
        let other_charter = derive_mask(SEED_A, "a different commission", 1_000);
        let other_tick = derive_mask(SEED_A, CHARTER_A, 1_001);
        assert_ne!(base, other_seed);
        assert_ne!(base, other_charter);
        assert_ne!(base, other_tick);
    }

    #[test]
    fn every_channel_grows_somewhere() {
        // Across a thousand growths, every channel appears in some lattice,
        // and no lattice is empty (the never-grown-blind fallback).
        let mut union = BTreeSet::new();
        for tick in 0..1000_u64 {
            let mask = derive_mask(SEED_A, CHARTER_A, tick);
            assert!(mask.is_grown(), "a grown room is never blind");
            union.extend(mask.channels);
        }
        let expected: BTreeSet<_> = ALL_CHANNELS.into_iter().collect();
        assert_eq!(union, expected);
    }

    #[test]
    fn mask_serializes_round_trip() {
        let mask = derive_mask(SEED_A, CHARTER_A, 42);
        let json = serde_json::to_string(&mask).unwrap();
        let back: RoomMask = serde_json::from_str(&json).unwrap();
        assert_eq!(mask, back);
        // The keyword channel serializes as the plain word "self".
        let self_json = serde_json::to_string(&MaskChannel::Self_).unwrap();
        assert_eq!(self_json, "\"self\"");
    }
}
