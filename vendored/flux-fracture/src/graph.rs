//! Dependency graph, connected-component fracture, and adaptive re-fracture.

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// DependencyGraph — Bipartite constraint×dimension adjacency
// ---------------------------------------------------------------------------

/// Bipartite graph: constraints (rows) × dimensions (columns).
///
/// Edge (i, j) exists iff constraint `i` involves dimension `j`.
#[derive(Clone, Debug)]
pub struct DependencyGraph {
    /// Row-major adjacency: `adj[c * n_dimensions + d]` is 1 if connected.
    adj: Vec<u8>,
    pub n_constraints: usize,
    pub n_dimensions: usize,
    pub constraint_names: Vec<String>,
    pub dimension_names: Vec<String>,
}

impl DependencyGraph {
    /// Build from a flat adjacency matrix (row-major, `n_constraints × n_dimensions`).
    pub fn from_adjacency(
        adj: Vec<u8>,
        n_constraints: usize,
        n_dimensions: usize,
    ) -> Self {
        assert_eq!(adj.len(), n_constraints * n_dimensions, "adjacency size mismatch");
        let constraint_names = (0..n_constraints).map(|i| format!("c{i}")).collect();
        let dimension_names = (0..n_dimensions).map(|j| format!("d{j}")).collect();
        Self { adj, n_constraints, n_dimensions, constraint_names, dimension_names }
    }

    /// Build from per-constraint dimension masks.
    ///
    /// `masks[i]` contains the dimension indices that constraint `i` involves.
    pub fn from_masks(masks: &[&[usize]]) -> Self {
        let n_c = masks.len();
        let n_d = masks.iter().flat_map(|m| m.iter().copied()).max().map_or(0, |m| m + 1);
        let mut adj = vec![0u8; n_c * n_d];
        for (i, m) in masks.iter().enumerate() {
            for &d in *m {
                adj[i * n_d + d] = 1;
            }
        }
        let constraint_names = (0..n_c).map(|i| format!("c{i}")).collect();
        let dimension_names = (0..n_d).map(|j| format!("d{j}")).collect();
        Self { adj, n_constraints: n_c, n_dimensions: n_d, constraint_names, dimension_names }
    }

    /// Build an identity graph (each constraint touches exactly its own dimension).
    pub fn identity(n: usize) -> Self {
        let mut adj = vec![0u8; n * n];
        for i in 0..n {
            adj[i * n + i] = 1;
        }
        let constraint_names = (0..n).map(|i| format!("c{i}")).collect();
        let dimension_names = (0..n).map(|j| format!("d{j}")).collect();
        Self { adj, n_constraints: n, n_dimensions: n, constraint_names, dimension_names }
    }

    #[inline]
    fn idx(&self, c: usize, d: usize) -> usize {
        c * self.n_dimensions + d
    }

    /// Whether constraint `c` involves dimension `d`.
    #[inline]
    pub fn involves(&self, c: usize, d: usize) -> bool {
        self.adj[self.idx(c, d)] != 0
    }

    /// Dimensions involved in constraint `c`.
    pub fn constraint_dims(&self, c: usize) -> Vec<usize> {
        let base = c * self.n_dimensions;
        (0..self.n_dimensions)
            .filter(|&d| self.adj[base + d] != 0)
            .collect()
    }

    /// Constraints involving dimension `d`.
    pub fn dim_constraints(&self, d: usize) -> Vec<usize> {
        (0..self.n_constraints)
            .filter(|&c| self.adj[self.idx(c, d)] != 0)
            .collect()
    }

    /// Mutable access to set an edge.
    pub fn set_edge(&mut self, c: usize, d: usize, val: u8) {
        let idx = c * self.n_dimensions + d;
        self.adj[idx] = val;
    }

    /// Fill a rectangular block with 1s.
    pub fn fill_block(&mut self, c_start: usize, c_end: usize, d_start: usize, d_end: usize) {
        let nd = self.n_dimensions;
        for c in c_start..c_end {
            for d in d_start..d_end {
                self.adj[c * nd + d] = 1;
            }
        }
    }

    /// Graph statistics.
    pub fn stats(&self) -> GraphStats {
        // Quick connected-component count without allocating full FractureResult
        let mut visited_c = vec![false; self.n_constraints];
        let mut visited_d = vec![false; self.n_dimensions];
        let mut n_blocks = 0usize;
        let mut largest = 0usize;

        for seed in 0..self.n_constraints {
            if visited_c[seed] {
                continue;
            }
            let mut comp_c_count = 0usize;
            let mut queue: VecDeque<(bool, usize)> = VecDeque::new(); // true=constraint, false=dim
            queue.push_back((true, seed));
            while let Some((is_c, idx)) = queue.pop_front() {
                if is_c {
                    if visited_c[idx] { continue; }
                    visited_c[idx] = true;
                    comp_c_count += 1;
                    for d in self.constraint_dims(idx) {
                        if !visited_d[d] { queue.push_back((false, d)); }
                    }
                } else {
                    if visited_d[idx] { continue; }
                    visited_d[idx] = true;
                    for c in self.dim_constraints(idx) {
                        if !visited_c[c] { queue.push_back((true, c)); }
                    }
                }
            }
            n_blocks += 1;
            largest = largest.max(comp_c_count);
        }

        let speedup = if largest > 0 && self.n_constraints > 0 {
            self.n_constraints as f64 / largest as f64
        } else {
            1.0
        };

        GraphStats { n_blocks, largest_block: largest, speedup_potential: speedup }
    }
}

/// Statistics about the graph's connected-component structure.
#[derive(Clone, Debug)]
pub struct GraphStats {
    pub n_blocks: usize,
    pub largest_block: usize,
    pub speedup_potential: f64,
}

// ---------------------------------------------------------------------------
// Block — One independent block of the fractured system
// ---------------------------------------------------------------------------

/// One independent block of the fractured system.
#[derive(Clone, Debug)]
pub struct Block {
    pub constraint_indices: Vec<usize>,
    pub dimension_indices: Vec<usize>,
    pub size: usize,
}

// ---------------------------------------------------------------------------
// FractureResult — The fractured system
// ---------------------------------------------------------------------------

/// Result of fracturing a constraint system into independent blocks.
#[derive(Clone, Debug)]
pub struct FractureResult {
    pub blocks: Vec<Block>,
    pub graph: DependencyGraph,
    pub n_blocks: usize,
    pub largest_block_size: usize,
    pub speedup_potential: f64,
}

impl FractureResult {
    fn new(blocks: Vec<Block>, graph: DependencyGraph) -> Self {
        let n_blocks = blocks.len();
        let largest_block_size = blocks.iter().map(|b| b.size).max().unwrap_or(0);
        let speedup_potential = if largest_block_size > 0 && graph.n_constraints > 0 {
            graph.n_constraints as f64 / largest_block_size as f64
        } else {
            1.0
        };
        Self { blocks, graph, n_blocks, largest_block_size, speedup_potential }
    }
}

// ---------------------------------------------------------------------------
// Fracturer — BFS connected-component detection
// ---------------------------------------------------------------------------

/// Fractures a constraint system by finding connected components
/// of the constraint–dimension bipartite dependency graph.
pub struct Fracturer;

impl Fracturer {
    pub fn new() -> Self { Self }

    /// Find connected components via BFS on the bipartite graph.
    pub fn fracture(&self, graph: &DependencyGraph) -> FractureResult {
        let mut visited_c = vec![false; graph.n_constraints];
        let mut visited_d = vec![false; graph.n_dimensions];
        let mut blocks: Vec<Block> = Vec::new();

        for seed_c in 0..graph.n_constraints {
            if visited_c[seed_c] {
                continue;
            }
            let mut comp_c: Vec<usize> = Vec::new();
            let mut comp_d: Vec<usize> = Vec::new();
            let mut queue: VecDeque<(bool, usize)> = VecDeque::new();
            queue.push_back((true, seed_c));

            while let Some((is_constraint, idx)) = queue.pop_front() {
                if is_constraint {
                    if visited_c[idx] { continue; }
                    visited_c[idx] = true;
                    comp_c.push(idx);
                    for d in graph.constraint_dims(idx) {
                        if !visited_d[d] { queue.push_back((false, d)); }
                    }
                } else {
                    if visited_d[idx] { continue; }
                    visited_d[idx] = true;
                    comp_d.push(idx);
                    for c in graph.dim_constraints(idx) {
                        if !visited_c[c] { queue.push_back((true, c)); }
                    }
                }
            }

            comp_c.sort_unstable();
            comp_d.sort_unstable();
            let size = comp_c.len();
            blocks.push(Block { constraint_indices: comp_c, dimension_indices: comp_d, size });
        }

        // Isolated dimensions (no constraints)
        for d in 0..graph.n_dimensions {
            if !visited_d[d] {
                blocks.push(Block {
                    constraint_indices: Vec::new(),
                    dimension_indices: vec![d],
                    size: 0,
                });
            }
        }

        FractureResult::new(blocks, graph.clone())
    }
}

// ---------------------------------------------------------------------------
// AdaptiveFracturer — Dynamic re-fracturing on structure change
// ---------------------------------------------------------------------------

/// Change in fracture structure between two states.
#[derive(Clone, Debug)]
pub struct FractureDelta {
    pub blocks_before: usize,
    pub blocks_after: usize,
    pub structure_changed: bool,
    pub max_block_size_delta: isize,
    pub speedup_delta: f64,
}

/// Monitors constraint system structure and re-fractures when needed.
pub struct AdaptiveFracturer {
    last_result: Option<FractureResult>,
    fracturer: Fracturer,
    pub refracture_count: usize,
}

impl AdaptiveFracturer {
    pub fn new() -> Self {
        Self { last_result: None, fracturer: Fracturer::new(), refracture_count: 0 }
    }

    /// Re-fracture with new graph, returning result and delta from last.
    pub fn update(&mut self, graph: &DependencyGraph) -> (FractureResult, FractureDelta) {
        let new_result = self.fracturer.fracture(graph);

        let delta = match &self.last_result {
            None => FractureDelta {
                blocks_before: 0,
                blocks_after: new_result.n_blocks,
                structure_changed: true,
                max_block_size_delta: new_result.largest_block_size as isize,
                speedup_delta: new_result.speedup_potential,
            },
            Some(old) => {
                let old_sizes: Vec<usize> = old.blocks.iter().map(|b| b.size).collect();
                let new_sizes: Vec<usize> = new_result.blocks.iter().map(|b| b.size).collect();
                let mut old_sorted = old_sizes.clone();
                let mut new_sorted = new_sizes.clone();
                old_sorted.sort_unstable();
                new_sorted.sort_unstable();
                let changed = old_sorted != new_sorted;
                FractureDelta {
                    blocks_before: old.n_blocks,
                    blocks_after: new_result.n_blocks,
                    structure_changed: changed,
                    max_block_size_delta: new_result.largest_block_size as isize - old.largest_block_size as isize,
                    speedup_delta: new_result.speedup_potential - old.speedup_potential,
                }
            }
        };

        if delta.structure_changed {
            self.refracture_count += 1;
        }

        self.last_result = Some(new_result.clone());
        (new_result, delta)
    }

    pub fn current(&self) -> Option<&FractureResult> {
        self.last_result.as_ref()
    }
}
