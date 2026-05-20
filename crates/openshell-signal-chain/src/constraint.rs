// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Constraint Spline — the fare curvatures of truth, bounds that bend.
//!
//! The core insight: constraint bounds are not binary. They have curvature.
//! A value at the edge of a hard constraint (lo=0, hi=127, value=126) carries
//! different information than the same value at the center (value=63). The
//! curvature encodes how close you are to the boundary and what the cost of
//! crossing it is.
//!
//! The signal chain's dial (0.0 hard → 1.0 soft) controls how those curvatures
//! are interpreted:
//! - **0.0 (hard):** curvature is existential — crossing the boundary is failure
//! - **0.5 (balanced):** curvature is informational — crossing the boundary is cost
//! - **1.0 (soft):** curvature is optional — crossing the boundary is guidance
//!
//! This module provides the bridge between FLUX constraint engines and the
//! signal chain's dial-controlled reasoning.

use serde::{Deserialize, Serialize};

use crate::{Dial, SignalChainError};

/// A constraint with curvature — bounds that know their own shape.
///
/// Unlike a raw FLUX `Constraint` (lo, hi, name), a `SplineConstraint`
/// carries the "fare" of approaching each boundary. The curvature encodes
/// how expensive it is to be near the edge vs. near the center.
///
/// Example: temperature in Celsius
/// - Hard (0.0): -50°C to 85°C. Crossing either boundary = aviation failure.
/// - Soft (1.0): same bounds, but approaching 85°C means "consider landing soon."
///
/// The bounds don't change. The interpretation of proximity changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplineConstraint {
    /// Lower bound (inclusive in hard mode)
    pub lo: i32,
    /// Upper bound (inclusive in hard mode)
    pub hi: i32,
    /// Constraint name
    pub name: String,
    /// Curvature: how costly is proximity to the lo-boundary?
    /// Higher = more costly to approach the edge.
    /// Units are dial-specific: severity points at hard, advisory weight at soft.
    pub lo_curvature: f64,
    /// Curvature: how costly is proximity to the hi-boundary?
    pub hi_curvature: f64,
    /// Neutral point: where in the range is this constraint "relaxed"?
    /// Default is midpoint. A constraint with neutral=0.75 would have its
    /// softest interpretation at the top of the range.
    pub neutral: f64,
}

impl SplineConstraint {
    /// Create a new spline constraint.
    ///
    /// # Arguments
    ///
    /// * `lo` — Lower bound
    /// * `hi` — Upper bound (must be >= lo)
    /// * `name` — Human-readable name
    /// * `lo_curvature` — Cost of approaching lo-boundary (suggest 1.0 for symmetric)
    /// * `hi_curvature` — Cost of approaching hi-boundary
    /// * `neutral` — Normalized position of neutral zone [0.0, 1.0]
    ///
    /// # Panics
    ///
    /// Panics if `lo > hi` or if `neutral` is outside [0.0, 1.0].
    pub fn new(
        lo: i32,
        hi: i32,
        name: &str,
        lo_curvature: f64,
        hi_curvature: f64,
        neutral: f64,
    ) -> Self {
        assert!(lo <= hi, "lo must not exceed hi");
        assert!(
            (0.0..=1.0).contains(&neutral),
            "neutral must be in [0.0, 1.0]"
        );
        assert!(
            lo_curvature >= 0.0 && hi_curvature >= 0.0,
            "curvatures must be non-negative"
        );
        SplineConstraint {
            lo,
            hi,
            name: name.to_string(),
            lo_curvature,
            hi_curvature,
            neutral,
        }
    }

    /// Create a symmetric spline constraint (equal curvature both sides).
    pub fn symmetric(lo: i32, hi: i32, name: &str, curvature: f64) -> Self {
        Self::new(lo, hi, name, curvature, curvature, 0.5)
    }

    /// Check a raw value and return a curvature distance.
    ///
    /// Returns `Ok(distance)` where distance is in [0.0, 1.0+]:
    /// - 0.0 means the value is at the neutral point (safest position)
    /// - 1.0 means the value is at a boundary
    /// - >1.0 means the value has crossed the boundary
    ///
    /// The dial position determines what crossing means:
    /// - hard (0.0): crossing = failure (return Err)
    /// - balanced (0.5): crossing = cost (return positive distance)
    /// - soft (1.0): crossing = advisory (return positive distance, no error)
    pub fn curvature_distance(&self, value: i32) -> f64 {
        let range = (self.hi - self.lo) as f64;
        if range <= 0.0 {
            return 0.0;
        }

        let normalized = (value - self.lo) as f64 / range;

        // Distance from neutral point
        let dist = if normalized < self.neutral {
            self.neutral - normalized
        } else {
            normalized - self.neutral
        };

        // Normalize: distance from neutral to nearest boundary
        let max_dist = self.neutral.max(1.0 - self.neutral);
        if max_dist <= 0.0 {
            0.0
        } else {
            dist / max_dist
        }
    }

    /// Evaluate this constraint at a given dial position.
    ///
    /// # Arguments
    ///
    /// * `value` — The value to check
    /// * `dial` — The signal chain dial position
    ///
    /// # Returns
    ///
    /// `Ok(curvature)` if the constraint is satisfied (or softly satisfied),
    /// `Err((severity, curvature))` if the constraint is violated.
    /// The severity is dial-dependent: at hard (0.0), severity maps to
    /// aviation/failure levels. At soft (1.0), severity maps to advisory weight.
    pub fn evaluate(&self, value: i32, dial: Dial) -> Result<f64, ConstraintViolation> {
        let curvature = self.curvature_distance(value);

        // At hard (0.0): boundary crossing is failure
        if value < self.lo || value > self.hi {
            let severity = if dial.position < 0.25 {
                // Hard enforcement: crossing is critical failure
                ViolationSeverity::Critical
            } else if dial.position < 0.5 {
                // Transition: crossing is warning
                ViolationSeverity::Warning
            } else {
                // Soft: crossing is advisory only
                ViolationSeverity::Advisory
            };
            return Err(ConstraintViolation {
                constraint: self.name.clone(),
                value,
                curvature,
                severity,
            });
        }

        // Within bounds: return the curvature distance
        Ok(curvature)
    }
}

/// Severity level for constraint violations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Advisory: soft constraint, suggestion
    Advisory = 0,
    /// Warning: balanced constraint, cost incurred
    Warning = 1,
    /// Critical: hard constraint, failure
    Critical = 2,
}

impl ViolationSeverity {
    /// Convert to a dial-dependent advisory weight.
    ///
    /// At hard (0.0): maps to severity (Critical=2 → high cost)
    /// At soft (1.0): maps to advisory (Advisory=0 → low cost)
    pub fn dial_weight(&self, dial_position: f64) -> f64 {
        let base = match self {
            ViolationSeverity::Advisory => 0.1,
            ViolationSeverity::Warning => 0.5,
            ViolationSeverity::Critical => 1.0,
        };
        // At hard end, use raw severity. At soft end, dampen.
        base * (1.0 - dial_position * 0.5)
    }
}

/// A constraint violation with full context.
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub constraint: String,
    pub value: i32,
    pub curvature: f64,
    pub severity: ViolationSeverity,
}

/// A spline evaluation result — all constraints checked at a given dial.
#[derive(Debug, Clone)]
pub struct SplineResult {
    /// Total curvature cost (sum of all constraint distances)
    pub total_curvature: f64,
    /// Number of constraints evaluated
    pub constraint_count: usize,
    /// Violations encountered (if any)
    pub violations: Vec<ConstraintViolation>,
    /// Whether all hard constraints passed
    pub is_hard_pass: bool,
}

impl SplineResult {
    /// Create a clean result (no violations).
    pub fn clean(total_curvature: f64, constraint_count: usize) -> Self {
        SplineResult {
            total_curvature,
            constraint_count,
            violations: Vec::new(),
            is_hard_pass: true,
        }
    }

    /// Whether this result passes the given dial's hard threshold.
    pub fn passes(&self, dial: Dial) -> bool {
        if dial.position < 0.25 {
            // Hard mode: no violations allowed
            self.violations.is_empty()
        } else if dial.position < 0.75 {
            // Balanced mode: warnings only
            !self.violations.iter().any(|v| v.severity == ViolationSeverity::Critical)
        } else {
            // Soft mode: anything goes
            true
        }
    }

    /// Weighted cost — curvature adjusted by dial position and violation severity.
    pub fn weighted_cost(&self, dial: Dial) -> f64 {
        let mut cost = self.total_curvature;

        // Add violation costs
        for v in &self.violations {
            cost += v.curvature * v.severity.dial_weight(dial.position);
        }

        // At hard end, multiply cost (hard constraints are expensive)
        if dial.position < 0.25 {
            cost *= 2.0;
        }

        cost
    }
}

/// Evaluate a set of spline constraints at a given dial position.
pub fn evaluate_spline(
    constraints: &[SplineConstraint],
    values: &[i32],
    dial: Dial,
) -> Result<SplineResult, Vec<ConstraintViolation>> {
    assert_eq!(
        constraints.len(),
        values.len(),
        "constraints and values must have same length"
    );

    let mut total_curvature = 0.0;
    let mut violations = Vec::new();

    for (c, &v) in constraints.iter().zip(values.iter()) {
        match c.evaluate(v, dial) {
            Ok(curvature) => total_curvature += curvature,
            Err(violation) => violations.push(violation),
        }
    }

    let result = SplineResult {
        total_curvature,
        constraint_count: constraints.len(),
        violations: violations.clone(),
        is_hard_pass: violations.is_empty(),
    };

    if violations.is_empty() || result.passes(dial) {
        Ok(result)
    } else {
        Err(violations)
    }
}

/// Maritime preset spline constraints — the bounds that define safe passage.
///
/// These are the anchor points of the fare curvatures. Each constraint
/// encodes both the boundary and the cost of approaching it.
pub fn maritime_spline() -> Vec<SplineConstraint> {
    vec![
        // Sea temperature: -40°C to 60°C
        // Curvature is higher at cold end (hypothermia risk) than warm end
        SplineConstraint::new(-40, 60, "sea_temp_celsius", 1.2, 1.0, 0.5),
        // Wave height: 0 to 127 decimeters
        // Curvature is symmetric but high — even moderate waves are costly
        SplineConstraint::symmetric(0, 127, "wave_height_dm", 1.5),
        // Wind speed: 0 to 120 knots
        // Low curvature at low end (calm is normal), higher at high end (storm)
        SplineConstraint::new(0, 120, "wind_speed_knots", 0.5, 1.3, 0.3),
        // Pressure deviation: -50 to 50 hPa from nominal
        // Symmetric around nominal — deviation in either direction is costly
        SplineConstraint::symmetric(-50, 50, "pressure_deviation_hpa", 1.1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spline_basic() {
        let c = SplineConstraint::symmetric(-50, 50, "test", 1.0);

        // At neutral (0): should have 0 curvature distance (center of range)
        // But curvature_distance is distance from neutral, and neutral=0.5
        // so at value=0 (which is normalized=0.5), distance from neutral=0
        assert!((c.curvature_distance(0)).abs() < 0.01);

        // At boundary: should have curvature distance > 1.0 (crossed)
        assert!(c.curvature_distance(-51) > 1.0);
        assert!(c.curvature_distance(51) > 1.0);
    }

    #[test]
    fn test_spline_evaluate_hard_pass() {
        let c = SplineConstraint::symmetric(0, 100, "test", 1.0);
        let dial = Dial::hard(); // position = 0.0

        let result = c.evaluate(50, dial);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.0).abs() < 0.01); // at neutral
    }

    #[test]
    fn test_spline_evaluate_hard_fail() {
        let c = SplineConstraint::symmetric(0, 100, "test", 1.0);
        let dial = Dial::hard(); // position = 0.0

        let result = c.evaluate(-1, dial);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_spline_evaluate_soft_advisory() {
        let c = SplineConstraint::symmetric(0, 100, "test", 1.0);
        let dial = Dial::new(0.9); // soft

        let result = c.evaluate(-1, dial);
        assert!(result.is_err());
        let violation = result.unwrap_err();
        assert_eq!(violation.severity, ViolationSeverity::Advisory);
    }

    #[test]
    fn test_maritime_spline() {
        let constraints = maritime_spline();
        assert_eq!(constraints.len(), 4);

        // Wave height should be most expensive near boundaries
        let wave = &constraints[1]; // wave_height_dm
        assert_eq!(wave.lo_curvature, 1.5);
        assert_eq!(wave.hi_curvature, 1.5);
    }

    #[test]
    fn test_evaluate_spline() {
        let constraints = maritime_spline();
        let values = &[20, 60, 30, -10]; // normal values
        let dial = Dial::hard();

        let result = evaluate_spline(&constraints, values, dial).unwrap();
        assert!(result.is_hard_pass);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_evaluate_spline_violation() {
        let constraints = maritime_spline();
        let values = &[20, 200, 30, -10]; // wave too high
        let dial = Dial::hard();

        let result = evaluate_spline(&constraints, values, dial);
        assert!(result.is_err());
    }
}
