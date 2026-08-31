// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The sandbox process is a room (RFC 0004 runtime wiring).
//!
//! [`super::run_sandbox`] calls [`spawn_residency`] once per supervised
//! sandbox at spawn: the room is grown-or-loaded from the sandbox id
//! (seed derivation and the charter gap are documented in
//! `openshell_construct::runtime`), and a dedicated supervision loop
//! feeds it one walk per tick.
//!
//! Honest limits of the v1 walk:
//!
//! - **Road** names the transport the supervisor itself uses —
//!   `"gateway"` when a gateway endpoint is configured (the supervisor
//!   session's gRPC channel), `"local"` otherwise. It is constant for
//!   the process's life.
//! - **Link quality is neutral (0.5).** No RTT or link-quality signal is
//!   plumbed to this loop today — the gap is real and the walk says so
//!   by carrying no variance. A room ticking on these walks alone reads
//!   cold (local) or cooling (gateway): the honest temperature of
//!   synthetic telemetry, per the RFC's "subtext is observed, not
//!   declared".
//!
//! Configuration (environment):
//!
//! - `OPENSHELL_ROOM_TICK_SECS` — supervision tick interval (default
//!   60, clamped to ≥ 5).
//! - `OPENSHELL_ROOM_GRAVITY` — the gravity dial every call bends
//!   through (default 0.5; see `openshell_construct::gravity`).
//! - `OPENSHELL_ROOM_DIR` — where room files persist (default
//!   `/var/log/openshell-rooms`; `"none"` disables persistence — the
//!   room lives in memory and its walks restart with the process).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use openshell_construct::{RoomResidency, WalkRecord};
use tokio::time::MissedTickBehavior;

/// Supervision tick interval, seconds.
const DEFAULT_TICK_SECS: u64 = 60;
/// The gravity dial (see `openshell_construct::gravity`).
const DEFAULT_GRAVITY: f64 = 0.5;
/// Where room files persist.
const DEFAULT_DIR: &str = "/var/log/openshell-rooms";
/// A tick faster than this would heat the room by metronome alone.
const MIN_TICK_SECS: u64 = 5;

/// The road name for arrivals over the supervisor's gateway channel.
const GATEWAY_ROAD: &str = "gateway";
/// The road name for standalone arrivals that never left the room.
const LOCAL_ROAD: &str = "local";

/// Neutral link quality — the documented v1 gap.
const NEUTRAL_LINK_QUALITY: f32 = 0.5;

struct ResidencyConfig {
    tick_secs: u64,
    gravity: f64,
    dir: Option<PathBuf>,
}

/// Read the residency configuration from the environment.
fn residency_config() -> ResidencyConfig {
    ResidencyConfig {
        tick_secs: parse_tick(std::env::var("OPENSHELL_ROOM_TICK_SECS").ok().as_deref()),
        gravity: parse_gravity(std::env::var("OPENSHELL_ROOM_GRAVITY").ok().as_deref()),
        dir: parse_dir(std::env::var("OPENSHELL_ROOM_DIR").ok().as_deref()),
    }
}

/// Tick interval: default 60s, clamped to ≥ 5s (garbage parses as the
/// default — a mis-set knob must not spin the loop or panic the interval).
fn parse_tick(raw: Option<&str>) -> u64 {
    raw.and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_TICK_SECS)
        .max(MIN_TICK_SECS)
}

/// Gravity: default 0.5; NaN and non-finite values parse as the default
/// (`modulate` clamps the rest to [0, 1]).
fn parse_gravity(raw: Option<&str>) -> f64 {
    raw.and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|g| g.is_finite())
        .unwrap_or(DEFAULT_GRAVITY)
}

/// Persist dir: unset → the default; `"none"` → in-memory rooms only.
fn parse_dir(raw: Option<&str>) -> Option<PathBuf> {
    match raw.map(str::trim) {
        None | Some("") => Some(PathBuf::from(DEFAULT_DIR)),
        Some("none") => None,
        Some(path) => Some(PathBuf::from(path)),
    }
}

/// The supervision tick's walk, at tick `ts`.
///
/// Road = the transport used (gateway channel vs local); link quality is
/// neutral — the v1 signal gap, stated in the module docs.
fn neutral_walk(ts: u64, gateway: bool) -> WalkRecord {
    WalkRecord {
        ts,
        road: if gateway { GATEWAY_ROAD } else { LOCAL_ROAD }.to_owned(),
        link_quality: NEUTRAL_LINK_QUALITY,
        arrival_meta: None,
    }
}

/// Wall-clock seconds for walk timestamps (rooms tick on observed time).
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Grow-or-load the sandbox's room and spawn its supervision loop.
///
/// Called from `run_sandbox` for supervised sandboxes (those carrying a
/// sandbox id). Returns the shared registry so the model-call path can
/// consult it (see `proxy::InferenceContext`), or `None` when residency
/// is unavailable (no id, or the room file refused to load — a tampered
/// chain disables residency rather than running over evidence).
pub fn spawn_residency(
    sandbox_id: &str,
    gateway_endpoint: Option<&str>,
) -> Option<Arc<RoomResidency>> {
    let config = residency_config();
    let residency = Arc::new(RoomResidency::with_persist_dir(
        config.gravity,
        config.dir.clone(),
    ));

    // What a prior does here: the keeper-facing log, keyed by urgency
    // (the elephant's nudge — correlate attention, never replace policy).
    residency.set_prior_sink(Arc::new(|room_id, prior| {
        if prior.urgency >= 0.8 {
            tracing::warn!(
                room = %room_id,
                urgency = prior.urgency,
                reason = ?prior.reason,
                "room attention prior — the keeper should look: {}",
                prior.detail
            );
        } else if prior.urgency > 0.0 {
            tracing::info!(
                room = %room_id,
                urgency = prior.urgency,
                reason = ?prior.reason,
                "room attention prior: {}",
                prior.detail
            );
        } else {
            tracing::debug!(room = %room_id, "room steady: {}", prior.detail);
        }
    }));

    let room_id = match residency.attach(sandbox_id, None, now_secs()) {
        Ok(id) => id,
        Err(error) => {
            tracing::warn!(
                sandbox_id = %sandbox_id,
                error = %error,
                "residency room refused to load — running without a room \
                 (a tampered walk chain disables residency rather than \
                 running over evidence)"
            );
            return None;
        }
    };

    tracing::info!(
        room = %room_id,
        sandbox_id = %sandbox_id,
        gravity = residency.gravity(),
        tick_secs = config.tick_secs,
        persist = ?config.dir,
        "residency room grown or loaded — the first tick is a temperature"
    );

    let tick_room = Arc::clone(&residency);
    let tick_room_id = room_id;
    let gateway = gateway_endpoint.is_some();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(config.tick_secs));
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            match tick_room.tick(&tick_room_id, neutral_walk(now_secs(), gateway)) {
                Some((reading, _prior)) => {
                    tracing::debug!(
                        room = %tick_room_id,
                        heat = reading.state.label(),
                        "room supervision tick"
                    );
                }
                None => {
                    // The room was unregistered — nothing left to tick.
                    break;
                }
            }
        }
    });

    Some(residency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_walk_names_the_transport_and_carries_the_gap() {
        let gateway = neutral_walk(1_000, true);
        assert_eq!(gateway.road, "gateway");
        assert!((gateway.link_quality - NEUTRAL_LINK_QUALITY).abs() < f32::EPSILON);
        assert_eq!(gateway.ts, 1_000);
        assert!(gateway.arrival_meta.is_none());

        let local = neutral_walk(1_000, false);
        assert_eq!(local.road, "local");
        assert!((local.link_quality - NEUTRAL_LINK_QUALITY).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_parses_default_clamp_and_garbage() {
        assert_eq!(parse_tick(None), 60);
        assert_eq!(parse_tick(Some("120")), 120);
        assert_eq!(parse_tick(Some("1")), 5, "a too-fast tick clamps to 5s");
        assert_eq!(parse_tick(Some("0")), 5);
        assert_eq!(parse_tick(Some("soon")), 60, "garbage parses as default");
        assert_eq!(parse_tick(Some("  90  ")), 90);
    }

    #[test]
    fn gravity_parses_default_and_filters_non_finite() {
        assert!((parse_gravity(None) - 0.5).abs() < 1e-9);
        assert!((parse_gravity(Some("0.9")) - 0.9).abs() < 1e-9);
        assert!(
            (parse_gravity(Some("2")) - 2.0).abs() < 1e-9,
            "modulate clamps"
        );
        assert!(
            (parse_gravity(Some("NaN")) - 0.5).abs() < 1e-9,
            "NaN parses as the default"
        );
        assert!((parse_gravity(Some("junk")) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn dir_parses_default_none_and_override() {
        assert_eq!(
            parse_dir(None),
            Some(PathBuf::from("/var/log/openshell-rooms"))
        );
        assert_eq!(
            parse_dir(Some("")),
            Some(PathBuf::from("/var/log/openshell-rooms"))
        );
        assert_eq!(parse_dir(Some("none")), None);
        assert_eq!(
            parse_dir(Some("/tmp/rooms")),
            Some(PathBuf::from("/tmp/rooms"))
        );
    }
}
