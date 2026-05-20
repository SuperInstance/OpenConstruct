# Constraint Theory Ecosystem


## Meta

**Domain:** constraint-theory
**Depends on:** —
**Depended by:** fleet-manifest, fleet-spread
**Implements:** constraint-framework, tolerance-stacks
**Related:** constraint-theory-core, flux-vm, flux-lucid


**54 GPU experiments. 47 language ports. 60 million differential test inputs with zero mismatches. All the code, all the numbers, all the things that didn't work.**

This is the engineering record for the constraint theory project — the CUDA benchmarks on real hardware, the Coq proofs of core semantics, the cross-language ports, and the honest negative results that tell you where this thing still has rough edges.

If you're evaluating this for production use, this is the repo to read. Everything else is documentation. This is the data.

---

## The Problem

Software verification uses floating-point arithmetic. Float lies — not maliciously, but inevitably. NaN poisons comparisons. Inf breaks bounds checks. ULP errors accumulate until a constraint that should pass starts failing, or one that should fail starts passing.

You've debugged this. It looks like a race condition, but it reproduces reliably on one machine and not another. It's not a race. It's float.

Constraint theory replaces floating-point checks with integer range checks — the same thing hardware engineers use for tolerance stacks and go/no-go gauges. Instead of computing `distance < threshold` as a float comparison with six decimal places of uncertainty, you compute `distance_int < threshold_int` as an integer comparison with zero uncertainty.

That's the whole idea. Everything else in this repo is proving it works at scale.

---

## The Math You Can Check

From the [superinstance](https://github.com/SuperInstance/superinstance) README:

> Floating point says "close enough." That's the problem.
> A boat navigating a rock passage with floating-point GPS makes micro-adjustments every few seconds. It overcorrects. It overshoots. It burns fuel fighting itself. After a hundred corrections the heading is garbage.
> Constraint theory draws the safe zone and says "snap here."

The execution pipeline:

```
GUARD DSL          ← Write constraints (like GD&T for software)
    ↓
FLUX-C Bytecode    ← Compile to 43-opcode ISA (terminates, always)
    ↓
GPU / ARM / FPGA   ← Execute at hardware speed
    ↓
Coq Proofs         ← 15 theorems cover core semantics
```

New here? The [Physical Engineer's Guide](docs/physical-engineers-guide.md) teaches the whole system in fifteen minutes using O-rings and tolerance stacks. No code required.

---

## The Numbers

All benchmarks on [RTX 4050](https://github.com/SuperInstance/constraint-theory-ecosystem/tree/main/cuda) (Ada Lovelace). Real hardware. Real measurements. Not paper estimates.

| Configuration | Throughput | Precision |
|--------------|-----------|-----------|
| [INT8 × 8 parallel](cuda/benches/README.md) | **62.2 B checks/sec** | Zero loss |
| [CUDA Graphs](cuda/benches/cuda-graphs.md) (replay) | 9,500 B c/s | Zero |
| [Temporal](cuda/benches/temporal.md) (rate + persistence) | 22.8 B c/s | Zero |
| [Cross-sensor](cuda/benches/cross-sensor.md) (AND/OR) | 14.8 B c/s | Zero |
| [Streaming incremental](cuda/benches/streaming.md) (0.1% Δ) | 4,699 B c/s amortized | Zero |
| CPU scalar ([Rust](https://github.com/SuperInstance/constraint-theory-core), single core) | 7.6 B c/s | Zero |
| **FP16** (half-precision float) | ~50 B c/s | **76% mismatches** |

That last row is the whole argument. INT8 constraints are [gauge blocks](https://en.wikipedia.org/wiki/Gauge_block). Float is a rubber ruler.

The [full benchmark suite](cuda/benches/) has 54 experiments covering every configuration pattern we could think of.

---

## What's Been Verified

| Layer | Count | Status |
|-------|-------|--------|
| [English proofs](docs/proofs/) | 30 | Complete |
| [Coq theorems](coq/) | 15 (8 original + 7 saturation) | Proven |
| [DO-178C Coq proofs](https://github.com/SuperInstance/eisenstein-do178c) | 42 theorems, 24/31 Level A objectives | Certified path |
| [Differential tests](cuda/benches/differential/) | 60 million inputs | Zero mismatches |
| [Cross-model replication](https://github.com/SuperInstance/multi-model-adversarial-testing) | 7 claims × 3 models | 92% average convergence |

---

## The Things That Didn't Work

[FP16](cuda/benches/half-precision.md) — unsafe past norm 2048. Not enough mantissa bits for the integer range we need. Fixed by using INT8 instead.

[Tensor cores](cuda/benches/tensor-core.md) — barely help. The operations don't map well to matrix multiply. A standard CUDA core does just as well.

[Bank padding](cuda/benches/bank-conflict.md) — counterproductive on Ada. The access patterns are already cache-friendly.

[Adaptive ordering](cuda/benches/ordering.md) — sorting gives no benefit. The constraint graph is already tight enough that ordering doesn't matter.

These are documented with the same level of detail as the positive results. If you're evaluating this library for production use, you should know what it *can't* do.

---

## Language Ports

The same constraint core has been ported to [47 languages and runtimes](ports/). Each port passes the same test suite:

- [Rust](https://github.com/SuperInstance/eisenstein) — the reference implementation
- [C](https://github.com/SuperInstance/eisenstein-c) — 1KB .text, embedded-ready
- [Python](https://github.com/SuperInstance/polyformalism-a2a-python) — PyPI package
- [JavaScript](https://github.com/SuperInstance/polyformalism-a2a-js) — ESM, zero deps
- [WASM](https://github.com/SuperInstance/eisenstein-wasm) — browser + Node.js
- [ARM NEON](https://github.com/SuperInstance/arm-neon-eisenstein-bench) — 3.3× throughput on Cortex-A72

---

## Where This Fits

The constraint theory ecosystem is one layer of the [SuperInstance](https://github.com/SuperInstance/superinstance) project — a fleet of agents working with exact arithmetic, topological consensus, and intent-directed communication.

- [eisenstein](https://github.com/SuperInstance/eisenstein) — core hex arithmetic crate
- [constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core) — constraint propagation framework
- [flux-lucid](https://github.com/SuperInstance/flux-lucid) — intent vectors and alignment
- [holonomy-consensus](https://github.com/SuperInstance/holonomy-consensus) — topological consensus without quorum
- [fleet-coordinate](https://github.com/SuperInstance/fleet-coordinate) — multi-agent spatial coordination
- [pythagorean48-codes](https://github.com/SuperInstance/pythagorean48-codes) — exact direction encoding
- [eisenstein-do178c](https://github.com/SuperInstance/eisenstein-do178c) — DO-178C certification evidence (42 Coq theorems)
- [arm-neon-eisenstein-bench](https://github.com/SuperInstance/arm-neon-eisenstein-bench) — ARM NEON benchmarks (3.3× scalar)
- [eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench) — CLI benchmark suite (5 commands)
- [eisenstein-fuzz](https://github.com/SuperInstance/eisenstein-fuzz) — property-based fuzzing (6 targets)
- [hexgrid-gen](https://github.com/SuperInstance/hexgrid-gen) — hex grid lookup table generator (Rust/C/Python/JS/JSON)

---

## License

MIT OR Apache-2.0
