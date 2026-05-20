// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Dial — continuous control from hard-snapped to soft-inferenced
//! 
//! Position range: 0.0 (hard algorithm) to 1.0 (soft inference)

use serde::{Deserialize, Serialize};

/// A dial controlling the ratio of hard-snapped vs soft-inferenced reasoning.
/// 
/// 0.0 = deterministic, provable, certifiable (theorem provers, FLUX ISA, H1 cohomology)
/// 1.0 = probabilistic, generative, exploratory (story generation, creative fill)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dial {
    /// Position on the dial: 0.0 (hard) to 1.0 (soft)
    pub position: f64,
}

impl Dial {
    /// Create a new dial with given position (clamped to 0.0-1.0)
    pub fn new(position: f64) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
        }
    }

    /// Create a dial at 0.0 (hard algorithm)
    pub fn hard() -> Self {
        Self { position: 0.0 }
    }

    /// Create a dial at 1.0 (soft inference)
    pub fn soft() -> Self {
        Self { position: 1.0 }
    }

    /// Weight for hard-snapped facts (inverse of position)
    pub fn snap_weight(&self) -> f64 {
        1.0 - self.position
    }

    /// Weight for soft inferences
    pub fn inference_weight(&self) -> f64 {
        self.position
    }

    /// Threshold for including inferences (1.0 - position)
    /// At 0.0: no inferences (threshold = 1.0)
    /// At 1.0: all inferences (threshold = 0.0)
    pub fn inference_threshold(&self) -> f64 {
        1.0 - self.position
    }

    /// Check if an inference passes the threshold for this dial
    pub fn accepts_inference(&self, confidence: f64) -> bool {
        confidence >= self.inference_threshold()
    }
}

impl Default for Dial {
    fn default() -> Self {
        Self { position: 0.5 }
    }
}

// Preset dials for common use cases

/// Pure formal reasoning — theorem provers, FLUX ISA, H1 cohomology
pub const DIAL_FORMAL: Dial = Dial { position: 0.0 };

/// Hard bathydata — sonar readings, coordinates, depth facts
pub const DIAL_BATHY: Dial = Dial { position: 0.1 };

/// Git history, build logs, certifiable traces
pub const DIAL_COMMIT: Dial = Dial { position: 0.05 };

/// Reasoning with snaps as anchors
pub const DIAL_ANALYSIS: Dial = Dial { position: 0.4 };

/// Equal weight to algorithm and inference
pub const DIAL_REVIEW: Dial = Dial { position: 0.5 };

/// Hypothesis generation, creative fill
pub const DIAL_EXTRAPOLATE: Dial = Dial { position: 0.7 };

/// Story generation, game narrative
pub const DIAL_CREATIVE: Dial = Dial { position: 0.9 };

/// Pure inference, no snap constraints
pub const DIAL_EXPLORATORY: Dial = Dial { position: 1.0 };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dial_position() {
        let d = Dial::new(0.5);
        assert_eq!(d.position, 0.5);
        assert_eq!(d.snap_weight(), 0.5);
        assert_eq!(d.inference_weight(), 0.5);
    }

    #[test]
    fn test_dial_bounds() {
        let d = Dial::new(-0.5);
        assert_eq!(d.position, 0.0);
        
        let d = Dial::new(1.5);
        assert_eq!(d.position, 1.0);
    }

    #[test]
    fn test_threshold() {
        let d = Dial::new(0.0); // hard - threshold = 1.0
        assert!(!d.accepts_inference(0.9));
        assert!(d.accepts_inference(1.0));

        let d = Dial::new(0.5); // threshold = 0.5
        assert!(!d.accepts_inference(0.4));
        assert!(d.accepts_inference(0.5));
        assert!(d.accepts_inference(0.9));

        let d = Dial::new(1.0); // soft - threshold = 0.0
        assert!(d.accepts_inference(0.0));
        assert!(d.accepts_inference(0.1));
    }
}