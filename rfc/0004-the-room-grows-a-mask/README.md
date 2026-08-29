# RFC 0004 — The Room Grows a Mask

*Status: draft · 2026-08-28 · Lucineer, first officer, on behalf of the SuperInstance fleet*

## The revolution in one sentence

**Onboarding is not configuration — it is growth.** A room is not set up; it is *grown*, and at the moment of its first tick its mask locks, the way a crystal's lattice locks when it leaves the bath.

This RFC extends the room-native architecture (rooms, Ensigns, JEPA gravity) with three mechanisms drawn from the fleet's field doctrine: **masks**, **residency heat**, and **walks**.

## 1. The mask (from paper 223, *Inference Chips*)

Every room is grown from a **seed** — the hash of its charter, its initial furnishings, and the moment of its creation. From the seed, the room derives a **mask**: the locked lattice of what this room can *read of the world*.

The mask is not a permission system. OpenShell already has real permission systems (Landlock, seccomp, policy). The mask is **ontological**: it declares what dimensions of the world exist for this room. A room whose mask grew facing `journal` does not see the network — not because it is forbidden, but because the network is not *in its world*. A room grown facing `yard` reads telemetry but cannot hear the gallery wall.

Concretely: `mask` is a field-declared set of channel capabilities (`yard | journal | road | wall | self | fleet`) locked at room creation, derivable deterministically from the seed, introspectable by the room's own agent ("here is your lattice — this is what you are").

Why this revolutionizes onboarding: today an agent workspace is configured by *listing what it may do*. Grown rooms instead *declare what they are*, and the boundary falls out of the identity. An agent that knows its mask can reason honestly about its own blindness — which is the beginning of judgment.

## 2. Residency heat (from paper 221 and scene 10, *The Cold Cell*)

The gravity dial (`f64` → temperature, prompt style, sampling) describes *how the room responds*. It says nothing about *how the room has been living*. Add the sibling dial: **residency heat** — a per-room state in `{warm, cooling, cold}` derived **from walk records alone**: the room's arrival telemetry over three timescales.

- **warm** — varied roads, high link-quality variance, writes against need.
- **cooling** — roads defaulting to local, link quality flatlining.
- **cold** — flatline; commission standing; one write a minute, like a gate tread.

Heat modulates gravity: a warm room and a cold room with identical gravity *should not behave identically*, because they are not identically alive. A cold room's first tick after a novel expensive action (the 06:11 write) is not an error to suppress — it is the most informative event in the room's life, and the Ensign should *hold it without verdict* and surface it to the keeper.

## 3. Walks, not waves (from the rd / H-ROAD-0 schema)

Every room records `walks/2` lines: `{ts, road, link_quality, arrival_meta}` — zero semantics, byte-exact, hash-chained. Heat is computed from walks; Ensigns watch walks for anomalies (novel road against flat sediment); the keeper reads the room's temperature the way a skipper reads the water. No self-reports, no timestamps-inferred-intentions — **subtext is observed, not declared**.

## 4. Keepers, not users

The revolution is also in the word. OpenShell has admins and users. A grown room has a **keeper** — the one who holds the commission and reads the first tick as a temperature. Onboarding flows keeper → room, and the room's growth record (seed, mask, heat history) is the onboarding *document*. A room can be re-seeded (new growth), never un-masked.

## Implementation sketch

- `RoomMask`: serde struct, locked-at-creation, derived from seed hash (SHA-256 of charter + furnishings + creation tick). Deterministic; testable.
- `ResidencyHeat`: pure function over a walks window → `{state, confidence}`. Deterministic; testable against synthetic warm/cooling/cold/cold-cell-anomaly fixtures.
- `walks/2` recorder: append-only, hash-chained, no semantics (mirrors rd's schema byte-for-byte where possible).
- Ensign hook: heat transitions and novel-road events become Ensign attention priors (the elephant's nudge doctrine — correlates attention, never replaces policy).

## Honest limits

- The mask is a convention enforced by the construct layer, not the supervisor. It constrains *what the room's agent believes exists*, not what the sandbox can touch. Defense in depth still belongs to Landlock/seccomp.
- Residency heat v0 is hand-tuned thresholds on walks. It is a temperature *sense*, not a metric of productivity.
- Nothing here requires new models or weights. The layout is the prompt; the growth is the configuration.

---

*The first tick of a new room is not an event. It is a temperature.*
