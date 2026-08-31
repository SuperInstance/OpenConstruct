// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! CLI support — the three room verbs, as thin functions (RFC 0004).
//!
//! The `openconstruct` binary wires these under `room grow` / `room walk`
//! / `room show` (see `crates/openconstruct-cli`). The logic lives here,
//! behind the crate's public API, so the verbs are exercised by
//! integration tests (`tests/cli.rs`) without building the binary — the
//! arg-parser layer stays a thin seam: it parses flags and supplies the
//! wall-clock tick; everything a keeper would want tested happens in
//! these functions.
//!
//! The verbs:
//!
//! - [`grow_room`] — grow a room from a seed and charter, save it, and
//!   return its onboarding document.
//! - [`walk_room`] — record one arrival on a saved room and return the
//!   room's temperature plus any Ensign prior.
//! - [`show_room`] — load a saved room and return its growth record,
//!   told as the onboarding document.

use std::fmt::Write as _;
use std::path::Path;

use crate::residency::WalkRecord;
use crate::room::{Room, RoomError};

/// The `room grow` verb: grow a room from `seed` and `charter` at
/// `creation_tick`, save it to `out`, and return the onboarding document.
pub fn grow_room(
    seed: &str,
    charter: &str,
    creation_tick: u64,
    out: &Path,
) -> Result<String, RoomError> {
    let (room, doc) = Room::grow(seed.as_bytes(), charter, creation_tick);
    room.save(out)?;
    Ok(doc)
}

/// The `room walk` verb.
///
/// Record one arrival (`road`, `link_quality`, at tick `ts`) on the room
/// saved at `file`, persist it, and return the room's temperature plus
/// any attention prior — one line each, for the keeper's console.
pub fn walk_room(file: &Path, road: &str, link_quality: f32, ts: u64) -> Result<String, RoomError> {
    let mut room = Room::load(file)?;
    let (reading, prior) = room.tick(WalkRecord {
        ts,
        road: road.to_owned(),
        link_quality,
        arrival_meta: None,
    });
    room.save(file)?;

    let mut out = String::new();
    if reading.novel_road_detected {
        let _ = writeln!(
            out,
            "heat: {} (novel road — held without verdict)",
            reading.state.label()
        );
    } else {
        let _ = writeln!(out, "heat: {}", reading.state.label());
    }
    let _ = write!(
        out,
        "prior: {} ({:.2}) — {}",
        reason_label(prior.reason),
        prior.urgency,
        prior.detail
    );
    Ok(out)
}

/// The `room show` verb: load the room saved at `file` and return its
/// growth record as the onboarding document.
pub fn show_room(file: &Path) -> Result<String, RoomError> {
    let room = Room::load(file)?;
    Ok(room.onboarding_doc())
}

/// The snake-case word for a prior's reason (matches the serde spelling).
fn reason_label(reason: crate::AttentionReason) -> &'static str {
    match reason {
        crate::AttentionReason::HeatTransition => "heat_transition",
        crate::AttentionReason::NovelRoad => "novel_road",
        crate::AttentionReason::ChainBreak => "chain_break",
        crate::AttentionReason::None => "none",
    }
}
