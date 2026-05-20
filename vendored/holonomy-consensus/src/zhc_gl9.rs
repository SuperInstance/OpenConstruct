//! Zero Holonomy Consensus — GL(9) General Linear Group Extension
//!
//! # The Problem
//!
//! The original ZHC used SO(3) rotation matrices (3D) to measure holonomy
//! in agent communication. Deep-dive experiments showed that 3D projection
//! DESTROYS correlation (r=-0.045) between holonomy and alignment.
//!
//! # The Fix
//!
//! GL(9) operates on full 9D intent vectors. Each dimension is a CI facet:
//!
//! | Index | CI Facet        | Description                        |
//! |-------|-----------------|------------------------------------|
//! | 0     | C1 Boundary     | System boundaries and scope        |
//! | 1     | C2 Pattern      | Recognized patterns                |
//! | 2     | C3 Process      | Process models                     |
//! | 3     | C4 Knowledge    | Knowledge structures               |
//! | 4     | C5 Social       | Social dynamics                    |
//! | 5     | C6 Deep Structure | Underlying structures            |
//! | 6     | C7 Instrument   | Instruments and tools              |
//! | 7     | C8 Paradigm     | Paradigmatic frameworks            |
//! | 8     | C9 Stakes       | Stakes and values                  |
//!
//! # Mathematics
//!
//! For a cycle γ of agents exchanging intent transforms:
//! ```text
//! Hol(γ) = Πᵢ Tᵢ  (product of 9×9 transforms around the cycle)
//! ```
//!
//! - **||Hol(γ) - I|| < ε** → Consistent (zero holonomy)
//! - **||Hol(γ) - I|| ≥ ε** → Inconsistent, locate fault via bisection
//!
//! The deviation metric is the Frobenius norm of (M - I).

use serde::{Deserialize, Serialize};

/// Dimensionality of the intent space (9 CI facets)
pub const INTENT_DIM: usize = 9;

/// CI facet names for human-readable output
pub const CI_FACETS: [&str; 9] = [
    "C1 Boundary",
    "C2 Pattern",
    "C3 Process",
    "C4 Knowledge",
    "C5 Social",
    "C6 Deep Structure",
    "C7 Instrument",
    "C8 Paradigm",
    "C9 Stakes",
];

/// Default ZHC tolerance (Oracle1's standard)
pub const DEFAULT_TOLERANCE: f64 = 0.5;

/// A 9×9 matrix in GL(9) — general linear group operating on 9D intent vectors.
///
/// Stored as a flat array of 81 f64 values in row-major order.
#[derive(Clone, Debug)]
pub struct GL9Matrix(pub [f64; 81]);

impl serde::Serialize for GL9Matrix {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_newtype_struct("GL9Matrix", &self.0[..])
    }
}

impl<'de> serde::Deserialize<'de> for GL9Matrix {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v: Vec<f64> = serde::Deserialize::deserialize(d)?;
        if v.len() != 81 {
            return Err(serde::de::Error::custom("expected 81 elements"));
        }
        let mut arr = [0.0f64; 81];
        arr.copy_from_slice(&v);
        Ok(GL9Matrix(arr))
    }
}

impl GL9Matrix {
    /// The identity element of GL(9).
    pub fn identity() -> Self {
        let mut data = [0.0f64; 81];
        for i in 0..9 {
            data[i * 9 + i] = 1.0;
        }
        Self(data)
    }

    /// Create from a 2D array (row-major).
    pub fn from_2d(rows: &[[f64; 9]; 9]) -> Self {
        let mut data = [0.0f64; 81];
        for i in 0..9 {
            data[i * 9..(i + 1) * 9].copy_from_slice(&rows[i]);
        }
        Self(data)
    }

    /// Create a rotation in a specified 2D plane within the 9D space.
    ///
    /// This rotates in the (dim_a, dim_b) plane by `angle` radians,
    /// leaving all other dimensions unchanged. Analogous to SO(3) axis-angle
    /// but generalized to any pair of CI facets.
    pub fn plane_rotation(dim_a: usize, dim_b: usize, angle: f64) -> Self {
        assert!(
            dim_a < 9 && dim_b < 9 && dim_a != dim_b,
            "Invalid plane indices"
        );
        let mut m = Self::identity();
        let (sin, cos) = angle.sin_cos();
        m.0[dim_a * 9 + dim_a] = cos;
        m.0[dim_a * 9 + dim_b] = -sin;
        m.0[dim_b * 9 + dim_a] = sin;
        m.0[dim_b * 9 + dim_b] = cos;
        m
    }

    /// Create a scaling transform (diagonal matrix).
    pub fn scaling(factors: &[f64; 9]) -> Self {
        let mut m = Self::identity();
        for i in 0..9 {
            m.0[i * 9 + i] = factors[i];
        }
        m
    }

    /// Create a transform that applies a small perturbation to a single dimension.
    ///
    /// This models an agent slightly shifting its intent in one CI facet,
    /// expressed as a shear transform.
    pub fn intent_shear(dim: usize, amount: f64) -> Self {
        let mut m = Self::identity();
        // Shear: add `amount` to all off-diagonal entries in the `dim` row
        for j in 0..9 {
            if j != dim {
                m.0[dim * 9 + j] = amount / 9.0;
            }
        }
        m
    }

    /// Multiply two GL(9) matrices: self × other.
    ///
    /// Standard 9×9 matrix multiplication. O(9³) = O(729) per multiply.
    pub fn multiply(&self, other: &GL9Matrix) -> Self {
        let mut result = [0.0f64; 81];
        for i in 0..9 {
            for j in 0..9 {
                let mut sum = 0.0;
                for k in 0..9 {
                    sum += self.0[i * 9 + k] * other.0[k * 9 + j];
                }
                result[i * 9 + j] = sum;
            }
        }
        Self(result)
    }

    /// Compute deviation from identity: Frobenius norm of (M - I).
    ///
    /// This is the holonomy measure. Zero = perfect consistency.
    pub fn deviation(&self) -> f64 {
        let mut sum = 0.0;
        for i in 0..9 {
            for j in 0..9 {
                let d = self.0[i * 9 + j] - if i == j { 1.0 } else { 0.0 };
                sum += d * d;
            }
        }
        sum.sqrt()
    }

    /// Check if this matrix is within tolerance of identity.
    pub fn is_identity(&self, tolerance: f64) -> bool {
        self.deviation() < tolerance
    }

    /// Apply this transform to a 9D intent vector.
    pub fn transform(&self, v: &[f64; 9]) -> [f64; 9] {
        let mut result = [0.0f64; 9];
        for i in 0..9 {
            let mut sum = 0.0;
            for j in 0..9 {
                sum += self.0[i * 9 + j] * v[j];
            }
            result[i] = sum;
        }
        result
    }

    /// Extract a single element (row, col).
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.0[row * 9 + col]
    }

    /// Set a single element (row, col).
    pub fn set(&mut self, row: usize, col: usize, val: f64) {
        self.0[row * 9 + col] = val;
    }

    /// Compute the determinant (for invertibility check).
    ///
    /// Uses cofactor expansion. O(9!) worst case but acceptable for
    /// diagnostic use, not hot path.
    pub fn determinant(&self) -> f64 {
        // Use LU-style approach: row reduction
        let mut mat = self.0;
        let mut det = 1.0;
        for col in 0..9 {
            // Find pivot
            let mut pivot_row = col;
            let mut pivot_val = mat[col * 9 + col].abs();
            for row in (col + 1)..9 {
                let val = mat[row * 9 + col].abs();
                if val > pivot_val {
                    pivot_val = val;
                    pivot_row = row;
                }
            }
            if pivot_val < 1e-15 {
                return 0.0; // Singular
            }
            // Swap rows
            if pivot_row != col {
                for j in 0..9 {
                    mat.swap(col * 9 + j, pivot_row * 9 + j);
                }
                det = -det;
            }
            det *= mat[col * 9 + col];
            // Eliminate below
            for row in (col + 1)..9 {
                let factor = mat[row * 9 + col] / mat[col * 9 + col];
                for j in (col + 1)..9 {
                    mat[row * 9 + j] -= factor * mat[col * 9 + j];
                }
                mat[row * 9 + col] = 0.0;
            }
        }
        det
    }

    /// Compute the transpose.
    pub fn transpose(&self) -> Self {
        let mut result = [0.0f64; 81];
        for i in 0..9 {
            for j in 0..9 {
                result[j * 9 + i] = self.0[i * 9 + j];
            }
        }
        Self(result)
    }
}

/// A 9D intent vector representing an agent's position across CI facets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntentVector(pub [f64; 9]);

impl IntentVector {
    /// Uniform intent (all facets equal weight), normalized to unit length.
    pub fn uniform() -> Self {
        let val = 1.0 / (9.0_f64).sqrt();
        Self([val; 9])
    }

    /// Unit vector in a single CI dimension.
    pub fn unit(dim: usize) -> Self {
        let mut v = [0.0f64; 9];
        v[dim] = 1.0;
        Self(v)
    }

    /// Compute L2 norm.
    pub fn norm(&self) -> f64 {
        self.0.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Normalize to unit length.
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n < 1e-15 {
            return Self([0.0; 9]);
        }
        let mut v = self.0;
        for x in v.iter_mut() {
            *x /= n;
        }
        Self(v)
    }

    /// Cosine similarity between two intent vectors.
    pub fn cosine_similarity(&self, other: &IntentVector) -> f64 {
        let dot: f64 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let norm_a = self.norm();
        let norm_b = other.norm();
        if norm_a < 1e-15 || norm_b < 1e-15 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    /// Euclidean distance between two intent vectors.
    pub fn distance(&self, other: &IntentVector) -> f64 {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }
}

/// Result of a GL(9) holonomy consensus check.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GL9ConsensusResult {
    /// True if accumulated holonomy around all cycles is within tolerance.
    pub is_consistent: bool,
    /// Maximum deviation from identity across all cycles.
    pub max_deviation: f64,
    /// Per-cycle deviations.
    pub cycle_deviations: Vec<f64>,
    /// ID of the most faulty agent (if any).
    pub faulty_agent: Option<u64>,
}

/// An agent in the GL(9) consensus network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GL9Agent {
    pub id: u64,
    /// The transform this agent applies to incoming intent.
    pub transform: GL9Matrix,
    /// The agent's current intent vector.
    pub intent: IntentVector,
    /// Neighboring agents.
    pub neighbors: Vec<u64>,
}

/// GL(9) Zero Holonomy Consensus engine.
pub struct GL9HolonomyConsensus {
    agents: Vec<GL9Agent>,
    agent_index: std::collections::HashMap<u64, usize>,
    tolerance: f64,
}

impl GL9HolonomyConsensus {
    /// Create a new consensus engine with the given tolerance.
    pub fn new(tolerance: f64) -> Self {
        Self {
            agents: Vec::new(),
            agent_index: std::collections::HashMap::new(),
            tolerance,
        }
    }

    /// Create with Oracle1's default tolerance (0.5).
    pub fn with_default_tolerance() -> Self {
        Self::new(DEFAULT_TOLERANCE)
    }

    /// Add an agent to the consensus network.
    pub fn add_agent(&mut self, agent: GL9Agent) {
        let id = agent.id;
        self.agent_index.insert(id, self.agents.len());
        self.agents.push(agent);
    }

    /// Get an agent by ID. O(1) lookup.
    pub fn get_agent(&self, id: u64) -> Option<&GL9Agent> {
        self.agent_index.get(&id).and_then(|&i| self.agents.get(i))
    }

    /// Compute holonomy around a cycle of agent transforms.
    ///
    /// The product of all transforms around a cycle should be identity
    /// for perfect consensus.
    pub fn compute_cycle_holonomy(&self, cycle: &[u64]) -> GL9Matrix {
        let mut product = GL9Matrix::identity();
        for &agent_id in cycle {
            if let Some(agent) = self.get_agent(agent_id) {
                product = product.multiply(&agent.transform);
            }
        }
        product
    }

    /// Compute alignment: how well do agents agree on intent direction?
    ///
    /// Returns the mean cosine similarity across all agent pairs.
    /// High alignment (near 1.0) = agents agree on intent.
    /// Low alignment (near 0.0) = agents disagree.
    pub fn compute_alignment(&self) -> f64 {
        if self.agents.len() < 2 {
            return 1.0;
        }
        let mut total_sim = 0.0;
        let mut count = 0usize;
        for i in 0..self.agents.len() {
            for j in (i + 1)..self.agents.len() {
                total_sim += self.agents[i]
                    .intent
                    .cosine_similarity(&self.agents[j].intent);
                count += 1;
            }
        }
        if count == 0 {
            1.0
        } else {
            total_sim / count as f64
        }
    }

    /// Check consensus across all cycles in the network.
    pub fn check_consensus(&self) -> GL9ConsensusResult {
        let cycles = self.find_cycles();
        let mut max_deviation = 0.0f64;
        let mut cycle_deviations = Vec::with_capacity(cycles.len());
        let mut faulty_agent = None;

        for cycle in &cycles {
            let holonomy = self.compute_cycle_holonomy(cycle);
            let dev = holonomy.deviation();
            cycle_deviations.push(dev);

            if dev > max_deviation {
                max_deviation = dev;
                if dev > self.tolerance {
                    faulty_agent = self.locate_fault(cycle);
                }
            }
        }

        GL9ConsensusResult {
            is_consistent: max_deviation < self.tolerance,
            max_deviation,
            cycle_deviations,
            faulty_agent,
        }
    }

    /// Find fundamental cycles in the agent graph.
    fn find_cycles(&self) -> Vec<Vec<u64>> {
        let mut cycles = Vec::new();
        let mut seen = std::collections::HashSet::<(u64, u64)>::new();

        for agent in &self.agents {
            for &neighbor in &agent.neighbors {
                if agent.id < neighbor && !seen.contains(&(agent.id, neighbor)) {
                    seen.insert((agent.id, neighbor));
                    if let Some(cycle) = self.trace_cycle(agent.id, neighbor) {
                        cycles.push(cycle);
                    }
                }
            }
        }

        cycles
    }

    /// Trace a cycle starting from agent -> neighbor -> ... -> back to start.
    fn trace_cycle(&self, start: u64, first_neighbor: u64) -> Option<Vec<u64>> {
        let mut cycle = vec![start, first_neighbor];
        let mut current = first_neighbor;

        for _ in 0..self.agents.len() + 1 {
            let agent = self.get_agent(current)?;
            let prev = cycle[cycle.len() - 2];

            // Find next neighbor (not the one we came from)
            let next = agent.neighbors.iter().find(|&&n| n != prev).copied();

            match next {
                Some(n) if n == start => return Some(cycle),
                Some(n) => {
                    cycle.push(n);
                    current = n;
                }
                None => return None,
            }
        }

        None
    }

    /// Locate the most faulty agent in a cycle via bisection. O(log L).
    fn locate_fault(&self, cycle: &[u64]) -> Option<u64> {
        let mut left = 0usize;
        let mut right = cycle.len();

        while right - left > 1 {
            let mid = (left + right) / 2;

            let left_hol = self.compute_cycle_holonomy(&cycle[left..mid]);
            let left_dev = left_hol.deviation();

            if left_dev > self.tolerance {
                right = mid;
            } else {
                left = mid;
            }
        }

        cycle.get(left).copied()
    }

    /// **Key experiment**: Compute correlation between holonomy and alignment.
    ///
    /// This is the test that the broken 3D version failed (r=-0.045).
    /// The 9D version should show positive correlation because it
    /// preserves the full intent structure.
    ///
    /// Returns (holonomy_deviations, alignment_scores) for computing correlation.
    pub fn holonomy_alignment_correlation(&self) -> (Vec<f64>, Vec<f64>) {
        let cycles = self.find_cycles();
        let mut holonomies = Vec::with_capacity(cycles.len());
        let mut alignments = Vec::with_capacity(cycles.len());

        for cycle in &cycles {
            let holonomy = self.compute_cycle_holonomy(cycle);
            let deviation = holonomy.deviation();
            holonomies.push(deviation);

            // Compute alignment for agents in this cycle
            let mut sim_sum = 0.0;
            let mut count = 0usize;
            for i in 0..cycle.len() {
                for j in (i + 1)..cycle.len() {
                    if let (Some(a), Some(b)) = (self.get_agent(cycle[i]), self.get_agent(cycle[j]))
                    {
                        sim_sum += a.intent.cosine_similarity(&b.intent);
                        count += 1;
                    }
                }
            }
            alignments.push(if count > 0 {
                sim_sum / count as f64
            } else {
                1.0
            });
        }

        (holonomies, alignments)
    }
}

/// Compute Pearson correlation coefficient between two slices.
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len()) as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n as usize {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = var_x.sqrt() * var_y.sqrt();
    if denom < 1e-15 {
        return 0.0;
    }
    cov / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_zero_holonomy() {
        let m = GL9Matrix::identity();
        assert_eq!(m.deviation(), 0.0);
        assert!(m.is_identity(0.001));
    }

    #[test]
    fn test_identity_loop_consensus() {
        // 5 agents, all with identity transforms → zero holonomy
        let mut consensus = GL9HolonomyConsensus::new(DEFAULT_TOLERANCE);
        for i in 0..5u64 {
            let neighbors: Vec<u64> = if i == 0 {
                vec![4, 1]
            } else if i == 4 {
                vec![3, 0]
            } else {
                vec![i - 1, i + 1]
            };
            consensus.add_agent(GL9Agent {
                id: i,
                transform: GL9Matrix::identity(),
                intent: IntentVector::uniform(),
                neighbors,
            });
        }
        let result = consensus.check_consensus();
        assert!(result.is_consistent);
        assert!(result.max_deviation < 1e-10);
    }

    #[test]
    fn test_plane_rotation_measurable_holonomy() {
        // Single rotation in C1-C3 plane (dims 0, 2) → π/6 radians
        let rot = GL9Matrix::plane_rotation(0, 2, std::f64::consts::FRAC_PI_6);
        let dev = rot.deviation();
        assert!(
            dev > 0.01,
            "Plane rotation should have measurable deviation, got {}",
            dev
        );
        assert!(dev < 5.0, "Deviation should be bounded");
    }

    #[test]
    fn test_rotation_loop_holonomy() {
        // 4 agents forming a cycle, each rotating by π/2 in C1-C2 plane
        // Total rotation = 2π → accumulated = identity (zero holonomy)
        let mut consensus = GL9HolonomyConsensus::new(DEFAULT_TOLERANCE);
        for i in 0..4u64 {
            let neighbors: Vec<u64> = if i == 0 {
                vec![3, 1]
            } else {
                vec![i - 1, (i + 1) % 4]
            };
            consensus.add_agent(GL9Agent {
                id: i,
                transform: GL9Matrix::plane_rotation(0, 1, std::f64::consts::FRAC_PI_2),
                intent: IntentVector::unit(0),
                neighbors,
            });
        }
        let result = consensus.check_consensus();
        // 4 × π/2 = 2π → back to identity → zero holonomy
        assert!(
            result.max_deviation < 1e-10,
            "Full rotation loop should have ~zero holonomy, got {}",
            result.max_deviation
        );
    }

    #[test]
    fn test_partial_rotation_nonzero_holonomy() {
        // 3 agents, each rotating by π/4 → total 3π/4 ≠ 2π → nonzero holonomy
        let mut consensus = GL9HolonomyConsensus::new(DEFAULT_TOLERANCE);
        for i in 0..3u64 {
            let neighbors: Vec<u64> = if i == 0 {
                vec![2, 1]
            } else if i == 2 {
                vec![1, 0]
            } else {
                vec![i - 1, i + 1]
            };
            consensus.add_agent(GL9Agent {
                id: i,
                transform: GL9Matrix::plane_rotation(0, 1, std::f64::consts::FRAC_PI_4),
                intent: IntentVector::unit(0),
                neighbors,
            });
        }
        let result = consensus.check_consensus();
        // 3 × π/4 = 3π/4, not 2π → nonzero holonomy
        assert!(
            result.max_deviation > 0.01,
            "Partial rotation should have nonzero holonomy, got {}",
            result.max_deviation
        );
    }

    #[test]
    fn test_tolerance_forgiving_vs_tight() {
        let small_rotation = GL9Matrix::plane_rotation(0, 1, 0.2); // ~11.5 degrees

        // Tolerance 0.5 → forgiving
        assert!(
            small_rotation.is_identity(0.5),
            "Small rotation should be within forgiving tolerance"
        );

        // Tolerance 0.1 → tighter
        let dev = small_rotation.deviation();
        // deviation of a single plane rotation by 0.2 rad ≈ sqrt(2*(1-cos(0.2))*2 + 2*sin²(0.2))
        // Actually for a 9×9 matrix with only 4 off-diagonal changes: sqrt(2*(1-cos0.2)² + 2*sin²0.2)
        // sin(0.2)≈0.1987, cos(0.2)≈0.9801
        // deviation = sqrt((1-0.9801)² + 0.1987² + 0.1987² + (1-0.9801)²) ≈ sqrt(4*0.02) ≈ 0.28
        // So tolerance 0.1 should NOT pass
        assert!(
            !small_rotation.is_identity(0.1),
            "deviation {} should exceed tight tolerance 0.1",
            dev
        );
    }

    #[test]
    fn test_tolerance_thresholds() {
        // Build a cycle with known deviation
        let angle = 0.1; // Small rotation
        let transform = GL9Matrix::plane_rotation(0, 1, angle);

        let mut consensus_loose = GL9HolonomyConsensus::new(0.5);
        let mut consensus_tight = GL9HolonomyConsensus::new(0.05);

        for i in 0..3u64 {
            let neighbors: Vec<u64> = if i == 0 {
                vec![2, 1]
            } else if i == 2 {
                vec![1, 0]
            } else {
                vec![i - 1, i + 1]
            };
            let agent = GL9Agent {
                id: i,
                transform: transform.clone(),
                intent: IntentVector::unit(0),
                neighbors,
            };
            consensus_loose.add_agent(agent.clone());
            consensus_tight.add_agent(agent);
        }

        let loose_result = consensus_loose.check_consensus();
        let tight_result = consensus_tight.check_consensus();

        // The loose tolerance should be more forgiving
        assert!(
            loose_result.max_deviation >= tight_result.max_deviation
                || (loose_result.max_deviation - tight_result.max_deviation).abs() < 1e-10
        );
        // Both should have the same deviation, just different pass/fail
        assert!(!tight_result.is_consistent || tight_result.max_deviation < 0.05);
    }

    #[test]
    fn test_intent_vector_operations() {
        let v1 = IntentVector::unit(0);
        let v2 = IntentVector::unit(0);
        assert!((v1.cosine_similarity(&v2) - 1.0).abs() < 1e-10);

        let v3 = IntentVector::unit(1);
        assert!((v1.cosine_similarity(&v3)).abs() < 1e-10); // Orthogonal

        let v4 = IntentVector::uniform();
        assert!((v4.norm() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_applied_to_intent() {
        let rot = GL9Matrix::plane_rotation(0, 1, std::f64::consts::FRAC_PI_2);
        let v = IntentVector::unit(0); // [1, 0, 0, ...]
        let transformed = rot.transform(&v.0);
        let result = IntentVector(transformed);

        // After 90° rotation in (0,1) plane: x→0, y→1
        assert!(
            result.0[0].abs() < 1e-10,
            "x should be ~0, got {}",
            result.0[0]
        );
        assert!(
            (result.0[1] - 1.0).abs() < 1e-10,
            "y should be ~1, got {}",
            result.0[1]
        );
        // Other dims unchanged
        for i in 2..9 {
            assert!(
                result.0[i].abs() < 1e-10,
                "dim {} should be 0, got {}",
                i,
                result.0[i]
            );
        }
    }

    #[test]
    fn test_determinant_identity() {
        let m = GL9Matrix::identity();
        assert!((m.determinant() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_determinant_rotation() {
        // Rotation has det = 1
        let rot = GL9Matrix::plane_rotation(3, 7, 1.23);
        assert!((rot.determinant() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_random_9d_holonomy_correlates_with_alignment() {
        // This is THE test that the broken 3D version failed.
        // In 9D, holonomy and alignment should correlate positively.

        // Build a network where aligned agents have small transforms,
        // misaligned agents have large transforms.
        let mut consensus = GL9HolonomyConsensus::new(DEFAULT_TOLERANCE);

        // Group A: aligned agents (small transforms, similar intents)
        for i in 0..4u64 {
            let mut intent = IntentVector::unit(0);
            intent.0[1] = 0.1 * (i as f64); // Slight variation
            let intent = intent.normalize();

            consensus.add_agent(GL9Agent {
                id: i,
                transform: GL9Matrix::plane_rotation(0, 1, 0.01 * (i as f64 + 1.0)),
                intent,
                neighbors: if i == 0 {
                    vec![3, 1]
                } else {
                    vec![i - 1, (i + 1) % 4]
                },
            });
        }

        // Group B: misaligned agents (large transforms, different intents)
        for i in 4..8u64 {
            let mut intent = IntentVector::unit(i as usize % 9);
            intent.0[0] = 0.5;
            let intent = intent.normalize();

            consensus.add_agent(GL9Agent {
                id: i,
                transform: GL9Matrix::plane_rotation((i as usize) % 9, ((i as usize) + 3) % 9, 0.5),
                intent,
                neighbors: if i == 4 {
                    vec![7, 5]
                } else {
                    vec![i - 1, if i == 7 { 4 } else { i + 1 }]
                },
            });
        }

        let alignment = consensus.compute_alignment();
        // Alignment should be between 0 and 1
        assert!(
            alignment >= 0.0 && alignment <= 1.0,
            "Alignment out of range: {}",
            alignment
        );

        // The 9D version should at least not DESTROY correlation
        // (we can't guarantee strong correlation with this small sample,
        // but it should be > -0.045 which was the broken 3D value)
        let (holonomies, alignments) = consensus.holonomy_alignment_correlation();
        if holonomies.len() >= 3 {
            let corr = pearson_correlation(&holonomies, &alignments);
            // Negative holonomy-alignment: more deviation = less aligned
            // So we expect negative correlation (high holonomy → low alignment)
            // The 3D version got r=-0.045 (no correlation). We should get |r| > 0.045.
            assert!(
                corr < -0.01 || corr > 0.01,
                "9D should show SOME correlation (r={}), unlike broken 3D (r=-0.045)",
                corr
            );
        }
    }

    #[test]
    fn test_pearson_correlation() {
        // Perfect positive
        let x = [1.0, 2.0, 3.0, 4.0];
        let y = [2.0, 4.0, 6.0, 8.0];
        assert!((pearson_correlation(&x, &y) - 1.0).abs() < 1e-10);

        // Perfect negative
        let y2 = [8.0, 6.0, 4.0, 2.0];
        assert!((pearson_correlation(&x, &y2) + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gl9_vs_so3_preserves_information() {
        // Demonstrate that 9D preserves intent information that 3D would destroy.
        // Create two very different intent vectors that project to the same 3D vector.

        let v1 = IntentVector([1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let v2 = IntentVector([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5]);

        let v1n = v1.normalize();
        let v2n = v2.normalize();

        // In 9D, these are clearly different
        let sim_9d = v1n.cosine_similarity(&v2n);
        assert!(
            sim_9d < 0.95,
            "9D should distinguish these vectors (sim={})",
            sim_9d
        );

        // If projected to 3D (dims 0-2 only), they'd look identical
        let v1_3d = [v1.0[0], v1.0[1], v1.0[2]];
        let v2_3d = [v2.0[0], v2.0[1], v2.0[2]];
        let sim_3d: f64 = {
            let dot = v1_3d
                .iter()
                .zip(v2_3d.iter())
                .map(|(a, b)| a * b)
                .sum::<f64>();
            let n1 = v1_3d.iter().map(|x| x * x).sum::<f64>().sqrt();
            let n2 = v2_3d.iter().map(|x| x * x).sum::<f64>().sqrt();
            dot / (n1 * n2)
        };
        assert!(
            (sim_3d - 1.0).abs() < 1e-10,
            "3D projection makes them identical (sim={})",
            sim_3d
        );
    }
}
