// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Walks, not waves — and the heat they leave (RFC 0004, §2–§3).
//!
//! Every room records `walks/2` lines: `{ts, road, link_quality,
//! arrival_meta}` — zero semantics, byte-exact, hash-chained. Residency heat
//! is derived from walk records alone: the room's arrival telemetry over
//! three timescales. No self-reports, no timestamps-inferred-intentions —
//! **subtext is observed, not declared**.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Walk records ─────────────────────────────────────────────────────────

/// One arrival at this room. Zero semantics; the recorder never interprets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkRecord {
    /// Monotonic tick of arrival.
    pub ts: u64,
    /// The road the arrival came by (e.g. `"local"`, `"h-road-0"`).
    pub road: String,
    /// Link quality of the arrival, 0.0–1.0.
    pub link_quality: f32,
    /// Free-form arrival metadata, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival_meta: Option<String>,
}

impl WalkRecord {
    /// Deterministic byte encoding used by the hash chain: the compact
    /// serde-JSON encoding of the record. Struct fields serialize in
    /// declaration order, so the encoding is byte-stable for a given value.
    ///
    /// # Panics
    ///
    /// Only if `serde_json` fails to serialize a plain struct, which it
    /// cannot for this shape.
    fn record_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("walk record serializes")
    }
}

// ── The hash-chained walk log ────────────────────────────────────────────

/// Genesis chain value for an empty log.
pub const GENESIS_CHAIN: [u8; 32] = [0_u8; 32];

/// Append-only, hash-chained walk log (mirrors the rd `walks/2` schema).
///
/// Chain rule: `chain[i] = SHA-256(chain[i-1] || record_bytes(records[i]))`
/// with `chain[-1] = GENESIS_CHAIN`. Any tampered record, dropped entry, or
/// forged link breaks [`WalkLog::verify`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WalkLog {
    records: Vec<WalkRecord>,
    chain: Vec<[u8; 32]>,
}

impl WalkLog {
    /// A new, empty log (head = genesis).
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one record, extending the chain. Returns the new head hash.
    pub fn append(&mut self, record: WalkRecord) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.head());
        hasher.update(record.record_bytes());
        let head: [u8; 32] = hasher.finalize().into();
        self.records.push(record);
        self.chain.push(head);
        head
    }

    /// All recorded walks, in order.
    pub fn records(&self) -> &[WalkRecord] {
        &self.records
    }

    /// Current chain head (genesis for an empty log).
    pub fn head(&self) -> [u8; 32] {
        self.chain.last().copied().unwrap_or(GENESIS_CHAIN)
    }

    /// Recompute the chain from genesis and compare. `true` iff no record
    /// was mutated, no entry dropped, and no link forged.
    pub fn verify(&self) -> bool {
        if self.records.len() != self.chain.len() {
            return false; // an entry was dropped or a link spliced
        }
        let mut prev = GENESIS_CHAIN;
        for (record, link) in self.records.iter().zip(self.chain.iter()) {
            let mut hasher = Sha256::new();
            hasher.update(prev);
            hasher.update(record.record_bytes());
            let expect: [u8; 32] = hasher.finalize().into();
            if &expect != link {
                return false;
            }
            prev = *link;
        }
        true
    }
}

// ── Residency heat ───────────────────────────────────────────────────────

/// The room's temperature, read from walks alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatState {
    /// Varied roads, high link-quality variance, writes against need.
    Warm,
    /// Roads defaulting to local, link quality flatlining.
    Cooling,
    /// Flatline; commission standing; one write a minute, like a gate tread.
    Cold,
}

/// A heat reading plus the Ensign signal.
///
/// `novel_road_detected` is the cold-cell anomaly: a novel road after
/// sustained cold. The room must NOT flip to `Warm` — the Ensign holds the
/// event without verdict and surfaces it to the keeper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeatReading {
    /// The heat state.
    pub state: HeatState,
    /// A novel road arrived against flat sediment (cold-cell anomaly).
    pub novel_road_detected: bool,
}

/// The default road — arrivals that never left the room's own lane.
pub const LOCAL_ROAD: &str = "local";

// Hand-tuned thresholds (RFC 0004 honest limits: a temperature sense, not a
// metric of productivity). All comparisons over the most recent `window`
// walks, in f64 for stability.

/// Population variance of link quality at or below this is a flatline
/// (stdev ≤ 0.01).
const COLD_LQ_VARIANCE_MAX: f64 = 1e-4;
/// Link-quality variance at or above this means varied arrivals
/// (stdev ≥ 0.1).
const WARM_LQ_VARIANCE_MIN: f64 = 1e-2;
/// Fraction of `local` roads at or above this means defaulting to local.
const COLD_LOCAL_FRACTION_MIN: f64 = 0.8;
/// Fraction of `local` roads at or below this means varied roads.
const WARM_LOCAL_FRACTION_MAX: f64 = 0.3;
/// Coefficient of variation of inter-arrival gaps at or below this is a
/// metronome — one write a minute, like a gate tread.
const COLD_CADENCE_CV_MAX: f64 = 0.25;
/// Warm rooms write against need, not on a metronome: cadence CV must
/// exceed this when it can be judged.
const WARM_CADENCE_CV_MIN: f64 = 0.10;
/// A prefix must hold at least this many walks to count as sustained cold.
const SUSTAINED_COLD_MIN_WALKS: usize = 4;
/// Cadence needs at least this many gaps to be judged at all.
const MIN_GAPS_FOR_CADENCE: usize = 3;

/// Read the room's temperature from its walk records.
///
/// `window` is the number of most recent walks considered (all of them if
/// `window` exceeds the log length; nothing read if `window == 0` or the
/// log is empty → `Cold`).
///
/// Deterministic, pure, hand-tuned per RFC 0004:
///
/// - **Warm** — link-quality variance ≥ [`WARM_LQ_VARIANCE_MIN`] AND
///   local-road fraction ≤ [`WARM_LOCAL_FRACTION_MAX`] AND non-metronomic
///   cadence (CV > [`WARM_CADENCE_CV_MIN`]) whenever cadence is judgeable
///   (≥ [`MIN_GAPS_FOR_CADENCE`] gaps).
/// - **Cold** — link-quality variance ≤ [`COLD_LQ_VARIANCE_MAX`] AND
///   local-road fraction ≥ [`COLD_LOCAL_FRACTION_MIN`] AND metronomic
///   cadence (CV ≤ [`COLD_CADENCE_CV_MAX`]) whenever judgeable.
/// - **Cooling** — everything between the flatline and the flame.
/// - Fewer than two walks in the window read as `Cold`: a single arrival
///   is not a residency.
///
/// **Cold-cell anomaly** — if the walks before the last one are sustained
/// cold (≥ [`SUSTAINED_COLD_MIN_WALKS`] walks) and the last walk arrives by
/// a road never seen before, the reading is `(Cold,
/// novel_road_detected: true)`. The first tick after a novel expensive
/// action is not an error to suppress — it is the most informative event in
/// the room's life. Hold it without verdict; surface it to the keeper.
pub fn heat(walks: &[WalkRecord], window: usize) -> HeatReading {
    let window = if window == 0 {
        &[][..]
    } else {
        &walks[walks.len().saturating_sub(window)..]
    };

    // Cold-cell anomaly: novel road against flat sediment.
    if window.len() > SUSTAINED_COLD_MIN_WALKS {
        let (prefix, last) = window.split_at(window.len() - 1);
        let last = &last[0];
        if classify(prefix) == HeatState::Cold
            && last.road != LOCAL_ROAD
            && !prefix.iter().any(|r| r.road == last.road)
        {
            return HeatReading {
                state: HeatState::Cold,
                novel_road_detected: true,
            };
        }
    }

    HeatReading {
        state: classify(window),
        novel_road_detected: false,
    }
}

/// Threshold ladder over one window of walks (no anomaly rule).
fn classify(walks: &[WalkRecord]) -> HeatState {
    if walks.len() < 2 {
        return HeatState::Cold;
    }

    let n = f64::from(u32::try_from(walks.len()).expect("window fits u32"));

    // Link-quality population variance.
    let mean_lq: f64 = walks.iter().map(|w| f64::from(w.link_quality)).sum::<f64>() / n;
    let lq_var: f64 = walks
        .iter()
        .map(|w| {
            let d = f64::from(w.link_quality) - mean_lq;
            d * d
        })
        .sum::<f64>()
        / n;

    // Fraction of arrivals on the default road.
    let local_frac: f64 = f64::from(
        u32::try_from(walks.iter().filter(|w| w.road == LOCAL_ROAD).count())
            .expect("window fits u32"),
    ) / n;

    // Write cadence: coefficient of variation of inter-arrival gaps.
    let gaps: Vec<f64> = walks
        .windows(2)
        .map(|pair| f64::from(u32::try_from(pair[1].ts.saturating_sub(pair[0].ts)).expect("gap fits u32")))
        .collect();
    let cadence_cv = |threshold: f64, below: bool| -> bool {
        if gaps.len() < MIN_GAPS_FOR_CADENCE {
            return false; // cadence not judgeable
        }
        let m = f64::from(u32::try_from(gaps.len()).expect("gaps fit u32"));
        let mean = gaps.iter().sum::<f64>() / m;
        if mean == 0.0 {
            return below; // all-zero gaps: perfectly metronomic
        }
        let var = gaps.iter().map(|g| (g - mean) * (g - mean)).sum::<f64>() / m;
        let cv = var.sqrt() / mean;
        if below {
            cv <= threshold
        } else {
            cv > threshold
        }
    };

    let warm = lq_var >= WARM_LQ_VARIANCE_MIN
        && local_frac <= WARM_LOCAL_FRACTION_MAX
        && (gaps.len() < MIN_GAPS_FOR_CADENCE || cadence_cv(WARM_CADENCE_CV_MIN, false));
    let cold = lq_var <= COLD_LQ_VARIANCE_MAX
        && local_frac >= COLD_LOCAL_FRACTION_MIN
        && (gaps.len() < MIN_GAPS_FOR_CADENCE || cadence_cv(COLD_CADENCE_CV_MAX, true));

    if warm {
        HeatState::Warm
    } else if cold {
        HeatState::Cold
    } else {
        HeatState::Cooling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(ts: u64, road: &str, lq: f32) -> WalkRecord {
        WalkRecord {
            ts,
            road: road.into(),
            link_quality: lq,
            arrival_meta: None,
        }
    }

    // ── Chain tests ──────────────────────────────────────────────────────

    #[test]
    fn chain_is_hash_chained_and_verifies() {
        let mut log = WalkLog::new();
        assert_eq!(log.head(), GENESIS_CHAIN);
        assert!(log.verify());

        let h1 = log.append(rec(0, "local", 0.5));
        let h2 = log.append(rec(60, "local", 0.5));
        assert_ne!(h1, h2);
        assert_ne!(h1, GENESIS_CHAIN);
        assert_eq!(log.head(), h2);
        assert_eq!(log.records().len(), 2);
        assert!(log.verify());
    }

    #[test]
    fn chain_detects_a_tampered_record() {
        let mut log = WalkLog::new();
        log.append(rec(0, "local", 0.5));
        log.append(rec(60, "h-road-0", 0.8));
        log.append(rec(120, "local", 0.5));
        assert!(log.verify());

        // Forge the road of the middle record.
        log.records[1].road = "h-road-9".into();
        assert!(!log.verify(), "a rewritten walk must break the chain");
    }

    #[test]
    fn chain_detects_a_forged_link() {
        let mut log = WalkLog::new();
        log.append(rec(0, "local", 0.5));
        log.append(rec(60, "local", 0.5));
        assert!(log.verify());

        log.chain[1][0] ^= 0xFF;
        assert!(!log.verify(), "a forged link must break the chain");
    }

    #[test]
    fn chain_detects_a_dropped_entry() {
        let mut log = WalkLog::new();
        log.append(rec(0, "local", 0.5));
        log.append(rec(60, "local", 0.5));
        log.append(rec(120, "local", 0.5));
        assert!(log.verify());

        // Drop the middle record but keep its link.
        let rec1 = log.records.remove(1);
        assert!(rec1.ts == 60);
        assert!(!log.verify(), "a dropped walk must break the chain");
    }

    // ── Heat fixtures ────────────────────────────────────────────────────

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

    /// Cooling: roads defaulted to local, link quality flatlining but not dead.
    fn cooling_walks() -> Vec<WalkRecord> {
        let lqs = [0.50, 0.52, 0.48, 0.51, 0.49, 0.52, 0.48, 0.50];
        lqs.iter()
            .enumerate()
            .map(|(i, lq)| rec(u64::try_from(i).expect("index fits u64") * 70, "local", *lq))
            .collect()
    }

    /// Cold: flatline, commission standing, one write a minute.
    fn cold_walks() -> Vec<WalkRecord> {
        (0..8_u64).map(|i| rec(i * 60, "local", 0.50)).collect()
    }

    #[test]
    fn warm_room_reads_warm() {
        let walks = warm_walks();
        let reading = heat(&walks, usize::MAX);
        assert_eq!(reading, HeatReading { state: HeatState::Warm, novel_road_detected: false });
    }

    #[test]
    fn cooling_room_reads_cooling() {
        let walks = cooling_walks();
        let reading = heat(&walks, usize::MAX);
        assert_eq!(reading, HeatReading { state: HeatState::Cooling, novel_road_detected: false });
    }

    #[test]
    fn cold_room_reads_cold() {
        let walks = cold_walks();
        let reading = heat(&walks, usize::MAX);
        assert_eq!(reading, HeatReading { state: HeatState::Cold, novel_road_detected: false });
    }

    #[test]
    fn cold_cell_anomaly_holds_without_verdict() {
        // Sustained cold, then the 06:11 write: one novel road with a spike.
        let mut walks = cold_walks();
        let last_ts = walks.last().unwrap().ts + 60;
        walks.push(rec(last_ts, "h-road-9", 0.95));

        // The prefix alone reads cold.
        let prefix_reading = heat(&walks[..8], usize::MAX);
        assert_eq!(prefix_reading.state, HeatState::Cold);

        // The full window must NOT warm — hold without verdict, signal Ensigns.
        let reading = heat(&walks, usize::MAX);
        assert_eq!(
            reading,
            HeatReading { state: HeatState::Cold, novel_road_detected: true }
        );
    }

    #[test]
    fn known_road_spike_is_not_the_anomaly() {
        // Same spike, but by a road already walked: informative, not novel.
        let mut walks = cold_walks();
        walks[0] = rec(0, "h-road-9", 0.50);
        walks.push(rec(walks.last().unwrap().ts + 60, "h-road-9", 0.95));

        let reading = heat(&walks, usize::MAX);
        assert!(!reading.novel_road_detected);
    }

    #[test]
    fn empty_window_reads_cold() {
        assert_eq!(
            heat(&[], usize::MAX),
            HeatReading { state: HeatState::Cold, novel_road_detected: false }
        );
        assert_eq!(
            heat(&cold_walks(), 0),
            HeatReading { state: HeatState::Cold, novel_road_detected: false }
        );
    }

    #[test]
    fn window_reads_only_the_recent_tail() {
        // Cold sediment, then a warm tail: a small window sees only the tail.
        let mut walks = cold_walks();
        walks.extend(warm_walks());
        let reading = heat(&walks, 8);
        assert_eq!(reading.state, HeatState::Warm);
    }
}
