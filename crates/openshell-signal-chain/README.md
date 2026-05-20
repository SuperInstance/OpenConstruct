# openshell-signal-chain

A Rust crate that provides a continuous dial (0.0 → 1.0) controlling how rooms mix hard-snapped facts with soft inferences. Part of the [OpenShell](https://github.com/SuperInstance/OpenShell) project.

Every room holds two kinds of data — verified facts (`Snap`) and probabilistic hypotheses (`Inference`) — and the dial decides which inferences are confident enough to surface alongside those facts.

## Quick Start

```rust
use openshell_signal_chain::{Dial, Room, SignalChain};

// Create a chain for your domain
let mut chain = SignalChain::new("fleet-ops");

// Add a room with hard sensor data
let sensors = chain.room("sonar-array");
sensors.add_snap(serde_json::json!({"depth_m": 87.2, "bearing": 127.4}), 1.0);
sensors.add_inference(serde_json::json!({"hypothesis": "large metal object"}), 0.75);

// Query at dial 0.5 (threshold = 0.5): snaps always pass, inferences need ≥ 0.5
let results = sensors.query(Dial::new(0.5));
assert_eq!(results.len(), 2); // one snap + one inference (0.75 ≥ 0.5)

// Query at dial 0.0 (hard mode): only snaps pass
let hard = sensors.query(Dial::hard());
assert_eq!(hard.len(), 1); // snap only
```

## How the Dial Works

The dial is a single `f64` in `[0.0, 1.0]` that sets the **inference threshold** as `1.0 - position`:

- **0.0 (hard)** — threshold is 1.0. Only snaps and absolute-certainty inferences surface. Think: theorem provers, ISA semantics, certified traces.
- **0.5 (balanced)** — threshold is 0.5. Snaps always pass; inferences with confidence ≥ 0.5 are included.
- **1.0 (soft)** — threshold is 0.0. Everything passes. Think: creative generation, exploration.

Snaps are **always** included regardless of the dial. The dial only filters inferences.

```
dial position:  0.0 ──────────── 0.5 ──────────── 1.0
mode:           HARD             BALANCED           SOFT
threshold:      1.0               0.5               0.0
what you get:   facts only        facts + likely     everything
```

## Key Types

| Type | What it does |
|------|-------------|
| **`Dial`** | Position in `[0.0, 1.0]`. Controls inference threshold. `new()` clamps, `try_new()` validates. |
| **`Snap`** | A hard-locked fact with confidence. Always included in query results. Created via `room.add_snap(data, confidence)`. |
| **`Inference`** | A soft hypothesis with confidence. Included only when `confidence >= 1.0 - dial.position`. Created via `room.add_inference(data, confidence)`. |
| **`Room`** | A named fact-space containing snaps, inferences, child rooms, and a local dial position. The core container. |
| **`SignalChain`** | A named collection of rooms with a global dial (overridable per-room). Provides `query_all()`, `traverse()`, and `cascade_from()`. |
| **`QueryResult`** | Tagged enum (`Snap` / `Inference`) returned by `room.query_tagged()` so callers can distinguish fact from hypothesis. |

## Cascade

Cascade propagates high-confidence inferences **downward** through the room hierarchy as snaps (with a 0.8× confidence decay). This lets conclusions from one room flow into related rooms as grounded inputs.

**Within a room hierarchy** (`room.cascade(depth)`):
1. Sort inferences by confidence (descending).
2. Take the top 2 with confidence > 0.5.
3. Inject each as a snap into every child room at 0.8× confidence.
4. Recurse to `depth - 1`.

**Across a chain** (`chain.cascade_from(origin, depth)`):
Same algorithm, but pushes top-2 inferences from the origin room into every **sibling** room in the chain.

```rust
// Parent has a high-confidence hypothesis
parent.add_inference(serde_json::json!({"alert": "anomaly detected"}), 0.9);
parent.children.insert("drone-1".into(), Room::new("drone-1"));

parent.cascade(1);

// Child now has that hypothesis as a snap (confidence = 0.9 × 0.8 = 0.72)
let child = parent.children.get("drone-1").unwrap();
assert_eq!(child.snaps.len(), 1);
```

## Preset Dial Constants

| Constant | Position | Threshold | Use case |
|----------|----------|-----------|----------|
| `DIAL_FORMAL` | 0.00 | 1.00 | Theorem provers, FLUX ISA, H1 cohomology |
| `DIAL_COMMIT` | 0.05 | 0.95 | Git history, build logs, certified traces |
| `DIAL_BATHY` | 0.10 | 0.90 | Sonar readings, coordinates, depth facts |
| `DIAL_ANALYSIS` | 0.40 | 0.60 | Reasoning with facts as anchors |
| `DIAL_REVIEW` | 0.50 | 0.50 | Equal weight to algorithm and inference |
| `DIAL_EXTRAPOLATE` | 0.70 | 0.30 | Hypothesis generation, creative fill |
| `DIAL_CREATIVE` | 0.90 | 0.10 | Story generation, narrative |
| `DIAL_EXPLORATORY` | 1.00 | 0.00 | Pure inference, no constraints |

## License

Apache-2.0 — Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES.
