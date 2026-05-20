//! fleet-homology — Topological invariants for fleet emergence detection
//!
//! Mathematical foundation:
//! - β₀ (H⁰) = number of connected components (fleet unity)
//! - β₁ (H¹) = E - V + C = number of independent cycles = redundant paths
//! - β₂ (H²) = voids in the constraint network (not used for 2D fleet)
//!
//! The key insight: β₁ > 0 means the fleet has redundant constraint paths.
//! Redundant paths = emergent behavior (the whole is more than the sum of parts).
//!
//! # Emergence Detection Threshold
//! 
//! | β₁ | Fleet State | Interpretation |
//! |-----|-------------|----------------|
//! | 0 | Rigidity only | No redundancy, no emergence |
//! | 1 | Minimal cycle | One redundant path |
//! | V-2 | Laman boundary | Exactly rigid, no overconstraint |
//! | > V-2 | Over-constrained | Emergence detected (≥ Laman + extra edges) |
//!
//! For V=5 fleet with complete graph (E=10):
//! β₁ = 10 - 5 + 1 = 6 > 3 (V-2) → OVER-CONSTRAINED (emergence)

use serde::{Deserialize, Serialize};

/// A fleet agent (vertex in the cellular complex)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vertex(pub u64);

/// An edge between two vertices (1-cell in the cellular complex)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge(pub u64, pub u64);

impl Edge {
    pub fn contains(&self, v: u64) -> bool { self.0 == v || self.1 == v }
    pub fn other(&self, v: u64) -> Option<u64> {
        if self.0 == v { Some(self.1) } else if self.1 == v { Some(self.0) } else { None }
    }
}

/// The fleet cellular complex (vertices + edges)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Complex {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

impl Complex {
    pub fn new() -> Self { Self { vertices: Vec::new(), edges: Vec::new() } }

    pub fn add_vertex(&mut self, v: u64) {
        if !self.vertices.iter().any(|Vertex(x)| *x == v) {
            self.vertices.push(Vertex(v));
        }
    }

    pub fn add_edge(&mut self, a: u64, b: u64) {
        self.add_vertex(a);
        self.add_vertex(b);
        if !self.edges.iter().any(|e| e.0 == a && e.1 == b || e.0 == b && e.1 == a) {
            self.edges.push(Edge(a, b));
        }
    }

    pub fn V(&self) -> usize { self.vertices.len() }
    pub fn E(&self) -> usize { self.edges.len() }

    /// Find connected components using BFS
    pub fn components(&self) -> Vec<Vec<u64>> {
        let mut visited = vec![false; self.V()];
        let verts: Vec<u64> = self.vertices.iter().map(|Vertex(v)| *v).collect();
        let mut result = Vec::new();

        for (i, v) in verts.iter().enumerate() {
            if !visited[i] {
                let mut component = Vec::new();
                let mut queue = vec![*v];
                visited[i] = true;

                while let Some(current) = queue.pop() {
                    component.push(current);
                    for (j, edge) in self.edges.iter().enumerate() {
                        if !visited[self.vertices.iter().position(|Vertex(x)| *x == edge.0).unwrap()] {
                            continue;
                        }
                        // BFS from current
                        if let Some(nbr) = edge.other(current) {
                            let nbr_idx = verts.iter().position(|x| *x == nbr);
                            if let Some(idx) = nbr_idx {
                                if !visited[idx] {
                                    visited[idx] = true;
                                    queue.push(nbr);
                                }
                            }
                        }
                    }
                }
                result.push(component);
            }
        }
        result
    }

    /// Betti number β₀ = number of connected components
    pub fn beta_0(&self) -> usize { self.components().len() }

    /// Betti number β₁ = E - V + C (independent cycles)
    pub fn beta_1(&self) -> usize {
        let V = self.V();
        let E = self.E();
        let C = self.beta_0();
        if E >= V { E - V + C } else { 0 }
    }

    /// The chain complex dimension formula:
    /// rank(C₁) - rank(C₀) = β₁ where C₀, C₁ are boundary operators
    /// For our purposes: β₁ = E - V + C

    /// Check if fleet has emergence (β₁ > V - 2 for V >= 3)
    /// This means there are redundant paths beyond Laman rigidity
    pub fn has_emergence(&self) -> bool {
        let V = self.V();
        if V < 3 { return false; }
        let threshold = V.saturating_sub(2);  // V-2
        self.beta_1() > threshold
    }

    /// Laman boundary: β₁ = V - 2 exactly
    pub fn is_laman_rigid(&self) -> bool {
        let V = self.V();
        if V < 3 { return true; }  // Small graphs are trivially rigid
        self.beta_1() == V.saturating_sub(2)
    }

    /// Full homology report
    pub fn homology_report(&self) -> HomologyReport {
        let beta_0 = self.beta_0();
        let beta_1 = self.beta_1();
        let beta_2 = 0;  // 2D fleet, no voids
        let V = self.V();
        let E = self.E();
        let C = beta_0;

        HomologyReport {
            V, E, C,
            beta_0,
            beta_1,
            beta_2,
            is_rigid: self.is_laman_rigid(),
            has_emergence: self.has_emergence(),
            lamant_boundary: V.saturating_sub(2),
            interpretation: Self::interpret(beta_0, beta_1, V),
        }
    }

    fn interpret(beta_0: usize, beta_1: usize, V: usize) -> String {
        if beta_0 > 1 {
            format!("FLEET SPLIT: {} disconnected components", beta_0)
        } else if V < 3 {
            "TOO SMALL: fleet needs ≥3 agents for meaningful homology".to_string()
        } else if beta_1 < V - 2 {
            format!("UNDER-CONSTRAINED: β₁={} < V-2={} (need more trust edges)", beta_1, V-2)
        } else if beta_1 == V - 2 {
            "LAMAN-RIGID: exactly rigid, no redundancy, no emergence".to_string()
        } else {
            format!("EMERGENCE: β₁={} > V-2={} ({} redundant paths detected)", 
                beta_1, V-2, beta_1 - (V - 2))
        }
    }

    /// Build from edge list
    pub fn from_edges(edges: &[(u64, u64)]) -> Self {
        let mut c = Self::new();
        for &(a, b) in edges {
            c.add_edge(a, b);
        }
        c
    }
}

impl Default for Complex { fn default() -> Self { Self::new() } }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomologyReport {
    pub V: usize,
    pub E: usize,
    pub C: usize,
    pub beta_0: usize,  // Connected components
    pub beta_1: usize,  // Independent cycles (redundant paths)
    pub beta_2: usize,  // Voids (0 for 2D fleet)
    pub is_rigid: bool,
    pub has_emergence: bool,
    pub lamant_boundary: usize,
    pub interpretation: String,
}

impl HomologyReport {
    pub fn summary(&self) -> String {
        format!(
            "Fleet homology: V={}, E={}, C={}\n\
             β₀={} (components), β₁={} (cycles), β₂={} (voids)\n\
             Laman boundary: β₁={} expected, {} observed\n\
             Status: {}",
            self.V, self.E, self.C,
            self.beta_0, self.beta_1, self.beta_2,
            self.lamant_boundary, self.beta_1,
            if self.has_emergence { "⚡ EMERGENCE DETECTED" } else if self.is_rigid { "✓ RIGID" } else { "✗ UNDER-CONSTRAINED" }
        )
    }
}

/// Cycle basis computation (for ZHC consensus)
/// Returns a basis of independent cycles for computing holonomy sums
pub fn cycle_basis(complex: &Complex) -> Vec<Vec<(u64, u64)>> {
    // Simple cycle basis: for each edge that creates a cycle, extract the cycle
    let mut basis: Vec<Vec<(u64, u64)>> = Vec::new();
    let verts: Vec<u64> = complex.vertices.iter().map(|Vertex(v)| *v).collect();
    let edges = &complex.edges;

    // Build adjacency list
    let mut adj: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
    for &Vertex(v) in &complex.vertices {
        adj.entry(v).or_default();
    }
    for &Edge(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    // Find cycles by doing DFS and tracking back-edges
    let mut visited = std::collections::HashSet::new();
    let mut parent = std::collections::HashMap::new();
    let mut cycles = Vec::new();

    fn dfs(
        v: u64,
        p: Option<u64>,
        adj: &std::collections::HashMap<u64, Vec<u64>>,
        visited: &mut std::collections::HashSet<u64>,
        parent: &mut std::collections::HashMap<u64, u64>,
        cycles: &mut Vec<Vec<(u64, u64)>>,
    ) {
        visited.insert(v);
        if let Some(pp) = p {
            parent.insert(v, pp);
        }
        for &nbr in adj.get(&v).unwrap_or(&vec![]) {
            if !visited.contains(&nbr) {
                dfs(nbr, Some(v), adj, visited, parent, cycles);
            } else if Some(nbr) != p && parent.get(&nbr).is_some() {
                // Found a cycle
                let mut cycle = Vec::new();
                let mut cur = v;
                cycle.push((cur.min(nbr), cur.max(nbr)));
                while let Some(&par) = parent.get(&cur) {
                    cur = par;
                    if cur == nbr { break; }
                    if let Some(next) = parent.get(&cur) {
                        cycle.push((cur.min(*next), cur.max(*next)));
                    }
                }
                if !cycle.is_empty() {
                    cycles.push(cycle);
                }
            }
        }
    }

    for &v in &verts {
        if !visited.contains(&v) {
            dfs(v, None, &adj, &mut visited, &mut parent, &mut cycles);
        }
    }

    cycles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_beta() {
        // Triangle: V=3, E=3, C=1 → β₁ = 3-3+1 = 1
        let c = Complex::from_edges(&[(1,2), (2,3), (3,1)]);
        assert_eq!(c.V(), 3);
        assert_eq!(c.E(), 3);
        assert_eq!(c.beta_0(), 1);
        assert_eq!(c.beta_1(), 1); // One independent cycle
        assert!(c.is_laman_rigid()); // β₁ = V-2 = 1
        assert!(!c.has_emergence()); // β₁ = V-2, not >
    }

    #[test]
    fn test_complete_graph_K5() {
        // K5: V=5, E=10, C=1 → β₁ = 10-5+1 = 6
        let edges = vec![
            (1,2), (1,3), (1,4), (1,5),
            (2,3), (2,4), (2,5),
            (3,4), (3,5),
            (4,5),
        ];
        let c = Complex::from_edges(&edges);
        assert_eq!(c.V(), 5);
        assert_eq!(c.E(), 10);
        assert_eq!(c.beta_0(), 1);
        assert_eq!(c.beta_1(), 6); // 6 cycles
        assert!(c.has_emergence()); // 6 > 3 (V-2)
        assert!(!c.is_laman_rigid()); // β₁ ≠ V-2
    }

    #[test]
    fn test_line_graph() {
        // Line: V=4, E=3, C=1 → β₁ = 3-4+1 = 0
        let c = Complex::from_edges(&[(1,2), (2,3), (3,4)]);
        assert_eq!(c.beta_1(), 0); // No cycles
        assert!(!c.is_laman_rigid());
        assert!(!c.has_emergence());
    }

    #[test]
    fn test_homology_report() {
        let c = Complex::from_edges(&[(1,2), (2,3), (3,1), (3,4), (4,5), (5,6), (6,4)]);
        let report = c.homology_report();
        assert!(report.beta_0 >= 1);
        assert!(report.beta_1 >= 0);
        let summary = report.summary();
        assert!(summary.contains("β₁="));
    }
}
