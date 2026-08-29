// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Heat → gravity modulation (RFC 0004, §2: "Heat modulates gravity").
//!
//! The gravity dial describes *how the room responds*; residency heat
//! describes *how the room has been living*. A warm room and a cold room
//! with identical gravity should not behave identically, because they are
//! not identically alive. [`modulate`] bends one gravity value through the
//! room's heat into concrete model parameters.
//!
//! The mapping table, verbatim (`g` = gravity clamped to `[0, 1]`;
//! temperature and max_tokens are linear in `g`; style is the heat's
//! voice and does not bend with gravity):
//!
//! | heat    | temperature (g=0 → g=1) | max_tokens (g=0 → g=1) | prompt_style |
//! |---------|-------------------------|------------------------|--------------|
//! | warm    | 1.20 → 0.40             | 4096 → 1024            | "expansive"  |
//! | cooling | 0.80 → 0.30             | 3072 →  768            | "measured"   |
//! | cold    | 0.40 → 0.20             | 2048 →  512            | "terse"      |
//!
//! Reading of the table: gravity is *pull* — as it rises, every room
//! focuses (temperature falls, the token budget tightens) along a lane
//! set by its heat. Cold rooms live in the tight lane from the start:
//! lower ceiling, smaller budget, terse style. Warm rooms get the wide
//! lane: higher ceiling, larger budget, expansive style.

use crate::residency::HeatState;

/// Model parameters after heat modulates the gravity dial.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelParams {
    /// Sampling temperature (always within the heat's lane; see the table).
    pub temperature: f64,
    /// Token budget for a response.
    pub max_tokens: u32,
    /// Prompt style: `"expansive"` (warm), `"measured"` (cooling),
    /// `"terse"` (cold).
    pub prompt_style: &'static str,
}

/// Bend `gravity` through the room's `heat` into model parameters.
///
/// Deterministic and total: gravity is clamped to `[0, 1]` (`±∞` included);
/// an undefined (NaN) gravity reads as weightless (`0.0`). Same inputs,
/// same outputs — a room's response is a function of its state, not of its
/// moment.
pub fn modulate(gravity: f64, heat: HeatState) -> ModelParams {
    let g = if gravity.is_nan() {
        0.0
    } else {
        gravity.clamp(0.0, 1.0)
    };

    // Lane per heat: (temperature hi → lo, max_tokens hi → lo, style),
    // exactly the mapping table in the module docs.
    let (temp_hi, temp_lo, tok_hi, tok_lo, style) = match heat {
        HeatState::Warm => (1.20, 0.40, 4096_u32, 1024_u32, "expansive"),
        HeatState::Cooling => (0.80, 0.30, 3072, 768, "measured"),
        HeatState::Cold => (0.40, 0.20, 2048, 512, "terse"),
    };

    ModelParams {
        temperature: temp_hi - (temp_hi - temp_lo) * g,
        max_tokens: (f64::from(tok_hi) - f64::from(tok_hi - tok_lo) * g).round() as u32,
        prompt_style: style,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn same_inputs_same_params() {
        for heat in [HeatState::Warm, HeatState::Cooling, HeatState::Cold] {
            for g in [0.0_f64, 0.25, 0.5, 0.75, 1.0] {
                assert_eq!(modulate(g, heat), modulate(g, heat));
            }
        }
    }

    #[test]
    fn clamped_at_extremes_and_total() {
        for heat in [HeatState::Warm, HeatState::Cooling, HeatState::Cold] {
            assert_eq!(modulate(-5.0, heat), modulate(0.0, heat));
            assert_eq!(modulate(7.5, heat), modulate(1.0, heat));
            assert_eq!(modulate(f64::NEG_INFINITY, heat), modulate(0.0, heat));
            assert_eq!(modulate(f64::INFINITY, heat), modulate(1.0, heat));
            // Undefined gravity reads as weightless — total, not panicking.
            assert_eq!(modulate(f64::NAN, heat), modulate(0.0, heat));
        }
    }

    #[test]
    fn cold_runs_colder_than_warm_at_every_gravity() {
        for g in [0.0_f64, 0.25, 0.5, 0.75, 1.0] {
            let warm = modulate(g, HeatState::Warm);
            let cooling = modulate(g, HeatState::Cooling);
            let cold = modulate(g, HeatState::Cold);
            assert!(
                cold.temperature < cooling.temperature,
                "cold < cooling at g={g}"
            );
            assert!(
                cooling.temperature < warm.temperature,
                "cooling < warm at g={g}"
            );
            assert!(cold.max_tokens < cooling.max_tokens);
            assert!(cooling.max_tokens < warm.max_tokens);
        }
    }

    #[test]
    fn gravity_pulls_every_lane_tighter() {
        for heat in [HeatState::Warm, HeatState::Cooling, HeatState::Cold] {
            let mut prev = modulate(0.0, heat);
            for g in [0.25_f64, 0.5, 0.75, 1.0] {
                let now = modulate(g, heat);
                assert!(now.temperature < prev.temperature, "temp falls as gravity rises");
                assert!(now.max_tokens <= prev.max_tokens, "budget tightens as gravity rises");
                prev = now;
            }
        }
    }

    #[test]
    fn table_is_exact() {
        // Endpoints and midpoints of the mapping table, verbatim.
        let warm0 = modulate(0.0, HeatState::Warm);
        assert!(close(warm0.temperature, 1.20));
        assert_eq!(warm0.max_tokens, 4096);
        assert_eq!(warm0.prompt_style, "expansive");

        let warm1 = modulate(1.0, HeatState::Warm);
        assert!(close(warm1.temperature, 0.40));
        assert_eq!(warm1.max_tokens, 1024);

        let cool0 = modulate(0.0, HeatState::Cooling);
        assert!(close(cool0.temperature, 0.80));
        assert_eq!(cool0.max_tokens, 3072);
        assert_eq!(cool0.prompt_style, "measured");

        let cool1 = modulate(1.0, HeatState::Cooling);
        assert!(close(cool1.temperature, 0.30));
        assert_eq!(cool1.max_tokens, 768);

        let cold0 = modulate(0.0, HeatState::Cold);
        assert!(close(cold0.temperature, 0.40));
        assert_eq!(cold0.max_tokens, 2048);
        assert_eq!(cold0.prompt_style, "terse");

        let cold1 = modulate(1.0, HeatState::Cold);
        assert!(close(cold1.temperature, 0.20));
        assert_eq!(cold1.max_tokens, 512);

        // Midpoints: linear interpolation.
        assert!(close(modulate(0.5, HeatState::Warm).temperature, 0.80));
        assert!(close(modulate(0.5, HeatState::Cooling).temperature, 0.55));
        assert!(close(modulate(0.5, HeatState::Cold).temperature, 0.30));
        assert_eq!(modulate(0.5, HeatState::Warm).max_tokens, 2560);
        assert_eq!(modulate(0.5, HeatState::Cooling).max_tokens, 1920);
        assert_eq!(modulate(0.5, HeatState::Cold).max_tokens, 1280);
    }
}
