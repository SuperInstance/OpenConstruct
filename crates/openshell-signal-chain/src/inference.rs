// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Inference — soft extrapolation with confidence
//! 
//! An inference is a hypothesis that may or may not be true.
//! Confidence determines how likely it is to be correct.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A soft extrapolation/hypothesis in a room.
/// 
/// Inferences are predictions, hypotheses, or suggestions.
/// They can be elevated to snaps when verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inference {
    /// The hypothesis data (arbitrary JSON)
    pub hypothesis: serde_json::Value,
    /// Confidence: 1.0 = proven, 0.0 = wild guess
    pub confidence: f64,
    /// Dial position when this inference was created
    pub dialect: f64,
    /// Timestamp of when this inference was created
    pub timestamp: DateTime<Utc>,
}

impl Inference {
    /// Create a new inference with given hypothesis and confidence
    pub fn new(hypothesis: serde_json::Value, confidence: f64, dialect: f64) -> Self {
        Self {
            hypothesis,
            confidence: confidence.clamp(0.0, 1.0),
            dialect,
            timestamp: Utc::now(),
        }
    }

    /// Create a high-confidence inference (0.8+)
    pub fn likely(hypothesis: serde_json::Value, dialect: f64) -> Self {
        Self::new(hypothesis, 0.8, dialect)
    }

    /// Create a speculative inference (0.4-0.6)
    pub fn speculative(hypothesis: serde_json::Value, dialect: f64) -> Self {
        Self::new(hypothesis, 0.5, dialect)
    }
}