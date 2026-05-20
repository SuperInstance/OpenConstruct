// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! # OpenShell Signal Chain
//!
//! Every intelligent system needs a dial between hard-snapped algorithms
//! and soft-inferenced models. This crate implements the Signal Chain Thesis:
//! structured rooms that anchor facts in space/time, connected by a continuous
//! dial that controls how much hard truth vs. soft reasoning you get.
//!
//! # Core Primitives
//!
//! - **[`Dial`]**: Continuous control from 0.0 (hard/algorithmic) to 1.0 (soft/inferenced)
//! - **[`Snap`]**: Hard-locked fact with confidence — ground truth anchors
//! - **[`Inference`]**: Soft extrapolation with confidence — hypotheses and predictions
//! - **[`Room`]**: Fact-space containing snaps, inferences, child rooms, and a dial position
//! - **[`SignalChain`]**: Named collection of rooms with global dial control
//!
//! # Quick Start
//!
//! ```rust
//! use openshell_signal_chain::{Dial, Room, SignalChain};
//!
//! // Create a chain for your fleet
//! let mut chain = SignalChain::new("fleet-ops");
//!
//! // Add a room with hard sensor data
//! let sensors = chain.room("sonar-array");
//! sensors.add_snap(serde_json::json!({"depth": 87.2, "lat": 45.3}), 1.0);
//! sensors.add_inference(serde_json::json!({"possible": "wreckage"}), 0.7);
//!
//! // Query: at dial 0.5, get snaps + inferences with confidence >= 0.5
//! let results = sensors.query(Dial::new(0.5));
//! assert_eq!(results.len(), 2); // one snap + one inference (0.7 >= 0.5)
//!
//! // At dial 0.0 (hard), only snaps pass
//! let hard_results = sensors.query(Dial::hard());
//! assert_eq!(hard_results.len(), 1); // snap only
//! ```
//!
//! # The Dial Concept
//!
//! The dial is the core abstraction. Position 0.0 means "give me only verified,
//! deterministic facts". Position 1.0 means "include all inferences, even wild
//! guesses". The threshold for including an inference is `1.0 - dial_position`.
//!
//! | Dial Position | Threshold | What You Get |
//! |---------------|-----------|--------------|
//! | 0.0           | 1.0       | Only absolute snaps |
//! | 0.5           | 0.5       | Snaps + confident inferences |
//! | 1.0           | 0.0       | Everything |
//!
//! # Error Handling
//!
//! [`Dial::try_new`] returns a [`Result`] that rejects values outside [0.0, 1.0].
//! [`Dial::new`] is also available and clamps values silently for convenience.

mod dial;
mod error;
mod inference;
mod query;
mod room;
mod signal_chain;
mod snap;

pub use dial::Dial;
pub use error::SignalChainError;
pub use inference::Inference;
pub use query::QueryResult;
pub use room::Room;
pub use signal_chain::SignalChain;
pub use snap::Snap;

// Preset dials for common use cases
pub use dial::DIAL_FORMAL;
pub use dial::DIAL_BATHY;
pub use dial::DIAL_COMMIT;
pub use dial::DIAL_ANALYSIS;
pub use dial::DIAL_REVIEW;
pub use dial::DIAL_EXTRAPOLATE;
pub use dial::DIAL_CREATIVE;
pub use dial::DIAL_EXPLORATORY;

pub mod constraint;
pub use constraint::{
    SplineConstraint, SplineResult, ConstraintViolation, ViolationSeverity,
    evaluate_spline, maritime_spline,
};
pub mod spline_connector;
pub use spline_connector::{
    SplineRoom, SplineChain, SplineEvaluation,
};

pub mod holonomy_bridge;
pub use holonomy_bridge::{
    HolonomyRoom, HolonomyChain, HolonomyStatus, BettiResult,
};
