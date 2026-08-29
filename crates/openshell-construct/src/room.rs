// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The room itself (RFC 0004) — the binding of mask, growth, and walks.
//!
//! The pieces landed first: the locked lattice ([`RoomMask`]), the grown
//! onboarding document ([`GrowthRecord`]), the hash-chained [`WalkLog`],
//! the heat read from it, the Ensign's attention prior. [`Room`] is the
//! join: one identity (`id`), one mask, one growth record, one walk log —
//! and **one call**, [`Room::tick`], that is the room's whole inner life:
//!
//! ```text
//! walk arrives ──▶ append to the chain ──▶ re-read heat over the window
//!              ──▶ record the transition in the growth record (if any)
//!              ──▶ return (HeatReading, AttentionPrior)
//! ```
//!
//! Nothing else is needed to live in a room: the first tick of a new room
//! is not an event, it is a temperature.

use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::ensign::prior;
use crate::growth::{onboard, GrowthRecord};
use crate::mask::RoomMask;
use crate::residency::{heat, HeatReading, WalkLog, WalkRecord};
use crate::walks::encode_hex;

/// Domain-separation prefix for the room-id derivation.
const ROOM_ID_DOMAIN: &[u8] = b"openshell-construct/room-id/v1";

/// The heat window a room reads on every tick: the most recent
/// [`ROOM_WINDOW`] walks. The walks before the window are the sediment
/// Ensign transitions are measured against (see [`crate::ensign`]).
pub const ROOM_WINDOW: usize = 16;

/// A grown room: identity, lattice, growth record, and walks (RFC 0004).
///
/// Invariant: `mask == growth.mask` always — the lattice locks at
/// creation and the growth record is its witness. [`Room::save`] /
/// [`Room::load`] verify the invariant (and the walk chain) on load.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    /// Deterministic identity, derived from the growth inputs
    /// (`"room-<12 hex>"`). Stable across saves; recomputed on load.
    pub id: String,
    /// The locked lattice (always equal to `growth.mask`).
    pub mask: RoomMask,
    /// What the room grew from, and how it has lived since.
    pub growth: GrowthRecord,
    /// The hash-chained arrival telemetry heat is read from.
    pub walklog: WalkLog,
}

/// Derive the room's deterministic id from its growth record:
/// `room-<first 12 hex of SHA-256(room-id domain ‖ seed_hex ‖
/// charter_hash ‖ creation_tick_le)>`. Recomputable from the record alone,
/// so a room file's id is verifiable without any other witness.
fn derive_room_id(record: &GrowthRecord) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ROOM_ID_DOMAIN);
    hasher.update([0x00]);
    hasher.update(record.seed_hex.as_bytes());
    hasher.update(record.charter_hash.as_bytes());
    hasher.update(record.creation_tick.to_le_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    let hex = encode_hex(&digest);
    let mut id = String::with_capacity(5 + 12);
    let _ = write!(id, "room-{}", &hex[..12]);
    id
}

impl Room {
    /// Grow a room from its seed, charter, and creation tick: derives the
    /// mask (the lattice locks now, like a crystal leaving the bath) and
    /// the initial [`GrowthRecord`], and writes the room's onboarding
    /// document — returned alongside the room. The same growth inputs
    /// always grow the same room (id, mask, and document).
    pub fn grow(seed: &[u8], charter: &str, creation_tick: u64) -> (Self, String) {
        let growth = GrowthRecord::grow(seed, charter, creation_tick, Vec::new());
        let id = derive_room_id(&growth);
        let room = Self {
            id,
            mask: growth.mask.clone(),
            growth,
            walklog: WalkLog::new(),
        };
        let doc = room.onboarding_doc();
        (room, doc)
    }

    /// The room's onboarding document, rendered from the growth record.
    /// Deterministic: onboarding is a reading of the record, not an event.
    pub fn onboarding_doc(&self) -> String {
        onboard(&self.growth)
    }

    /// One tick of the room's inner life.
    ///
    /// 1. Appends the walk to the hash chain (keeping the chain's pinned
    ///    head in step, so the log stays verifiable in memory — mirrors
    ///    the walks-file recorder).
    /// 2. Recomputes residency heat over the last [`ROOM_WINDOW`] walks.
    /// 3. Records the reading in the growth record's heat history **iff
    ///    the state changed** (the first tick is always a temperature:
    ///    an empty history records it). One transition, one entry.
    /// 4. Returns `(heat, prior)` — how the room is living, and what (if
    ///    anything) the Ensign wants the keeper to look at.
    pub fn tick(&mut self, walk: WalkRecord) -> (HeatReading, crate::AttentionPrior) {
        let ts = walk.ts;
        let head = self.walklog.append(walk);
        self.walklog.expect_chain_head(head);

        let reading = heat(self.walklog.records(), ROOM_WINDOW);
        let state_changed = match self.growth.heat_history.last() {
            Some((_, last)) => *last != reading.state,
            None => true, // the first tick is a temperature: record it
        };
        if state_changed {
            self.growth.heat_history.push((ts, reading.state));
        }

        let prior = prior(&self.walklog, ROOM_WINDOW);
        (reading, prior)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ensign::{AttentionReason, NOVEL_ROAD_URGENCY};
    use crate::residency::HeatState;

    const SEED: &[u8] = b"seed-of-the-bound-room";
    const CHARTER: &str = "hold the commission; read the water";

    fn rec(ts: u64, road: &str, lq: f32) -> WalkRecord {
        WalkRecord {
            ts,
            road: road.into(),
            link_quality: lq,
            arrival_meta: None,
        }
    }

    /// Cold: flatline, commission standing, one write a minute.
    fn cold_walks(n: u64) -> Vec<WalkRecord> {
        (0..n).map(|i| rec(i * 60, "local", 0.50)).collect()
    }

    /// Warm: varied roads, swinging link quality, writes against need.
    fn warm_walks() -> Vec<WalkRecord> {
        let roads = ["h-road-0", "north", "h-road-2", "rim"];
        let lqs = [0.20, 0.55, 0.90, 0.35, 0.75, 0.45, 0.95, 0.60];
        let gaps = [30_u64, 90, 15, 120, 45, 75, 10];
        let mut ts = 10_000_u64;
        let mut walks = Vec::new();
        for i in 0..8 {
            walks.push(rec(ts, roads[i % roads.len()], lqs[i]));
            ts += gaps[i % gaps.len()];
        }
        walks
    }

    #[test]
    fn grow_binds_mask_growth_and_identity() {
        let (room, doc) = Room::grow(SEED, CHARTER, 1_000);

        assert!(room.id.starts_with("room-"), "id was: {}", room.id);
        assert_eq!(room.id.len(), "room-".len() + 12);
        assert_eq!(room.mask, room.growth.mask, "the lattice and its witness agree");
        assert!(room.mask.is_grown());
        assert!(room.growth.heat_history.is_empty());
        assert!(room.walklog.records().is_empty());

        // The document is the growth record, told in markdown.
        assert!(doc.contains("This room was grown, not configured."));
        assert!(doc.contains(&room.growth.seed_hex));
        assert!(doc.contains("No heat readings yet"));

        // Deterministic growth: same inputs, same room, same document.
        let (again, doc_again) = Room::grow(SEED, CHARTER, 1_000);
        assert_eq!(room, again);
        assert_eq!(doc, doc_again);
    }

    #[test]
    fn first_tick_records_the_first_temperature() {
        let (mut room, _doc) = Room::grow(SEED, CHARTER, 0);
        let (reading, prior) = room.tick(rec(60, "local", 0.5));

        // A single arrival is not a residency: it reads cold, quietly.
        assert_eq!(reading.state, HeatState::Cold);
        assert!(!reading.novel_road_detected);
        assert_eq!(prior.reason, AttentionReason::None);

        // ...but the first tick is still a temperature: it is recorded.
        assert_eq!(room.growth.heat_history, vec![(60, HeatState::Cold)]);
        assert!(room.walklog.verify(), "the chain pin must stay in step after a tick");
    }

    #[test]
    fn steady_room_appends_no_history() {
        let (mut room, _doc) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(8) {
            room.tick(walk);
        }
        // Cold at first tick, cold ever since: exactly one entry.
        assert_eq!(room.growth.heat_history, vec![(0, HeatState::Cold)]);
    }

    #[test]
    fn transitions_recorded_exactly_once_per_change() {
        let (mut room, _doc) = Room::grow(SEED, CHARTER, 0);
        let mut tick_n = 0_u64;

        // Cold life, then the room warms.
        for walk in cold_walks(8) {
            room.tick(walk);
            tick_n += 1;
        }
        for walk in warm_walks() {
            room.tick(walk);
            tick_n += 1;
        }
        for walk in warm_walks() {
            room.tick(rec(tick_n * 100 + walk.ts, &walk.road, walk.link_quality));
        }

        let history = &room.growth.heat_history;
        // Exactly-once: no two adjacent entries share a state, ever.
        for pair in history.windows(2) {
            assert_ne!(pair[0].1, pair[1].1, "adjacent history entries repeat a state: {history:?}");
        }
        // The last entry always matches the current reading.
        let reading = heat(room.walklog.records(), ROOM_WINDOW);
        assert_eq!(history.last().unwrap().1, reading.state);
        // The room did warm: at least one transition was recorded.
        assert!(history.len() >= 2, "expected at least one transition: {history:?}");
        assert_eq!(reading.state, HeatState::Warm);

        // Steady warm ticks after the transition add nothing more.
        let len_before = history.len();
        let (reading, _) = room.tick(rec(99_999, "north", 0.7));
        assert_eq!(reading.state, HeatState::Warm);
        assert_eq!(room.growth.heat_history.len(), len_before, "a steady state is not a transition");
    }

    #[test]
    fn tick_propagates_the_ensign_prior() {
        // Sustained cold, then the 06:11 write: the cold-cell anomaly.
        let (mut room, _doc) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(8) {
            room.tick(walk);
        }
        let (reading, prior) = room.tick(rec(8 * 60, "h-road-9", 0.95));

        // Held without verdict: still cold, but flagged for the keeper.
        assert_eq!(reading.state, HeatState::Cold);
        assert!(reading.novel_road_detected);
        assert_eq!(prior.reason, AttentionReason::NovelRoad);
        assert_eq!(prior.urgency, NOVEL_ROAD_URGENCY);
        assert!(prior.detail.contains("h-road-9"), "detail was: {}", prior.detail);
        // And no fake transition: cold held, so history does not move.
        assert_eq!(
            room.growth.heat_history.last(),
            Some(&(0, HeatState::Cold)),
            "the anomaly is held, not warmed"
        );
    }

    #[test]
    fn tick_reads_the_windowed_heat() {
        let (mut room, _doc) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(8).into_iter().chain(warm_walks()) {
            let (reading, _) = room.tick(walk);
            assert_eq!(
                reading,
                heat(room.walklog.records(), ROOM_WINDOW),
                "tick's reading must be heat() over the room window"
            );
        }
    }
}
