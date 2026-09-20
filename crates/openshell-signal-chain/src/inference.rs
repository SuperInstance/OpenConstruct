// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Inference — soft extrapolation with confidence.
//!
//! An inference is a hypothesis that may or may not be true. Unlike snaps,
//! inferences are filtered by the dial threshold during queries — at low dial
//! positions only high-confidence inferences appear, while at high positions
//! everything passes.
//!
//! # Confidence Levels
//!
//! - **0.8+**: Likely — multiple supporting signals, high prior probability
//! - **0.4-0.8**: Speculative — plausible but unverified
//! - **< 0.4**: Wild guess — included only at near-1.0 dial positions
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::Inference;
//!
//! let inf = Inference::likely(serde_json::json!({"anomaly": true}), 0.3);
//! assert!(inf.confidence >= 0.8);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A soft extrapolation/hypothesis in a room.
///
/// Inferences are predictions, hypotheses, or suggestions. They can be
/// elevated to snaps when verified. During queries, inferences are filtered
/// by the dial threshold — only those with `confidence >= threshold` pass.
///
/// # Fields
///
/// - `hypothesis`: Arbitrary JSON data representing the hypothesis.
/// - `confidence`: How likely the hypothesis is correct (0.0–1.0, clamped).
/// - `dial_position`: The dial position of the room when this was created.
/// - `timestamp`: When this inference was created (UTC).
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::Inference;
///
/// let inf = Inference::new(
///     serde_json::json!({"target": "vessel", "probability": 0.7}),
///     0.7,
///     0.5,
/// );
/// assert_eq!(inf.confidence, 0.7);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inference {
    /// The hypothesis data (arbitrary JSON).
    pub hypothesis: serde_json::Value,
    /// Confidence: 1.0 = proven, 0.0 = wild guess. Clamped to [0.0, 1.0].
    pub confidence: f64,
    /// Dial position of the room when this inference was created.
    pub dial_position: f64,
    /// Timestamp of when this inference was created (UTC).
    pub timestamp: DateTime<Utc>,
}

impl Inference {
    /// Create a new inference with given hypothesis and confidence.
    ///
    /// # Parameters
    ///
    /// - `hypothesis`: The hypothesis data as arbitrary JSON.
    /// - `confidence`: Confidence level (clamped to [0.0, 1.0]).
    /// - `dial_position`: The dial position of the room at creation time.
    ///
    /// # Returns
    ///
    /// A new [`Inference`] with the current UTC timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Inference;
    ///
    /// let inf = Inference::new(
    ///     serde_json::json!({"prediction": "rain"}),
    ///     0.65,
    ///     0.5,
    /// );
    /// assert_eq!(inf.confidence, 0.65);
    /// ```
    pub fn new(hypothesis: serde_json::Value, confidence: f64, dial_position: f64) -> Self {
        Self {
            hypothesis,
            confidence: confidence.clamp(0.0, 1.0),
            dial_position,
            timestamp: Utc::now(),
        }
    }

    /// Create a high-confidence inference (0.8+).
    ///
    /// Use for predictions backed by multiple supporting signals.
    ///
    /// # Parameters
    ///
    /// - `hypothesis`: The hypothesis data as arbitrary JSON.
    /// - `dial_position`: The dial position of the room at creation time.
    ///
    /// # Returns
    ///
    /// A new [`Inference`] with confidence = 0.8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Inference;
    ///
    /// let inf = Inference::likely(serde_json::json!({"status": "optimal"}), 0.3);
    /// assert!(inf.confidence >= 0.8);
    /// ```
    pub fn likely(hypothesis: serde_json::Value, dial_position: f64) -> Self {
        Self::new(hypothesis, 0.8, dial_position)
    }

    /// Create a speculative inference (0.5).
    ///
    /// Use for plausible but unverified hypotheses.
    ///
    /// # Parameters
    ///
    /// - `hypothesis`: The hypothesis data as arbitrary JSON.
    /// - `dial_position`: The dial position of the room at creation time.
    ///
    /// # Returns
    ///
    /// A new [`Inference`] with confidence = 0.5.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Inference;
    ///
    /// let inf = Inference::speculative(serde_json::json!({"maybe": true}), 0.5);
    /// assert_eq!(inf.confidence, 0.5);
    /// ```
    pub fn speculative(hypothesis: serde_json::Value, dial_position: f64) -> Self {
        Self::new(hypothesis, 0.5, dial_position)
    }
}
