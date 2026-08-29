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

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

use crate::ensign::prior;
use crate::growth::{onboard, GrowthRecord};
use crate::mask::RoomMask;
use crate::residency::{heat, HeatReading, WalkLog, WalkRecord};
use crate::walks::{decode_hex32, encode_hex};

/// Domain-separation prefix for the room-id derivation.
const ROOM_ID_DOMAIN: &[u8] = b"openshell-construct/room-id/v1";

/// The heat window a room reads on every tick: the most recent
/// [`ROOM_WINDOW`] walks. The walks before the window are the sediment
/// Ensign transitions are measured against (see [`crate::ensign`]).
pub const ROOM_WINDOW: usize = 16;

/// The format tag written into every room file (versioned: bump to change
/// the on-disk shape).
pub const ROOM_FILE_FORMAT: &str = "openshell-construct/room/v1";

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

    // ── The room file ───────────────────────────────────────────────

    /// Save the room as a single JSON file at `path` (atomic: temp file
    /// + rename).
    ///
    /// The file wraps everything the room is:
    ///
    /// ```json
    /// {
    ///   "format": "openshell-construct/room/v1",
    ///   "id": "room-…",
    ///   "mask": { … },
    ///   "growth": { …the growth record… },
    ///   "walks": [ …one walks/2 record per entry, in order… ],
    ///   "chain": { "head": "<64 hex>", "records": N }
    /// }
    /// ```
    ///
    /// The `walks` entries are exactly the walks/2 JSON encodings (the
    /// JSONL lines, embedded), and `chain` is the checkpoint pinning the
    /// head of exactly those `N` walks — the same continuity contract as
    /// the walks file recorder, folded into one file.
    ///
    /// Honest limit: the chain pins the **walks**; the growth record is
    /// cross-checked (id and mask must agree with what the record
    /// derives) but its heat history is not hash-chained in v1.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), RoomError> {
        let file = RoomFile {
            format: ROOM_FILE_FORMAT.to_owned(),
            id: self.id.clone(),
            mask: self.mask.clone(),
            growth: self.growth.clone(),
            walks: self.walklog.records().to_vec(),
            chain: ChainCheckpoint {
                head: encode_hex(&self.walklog.head()),
                records: u64::try_from(self.walklog.records().len())
                    .expect("walk count fits u64"),
            },
        };
        let json = serde_json::to_vec_pretty(&file)?;
        let path = path.as_ref();
        let tmp = path.with_extension("room-tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load a room from its file.
    ///
    /// Verifies before returning: the format tag, the walk chain (the
    /// recomputed head must equal the persisted checkpoint — any walk
    /// edited, dropped, or spliced reads as tamper), the record count,
    /// the room id (recomputed from the growth record), and the
    /// mask/growth invariant. **Refuses, never panics**: every mismatch
    /// returns [`RoomError::Tamper`] with the reason; a missing or
    /// unreadable file returns [`RoomError::Io`].
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RoomError> {
        let file: RoomFile = serde_json::from_str(&fs::read_to_string(path)?)?;

        if file.format != ROOM_FILE_FORMAT {
            return Err(RoomError::Tamper(format!(
                "unknown format tag `{}` (expected `{ROOM_FILE_FORMAT}`)",
                file.format
            )));
        }

        // The chain: rebuild from the walks, pin the checkpointed head.
        let declared = usize::try_from(file.chain.records)
            .map_err(|_| RoomError::Tamper(format!("record count overflows usize: {}", file.chain.records)))?;
        if declared != file.walks.len() {
            return Err(RoomError::Tamper(format!(
                "chain pins {} walks but the file carries {}",
                file.chain.records,
                file.walks.len()
            )));
        }
        let expected_head = decode_hex32(&file.chain.head).ok_or_else(|| {
            RoomError::Tamper(format!("chain head is not 64-char hex: `{}`", file.chain.head))
        })?;
        let mut walklog = WalkLog::new();
        for walk in file.walks {
            walklog.append(walk);
        }
        if walklog.head() != expected_head {
            return Err(RoomError::Tamper(
                "recomputed chain head does not match the persisted checkpoint — \
                 walks edited, dropped, or spliced since save"
                    .into(),
            ));
        }
        walklog.expect_chain_head(expected_head);

        // Identity and lattice: both must agree with what the growth
        // record derives (the record is the witness).
        let id = derive_room_id(&file.growth);
        if file.id != id {
            return Err(RoomError::Tamper(format!(
                "room id `{}` does not derive from the growth record (expected `{id}`)",
                file.id
            )));
        }
        if file.mask != file.growth.mask {
            return Err(RoomError::Tamper(
                "mask disagrees with the growth record's lattice — the mask \
                 locks at creation and cannot be reconfigured"
                    .into(),
            ));
        }

        Ok(Self {
            id,
            mask: file.growth.mask.clone(),
            growth: file.growth,
            walklog,
        })
    }
}

// ── The room file ─────────────────────────────────────────────────

/// One room, one file: growth record + embedded walks JSONL + chain
/// checkpoint (see [`Room::save`]).
#[derive(Serialize, Deserialize)]
struct RoomFile {
    format: String,
    id: String,
    mask: RoomMask,
    growth: GrowthRecord,
    walks: Vec<WalkRecord>,
    chain: ChainCheckpoint,
}

/// The persisted chain checkpoint: the head covering exactly `records`
/// walks (the room-file spelling of the walks-file checkpoint line).
#[derive(Serialize, Deserialize)]
struct ChainCheckpoint {
    head: String,
    records: u64,
}

/// What can go wrong growing, saving, or loading a room.
#[derive(Debug, Error)]
pub enum RoomError {
    /// The file could not be read or written.
    #[error("room file io: {0}")]
    Io(#[from] io::Error),
    /// The file is not valid JSON for the room shape.
    #[error("room file serialize: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The file failed verification — refuse, never panic.
    #[error("room file tampered: {0}")]
    Tamper(String),
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

    // ── Room file tests ─────────────────────────────────────────────

    use super::{RoomError, ROOM_FILE_FORMAT};
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tmp_room_path(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "openshell-room-{}-{}-{}.json",
            std::process::id(),
            tag,
            n
        ))
    }

    /// A saved-and-reloaded JSON value, with one field rewritten — the
    /// tamperer's pen.
    fn tamper_with(path: &Path, edit: impl FnOnce(&mut Value)) {
        let mut value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        edit(&mut value);
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    #[test]
    fn save_load_roundtrip_preserves_the_room() {
        let path = tmp_room_path("roundtrip");
        let (mut room, _) = Room::grow(SEED, CHARTER, 1_000);
        for walk in cold_walks(3) {
            room.tick(walk);
        }
        room.save(&path).unwrap();

        let loaded = Room::load(&path).unwrap();
        assert_eq!(loaded, room, "a roundtrip must preserve the whole room");
        assert!(loaded.walklog.verify());

        // The loaded room keeps living: the chain continues, and a second
        // save/load still agrees.
        let mut loaded = loaded;
        let (_, prior) = loaded.tick(rec(180, "h-road-0", 0.8));
        assert_ne!(prior.reason, AttentionReason::ChainBreak);
        assert!(loaded.walklog.verify(), "the loaded chain must continue, not restart");
        loaded.save(&path).unwrap();
        assert_eq!(Room::load(&path).unwrap(), loaded);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn tampered_walk_refuses_to_load() {
        let path = tmp_room_path("walk");
        let (mut room, _) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(3) {
            room.tick(walk);
        }
        room.save(&path).unwrap();
        assert!(Room::load(&path).is_ok());

        // Rewrite the middle walk's road: the recomputed head disagrees.
        tamper_with(&path, |v| v["walks"][1]["road"] = Value::from("h-road-9"));
        match Room::load(&path) {
            Err(RoomError::Tamper(detail)) => {
                assert!(detail.contains("head"), "detail was: {detail}");
            }
            other => panic!("expected Tamper, got {other:?}"),
        }

        // Drop the last walk: the count and head both disagree.
        let (mut room, _) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(3) {
            room.tick(walk);
        }
        room.save(&path).unwrap();
        tamper_with(&path, |v| {
            v["walks"].as_array_mut().unwrap().pop();
        });
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn tampered_checkpoint_or_identity_refuses_to_load() {
        let path = tmp_room_path("identity");
        let (mut room, _) = Room::grow(SEED, CHARTER, 0);
        for walk in cold_walks(3) {
            room.tick(walk);
        }
        room.save(&path).unwrap();

        // Forged checkpoint head.
        tamper_with(&path, |v| v["chain"]["head"] = Value::from("ab".repeat(32)));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        // Checkpoint count disagrees with the walks carried.
        room.save(&path).unwrap();
        tamper_with(&path, |v| v["chain"]["records"] = Value::from(99));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        // Swapped identity: the id no longer derives from the record.
        room.save(&path).unwrap();
        tamper_with(&path, |v| v["id"] = Value::from("room-deadbeefdeadbeef"));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        // Forged lattice: the mask no longer matches the growth record.
        room.save(&path).unwrap();
        tamper_with(&path, |v| v["mask"]["channels"] = serde_json::json!(["fleet"]));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        // Forged seed: the id (and mask) no longer derive.
        room.save(&path).unwrap();
        tamper_with(&path, |v| v["growth"]["seed_hex"] = Value::from("00".repeat(16)));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));

        // Unknown format tag.
        room.save(&path).unwrap();
        tamper_with(&path, |v| v["format"] = Value::from("some/other/v2"));
        assert!(matches!(Room::load(&path), Err(RoomError::Tamper(_))));
        assert_ne!(ROOM_FILE_FORMAT, "some/other/v2");

        // A missing file is an io error — refused, not a panic.
        let missing = tmp_room_path("missing");
        assert!(matches!(Room::load(&missing), Err(RoomError::Io(_))));

        let _ = fs::remove_file(&path);
    }
}
