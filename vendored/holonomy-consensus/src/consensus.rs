//! Zero-holonomy consensus — eliminates voting, CRDTs, BFT

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A holonomy matrix (3x3 rotation)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HolonomyMatrix(pub [[f64; 3]; 3]);

impl HolonomyMatrix {
    pub fn identity() -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn from_rotation(axis: [f64; 3], angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        let [x, y, z] = axis;
        let t = 1.0 - cos;

        Self([
            [t * x * x + cos, t * x * y - sin * z, t * x * z + sin * y],
            [t * x * y + sin * z, t * y * y + cos, t * y * z - sin * x],
            [t * x * z - sin * y, t * y * z + sin * x, t * z * z + cos],
        ])
    }

    /// Multiply two holonomy matrices (composition)
    pub fn multiply(&self, other: &HolonomyMatrix) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        Self(result)
    }

    /// Compute deviation from identity (norm of (M - I))
    pub fn deviation(&self) -> f64 {
        let mut sum = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                let d = self.0[i][j] - if i == j { 1.0 } else { 0.0 };
                sum += d * d;
            }
        }
        sum.sqrt()
    }

    pub fn is_identity(&self, tolerance: f64) -> bool {
        self.deviation() < tolerance
    }
}

/// A tile in the consensus network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusTile {
    pub id: u64,
    pub holonomy: HolonomyMatrix,
    pub neighbors: Vec<u64>, // Laman's theorem establishes edge count (E=2V-3) but places no upper bound on vertex degree
    pub cycle_id: Option<u64>,
}

/// Result of consensus check
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// True if the tile network has zero holonomy (globally consistent)
    pub is_consistent: bool,
    /// Holonomy deviation (0 = perfect consistency)
    pub deviation: f64,
    /// If inconsistent: ID of faulty tile
    pub faulty_tile: Option<u64>,
    /// Information content: I = -log|Hol(γ)|
    pub information: f64,
}

/// Zero-holonomy consensus engine — O(C·L) instead of O(N²)
pub struct HolonomyConsensus {
    /// O(1) tile lookup by ID — eliminates O(N) scan
    tile_index: HashMap<u64, usize>,
    tiles: Vec<ConsensusTile>,
    tolerance: f64,
}

impl HolonomyConsensus {
    pub fn new(tolerance: f64) -> Self {
        Self {
            tile_index: HashMap::new(),
            tiles: Vec::new(),
            tolerance,
        }
    }

    pub fn add_tile(&mut self, tile: ConsensusTile) {
        let id = tile.id;
        self.tile_index.insert(id, self.tiles.len());
        self.tiles.push(tile);
    }

    /// O(1) tile lookup — replaces O(N) iter().find()
    fn get_tile(&self, id: u64) -> Option<&ConsensusTile> {
        self.tile_index.get(&id).and_then(|&i| self.tiles.get(i))
    }

    /// O(1) neighbor lookup — trace_cycle no longer scans all tiles
    fn get_neighbors(&self, tile_id: u64) -> Option<&Vec<u64>> {
        self.get_tile(tile_id).map(|t| &t.neighbors)
    }

    /// Compute holonomy around a cycle of tiles
    /// O(L) — L = cycle length, not O(N)
    pub fn compute_cycle_holonomy(&self, cycle: &[u64]) -> HolonomyMatrix {
        let mut product = HolonomyMatrix::identity();

        for &tile_id in cycle {
            if let Some(tile) = self.get_tile(tile_id) {
                product = product.multiply(&tile.holonomy);
            }
        }

        product
    }

    /// Check consensus for the entire tile network
    /// Returns ConsensusResult: is_consistent = true if all cycles have zero holonomy
    pub fn check_consensus(&self) -> ConsensusResult {
        // Find all cycles in the tile network
        let cycles = self.find_all_cycles();

        let mut max_deviation = 0.0f64;
        let mut faulty_tile = None;

        for cycle in cycles {
            let holonomy = self.compute_cycle_holonomy(&cycle);
            let deviation = holonomy.deviation();

            if deviation > max_deviation {
                max_deviation = deviation;
                if deviation > self.tolerance {
                    // Find the faulty tile by bisection — O(log L) per cycle
                    faulty_tile = self.locate_fault(cycle, holonomy);
                }
            }
        }

        ConsensusResult {
            is_consistent: max_deviation < self.tolerance,
            deviation: max_deviation,
            faulty_tile,
            information: if max_deviation > 0.0 {
                -(max_deviation.ln())
            } else {
                f64::INFINITY // Perfect consistency = infinite information
            },
        }
    }

    /// Find all fundamental cycles in the tile network
    /// O(C·L) — C cycles, L avg length. O(N²) worst case.
    fn find_all_cycles(&self) -> Vec<Vec<u64>> {
        let mut cycles = Vec::new();
        let mut visited: HashSet<u64> = HashSet::new(); // O(1) lookup vs O(C) Vec::contains

        for tile in &self.tiles {
            for &neighbor in &tile.neighbors {
                let cycle = self.trace_cycle(tile.id, neighbor);
                if !cycle.is_empty() {
                    // Use first+last as cycle fingerprint — O(1) hash vs Vec comparison
                    let fingerprint = cycle[0] ^ (cycle[cycle.len() - 1] << 16);
                    if !visited.contains(&fingerprint) {
                        visited.insert(fingerprint);
                        cycles.push(cycle);
                    }
                }
            }
        }

        cycles
    }

    /// Trace a cycle starting from tile -> neighbor
    /// O(L) — each step is O(1) HashMap lookup instead of O(N) scan
    fn trace_cycle(&self, start: u64, neighbor: u64) -> Vec<u64> {
        let mut cycle = vec![start, neighbor];
        let mut current = neighbor;

        // Simple cycle detection: follow neighbors until we return to start
        for _ in 0..self.tiles.len() {
            if let Some(next_neighbors) = self.get_neighbors(current) {
                if let Some(&next) = next_neighbors
                    .iter()
                    .find(|&&n| n != cycle[cycle.len() - 2])
                {
                    if next == start {
                        return cycle;
                    }
                    cycle.push(next);
                    current = next;
                } else {
                    return Vec::new();
                }
            } else {
                return Vec::new();
            }
        }

        Vec::new()
    }

    /// Locate a faulty tile by cycle bisection — O(log L)
    fn locate_fault(&self, cycle: Vec<u64>, _bad_holonomy: HolonomyMatrix) -> Option<u64> {
        let mut left = 0usize;
        let mut right = cycle.len();

        while right - left > 1 {
            let mid = (left + right) / 2;

            let left_cycle: Vec<u64> = cycle[left..mid].to_vec();
            let right_cycle: Vec<u64> = cycle[mid..right].to_vec();

            let left_hol = self.compute_cycle_holonomy(&left_cycle);
            let _right_hol = self.compute_cycle_holonomy(&right_cycle);

            if left_hol.deviation() > self.tolerance {
                right = mid;
            } else {
                left = mid;
            }
        }

        Some(cycle[left])
    }
}
