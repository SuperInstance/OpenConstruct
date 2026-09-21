// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Snap — hard-locked fact with confidence.
//!
//! A snap is a ground-truth anchor in the signal chain. Once locked, it
//! constrains all downstream inferences. Think of it as a measurement from
//! a calibrated instrument — it's as close to "true" as you get.
//!
//! # Confidence Levels
//!
//! - **1.0**: Absolute ground truth (mathematical proof, verified sensor reading)
//! - **0.8-0.99**: High confidence (replicated measurement, consensus)
//! - **0.5-0.79**: Moderate (single measurement, plausible claim)
//! - **< 0.5**: Low (unverified, speculative)
//!
//! # Example
//!
//! ```rust
//! use openshell_signal_chain::Snap;
//!
//! let snap = Snap::absolute(serde_json::json!({"depth": 87.2}), 0.0);
//! assert_eq!(snap.confidence, 1.0);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A hard-locked fact in a room.
///
/// Snaps are the ground truth anchors in the signal chain. They always appear
/// in query results regardless of dial position (only inferences are filtered).
///
/// Use [`Snap::new`] for arbitrary confidence or [`Snap::absolute`] for
/// guaranteed facts (confidence = 1.0).
///
/// # Fields
///
/// - `fact`: Arbitrary JSON data representing the fact.
/// - `confidence`: How certain we are (0.0–1.0, clamped).
/// - `dial_position`: The dial position of the room when this snap was created.
/// - `timestamp`: When this snap was created (UTC).
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::Snap;
///
/// let snap = Snap::new(
///     serde_json::json!({"sonar": "contact at 200m"}),
///     0.95,
///     0.1,
/// );
/// assert_eq!(snap.confidence, 0.95);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snap {
    /// The fact data (arbitrary JSON).
    pub fact: serde_json::Value,
    /// Confidence: 1.0 = absolute ground truth, 0.0 = untrusted. Clamped to [0.0, 1.0].
    pub confidence: f64,
    /// Dial position of the room when this snap was created.
    pub dial_position: f64,
    /// Timestamp of when this snap was created (UTC).
    pub timestamp: DateTime<Utc>,
}

impl Snap {
    /// Create a new snap with given fact and confidence.
    ///
    /// # Parameters
    ///
    /// - `fact`: The fact data as arbitrary JSON.
    /// - `confidence`: Confidence level (clamped to [0.0, 1.0]).
    /// - `dial_position`: The dial position of the room at creation time.
    ///
    /// # Returns
    ///
    /// A new [`Snap`] with the current UTC timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Snap;
    ///
    /// let snap = Snap::new(
    ///     serde_json::json!({"temperature": 72.3}),
    ///     0.9,
    ///     0.0,
    /// );
    /// assert_eq!(snap.confidence, 0.9);
    /// ```
    pub fn new(fact: serde_json::Value, confidence: f64, dial_position: f64) -> Self {
        Self {
            fact,
            confidence: confidence.clamp(0.0, 1.0),
            dial_position,
            timestamp: Utc::now(),
        }
    }

    /// Create an absolute snap (confidence = 1.0).
    ///
    /// Use this for facts that are verified beyond any doubt — mathematical
    /// proofs, calibrated sensor readings, certified build outputs.
    ///
    /// # Parameters
    ///
    /// - `fact`: The fact data as arbitrary JSON.
    /// - `dial_position`: The dial position of the room at creation time.
    ///
    /// # Returns
    ///
    /// A new [`Snap`] with confidence = 1.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openshell_signal_chain::Snap;
    ///
    /// let snap = Snap::absolute(serde_json::json!({"proven": true}), 0.0);
    /// assert_eq!(snap.confidence, 1.0);
    /// ```
    pub fn absolute(fact: serde_json::Value, dial_position: f64) -> Self {
        Self::new(fact, 1.0, dial_position)
    }
}
