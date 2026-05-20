# pythagorean48-codes


![CI](https://github.com/SuperInstance/pythagorean48-codes/actions/workflows/rust-ci.yml/badge.svg)

## Meta

**Domain:** other
**Depends on:** —
**Depended by:** —
**Implements:** Pythagorean-48 directional encoding — 48 compass points as exact integer pairs (...
**Related:** —


**Forty-eight exact directions. Zero drift, unlimited hops.**

A 48-direction compass rose where every vector lands exactly on the unit circle. No floating-point encoding. No error accumulation. Each direction is represented as a Pythagorean triple `(x, y)` where `x² + y² = 1` exactly — integer numerators over matching denominators.

## Why 48?

Forty-eight directions (`log₂(48) ≈ 5.585 bits per vector`) is enough granularity for most navigation, trust propagation, and consensus routing. But the key property isn't the angular resolution — it's that you can hop between vectors indefinitely and the encoding never drifts. Each `compose` operation produces another vector in the codebook. No rounding. No approximation. No accumulation.

## Where it's used

- **fleet-coordinate** — trust topology encoding. Agents propagate direction vectors through the fleet. After 10,000 hops, the direction is exactly what it started as.
- **holonomy-consensus** — consensus verification. Cycle detection on directed trust graphs. Needs exact direction comparison. Float comparison would break on rounding.
- **aboracle** — research note encoding. Mapping ideas to exact directions for spatial indexing.

## Quick start

```rust
use pythagorean48_codes::TrustVector;

let v = TrustVector::from_f32(0.6, 0.8);
let (x, y) = v.to_f32(); // exact round-trip

let all = TrustVector::all_directions(); // the full 48-direction codebook
```

Python:

```python
from pythagorean48 import TrustVector

v = TrustVector.from_f32(0.6, 0.8)
x, y = v.to_f32()
```

## The math

Each direction is a Pythagorean triple `(xn/xd, yn/yd)` where `xn² + yn² = xd²`. The codebook draws from:

- 3-4-5 → 8 directions
- 5-12-13 → 8 directions
- 7-24-25 → 8 directions
- 8-15-17 → 8 directions
- 9-40-41 → 8 directions
- Cardinal axes → 4 directions
- Swapped variants → 4 directions

Every vector fits in 16 bits. No floats. No loss.

## Install

```bash
cargo add pythagorean48-codes
```

Python package published to PyPI.

## License

MIT OR Apache-2.0
