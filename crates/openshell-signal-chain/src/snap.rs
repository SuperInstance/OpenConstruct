// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Snap — hard-locked fact with confidence
//! 
//! A snap is a ground-truth anchor. confidence=1.0 means absolute fact.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A hard-locked fact in a room.
/// 
/// Snaps are the ground truth anchors in the signal chain.
/// Once locked, they constrain all downstream inferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snap {
    /// The fact data (arbitrary JSON)
    pub fact: serde_json::Value,
    /// Confidence: 1.0 = absolute ground truth, 0.0 = untrusted
    pub confidence: f64,
    /// Dial position when this snap was created
    pub dialect: f64,
    /// Timestamp of when this snap was created
    pub timestamp: DateTime<Utc>,
}

impl Snap {
    /// Create a new snap with given fact and confidence
    pub fn new(fact: serde_json::Value, confidence: f64, dialect: f64) -> Self {
        Self {
            fact,
            confidence: confidence.clamp(0.0, 1.0),
            dialect,
            timestamp: Utc::now(),
        }
    }

    /// Create an absolute snap (confidence = 1.0)
    pub fn absolute(fact: serde_json::Value, dialect: f64) -> Self {
        Self::new(fact, 1.0, dialect)
    }
}