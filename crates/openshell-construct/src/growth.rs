// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The growth record (RFC 0004, §4) — onboarding as a grown document.
//!
//! A grown room's onboarding flow keeper → room ends not in a config file
//! but in a **growth record**: the seed it grew from, the hash of the
//! charter it holds, the tick of its creation, the mask that locked at
//! that tick, and the heat it has run since. [`onboard`] renders that
//! record as the room's onboarding document — the room's origin story,
//! told in markdown, starting with the sentence that makes the whole
//! doctrine legible: *this room was grown, not configured*.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::mask::{MaskChannel, RoomMask, derive_mask};
use crate::residency::HeatState;
use crate::walks::encode_hex;

/// What a room grew from, and how it has lived since (RFC 0004, §4).
///
/// This is the onboarding document's source of truth: the growth record
/// *is* the configuration. A room can be re-seeded (new growth, new
/// record); it can never be un-masked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthRecord {
    /// The seed the room grew from (hex).
    pub seed_hex: String,
    /// SHA-256 of the room's charter (hex) — the commission it holds.
    pub charter_hash: String,
    /// The tick at which the room's mask locked.
    pub creation_tick: u64,
    /// The locked lattice: what dimensions of the world exist for this room.
    pub mask: RoomMask,
    /// Heat readings over the room's life so far: `(tick, state)` pairs,
    /// oldest first.
    pub heat_history: Vec<(u64, HeatState)>,
}

impl GrowthRecord {
    /// Grow a record from the seed inputs: derives the mask exactly as
    /// room creation does ([`derive_mask`]) and hashes the charter.
    pub fn grow(
        seed: &[u8],
        charter: &str,
        creation_tick: u64,
        heat_history: Vec<(u64, HeatState)>,
    ) -> Self {
        let charter_hash: [u8; 32] = Sha256::digest(charter.as_bytes()).into();
        Self {
            seed_hex: encode_hex(seed),
            charter_hash: encode_hex(&charter_hash),
            creation_tick,
            mask: derive_mask(seed, charter, creation_tick),
            heat_history,
        }
    }
}

/// One line of the mask lattice: the channel's word and what it means.
fn channel_line(channel: MaskChannel) -> &'static str {
    match channel {
        MaskChannel::Yard => "yard — telemetry of the shared yard",
        MaskChannel::Journal => "journal — the room's own journal, writes against need",
        MaskChannel::Road => "road — the road network, arrivals and link quality",
        MaskChannel::Wall => "wall — the gallery wall, what other rooms have left on display",
        MaskChannel::Self_ => "self — the room's own self-inspection",
        MaskChannel::Fleet => "fleet — other rooms, other keepers",
    }
}

/// Render the onboarding document (markdown) for a grown room.
///
/// Deterministic: the same growth record always renders the same
/// document — onboarding is a reading of the record, not an event.
pub fn onboard(record: &GrowthRecord) -> String {
    let mut doc = String::new();

    writeln!(doc, "# Room Growth Record").unwrap();
    writeln!(doc).unwrap();
    writeln!(doc, "This room was grown, not configured.").unwrap();
    writeln!(doc).unwrap();
    writeln!(doc, "- **Seed:** `{}`", record.seed_hex).unwrap();
    writeln!(doc, "- **Charter hash:** `{}`", record.charter_hash).unwrap();
    writeln!(doc, "- **Creation tick:** {}", record.creation_tick).unwrap();
    writeln!(doc).unwrap();

    writeln!(doc, "## Mask lattice").unwrap();
    writeln!(doc).unwrap();
    writeln!(doc, "The dimensions of the world that exist for this room:").unwrap();
    writeln!(doc).unwrap();
    for channel in &record.mask.channels {
        writeln!(doc, "- `{}`", channel_line(*channel)).unwrap();
    }
    writeln!(doc).unwrap();
    writeln!(
        doc,
        "The lattice locked at tick {} and cannot be reconfigured — a room \
         can be re-seeded, never un-masked.",
        record.creation_tick
    )
    .unwrap();
    writeln!(doc).unwrap();

    writeln!(doc, "## Heat timeline").unwrap();
    writeln!(doc).unwrap();
    if record.heat_history.is_empty() {
        writeln!(doc, "No heat readings yet — the room has not been read.").unwrap();
    } else {
        for (tick, state) in &record.heat_history {
            writeln!(doc, "- tick {tick} — {}", state.label()).unwrap();
        }
    }
    writeln!(doc).unwrap();

    writeln!(doc, "## First-tick temperature").unwrap();
    writeln!(doc).unwrap();
    if let Some((tick, state)) = record.heat_history.first() {
        writeln!(
            doc,
            "At tick {tick}, this room first read **{}** — the first tick \
             of a room is not an event, it is a temperature. Hold it \
             without verdict.",
            state.label()
        )
        .unwrap();
    } else {
        writeln!(
            doc,
            "No first tick yet. When it comes: the first tick of a room is \
             not an event, it is a temperature — hold it without verdict."
        )
        .unwrap();
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &[u8] = b"seed-of-the-grown-room";
    const CHARTER: &str = "hold the commission; read the water";

    /// A seed/tick pair whose mask locks at least three channels, so the
    /// rendered-doc assertions bite on real lattice content.
    fn rich_record() -> GrowthRecord {
        for tick in 0..1_000_u64 {
            let record = GrowthRecord::grow(SEED, CHARTER, tick, Vec::new());
            if record.mask.channels.len() >= 3 {
                return record;
            }
        }
        panic!("no three-channel mask in a thousand growths");
    }

    #[test]
    fn record_serializes_round_trip() {
        let mut record = rich_record();
        record.heat_history = vec![
            (0, HeatState::Warm),
            (600, HeatState::Cooling),
            (1_200, HeatState::Cold),
        ];

        let json = serde_json::to_string(&record).unwrap();
        let back: GrowthRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);

        // Tuples serialize as [tick, "state"] — the document stays legible.
        assert!(json.contains("[600,\"cooling\"]"), "json was: {json}");
    }

    #[test]
    fn charter_hash_is_sha256_of_charter() {
        let record = GrowthRecord::grow(SEED, CHARTER, 42, Vec::new());
        let digest: [u8; 32] = Sha256::digest(CHARTER.as_bytes()).into();
        assert_eq!(record.charter_hash, encode_hex(&digest));
        assert_eq!(record.mask, derive_mask(SEED, CHARTER, 42));
    }

    #[test]
    fn onboarding_document_carries_the_lattice_and_the_heat() {
        let mut record = rich_record();
        record.heat_history = vec![(0, HeatState::Warm), (600, HeatState::Cooling)];

        let doc = onboard(&record);

        assert!(doc.contains("This room was grown, not configured."));
        assert!(doc.contains(&record.seed_hex));
        assert!(doc.contains(&record.charter_hash));
        assert!(doc.contains("Mask lattice"));
        for channel in &record.mask.channels {
            assert!(
                doc.contains(&format!("- `{}`", channel_line(*channel))),
                "doc missing channel {channel:?}"
            );
        }
        assert!(doc.contains("Heat timeline"));
        for (tick, state) in &record.heat_history {
            assert!(
                doc.contains(&format!("tick {tick} — {}", state.label())),
                "doc missing heat entry {tick}/{state:?}"
            );
        }
        assert!(doc.contains("First-tick temperature"));
        assert!(doc.contains("At tick 0, this room first read **warm**"));
        assert!(doc.contains("never un-masked"));
    }

    #[test]
    fn onboarding_is_deterministic_and_tolerates_the_unread_room() {
        let mut record = rich_record();

        // Unread room: the document still renders, honestly.
        let doc = onboard(&record);
        assert!(doc.contains("No heat readings yet"));
        assert!(doc.contains("No first tick yet"));

        // Deterministic: same record, same document.
        record.heat_history = vec![(7, HeatState::Cold)];
        assert_eq!(onboard(&record), onboard(&record));
        assert_eq!(onboard(&record), onboard(&record.clone()));
    }
}
