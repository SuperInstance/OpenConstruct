// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! FluxVmBridge — connects signal chain to FLUX constraint engine.
//!
//! The bridge:
//! 1. Loads maritime spline constraints (wave, temp, wind, pressure)
//! 2. Checks values against constraint bounds
//! 3. Maps curvature distance to ViolationSeverity
//! 4. Converts results to snaps/inferences based on dial position
//!
//! # Usage
//!
//! ```
//! use openshell_signal_chain::{Dial, FluxVmBridge};
//!
//! let mut bridge = FluxVmBridge::new();
//! bridge.load_maritime();
//!
//! // Check a reading
//! let result = bridge.check("wave_height_dm", 35).unwrap();
//! assert!(result.passed); // 35dm is within [0, 127] bounds
//!
//! // Surface at hard dial: only verified snaps (need check_all first)
//! bridge.check_all(&[("wave_height_dm", 35_i32)]);
//! let (snaps, inferences) = bridge.surface_at(Dial::hard());
//! assert_eq!(snaps.len(), 1); // passed check surfaces as snap
//!
//! // Surface at soft dial: all pass, violations become advisories
//! let (snaps, inferences) = bridge.surface_at(Dial::soft());
//! ```

use serde::{Deserialize, Serialize};
use crate::{Dial, SplineConstraint, ViolationSeverity};

/// Result of checking a value against a constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmCheckResult {
    pub name: String,
    pub value: i32,
    pub passed: bool,
    pub severity: ViolationSeverity,
    /// Curvature distance — 0.0 at neutral, 1.0 at boundary, >1.0 outside
    pub curvature: f64,
    /// Hard confidence: at dial 0.0, what's the confidence?
    /// Critical violation → 0.0 (hard failure)
    /// Warning → 0.3
    /// Advisory → 0.7
    /// Passed → 1.0
    pub hard_confidence: f64,
    /// Soft confidence: at dial 1.0, everything passes as advisory
    pub soft_confidence: f64,
}

impl VmCheckResult {
    /// Get confidence at a specific dial position.
    pub fn confidence_at(&self, dial: Dial) -> f64 {
        if self.passed {
            return 1.0;
        }
        // At hard (position=0): severity directly determines confidence
        // At soft (position=1): all violations are advisories (1.0)
        match self.severity {
            ViolationSeverity::Critical => {
                if dial.position > 0.75 { 0.5 } else { 0.0 }
            }
            ViolationSeverity::Warning => {
                if dial.position > 0.75 { 0.8 } else { 0.3 }
            }
            ViolationSeverity::Advisory => {
                if dial.position > 0.75 { 1.0 } else { 0.7 }
            }
        }
    }

    /// Convert to a JSON value for the signal chain.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "constraint": self.name,
            "value": self.value,
            "passed": self.passed,
            "severity": self.severity,
            "curvature": self.curvature,
            "hard_confidence": self.hard_confidence,
            "soft_confidence": self.soft_confidence,
        })
    }
}

/// FluxVm bridge — loads constraints, checks values, surfaces results at dial.
///
/// The bridge is stateless per-check: call `check(name, value)` to evaluate
/// a single reading. Call `check_all(readings)` to evaluate a batch.
/// Call `surface_at(dial)` to get snaps + inferences for the current results.
#[derive(Debug, Clone)]
pub struct FluxVmBridge {
    constraints: Vec<SplineConstraint>,
    results: Vec<VmCheckResult>,
    last_error: Option<String>,
}

impl FluxVmBridge {
    /// Create a new bridge.
    pub fn new() -> Self {
        FluxVmBridge {
            constraints: Vec::new(),
            results: Vec::new(),
            last_error: None,
        }
    }

    /// Load the maritime spline preset (wave_height_dm, sea_temp_celsius, wind_speed_knots, pressure_deviation_hpa).
    pub fn load_maritime(&mut self) {
        use crate::maritime_spline;
        self.constraints = maritime_spline();
    }

    /// Load a custom constraint.
    pub fn add_constraint(&mut self, constraint: SplineConstraint) {
        self.constraints.push(constraint);
    }

    /// Number of constraints currently loaded.
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    /// List all constraint names.
    pub fn constraint_names(&self) -> Vec<&str> {
        self.constraints.iter().map(|c| c.name.as_str()).collect()
    }

    /// Check a value against a named constraint.
    ///
    /// Returns `Err` if the constraint name doesn't exist.
    /// Returns `Ok(VmCheckResult)` with passed/severity/curvature populated.
    pub fn check(&mut self, name: &str, value: i32) -> Result<VmCheckResult, String> {
        let constraint = match self.constraints.iter().find(|c| c.name == name) {
            Some(c) => c,
            None => {
                let msg = format!("constraint '{}' not found in bridge", name);
                self.last_error = Some(msg.clone());
                return Err(msg);
            }
        };

        // bounds-based pass/fail: within [lo, hi] = pass, outside = fail
        // severity maps raw |norm - 0.5| distance to Critical/Warning/Advisory
        let norm = (value - constraint.lo) as f64 / (constraint.hi - constraint.lo) as f64;
        let dist = (norm - 0.5).abs();

        let (passed, severity) = if value >= constraint.lo && value <= constraint.hi {
            (true, ViolationSeverity::Advisory) // within hard bounds = pass
        } else if dist > 0.25 {
            (false, ViolationSeverity::Critical) // >25% outside neutral = critical
        } else if dist > 0.15 {
            (false, ViolationSeverity::Warning) // >15% outside neutral = warning
        } else {
            (false, ViolationSeverity::Advisory) // outside but near = advisory
        };

        let curvature = constraint.curvature_distance(value);

        let result = VmCheckResult {
            name: name.to_string(),
            value,
            passed,
            severity,
            curvature,
            hard_confidence: if passed { 1.0 } else {
                match severity {
                    ViolationSeverity::Critical => 0.0,
                    ViolationSeverity::Warning => 0.3,
                    ViolationSeverity::Advisory => 0.7,
                }
            },
            soft_confidence: 1.0,
        };

        Ok(result)
    }

    /// Check multiple readings at once.
    ///
    /// Returns all results (one per reading). Errors are logged but don't stop the batch.
    pub fn check_all(&mut self, readings: &[(impl AsRef<str>, i32)]) -> &mut [VmCheckResult] {
        self.results.clear();
        for (name, value) in readings {
            match self.check(name.as_ref(), *value) {
                Ok(r) => self.results.push(r),
                Err(e) => {
                    self.last_error = Some(e);
                    // Push a placeholder error result
                    self.results.push(VmCheckResult {
                        name: name.as_ref().to_string(),
                        value: *value,
                        passed: false,
                        severity: ViolationSeverity::Critical,
                        curvature: f64::INFINITY,
                        hard_confidence: 0.0,
                        soft_confidence: 0.0,
                    });
                }
            }
        }
        &mut self.results
    }

    /// Get all results from the last check_all call.
    pub fn results(&self) -> &[VmCheckResult] {
        &self.results
    }

    /// Get only passed results.
    pub fn passed(&self) -> Vec<&VmCheckResult> {
        self.results.iter().filter(|r| r.passed).collect()
    }

    /// Get only violated results.
    pub fn violations(&self) -> Vec<&VmCheckResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    /// Get critical violations.
    pub fn critical_violations(&self) -> Vec<&VmCheckResult> {
        self.results.iter()
            .filter(|r| !r.passed && r.severity == ViolationSeverity::Critical)
            .collect()
    }

    /// Surface results at a specific dial position.
    ///
    /// Returns (snaps, inferences):
    /// - **Snaps**: passed checks + violations at soft dial (confidence 1.0)
    /// - **Inferences**: violations at hard/medium dial (confidence from severity)
    ///
    /// At hard (0.0): only passed checks become snaps.
    /// At soft (1.0): all checks surface, violations become low-confidence inferences.
    pub fn surface_at(&self, dial: Dial) -> (Vec<serde_json::Value>, Vec<(serde_json::Value, f64)>) {
        let mut snaps = Vec::new();
        let mut inferences = Vec::new();

        for result in &self.results {
            if result.passed {
                snaps.push(result.to_json());
            } else {
                let confidence = result.confidence_at(dial);
                if confidence >= 0.9 {
                    snaps.push(result.to_json());
                } else {
                    inferences.push((result.to_json(), confidence));
                }
            }
        }

        (snaps, inferences)
    }

    /// Last error message (from last check_all with unknown constraint).
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Print a human-readable constraint report.
    pub fn print_report(&self) {
        println!("  FluxVmBridge — {} constraints loaded:", self.constraint_count());
        for c in &self.constraints {
            println!("    {:25} range=[{:5}, {:5}]  neutral={:.1}",
                c.name, c.lo, c.hi, c.neutral);
        }
        if self.results.is_empty() {
            println!("    (no results — run check_all first)");
        } else {
            let passed = self.results.iter().filter(|r| r.passed).count();
            let violated = self.results.len() - passed;
            println!("  Results: {} passed, {} violated", passed, violated);
            for r in &self.results {
                let status = if r.passed { "✓ pass" } else { "✗ violation" };
                let sev = if r.passed { "".to_string() } else {
                    format!("{:?}", r.severity)
                };
                println!("    {:25} = {:5}  curv={:.3}  {} {}",
                    r.name, r.value, r.curvature, status, sev);
            }
        }
    }
}

impl Default for FluxVmBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_maritime() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();
        assert_eq!(bridge.constraint_count(), 4);
        let names: Vec<&str> = bridge.constraint_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"wave_height_dm"));
        assert!(names.contains(&"sea_temp_celsius"));
        assert!(names.contains(&"wind_speed_knots"));
        assert!(names.contains(&"pressure_deviation_hpa"));
    }

    #[test]
    fn test_check_pass() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();
        let r = bridge.check("wave_height_dm", 35).unwrap();
        assert!(r.passed);
        assert!(r.curvature < 1.0);
        assert_eq!(r.hard_confidence, 1.0);
    }

    #[test]
    fn test_check_violation_critical() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();
        let r = bridge.check("wave_height_dm", 130).unwrap(); // > 127 max → critical
        assert!(!r.passed);
        assert!(r.curvature > 1.0);
        assert_eq!(r.severity, ViolationSeverity::Critical);
        assert_eq!(r.hard_confidence, 0.0);
    }

    #[test]
    fn test_confidence_at_hard_vs_soft() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();
        let r = bridge.check("wave_height_dm", 130).unwrap(); // critical

        let hard = Dial::hard();
        let soft = Dial::soft();

        // At hard: critical = 0.0 confidence
        assert_eq!(r.confidence_at(hard), 0.0);
        // At soft: critical → 0.5
        assert_eq!(r.confidence_at(soft), 0.5);
    }

    #[test]
    fn test_check_unknown_constraint() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();
        let result = bridge.check("nonexistent", 42);
        assert!(result.is_err());
        assert!(bridge.last_error().is_some());
    }

    #[test]
    fn test_check_all_batch() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();

        let readings = vec![
            ("wave_height_dm", 35_i32),
            ("sea_temp_celsius", 12_i32),
            ("wind_speed_knots", 22_i32),
            ("pressure_deviation_hpa", -8_i32),
        ];

        let results = bridge.check_all(&readings);
        assert_eq!(results.len(), 4);
        assert_eq!(bridge.passed().len(), 4);
        assert_eq!(bridge.violations().len(), 0);
    }

    #[test]
    fn test_surface_at_hard_all_pass() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();

        bridge.check_all(&[
            ("wave_height_dm", 35_i32),
            ("sea_temp_celsius", 12_i32),
            ("wind_speed_knots", 22_i32),
            ("pressure_deviation_hpa", -8_i32),
        ]);

        let (snaps, inferences) = bridge.surface_at(Dial::hard());
        assert!(snaps.len() >= 4); // 4 readings, all pass → snaps at hard
        assert_eq!(inferences.len(), 0);
    }

    #[test]
    fn test_surface_at_soft_mixed() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();

        bridge.check_all(&[
            ("wave_height_dm", 35_i32),  // pass
            ("wave_height_dm", 130_i32), // violation (critical)
            ("sea_temp_celsius", 12_i32), // pass
        ]);

        let (snaps, inferences) = bridge.surface_at(Dial::soft());
        // At soft: passed checks → snaps, violations → inferences
        assert!(snaps.len() >= 2);
        assert!(inferences.len() >= 1);
    }

    #[test]
    fn test_critical_only() {
        let mut bridge = FluxVmBridge::new();
        bridge.load_maritime();

        bridge.check_all(&[
            ("wave_height_dm", 35_i32),
            ("wave_height_dm", 130_i32), // critical
            ("sea_temp_celsius", 12_i32),
        ]);

        let critical = bridge.critical_violations();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].name, "wave_height_dm");
        assert_eq!(critical[0].value, 130);
    }

    #[test]
    fn test_empty_bridge_error() {
        let mut bridge = FluxVmBridge::new();
        let result = bridge.check("wave_height_dm", 35);
        assert!(result.is_err());
    }
}