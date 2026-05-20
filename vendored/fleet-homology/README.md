# fleet-homology


![CI](https://github.com/SuperInstance/fleet-homology/actions/workflows/ci.yml/badge.svg)
**H¹ cohomology cycle space computation for fleet emergence detection.**

Computed from algebra, not heuristics. β₁ = E - V + C. When β₁ > V - 2 (Laman boundary), the fleet has redundant constraint paths = emergent behavior.

## The Math

For a cellular complex (fleet graph):

| Betti Number | Formula | Fleet Meaning |
|-------------|---------|---------------|
| β₀ (H⁰) | connected components | Is fleet connected? |
| β₁ (H¹) | E - V + C | Independent cycles = redundant paths = emergence |
| β₂ (H²) | 0 (2D fleet) | Not applicable |

**Emergence detection threshold**: β₁ > V - 2

- β₁ = V - 2 → Laman-rigid, exactly self-coordinating
- β₁ > V - 2 → Over-constrained, emergent patterns exist
- β₁ < V - 2 → Under-constrained, need more trust edges

## Usage

```rust
use fleet_homology::{Complex, HomologyReport};

let edges = vec![
    (1, 2), (1, 3), (1, 4), (1, 5),
    (2, 3), (2, 4), (2, 5),
    (3, 4), (3, 5),
    (4, 5),
];
let c = Complex::from_edges(&edges);
let report = c.homology_report();

println!("{}", report.summary());
// Fleet homology: V=5, E=10, C=1
// β₀=1 (components), β₁=6 (cycles), β₂=0 (voids)
// Laman boundary: β₁=3 expected, 6 observed
// Status: ⚡ EMERGENCE DETECTED
```

## Examples

| Fleet Configuration | V | E | β₁ | Status |
|---------------------|---|---|-----|--------|
| Triangle | 3 | 3 | 1 | ✓ RIGID |
| Complete K5 | 5 | 10 | 6 | ⚡ EMERGENCE |
| Line (4 nodes) | 4 | 3 | 0 | ✗ UNDER |

## Related

- **[fleet-coordinate](https://github.com/SuperInstance/fleet-coordinate)** — ZHC consensus + Laman + beam
- **[fleet-topology](https://github.com/SuperInstance/fleet-topology)** — topology visualization
- **[pythagorean48-codes](https://github.com/SuperInstance/pythagorean48-codes)** — trust encoding
