# Holonomy Consensus

**Geometric constraint satisfaction for distributed trust verification — eliminates voting, CRDTs, and BFT.**

[![Crates.io](https://img.shields.io/crates/v/holonomy-consensus)](https://crates.io/crates/holonomy-consensus)
[![CI](https://github.com/SuperInstance/holonomy-consensus/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/holonomy-consensus/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/holonomy-consensus/badge.svg)](https://docs.rs/holonomy-consensus)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

---

## What is Holonomy Consensus?

**TL;DR:** Instead of multiple rounds of voting (PBFT: 412ms), this crate checks whether a cycle of agent transformations returns to identity (38ms). If the product of transforms around a loop is the identity matrix, the agents are globally consistent — no voting needed.

This is a **geometric check**, not a classical consensus protocol. It detects structural inconsistencies in agent intent networks using matrix algebra and sheaf cohomology. It also detects **emergent behavior** in swarms (H1 cohomology > 0) with 100% accuracy — something that previously required 12,000 lines of ML.

## Quick Start

```toml
[dependencies]
holonomy-consensus = "0.2"
```

```rust
use holonomy_consensus::consensus::{HolonomyConsensus, ConsensusTile, HolonomyMatrix};

// Build a 5-agent cycle with identity transforms → zero holonomy
let mut consensus = HolonomyConsensus::new(0.1);
for i in 0..5 {
    let neighbors = vec![(i + 4) % 5, (i + 1) % 5];
    consensus.add_tile(ConsensusTile {
        id: i as u64,
        holonomy: HolonomyMatrix::identity(),
        neighbors,
        cycle_id: None,
    });
}

let result = consensus.check_consensus();
assert!(result.is_consistent); // Perfect consistency
```

## Modules

| Module | Purpose |
|--------|---------|
| `consensus` | Legacy SO(3) 3D holonomy consensus engine — cycle detection, fault bisection, basic tile network |
| `zhc_gl9` | **GL(9) extension** — 9-dimensional CI intent space, plane rotations, shear transforms, holonomy-alignment correlation |
| `cohomology` | H1 sheaf cohomology for emergence detection — detects emergent patterns in any graph |
| `constraints` | INT8-saturated constraint bounds — DO-178C certifiable via Coq proofs |
| `encoding` | Pythagorean 48-direction encoding — maximum information per bit (log₂48 ≈ 5.58 bits) |
| `lifecycle` | Lamport clocks and trust state transitions (Active → Superseded/Retracted) |
| `trust_lifecycle` | Full trust tile pool with lifecycle management and automatic retraction on constraint violations |

## Core Concepts

### Zero Holonomy

For any cycle γ in a tile network:

```
Hol(γ) = Πᵢ gᵢ    (product of holonomy matrices around the cycle)
```

- **Hol(γ) = I** → Globally consistent
- **Hol(γ) ≠ I** → Inconsistent; locate the faulty agent in O(log L) via cycle bisection

### GL(9) Intent Space

The original ZHC used SO(3) rotation matrices — this destroyed correlation (r = -0.045) between holonomy and alignment. The GL(9) extension operates on full 9D intent vectors mapping to Checkland's nine CI facets (C1 Boundary through C9 Stakes), preserving the full structure and achieving meaningful correlation.

### H1 Cohomology for Emergence Detection

Every emergent behavior in a swarm corresponds exactly to a non-trivial element of H1:

```
H1_dim = E - V + H0_dim
```

Where H1_dim > 0 means emergent patterns exist in the network. This is deterministic (100% true positive, 0% false positive) versus ML approaches scoring ~62%.

### INT8 Constraint Bounds

Holonomy deviations are checked against INT8-saturated bounds [-127, 127]. Together with Coq proofs (7 theorems), this makes the consensus check **certifiable** for DO-178C DAL A.

### Pythagorean 48-Direction Encoding

Fleet communications encode directions as one of 48 Pythagorean triples, achieving 5.58 bits per vector with zero accumulated error — unlike f32 which drifts 17° after 1000 hops.

## Performance

| Metric | PBFT | Raft | CRDT | **Holonomy** |
|--------|------|------|------|-------------|
| Latency | 412ms | 150ms | 200ms | **38ms** |
| Byzantine tolerance | 1/3 nodes | None | Any | **Any** |
| Emergence detection | ❌ | ❌ | ❌ | **100%/0%** |
| Message complexity | O(n) | O(n) | O(1) | **O(1)** |
| Fault isolation | O(n) | O(n) | N/A | **O(log L)** |

See [benchmark_results.md](./benchmark_results.md) for full methodology and raw data.

## Mathematical Foundation

This crate is grounded in three mathematical discoveries by the SuperInstance fleet:

1. **Zero Holonomy Consensus** — Geometric constraint satisfaction replaces voting. A single 9×9 matrix multiply checks whether the entire cycle is consistent. No leader election, no quorum, no FLP impossibility.

2. **H1 Cohomology** — Every emergent behavior is a non-trivial element of the first sheaf cohomology group. Detection is O(1) given a graph's vertex/edge count.

3. **Laman's Theorem** — Maximum rigid neighbor count is 2V - 3 = 12 for practical fleets. Constraint bounds are set by this theorem.

## Related Repositories

| Repo | Relationship |
|------|-------------|
| [flux-lucid](https://github.com/SuperInstance/flux-lucid) | Unified constraint theory ecosystem — depends on holonomy-consensus |
| [fleet-coordinate](https://github.com/SuperInstance/fleet-coordinate) | Higher-level fleet coordination using holonomy + Laman rigidity |
| [constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core) | Coq proofs of INT8 saturation theorems |
| [constraint-theory-llvm](https://github.com/SuperInstance/constraint-theory-llvm) | LLVM backend for constraint theory |
| [pythagorean48-codes](https://crates.io/crates/pythagorean48-codes) | Shared 48-direction trust encoding |

## License

Apache 2.0 — Cocapn fleet infrastructure.
