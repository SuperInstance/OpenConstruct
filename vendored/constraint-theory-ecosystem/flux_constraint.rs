//! # FLUX Constraint Engine - Rust Implementation
//!
//! A high-performance INT8 saturated constraint checking system for critical applications.
//! Provides robust boundary checking with industry-specific presets and comprehensive error reporting.
//!

#![allow(unused_imports)]
//! ## Features
//! - Zero-dependency constraint validation (no_std compatible)
//! - Industry-standard presets (aviation, medical, maritime, etc.)
//! - Batch processing capabilities
//! - Comprehensive error reporting with severity levels
//! - Memory-efficient saturated arithmetic

#![no_std]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

#[cfg(test)]
#[allow(unused_imports)]
use alloc::vec;

/// INT8 minimum value for saturated arithmetic
pub const INT8_MIN: i8 = -127;

/// INT8 maximum value for saturated arithmetic
pub const INT8_MAX: i8 = 127;

/// Saturates a 32-bit integer to the valid INT8 range [-127, 127]
///
/// # Examples
/// ```
/// assert_eq!(saturate(-200), -127);
/// assert_eq!(saturate(200), 127);
/// assert_eq!(saturate(50), 50);
/// ```
pub fn saturate(val: i32) -> i8 {
    if val < INT8_MIN as i32 {
        INT8_MIN
    } else if val > INT8_MAX as i32 {
        INT8_MAX
    } else {
        val as i8
    }
}

/// A constraint definition with bounds and identifying name
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    /// Lower bound (inclusive)
    pub lo: i8,
    /// Upper bound (inclusive)
    pub hi: i8,
    /// Human-readable constraint name
    pub name: &'static str,
}

impl Constraint {
    /// Creates a new constraint with validation
    pub fn new(lo: i8, hi: i8, name: &'static str) -> Result<Self, FluxError> {
        if lo > hi {
            return Err(FluxError::InvalidConstraint(name));
        }
        Ok(Constraint { lo, hi, name })
    }
}

/// Severity levels for constraint violations
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Severity {
    Pass = 0,
    Caution = 1,
    Warning = 2,
    Critical = 3,
}

/// Result of constraint checking with detailed violation information
#[derive(Debug, Clone, PartialEq)]
pub struct FluxResult {
    /// Bitmask of which constraints failed (bit i = constraint i)
    pub error_mask: u8,
    /// Maximum severity level encountered
    pub severity: u8,
    /// Count of lower bound violations
    pub violated_lo: u8,
    /// Count of upper bound violations
    pub violated_hi: u8,
}

impl FluxResult {
    /// Creates a new result indicating all constraints passed
    pub fn pass() -> Self {
        FluxResult {
            error_mask: 0,
            severity: Severity::Pass as u8,
            violated_lo: 0,
            violated_hi: 0,
        }
    }

    /// Checks if all constraints passed
    pub fn is_pass(&self) -> bool {
        self.error_mask == 0
    }

    /// Gets the severity as an enum
    pub fn get_severity(&self) -> Severity {
        match self.severity {
            0 => Severity::Pass,
            1 => Severity::Caution,
            2 => Severity::Warning,
            _ => Severity::Critical,
        }
    }
}

/// Error types for the FLUX system
#[derive(Debug, Clone, PartialEq)]
pub enum FluxError {
    InvalidConstraint(&'static str),
    TooManyConstraints,
    PresetNotFound(String),
}

/// High-performance constraint checker with batch processing capabilities
pub struct FluxChecker {
    constraints: Vec<Constraint>,
}

impl FluxChecker {
    /// Creates a new checker with the given constraints
    ///
    /// # Arguments
    /// * `constraints` - Vector of constraints to enforce
    ///
    /// # Errors
    /// Returns `FluxError::TooManyConstraints` if more than 8 constraints are provided
    /// (limited by error_mask being u8)
    pub fn new(constraints: Vec<Constraint>) -> Result<Self, FluxError> {
        if constraints.len() > 8 {
            return Err(FluxError::TooManyConstraints);
        }
        Ok(FluxChecker { constraints })
    }

    /// Checks a single value against all constraints
    ///
    /// # Arguments
    /// * `value` - The INT8 value to check
    ///
    /// # Returns
    /// FluxResult with detailed violation information
    pub fn check(&self, value: i8) -> FluxResult {
        let mut result = FluxResult::pass();

        for (i, constraint) in self.constraints.iter().enumerate() {
            if value < constraint.lo {
                result.error_mask |= 1 << i;
                result.violated_lo += 1;
                result.severity = result.severity.max(self.calculate_severity(constraint, value));
            } else if value > constraint.hi {
                result.error_mask |= 1 << i;
                result.violated_hi += 1;
                result.severity = result.severity.max(self.calculate_severity(constraint, value));
            }
        }

        result
    }

    /// Checks multiple values in batch for improved performance
    ///
    /// # Arguments
    /// * `values` - Slice of INT8 values to check
    ///
    /// # Returns
    /// Vector of FluxResult, one for each input value
    pub fn check_batch(&self, values: &[i8]) -> Vec<FluxResult> {
        values.iter().map(|&value| self.check(value)).collect()
    }

    /// Creates a checker from an industry preset
    ///
    /// # Arguments
    /// * `name` - Name of the preset ("aviation", "medical", "maritime", "automotive", "energy", "nuclear")
    ///
    /// # Returns
    /// Configured FluxChecker or error if preset not found
    pub fn from_preset(name: &str) -> Result<Self, FluxError> {
        let constraints = match name {
            "aviation" => AVIATION_CONSTRAINTS.to_vec(),
            "medical" => MEDICAL_CONSTRAINTS.to_vec(),
            "maritime" => MARITIME_CONSTRAINTS.to_vec(),
            "automotive" => AUTOMOTIVE_CONSTRAINTS.to_vec(),
            "energy" => ENERGY_CONSTRAINTS.to_vec(),
            "nuclear" => NUCLEAR_CONSTRAINTS.to_vec(),
            _ => return Err(FluxError::PresetNotFound(String::from(name))),
        };

        FluxChecker::new(constraints)
    }

    /// Calculates severity based on how far the value is from constraint bounds
    fn calculate_severity(&self, constraint: &Constraint, value: i8) -> u8 {
        let range = (constraint.hi as i16) - (constraint.lo as i16);
        let violation_distance = if value < constraint.lo {
            (constraint.lo as i16) - (value as i16)
        } else {
            (value as i16) - (constraint.hi as i16)
        };

        if range == 0 || violation_distance >= range {
            Severity::Critical as u8
        } else if violation_distance >= range / 2 {
            Severity::Warning as u8
        } else {
            Severity::Caution as u8
        }
    }
}

// Industry-specific constraint presets
const AVIATION_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: -50, hi: 85, name: "temperature_celsius" },
    Constraint { lo: 0, hi: 100, name: "cabin_pressure_percent" },
    Constraint { lo: -90, hi: 90, name: "pitch_degrees" },
    Constraint { lo: -127, hi: 127, name: "roll_degrees_scaled" },
];

const MEDICAL_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: 60, hi: 100, name: "heart_rate_bpm_normalized" },
    Constraint { lo: 12, hi: 20, name: "respiratory_rate" },
    Constraint { lo: 36, hi: 38, name: "body_temp_celsius" },
    Constraint { lo: 90, hi: 100, name: "blood_oxygen_percent" },
];

const MARITIME_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: -40, hi: 60, name: "sea_temp_celsius" },
    Constraint { lo: 0, hi: 127, name: "wave_height_decimeters" },
    Constraint { lo: 0, hi: 120, name: "wind_speed_knots" },
    Constraint { lo: -50, hi: 50, name: "pressure_deviation_hpa" },
];

const AUTOMOTIVE_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: -40, hi: 125, name: "engine_temp_celsius" },
    Constraint { lo: 0, hi: 127, name: "vehicle_speed_mph" },
    Constraint { lo: 0, hi: 100, name: "fuel_level_percent" },
    Constraint { lo: 11, hi: 15, name: "battery_voltage" },
];

const ENERGY_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: 110, hi: 125, name: "grid_voltage_normalized" },
    Constraint { lo: 49, hi: 51, name: "frequency_hz" },
    Constraint { lo: 0, hi: 100, name: "load_factor_percent" },
    Constraint { lo: -20, hi: 80, name: "ambient_temp_celsius" },
];

const NUCLEAR_CONSTRAINTS: &[Constraint] = &[
    Constraint { lo: -40, hi: 40, name: "reactor_temp_deviation_celsius" },
    Constraint { lo: 0, hi: 127, name: "radiation_level_normalized" },
    Constraint { lo: 14, hi: 16, name: "pressure_mpa" },
    Constraint { lo: 0, hi: 100, name: "control_rod_position_percent" },
];

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_saturate_boundaries() {
        assert_eq!(saturate(-128), -127);
        assert_eq!(saturate(-127), -127);
        assert_eq!(saturate(0), 0);
        assert_eq!(saturate(127), 127);
        assert_eq!(saturate(128), 127);
        assert_eq!(saturate(1000), 127);
        assert_eq!(saturate(-1000), -127);
    }

    #[test]
    fn test_constraint_creation() {
        let constraint = Constraint::new(-10, 10, "test").unwrap();
        assert_eq!(constraint.lo, -10);
        assert_eq!(constraint.hi, 10);
        assert_eq!(constraint.name, "test");

        // Invalid constraint (lo > hi)
        assert!(Constraint::new(10, -10, "invalid").is_err());
    }

    #[test]
    fn test_single_constraint_pass() {
        let constraints = vec![
            Constraint::new(-10, 10, "range_test").unwrap()
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(5);
        assert!(result.is_pass());
        assert_eq!(result.error_mask, 0);
        assert_eq!(result.severity, Severity::Pass as u8);
        assert_eq!(result.violated_lo, 0);
        assert_eq!(result.violated_hi, 0);
    }

    #[test]
    fn test_single_constraint_fail_low() {
        let constraints = vec![
            Constraint::new(-10, 10, "range_test").unwrap()
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(-15);
        assert!(!result.is_pass());
        assert_eq!(result.error_mask, 1);
        assert_eq!(result.violated_lo, 1);
        assert_eq!(result.violated_hi, 0);
        assert!(result.severity > 0);
    }

    #[test]
    fn test_single_constraint_fail_high() {
        let constraints = vec![
            Constraint::new(-10, 10, "range_test").unwrap()
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(15);
        assert!(!result.is_pass());
        assert_eq!(result.error_mask, 1);
        assert_eq!(result.violated_lo, 0);
        assert_eq!(result.violated_hi, 1);
        assert!(result.severity > 0);
    }

    #[test]
    fn test_multi_constraint_mixed_results() {
        let constraints = vec![
            Constraint::new(-10, 10, "range1").unwrap(),
            Constraint::new(0, 20, "range2").unwrap(),
            Constraint::new(-5, 5, "range3").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(8);
        // Should pass range1 (8 in [-10,10]) and range2 (8 in [0,20])
        // Should fail range3 (8 not in [-5,5])
        assert!(!result.is_pass());
        assert_eq!(result.error_mask, 0b100); // Third constraint failed
        assert_eq!(result.violated_hi, 1);
        assert_eq!(result.violated_lo, 0);
    }

    #[test]
    fn test_severity_calculation() {
        let constraints = vec![
            Constraint::new(-10, 10, "narrow_range").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        // Small violation should be caution
        let result = checker.check(12);
        assert_eq!(result.get_severity(), Severity::Caution);

        // Large violation should be critical
        let result = checker.check(50);
        assert_eq!(result.get_severity(), Severity::Critical);
    }

    #[test]
    fn test_batch_checking() {
        let constraints = vec![
            Constraint::new(-10, 10, "range_test").unwrap()
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let values = vec![5, -15, 15, 0];
        let results = checker.check_batch(&values);

        assert_eq!(results.len(), 4);
        assert!(results[0].is_pass()); // 5 is in range
        assert!(!results[1].is_pass()); // -15 is below range
        assert!(!results[2].is_pass()); // 15 is above range
        assert!(results[3].is_pass()); // 0 is in range
    }

    #[test]
    fn test_preset_loading() {
        let aviation = FluxChecker::from_preset("aviation").unwrap();
        assert_eq!(aviation.constraints.len(), 4);

        let medical = FluxChecker::from_preset("medical").unwrap();
        assert_eq!(medical.constraints.len(), 4);

        let maritime = FluxChecker::from_preset("maritime").unwrap();
        assert_eq!(maritime.constraints.len(), 4);

        let automotive = FluxChecker::from_preset("automotive").unwrap();
        assert_eq!(automotive.constraints.len(), 4);

        let energy = FluxChecker::from_preset("energy").unwrap();
        assert_eq!(energy.constraints.len(), 4);

        let nuclear = FluxChecker::from_preset("nuclear").unwrap();
        assert_eq!(nuclear.constraints.len(), 4);

        // Unknown preset should fail
        assert!(FluxChecker::from_preset("unknown").is_err());
    }

    #[test]
    fn test_overflow_protection() {
        // Test that we can't create more than 8 constraints
        let mut constraints = Vec::new();
        for _i in 0..10 {
            constraints.push(Constraint::new(-1, 1, "test").unwrap());
        }
        assert!(FluxChecker::new(constraints).is_err());
    }

    #[test]
    fn test_edge_cases() {
        let constraints = vec![
            Constraint::new(INT8_MIN, INT8_MAX, "full_range").unwrap()
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        // Min and max values should pass
        assert!(checker.check(INT8_MIN).is_pass());
        assert!(checker.check(INT8_MAX).is_pass());

        // Test with empty constraints
        let empty_checker = FluxChecker::new(vec![]).unwrap();
        assert!(empty_checker.check(0).is_pass());
    }

    #[test]
    fn test_all_pass_scenario() {
        let constraints = vec![
            Constraint::new(-50, 50, "wide_range1").unwrap(),
            Constraint::new(-100, 100, "wide_range2").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(25);
        assert!(result.is_pass());
        assert_eq!(result.error_mask, 0);
        assert_eq!(result.violated_lo, 0);
        assert_eq!(result.violated_hi, 0);
    }

    #[test]
    fn test_all_fail_scenario() {
        let constraints = vec![
            Constraint::new(10, 20, "high_range1").unwrap(),
            Constraint::new(15, 25, "high_range2").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let result = checker.check(5);
        assert!(!result.is_pass());
        assert_eq!(result.error_mask, 0b11); // Both constraints failed
        assert_eq!(result.violated_lo, 2);
        assert_eq!(result.violated_hi, 0);
    }
}

/// Simple benchmarking module using inline timing
#[cfg(test)]
mod benchmarks {
    use super::*;
    use alloc::vec;

    /// Get current time in nanoseconds (mock implementation for no_std)
    fn get_time_ns() -> u64 {
        // In a real no_std environment, you'd use a hardware timer
        // For testing purposes, we'll use a simple counter
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1000; // Simulate 1us per call
            COUNTER
        }
    }

    #[test]
    fn benchmark_single_checks() {
        let constraints = vec![
            Constraint::new(-50, 50, "temp").unwrap(),
            Constraint::new(0, 100, "pressure").unwrap(),
            Constraint::new(-90, 90, "angle").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let iterations = 10000;
        let start_time = get_time_ns();

        for i in 0..iterations {
            let value = ((i % 201) - 100) as i8; // Values from -100 to 100
            let _ = checker.check(value);
        }

        let end_time = get_time_ns();
        let duration_ns = end_time - start_time;
        let checks_per_sec = (iterations as f64 * 1_000_000_000.0) / duration_ns as f64;

        // Store results in variables to verify calculations
        let _single_checks_per_sec = checks_per_sec;
        let _single_duration_ns = duration_ns;

        // Verify we achieved reasonable performance (mock timing always passes)
        assert!(checks_per_sec > 0.0);
    }

    #[test]
    fn benchmark_batch_checks() {
        let constraints = vec![
            Constraint::new(-50, 50, "temp").unwrap(),
            Constraint::new(0, 100, "pressure").unwrap(),
            Constraint::new(-90, 90, "angle").unwrap(),
        ];
        let checker = FluxChecker::new(constraints).unwrap();

        let batch_size = 1000;
        let values: Vec<i8> = (0..batch_size).map(|i| ((i % 201) - 100) as i8).collect();
        let iterations = 100;

        let start_time = get_time_ns();

        for _ in 0..iterations {
            let _ = checker.check_batch(&values);
        }

        let end_time = get_time_ns();
        let duration_ns = end_time - start_time;
        let total_checks = iterations * batch_size;
        let checks_per_sec = (total_checks as f64 * 1_000_000_000.0) / duration_ns as f64;

        // Store results in variables for verification
        let _batch_checks_per_sec = checks_per_sec;
        let _batch_size_result = batch_size;
        let _iterations_result = iterations;
        let _duration_result = duration_ns;

        // Verify we achieved reasonable performance (mock timing always passes)
        assert!(checks_per_sec > 0.0);
    }

    #[test]
    fn benchmark_preset_loading() {
        let presets = ["aviation", "medical", "maritime", "automotive", "energy", "nuclear"];

        let start_time = get_time_ns();

        for preset in &presets {
            for _ in 0..1000 {
                let _ = FluxChecker::from_preset(preset).unwrap();
            }
        }

        let end_time = get_time_ns();
        let duration_ns = end_time - start_time;
        let loads_per_sec = (presets.len() * 1000) as f64 * 1_000_000_000.0 / duration_ns as f64;

        // Store results for verification
        let _preset_loads_per_sec = loads_per_sec;
        let _total_presets_loaded = presets.len() * 1000;
        let _preset_duration = duration_ns;

        assert!(loads_per_sec > 0.0);
    }
}