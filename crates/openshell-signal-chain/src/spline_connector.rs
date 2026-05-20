// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Spline Connector — constraint splines inside signal chain rooms.
//!
//! Bridges the constraint spline evaluation with the signal chain's
//! cascade architecture. A `SplineRoom` is a signal chain room that
//! holds `SplineConstraint`s and evaluates them at the room's dial position.
//!
//! The connector transforms:
//! - constraint boundaries → curvature distances
//! - dial positions → violation severities
//! - evaluation results → inference-weighted room outputs
//!
//! This is where the fare curves meet the cascade.

use serde::{Deserialize, Serialize};

use crate::{
    Dial, Inference, Room, SignalChain,
    constraint::{SplineConstraint, ViolationSeverity},
};

/// Inference level — discrete categories of reasoning depth.
///
/// Maps to dial positions:
/// - 0.0-0.05 → Formal (hard snap)
/// - 0.05-0.2  → Bathy (depth sensing)
/// - 0.2-0.35  → Commit (commitment point)
/// - 0.35-0.5  → Analysis (balanced)
/// - 0.5-0.65  → Review (soft review)
/// - 0.65-0.8  → Extrapolate (speculative)
/// - 0.8-0.95  → Creative (generative)
/// - ~1.0      → Exploratory (wild exploration)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceLevel {
    Formal,
    Bathy,
    Commit,
    Analysis,
    Review,
    Extrapolate,
    Creative,
    Exploratory,
}

impl InferenceLevel {
    /// Derive inference level from dial position.
    pub fn from_dial(dial: Dial) -> Self {
        match dial.position {
            p if p <= 0.05 => InferenceLevel::Formal,
            p if p <= 0.2  => InferenceLevel::Bathy,
            p if p <= 0.35 => InferenceLevel::Commit,
            p if p <= 0.5  => InferenceLevel::Analysis,
            p if p <= 0.65 => InferenceLevel::Review,
            p if p <= 0.8  => InferenceLevel::Extrapolate,
            p if p <= 0.95 => InferenceLevel::Creative,
            _              => InferenceLevel::Exploratory,
        }
    }

    /// Convert level to a dial position (round-trip).
    pub fn to_dial(&self) -> Dial {
        match self {
            InferenceLevel::Formal => Dial::hard(),
            InferenceLevel::Bathy => Dial::new(0.1),
            InferenceLevel::Commit => Dial::new(0.25),
            InferenceLevel::Analysis => Dial::new(0.45),
            InferenceLevel::Review => Dial::new(0.55),
            InferenceLevel::Extrapolate => Dial::new(0.75),
            InferenceLevel::Creative => Dial::new(0.9),
            InferenceLevel::Exploratory => Dial::soft(),
        }
    }
}

/// A signal chain room with constraint spline evaluation built in.
///
/// Add constraints, push values, and the room evaluates them at its
/// dial position every time you query. Violations become inferences
/// with severity-weighted confidence.
#[derive(Debug, Clone)]
pub struct SplineRoom {
    pub name: String,
    pub dial: Dial,
    constraints: Vec<SplineConstraint>,
    value_history: Vec<i32>,
    max_history: usize,
}

impl SplineRoom {
    pub fn new(name: &str, dial: Dial) -> Self {
        SplineRoom {
            name: name.to_string(),
            dial,
            constraints: Vec::new(),
            value_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn add_constraint(&mut self, constraint: SplineConstraint) {
        self.constraints.push(constraint);
    }

    pub fn add_constraints(&mut self, constraints: impl IntoIterator<Item = SplineConstraint>) {
        self.constraints.extend(constraints);
    }

    pub fn push_value(&mut self, value: i32) {
        if self.value_history.len() >= self.max_history {
            self.value_history.remove(0);
        }
        self.value_history.push(value);
    }

    pub fn push_values(&mut self, values: impl IntoIterator<Item = i32>) {
        for v in values {
            self.push_value(v);
        }
    }

    fn evaluate_single(&self, constraint: &SplineConstraint, value: i32, dial: Dial) -> SplineEvaluation {
        match constraint.evaluate(value, dial) {
            Ok(curvature) => SplineEvaluation {
                constraint_name: constraint.name.clone(),
                value,
                curvature,
                severity: None,
                is_violation: false,
                inference: Inference::new(
                    serde_json::json!({
                        "constraint": constraint.name,
                        "curvature": curvature,
                        "neutral_distance": curvature,
                    }),
                    1.0 - curvature.min(1.0),
                    dial.position,
                ),
            },
            Err(violation) => SplineEvaluation {
                constraint_name: constraint.name.clone(),
                value,
                curvature: violation.curvature,
                severity: Some(violation.severity),
                is_violation: true,
                inference: Inference::new(
                    serde_json::json!({
                        "constraint": constraint.name,
                        "violation": violation.constraint,
                        "value": violation.value,
                        "severity": format!("{:?}", violation.severity),
                    }),
                    violation.severity.confidence_from_dial(dial),
                    dial.position,
                ),
            },
        }
    }

    pub fn query(&self, override_dial: Option<Dial>) -> Vec<SplineEvaluation> {
        let dial = override_dial.unwrap_or(self.dial);
        if let Some(&latest) = self.value_history.last() {
            self.constraints
                .iter()
                .map(|c| self.evaluate_single(c, latest, dial))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplineEvaluation {
    pub constraint_name: String,
    pub value: i32,
    pub curvature: f64,
    pub severity: Option<ViolationSeverity>,
    pub is_violation: bool,
    pub inference: Inference,
}

impl SplineEvaluation {
    pub fn should_cascade(&self, threshold: f64) -> bool {
        self.inference.confidence >= threshold
    }
}

impl ViolationSeverity {
    pub fn confidence_from_dial(&self, dial: Dial) -> f64 {
        let base = match self {
            ViolationSeverity::Critical => 0.95,
            ViolationSeverity::Warning => 0.7,
            ViolationSeverity::Advisory => 0.4,
        };
        base * (1.0 - dial.position * 0.4)
    }
}

#[derive(Debug, Clone)]
pub struct SplineChain {
    inner: SignalChain,
    spline_rooms: std::collections::HashMap<String, SplineRoom>,
}

impl SplineChain {
    pub fn new(name: &str) -> Self {
        SplineChain {
            inner: SignalChain::new(name),
            spline_rooms: std::collections::HashMap::new(),
        }
    }

    pub fn spline_room(&mut self, name: &str, dial: Dial) -> &mut SplineRoom {
        if !self.spline_rooms.contains_key(name) {
            self.spline_rooms.insert(name.to_string(), SplineRoom::new(name, dial));
        }
        self.spline_rooms.get_mut(name).unwrap()
    }

    pub fn add_spline_room(&mut self, name: &str, dial: Dial, constraints: Vec<SplineConstraint>) -> &mut SplineRoom {
        let mut room = SplineRoom::new(name, dial);
        room.add_constraints(constraints);
        self.spline_rooms.insert(name.to_string(), room);
        self.spline_rooms.get_mut(name).unwrap()
    }

    pub fn query_spline_room(&self, name: &str, override_dial: Option<Dial>) -> Option<Vec<SplineEvaluation>> {
        self.spline_rooms.get(name).map(|r| r.query(override_dial))
    }

    pub fn query_all(&self, override_dial: Option<Dial>) -> Vec<(String, Vec<SplineEvaluation>)> {
        self.spline_rooms
            .iter()
            .map(|(name, room)| (name.clone(), room.query(override_dial)))
            .collect()
    }

    pub fn push(&mut self, room_name: &str, values: impl IntoIterator<Item = i32>) -> Option<Vec<SplineEvaluation>> {
        self.spline_rooms.get_mut(room_name).map(|r| {
            r.push_values(values);
            r.query(None)
        })
    }

    pub fn spline_room_count(&self) -> usize {
        self.spline_rooms.len()
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spline_room_basic() {
        let mut room = SplineRoom::new("test", Dial::hard());
        room.add_constraint(SplineConstraint::symmetric(0, 100, "test_constraint", 1.0));
        room.push_value(50);
        let results = room.query(None);
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_violation);
    }

    #[test]
    fn test_spline_room_violation() {
        let mut room = SplineRoom::new("test", Dial::hard());
        room.add_constraint(SplineConstraint::symmetric(0, 100, "test_constraint", 1.0));
        room.push_value(-10);
        let results = room.query(None);
        assert!(results[0].is_violation);
        assert_eq!(results[0].severity, Some(ViolationSeverity::Critical));
    }

    #[test]
    fn test_inference_level_round_trip() {
        let levels = [
            (InferenceLevel::Formal, 0.0),
            (InferenceLevel::Bathy, 0.1),
            (InferenceLevel::Commit, 0.25),
            (InferenceLevel::Analysis, 0.45),
            (InferenceLevel::Review, 0.55),
            (InferenceLevel::Extrapolate, 0.75),
            (InferenceLevel::Creative, 0.9),
            (InferenceLevel::Exploratory, 1.0),
        ];
        for (level, expected_pos) in levels {
            let dial = level.to_dial();
            assert!((dial.position - expected_pos).abs() < 0.01, "level {:?} got {:?}", level, dial.position);
            let round_trip = InferenceLevel::from_dial(dial);
            assert_eq!(level, round_trip);
        }
    }

    #[test]
    fn test_spline_chain() {
        let mut chain = SplineChain::new("fleet-spline");
        chain.add_spline_room("engine", Dial::hard(), crate::constraint::maritime_spline());
        chain.push("engine", vec![20, 60, 30, -10]);
        let results = chain.query_spline_room("engine", None);
        assert!(results.is_some());
        assert_eq!(results.unwrap().len(), 4);
    }
}
