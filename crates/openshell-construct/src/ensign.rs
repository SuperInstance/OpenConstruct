// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Ensign attention priors (RFC 0004, §2 — the elephant's nudge doctrine).
//!
//! An Ensign watching a room does not replace policy; it **correlates
//! attention**. [`prior`] reads the room's walk log and returns one
//! [`AttentionPrior`]: what to look at, how hard, and why — heat state
//! transitions inside the window, the cold-cell novel road, or a chain
//! break on load. A quiet room reads [`AttentionReason::None`].
//!
//! The urgency ladder (monotonic; tests pin it):
//!
//! | reason                          | urgency |
//! |---------------------------------|---------|
//! | novel road after sustained cold | 0.90    |
//! | chain break on load             | 0.80    |
//! | heat transition, coldward       | 0.60    |
//! | heat transition, warmward       | 0.45    |
//! | none                            | 0.00    |
//!
//! Integrity gates interpretation: a broken chain taints every other
//! signal, so it is reported before novelty even though a novel road
//! carries the higher urgency on the ladder.

use serde::{Deserialize, Serialize};

use crate::residency::{heat, novel_road_against_cold, windowed, HeatState, WalkLog};

/// The attention urgency of a novel road after sustained cold — the most
/// informative event in the room's life (RFC 0004, §2).
pub const NOVEL_ROAD_URGENCY: f32 = 0.90;
/// The attention urgency of a chain break on load.
pub const CHAIN_BREAK_URGENCY: f32 = 0.80;
/// The attention urgency of a heat transition toward cold (the room is
/// cooling — warm → cooling → cold).
pub const TRANSITION_COLDWARD_URGENCY: f32 = 0.60;
/// The attention urgency of a heat transition toward warm (the room is
/// warming).
pub const TRANSITION_WARMWARD_URGENCY: f32 = 0.45;
/// The attention urgency of a quiet room.
pub const NO_PRIOR_URGENCY: f32 = 0.0;

/// Why the Ensign is looking at this room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionReason {
    /// The room's heat changed state inside the window
    /// (warm → cooling → cold, or the reverse).
    HeatTransition,
    /// A novel road arrived after sustained cold (the cold-cell anomaly).
    NovelRoad,
    /// The walks file's recomputed chain disagrees with its persisted
    /// prev_chain — edited, truncated, or corrupted between restarts.
    ChainBreak,
    /// Nothing worth correlating.
    None,
}

/// One Ensign nudge: look here, this hard, for this reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionPrior {
    /// How hard to look, 0.0–1.0 (monotonic ladder, see module docs).
    pub urgency: f32,
    /// Why.
    pub reason: AttentionReason,
    /// Human-keeper detail, e.g. `"novel road `h-road-9` after sustained
    /// cold"` or `"heat transition in window: warm → cold"`.
    pub detail: String,
}

impl AttentionPrior {
    /// The quiet-room prior.
    fn none(detail: impl Into<String>) -> Self {
        Self {
            urgency: NO_PRIOR_URGENCY,
            reason: AttentionReason::None,
            detail: detail.into(),
        }
    }
}

/// Read the room's walk log and return the current attention prior.
///
/// `window` is the number of most recent walks judged for heat (same
/// window rule as [`heat`]); the walks before it are the sediment the
/// transition is measured against. Pure: reads the log, computes, returns
/// — no I/O, no clocks, no verdicts.
pub fn prior(walks: &WalkLog, window: usize) -> AttentionPrior {
    // Integrity gates interpretation: a broken chain taints every other
    // signal, novelty included.
    if !walks.verify() {
        return AttentionPrior {
            urgency: CHAIN_BREAK_URGENCY,
            reason: AttentionReason::ChainBreak,
            detail: "chain break on load: recomputed head does not match the \
                     persisted prev_chain — walks file edited, truncated, or \
                     corrupted between restarts"
                .into(),
        };
    }

    let records = walks.records();
    let suffix = windowed(records, window);
    if suffix.is_empty() {
        return AttentionPrior::none("no walks in window — nothing to read");
    }

    // Cold-cell anomaly, through heat()'s own logic (shared helpers, never
    // a second opinion).
    let reading = heat(records, window);
    if let Some(novel) = novel_road_against_cold(suffix) {
        return AttentionPrior {
            urgency: NOVEL_ROAD_URGENCY,
            reason: AttentionReason::NovelRoad,
            detail: format!(
                "novel road `{}` after sustained cold — the most informative \
                 event in the room's life; hold without verdict, surface to \
                 the keeper",
                novel.road
            ),
        };
    }

    // Heat state transition inside the window, read against the sediment
    // before it (a window covering every walk has no sediment to compare).
    let prefix = &records[..records.len() - suffix.len()];
    if !prefix.is_empty() {
        let before = heat(prefix, usize::MAX).state;
        let now = reading.state;
        if before != now {
            let (urgency, direction) = if coldness(now) > coldness(before) {
                (TRANSITION_COLDWARD_URGENCY, "cooling")
            } else {
                (TRANSITION_WARMWARD_URGENCY, "warming")
            };
            return AttentionPrior {
                urgency,
                reason: AttentionReason::HeatTransition,
                detail: format!(
                    "heat transition in window: {} → {} — the room is {}",
                    before.label(),
                    now.label(),
                    direction
                ),
            };
        }
    }

    AttentionPrior::none(format!(
        "steady {} — no attention prior",
        reading.state.label()
    ))
}

/// Where a heat state sits on the warm↔cold axis (for transition
/// direction; ordinal, not a metric).
fn coldness(state: HeatState) -> u8 {
    match state {
        HeatState::Warm => 0,
        HeatState::Cooling => 1,
        HeatState::Cold => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residency::WalkRecord;

    fn rec(ts: u64, road: &str, lq: f32) -> WalkRecord {
        WalkRecord {
            ts,
            road: road.into(),
            link_quality: lq,
            arrival_meta: None,
        }
    }

    /// Warm: varied roads, swinging link quality, writes against need.
    fn warm_walks() -> Vec<WalkRecord> {
        let roads = ["h-road-0", "north", "h-road-2", "rim"];
        let lqs = [0.20, 0.55, 0.90, 0.35, 0.75, 0.45, 0.95, 0.60];
        let gaps = [30_u64, 90, 15, 120, 45, 75, 10];
        let mut ts = 1_000_u64;
        let mut walks = Vec::new();
        for i in 0..8 {
            walks.push(rec(ts, roads[i % roads.len()], lqs[i]));
            ts += gaps[i % gaps.len()];
        }
        walks
    }

    /// Cold: flatline, commission standing, one write a minute.
    fn cold_walks() -> Vec<WalkRecord> {
        (0..8_u64).map(|i| rec(i * 60, "local", 0.50)).collect()
    }

    fn log_of(walks: Vec<WalkRecord>) -> WalkLog {
        let mut log = WalkLog::new();
        for walk in walks {
            log.append(walk);
        }
        log
    }

    #[test]
    fn steady_room_reads_no_prior() {
        let p = prior(&log_of(warm_walks()), 8);
        assert_eq!(p.reason, AttentionReason::None);
        assert_eq!(p.urgency, NO_PRIOR_URGENCY);
        assert!(p.detail.contains("warm"));

        // The prior itself serializes (Ensigns ship these across the wire).
        let json = serde_json::to_string(&p).unwrap();
        let back: AttentionPrior = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn empty_log_reads_no_prior() {
        let p = prior(&WalkLog::new(), 16);
        assert_eq!(p.reason, AttentionReason::None);
        assert_eq!(p.urgency, NO_PRIOR_URGENCY);
    }

    #[test]
    fn coldward_transition_fires() {
        // Warm sediment, then a cold tail: warm → cooling → cold.
        let mut walks = warm_walks();
        walks.extend(cold_walks());
        let p = prior(&log_of(walks), 8);
        assert_eq!(p.reason, AttentionReason::HeatTransition);
        assert_eq!(p.urgency, TRANSITION_COLDWARD_URGENCY);
        assert!(p.detail.contains("warm → cold"), "detail was: {}", p.detail);
        assert!(p.detail.contains("cooling"));
    }

    #[test]
    fn warmward_transition_fires_softer() {
        // Cold sediment, then a warm tail: the room is warming.
        let mut walks = cold_walks();
        walks.extend(warm_walks());
        let p = prior(&log_of(walks), 8);
        assert_eq!(p.reason, AttentionReason::HeatTransition);
        assert_eq!(p.urgency, TRANSITION_WARMWARD_URGENCY);
        assert!(p.detail.contains("cold → warm"), "detail was: {}", p.detail);
    }

    #[test]
    fn novel_road_after_cold_fires() {
        // Sustained cold, then the 06:11 write: one novel road with a spike.
        let mut walks = cold_walks();
        let last_ts = walks.last().unwrap().ts + 60;
        walks.push(rec(last_ts, "h-road-9", 0.95));

        let p = prior(&log_of(walks), usize::MAX);
        assert_eq!(p.reason, AttentionReason::NovelRoad);
        assert_eq!(p.urgency, NOVEL_ROAD_URGENCY);
        assert!(p.detail.contains("h-road-9"), "detail was: {}", p.detail);
    }

    #[test]
    fn chain_break_on_load_fires() {
        // A head pin that disagrees with the recomputed chain.
        let mut log = log_of(cold_walks());
        assert!(log.verify());
        log.expect_chain_head([0xAB; 32]);
        assert!(!log.verify(), "a pinned head that disagrees breaks the chain");

        let p = prior(&log, usize::MAX);
        assert_eq!(p.reason, AttentionReason::ChainBreak);
        assert_eq!(p.urgency, CHAIN_BREAK_URGENCY);
        assert!(p.detail.contains("chain break"));
    }

    #[test]
    fn urgency_ladder_is_monotonic() {
        // novel road > chain break > coldward transition > warmward
        // transition > none — and none is exactly quiet.
        let mut cold_then_novel = cold_walks();
        let last_ts = cold_then_novel.last().unwrap().ts + 60;
        cold_then_novel.push(rec(last_ts, "h-road-9", 0.95));
        let novel = prior(&log_of(cold_then_novel), usize::MAX);

        let mut broken = log_of(cold_walks());
        broken.expect_chain_head([0xAB; 32]);
        let chain = prior(&broken, usize::MAX);

        let mut cooling = warm_walks();
        cooling.extend(cold_walks());
        let coldward = prior(&log_of(cooling), 8);

        let mut warming = cold_walks();
        warming.extend(warm_walks());
        let warmward = prior(&log_of(warming), 8);

        let none = prior(&log_of(warm_walks()), 8);

        assert!(novel.urgency > chain.urgency);
        assert!(chain.urgency > coldward.urgency);
        assert!(coldward.urgency > warmward.urgency);
        assert!(warmward.urgency > none.urgency);
        assert_eq!(none.urgency, 0.0);
        // The task's floor: novel road > heat transition > none.
        assert!(novel.urgency > coldward.urgency);
        assert!(coldward.urgency > none.urgency);
    }
}
