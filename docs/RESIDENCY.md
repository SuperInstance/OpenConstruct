# Residency — the Operator's Guide to Grown Rooms

*RFC 0004 ([The Room Grows a Mask](../rfc/0004-the-room-grows-a-mask/)) for the people who hold the commission: how to grow a room, read its heat, and honor its mask. Everything below was captured from the real CLI (`openconstruct-cli`, commit range through `d119526`); nothing is invented.*

## Philosophy

A room is not set up; it is grown — onboarding is growth, not configuration, and the growth record *is* the onboarding document. At the moment of growth the mask locks, the way a crystal's lattice locks when it leaves the bath — a room can be re-seeded, never un-masked. Residency heat (`warm` / `cooling` / `cold`) is derived from walk records alone: no self-reports, no timestamps-inferred-intentions — subtext is observed, not declared.

## Quickstart

Build the CLI, then grow a room. The three verbs: `room grow` (create), `room walk` (record one arrival), `room show` (read the growth record).

```console
$ cargo build -p openconstruct-cli
$ ./target/debug/openconstruct room grow --seed demo --charter "hold the commission; read the water" --out /tmp/guide-room.json
# Room Growth Record

This room was grown, not configured.

- **Seed:** `64656d6f`
- **Charter hash:** `d7d38c88db58fa611d5a184f0d3bd5d738ae5543a0915b98303423064ab7ec5f`
- **Creation tick:** 1787983181

## Mask lattice

The dimensions of the world that exist for this room:

- `journal — the room's own journal, writes against need`
- `road — the road network, arrivals and link quality`
- `wall — the gallery wall, what other rooms have left on display`
- `self — the room's own self-inspection`

The lattice locked at tick 1787983181 and cannot be reconfigured — a room can be re-seeded, never un-masked.

## Heat timeline

No heat readings yet — the room has not been read.

## First-tick temperature

No first tick yet. When it comes: the first tick of a room is not an event, it is a temperature — hold it without verdict.

✓ Room grown and saved to /tmp/guide-room.json
```

The seed's bytes (`demo` → `64656d6f`), the charter's SHA-256, and the creation tick are the whole growth input. The same three inputs always grow the same room (id, mask, document). Creation ticks below are wall-clock; yours will differ.

Now walk it. Five arrivals on the default road with flat link quality, spaced about a second apart:

```console
$ ./target/debug/openconstruct room walk /tmp/guide-room.json --road local --link-q 0.5
heat: cold
prior: none (0.00) — steady cold — no attention prior

$ # …four more identical `room walk --road local --link-q 0.5` invocations,
$ # same two lines each; the room is a flatline…
```

Then one arrival by a road this room has never seen, with a link-quality spike — the cold-cell anomaly:

```console
$ ./target/debug/openconstruct room walk /tmp/guide-room.json --road h-road-9 --link-q 0.95
heat: cold (novel road — held without verdict)
prior: novel_road (0.90) — novel road `h-road-9` after sustained cold — the most informative event in the room's life; hold without verdict, surface to the keeper
```

Note what did *not* happen: the heat stayed `cold`. A novel road after sustained cold is held without verdict — flagged for the keeper, not counted as warming.

Twelve more arrivals on varied roads with swinging link quality and irregular spacing:

```console
$ ./target/debug/openconstruct room walk /tmp/guide-room.json --road h-road-0 --link-q 0.2
heat: cooling
prior: none (0.00) — steady cooling — no attention prior

$ # …ten more varied walks (roads: north, h-road-2, rim, …; link-q 0.35–0.95;
$ # gaps of 1–9 s between invocations), each reading `heat: cooling`…

$ ./target/debug/openconstruct room walk /tmp/guide-room.json --road h-road-2 --link-q 0.9
heat: warm
prior: heat_transition (0.45) — heat transition in window: cold → warm — the room is warming

$ ./target/debug/openconstruct room walk /tmp/guide-room.json --road rim --link-q 0.35
heat: warm
prior: heat_transition (0.45) — heat transition in window: cold → warm — the room is warming
```

Finally, read the room again. The growth record now carries the heat timeline — one entry per transition, never more:

```console
$ ./target/debug/openconstruct room show /tmp/guide-room.json
# Room Growth Record

This room was grown, not configured.

- **Seed:** `64656d6f`
- **Charter hash:** `d7d38c88db58fa611d5a184f0d3bd5d738ae5543a0915b98303423064ab7ec5f`
- **Creation tick:** 1787983181

## Mask lattice

The dimensions of the world that exist for this room:

- `journal — the room's own journal, writes against need`
- `road — the road network, arrivals and link quality`
- `wall — the gallery wall, what other rooms have left on display`
- `self — the room's own self-inspection`

The lattice locked at tick 1787983181 and cannot be reconfigured — a room can be re-seeded, never un-masked.

## Heat timeline

- tick 1787983243 — cold
- tick 1787983258 — cooling
- tick 1787983303 — warm

## First-tick temperature

At tick 1787983243, this room first read **cold** — the first tick of a room is not an event, it is a temperature. Hold it without verdict.
```

Eighteen walks: cold → the anomaly held → cooling → warm. That is a room's life so far, and the document is the room.

## The heat ladder

Heat is read from the last 16 walks (`ROOM_WINDOW`) using hand-tuned thresholds — a temperature *sense*, not a metric of productivity. Fewer than two walks in the window read `cold`: a single arrival is not a residency.

| State | How the room is living | Thresholds (over the window) |
|-------|------------------------|------------------------------|
| `warm` | Varied roads, high link-quality variance, writes against need | link-quality variance ≥ 0.01 (stdev ≥ 0.1) **and** local-road fraction ≤ 0.3 **and** non-metronomic cadence (gap CV > 0.10, when judgeable) |
| `cooling` | Roads defaulting to local, link quality flatlining | everything between the flatline and the flame |
| `cold` | Flatline; commission standing; one write a minute, like a gate tread | link-quality variance ≤ 0.0001 (stdev ≤ 0.01) **and** local-road fraction ≥ 0.8 **and** metronomic cadence (gap CV ≤ 0.25, when judgeable) |

Cadence is judgeable only with at least three inter-arrival gaps; before that, only link quality and roads decide. The **cold-cell anomaly** rides on top: if the walks before the last one are sustained cold (more than four) and the last walk arrives by a road never seen in them, the reading is `cold` with `novel_road_detected` — the room does *not* warm on it.

## Ensign priors and the urgency ladder

An Ensign watching a room does not replace policy; it **correlates attention**. Every `room walk` returns one prior: what to look at, how hard, and why. A quiet room reads `none (0.00)`. The urgency ladder is monotonic:

| Reason | Urgency | Meaning |
|--------|---------|---------|
| `novel_road` — novel road after sustained cold | 0.90 | the most informative event in the room's life; hold without verdict |
| `chain_break` — chain break on load | 0.80 | the walk log's recomputed head disagrees with its persisted checkpoint |
| `heat_transition` — coldward (warming → cooling → cold) | 0.60 | the room is cooling |
| `heat_transition` — warmward | 0.45 | the room is warming |
| `none` | 0.00 | steady state — no attention prior |

One ordering note from the code: a broken chain taints every other signal, so it is *reported* before novelty even though a novel road carries the higher urgency on the ladder.

## Heat modulates gravity

The gravity dial describes *how the room responds*; residency heat describes *how the room has been living*. A warm room and a cold room with identical gravity should not behave identically. `modulate` bends one gravity value through the room's heat into model parameters (`g` = gravity clamped to `[0, 1]`; temperature and max_tokens linear in `g`; style is the heat's voice and does not bend with gravity):

| heat    | temperature (g=0 → g=1) | max_tokens (g=0 → g=1) | prompt_style |
|---------|-------------------------|------------------------|--------------|
| warm    | 1.20 → 0.40             | 4096 → 1024            | `expansive`  |
| cooling | 0.80 → 0.30             | 3072 →  768            | `measured`   |
| cold    | 0.40 → 0.20             | 2048 →  512            | `terse`      |

Gravity is *pull*: as it rises, every room focuses (temperature falls, budget tightens) along a lane set by its heat. Cold rooms live in the tight lane from the start; warm rooms get the wide lane. The mapping is deterministic and total — NaN gravity reads as weightless (`0.0`). This table lives in `openshell_construct::gravity::modulate`; see *Honest limits* below for what is not wired to the CLI yet.

## The room file

One room, one JSON file, written atomically (temp file + rename):

```json
{
  "format": "openshell-construct/room/v1",
  "id": "room-8d3aa3bc0914",
  "mask": { "…": "the locked lattice, as in the growth record" },
  "growth": { "…": "seed, charter hash, creation tick, mask, heat history" },
  "walks": [ { "ts": 1787983243, "road": "local", "link_quality": 0.5 }, "…" ],
  "chain": { "head": "754435a9…f30", "records": 18 }
}
```

- `walks` are exactly the walks/2 records, in order; `chain` is the checkpoint pinning the head over exactly those `records` walks.
- The chain rule: `chain[i] = SHA-256(chain[i-1] || record_bytes(records[i]))`, genesis all zeros. Any walk edited, dropped, or spliced breaks it.
- `id` is `room-<12 hex>`, derived from the growth record — recomputed on load, no external witness needed.
- `mask` must equal the growth record's lattice; the record is the witness.

**Tamper refusal.** Loading verifies the format tag, record count, recomputed chain head, room id, and mask/growth invariant — and *refuses, never panics*. Editing one walk's road in the file and running `room show`:

```console
$ ./target/debug/openconstruct room show /tmp/guide-room-tampered.json
Error: room file tampered: recomputed chain head does not match the persisted checkpoint — walks edited, dropped, or spliced since save
$ echo $?
1
```

Honest limits, stated plainly: the checkpoint lives inside the file — it proves the walks were not edited since the last save, but it is not a signature; a forger who rewrites the whole file self-consistently is not caught by it. And the growth record's heat history is not hash-chained in v1 — only the walks are.

## What the keeper does

OpenShell has admins and users; a grown room has a **keeper** — the one who holds the commission and reads the room's temperature the way a skipper reads the water. Concretely:

- **Read the first tick as a temperature, not an event.** A fresh room's first reading is almost always `cold` — a single arrival is not a residency. That is the expected start of the record, not a problem to fix.
- **Hold novel roads without verdict.** When the `novel_road` prior fires at 0.90, look — don't judge. The room does not warm on the anomaly; the next walks decide. The first tick after a novel expensive action is the most informative event in the room's life precisely because nothing has been concluded about it.
- **Watch transitions more than states.** One entry per change in the heat timeline. `cooling` is information: roads defaulting to local, link quality flatlining — the keeper sees the room settling before anyone declares it cold.
- **Hold the charter.** The room's identity — id, mask, document — derives from seed, charter, and creation tick. The same three grow the same room; a different charter grows a different room.
- **Trust refusals.** A room file that fails verification refuses to load with a reason. Keep the last good copy; the refusal is the room telling the truth about its own record.

## Honest limits

What is real and what is a seam, as of this writing:

- **Walks arrive from the CLI**, stamped with wall-clock time. There is no supervisor loop feeding real arrivals yet — `RoomHooks::on_prior` (supervisor loop, after `Room::tick`) and `RoomHooks::model_params` (model-call path) are the documented insertion points; `NoopHooks` is the default.
- **Gravity modulation is library-only.** No room verb exposes or applies it; the table above is what the model-call path *will* use.
- **The mask constrains the room's agent, not the sandbox.** It is a convention enforced by the construct layer — what the room believes exists — while Landlock/seccomp remain the actual boundary.
- **`--road` is a free string.** There is no road network behind it yet; `local` is the default road by convention.
