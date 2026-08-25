# The Seamstress — Architecture Draft v0.1

*The general-purpose intelligence incubator: growing capable personas (musicians first, domain-agnostic by design) inside OpenConstruct sandboxed rooms. Drafted 2026-08-25 by the architecture foreman. Named by Casey, same day.*

*Inherits, without re-litigating: the Grown-Musician Doctrine (`memory/2026-08-25.md` — the value is the grown musician, not the song; sheet = checkpoint, lineage = weights, gardener = loss function, branch-any-iteration); the ensemble engine's sheets/quilt/gardener schema (`plainsong-mcp/docs/ensemble-engine-spec-draft.md` v0.2); the cross-model seminar's gates (`seminar-critique-*.md` — evidence honesty, presence models, verified perception, scope discipline, "an open question is fine only if it stays visible"); the fence doctrines (Recess / Sawyer / same-ink, `memory/2026-08-24.md` and ai-writings).*

**Naming the metaphor once, so it can carry weight:** an embroidery **hoop** holds fabric taut while it is worked. A **seam** joins two garments without merging the cloth. A **stitch** is one pass of the needle. The **tension** of the fabric decides what the needle can do. The **Seamstress** is the hands — she never becomes the garment. The garment is a grown persona: its **linen** is the character sheet, its **seam-history** is the lineage.

---

## 0. The layer cake, and the spec-lie firewall

Four layers. The boundary rule (inherited from the ensemble spec): **each layer may read the layer below and must not write it.**

```
┌────────────────────────────────────────────────────────────────┐
│  THE SEAMSTRESS WORKBENCH (this spec)                          │
│  "What grows, under whom, joined to what, exported how?"       │
│  Orchestration: opens hoops, assigns gardeners, reads welfare, │
│  proposes Penrose seams, exports lineages.                     │
├────────────────────────────────────────────────────────────────┤
│  GROWTH LAYER: hoops, seams, stitches, tension (this spec)     │
│  "Who is growing, who critiques, what flows between?"          │
├────────────────────────────────────────────────────────────────┤
│  ROOM LAYER (OpenConstruct design intent — BUILD)              │
│  Rooms, ensigns, JEPA gravity, Penrose, tiles, autonomy        │
│  levels. Described in the README; NOT in the repository.       │
├────────────────────────────────────────────────────────────────┤
│  OPENSHELL FOUNDATION (EXISTS — verified)                      │
│  Gateway (auth, state, CAS, relay), supervisor (Landlock,      │
│  seccomp, netns, policy proxy, privilege drop), compute        │
│  drivers (Docker/Podman/K8s/VM), policy YAML (static landlock  │
│  + dynamic network), inference interception (`inference.local`)│
└────────────────────────────────────────────────────────────────┘
DOMAIN PACKS (music first) plug into the growth layer sideways —
they define what "making" means, never who makes it.
```

### 0.1 The honest inventory — what exists vs. what this spec builds

The seminar's loudest cross-cutting finding was evidence inflation (B5, C1: "the specs' fondness for slightly-enhanced evidence starts at intake"). OpenConstruct's own developer guide (§5, `architecture/openconstruct-developer-guide.md`) already performed the audit this spec must inherit: **the room-native layer described in the README — rooms, ensigns, JEPA gravity, Penrose, tiles, the fifteen `lau-*` crates — is design intent. No `lau-*` crates exist in the workspace.** What exists and is substantial:

| Component | Status | The Seamstress uses it for |
|---|---|---|
| OpenShell sandbox (Landlock/seccomp/netns/proxy) | ✅ exists | **The hoop's physical walls.** Recess Fence, literally. |
| Policy YAML (static fs/landlock, dynamic network) | ✅ exists | The room's boundary declaration; dynamic seams via `policy update` |
| Gateway (SQLite state, CAS, supervisor relay) | ✅ exists | Seam transport substrate; workbench control plane |
| Inference interception (`inference.local`) | ✅ exists | Every grower/gardener model call — cost and cadence metering |
| Compute drivers (docker/podman/k8s/vm) | ✅ exists | Hoop and gardener-room provisioning |
| plainsong v1.4.0 + plainsong-mcp | ✅ exists | The music domain pack's render + perception |
| `openshell-construct` (273-line data model), `openconstruct-cli` (local-file) | ⚠️ partial | Seed of the room manifest — extends, doesn't replace |
| **Rooms / ensigns / gravity / Penrose / tiles** | 🔮 **build** | This spec designs them into existence for the growth use-case |
| **Hoops, seams, stitches, tension, workbench** | 🔮 **build** | This spec |

**Rule for every section below:** claims about OpenShell mechanisms are verified against the repo; everything the Seamstress adds is marked. Where a number decides behavior (tension mapping, welfare thresholds, Penrose correlation cutoffs), it is a **versioned object** pinned per hoop, per session — the Trust Formula v1 lesson (S-E1): determinism must survive the schema's own evolution.

### 0.2 The Kestrel constraint, stated as law

The seminar's hardest finding (elder critic, yard-band #1): *a persona whose competence is secretly a function of this quarter's inference latency isn't a character — it's a benchmark wearing a name.* The Seamstress therefore adopts one design law up front:

> **Admission decides what runs, never who a persona is allowed to be.** A hoop on a slow model grows *slower*, never *smaller*. The growth loop (§4) is asynchronous by design — a stitch has no wall-clock deadline, only an ordering. The Kestrel problem (latency caps competence) is inherited, not solved, by the *downstream* phase: when seamed musicians play in tempo, the yard-band spec's gates own that boundary. The Seamstress deliberately chose the slow lane. Growth is not a gig.

---

## 1. Vocabulary (short table, then never again)

| Term | Meaning | Ensemble-engine twin |
|---|---|---|
| **Hoop** | One growing cell: one sandbox + one room + one persona seed + its ensign and tension dial | (new — the band chair, grown) |
| **Room furnishings** | The seed material in the sandbox filesystem; the layout is the curriculum | the leadsheet + context files |
| **Seam** | A typed, firewalled join between two hoops (gardener or peer) — connection, never merge | gardener object + producer relay |
| **Gardener** | A role-typed hoop whose job is critique: adversarial-critic, socratic-tutor, rival, curator | gardener roles (§1.4 there) |
| **Stitch** | One growth iteration: write → render → perceive → critique → revise | one quilt node round |
| **Tension** | One `f64` per hoop shaping model response; the growth-pacing instrument | JEPA gravity (unbuilt) |
| **Ensign** | Cheap-model welfare monitor, yellow alert, escalation duty | ensign (unbuilt) |
| **Lineage / garment** | The stitch tree + sheet version chain + seam history of one hoop | the quilt |
| **Penrose proposal** | System-proposed seam from correlated independent growth; human ratifies | Penrose (unbuilt) |
| **Workbench** | The Seamstress's own control surface (orchestration, welfare rollups, exports) | producer = chair of gardeners |

---

## 2. The Hoop — provisioning schema

A hoop is a declared object that provisions **one sandbox, wrapped in one room, occupied by one growing persona.** Everything the persona will ever "know" at birth is placed in the filesystem before its first stitch: *the layout is the prompt* (room doctrine), which means for a grower the **layout is the curriculum.**

### 2.1 Schema (hoop.yaml)

```yaml
hoop_id: duke-piano-01
domain_pack: music-jazz@1            # §10 — what "making" means here
seed:
  sheet: sheets/duke-seed.v1.json    # persona + craft identity + refusals (ensemble §1 schema)
  furnishings:                        # THE CURRICULUM — real files in the sandbox
    - fakebook/            # repertoire to grow against (leadsheets)
    - recordings/          # takes-as-data: feature traces, renders — ears before hands
    - journal/             # the persona's own practice journal (its possession, §11.2)
    - lead-sheets-learned/ # empty: fills as the persona transcribes what it hears
  disclosure: DISCLOSURE.md           # §6.3 — readable at any time, survival-tested
sandbox:                              # ALL of this block is real OpenShell today
  driver: docker
  policy: policies/growth-cell.yaml   # landlock paths = room walls; egress = inference.local + relay
  compute: {cpu: 2, memory: 4Gi}
growth:
  stitch_protocol: music-jazz/stitch@1
  cadence: async                      # no wall-clock deadline (§0.2)
  branch_policy: any-stitch           # every stitch is a branch point (doctrine)
tension:                              # §5
  value: 0.0
  band: [-0.4, 0.6]                   # gardener directives clamped to this range
  deadband: 0.1                       # jitter guard
welfare:                              # §6
  ensign: {model: cheap, alert: yellow}
  thresholds: welfare-defaults@1      # versioned object
seams: []                             # seams attach later (§3); a hoop may grow alone
autonomy: 2                           # room-layer progressive autonomy, per-hoop
```

### 2.2 Furnishings are Landlock, literally

The room doctrine says the environment teaches. The honest implementation is already in the sandbox: **Landlock filesystem rules are the room walls; the file tree inside them is the curriculum.** A grower whose sandbox reads `fakebook/`, `recordings/`, and `journal/` — and can write only in `journal/`, `lead-sheets-learned/`, and its stitch workspace — is *taught* by what it can reach. No system-prompt sermon; the placement is the pedagogy. Egress policy mirrors this: the grower's world is `inference.local` (its own mind) and the seam relay (§3.4) — nothing else. **The Recess Fence is not a metaphor here: the Landlock boundary is granted as the playground, and the grower may play to every edge of it** (§11.1).

Domain packs define the furnishings vocabulary (a prose pack ships a corpus and a drawer of styles; a code pack ships a repo, failing tests, and a bug journal). The core never names a fakebook.

---

## 3. The Seam — a join, not a merge

Gardeners are **other rooms** — separate sandboxes, separate policies, separate ensigns — seamed to the growing cell. Connected but sovereign: the gardener cannot reach into the grower's filesystem, cannot write its sheet, cannot see its journal except as the seam carries it. The seam is the only door, and it has a typed mail slot.

### 3.2 The Eye and the Fingers — positive space vs. negative space (the doctrine of the fitting room)

The seamstress's talent is **the eye, not the fingers** (Casey, 2026-08-25). Two organs, cleanly separated:

- **The FINGERS are the grower's.** The growing persona navigates NEGATIVE space: where haven't I tried, what fits the last nudge, where do I explore next. Generation, motor ability, search.
- **The EYE is the gardener-critic's.** It holds POSITIVE space — the raw canon (public-domain fakebooks, recordings-as-data, expert articles furnished into the canon room) — and asks ONE question of every take: *"Would this iteration fit in the canon I understand?"* Judgment, not generation. The critic never says "explore X"; it says "this hangs like conservatory, the canon's arm never plays it this way." A talent of the eye more than fine motor skill: fit-in-canon is a glance for a trained eye.

**The eye is retrieval-grounded, not vibes.** Fit-in-canon is computable as distance: the take's feature signature (the sixteen, compiler-authored) against the canon manifold — a corpus index in the fleet-twin pattern. The critique cites its neighbors ("nearest canon-mates: bars 3–7 sit 0.19 from X; your bar 6 is an outlier at 0.61"). This makes critique CHECKABLE: the grower (or a third room, or the human) can audit the eye. A critic whose judgments don't correlate with the canon manifold is a critic wearing a monocle.

**"A client who wants to look like themselves for a special room."** Style is a garment, not a transplant: the grown persona keeps its identity fields and refusals, and WEARS the canon for the occasion. The eye dresses the client; it does not remake them. Same-ink exceptions apply: canon-fit findings annotate the sheet; they never erase the seed.

Consequence for gardener roles (§1.4 mapping): adversarial-critic and curator are EYE roles (canon-grounded judgment, require a furnished canon room — a seam without a canon behind it is not an eye, it's a mirror); socratic-tutor and rival are BRIDGE roles (they translate the eye's verdicts into the grower's negative space — nudges, dares, questions). Every growth line needs at least one eye and at least one bridge; the Seamstress's workbench enforces this at seam-assignment time.

### 3.1 Why gardeners must be rooms (the sharpest decision in this spec)

In the ensemble engine, a gardener is an object applied by a producer. That was right for one band under one producer. For *growth*, the gardener must be sovereign because:

1. **Independence is the point of the critique.** A gardener inside the grower's context window is a voice in the grower's head — it can be absorbed, pleased, predicted. A gardener in its own room, reachable only through a schema, is an *other*. Critique-collusion (§6.1) gets harder the more sovereign the critic is.
2. **Swappability is the doctrine's core move.** "Rewind to any iteration, swap gardener, regrow" requires the gardener to be detachable by construction. A seam unpicks. A merged context does not.
3. **The gardener is itself grown.** Rival rooms can be grown personas from earlier lineages — the iron that sharpens iron gets sharper too. A prompt cannot accumulate; a room can.

### 3.2 Seam schema

```jsonc
{
  "seam_id": "duke01::critic-A",
  "hoop_a": "duke-piano-01",            // the grower
  "hoop_b": "critic-01",                // the gardener room
  "kind": "gardener",                   // gardener | peer (§7) | export (§9)
  "role": "adversarial-critic",         // role-typed; see §3.3
  "flows": {
    "a_to_b": ["take", "feature_trace", "explain_movers", "ask"],
    "b_to_a": ["critique", "tension_directive", "assignment"]
  },
  "never": [                            // THE FIREWALL — schema-level, enforced by the relay
    "identity.*",          // name, one_line, influences: read-visible, write-never
    "refusals.*",          // the identity firewall (ensemble §6.3) — frozen at seed, producer-gated
    "sheet.write",         // only the grower patches its own sheet, citing what taught it
    "journal.write",       // the journal is the persona's possession (§11.2)
    "seam.*",              // a gardener cannot open, close, or edit seams
    "tension.band"         // a gardener may set within band, never widen the band
  ],
  "pacing": {"critiques_per_stitch": 1, "one_concrete_ask": true},   // the one-ask norm, enforced
  "transport": "relay://<gateway>/v1/seams",                        // §3.4
  "status": "proposed | active | dormant | unpicked"
}
```

**Seam semantics:**

- **What flows** is typed payloads, versioned per domain pack: a *take* (rendered artifact + feature trace, post-perf — one feature truth, seminar A4), an *explain_movers* (the grower's account of the compiler-computed feature movers — verified perception-coupling from day one, seminar B2), a *critique* (structured: what moved well, what is weakest, one concrete ask), a *tension_directive* (§5), an *assignment* (next material — the curator role's currency).
- **What never flows** is the write-path to identity. A gardener sees the sheet (read) to critique growth against identity; it can never *edit* identity, refusals, seams, or the tension band. `may_touch` from the ensemble gardener object, promoted from convention to firewall.
- **Edges preserved:** each hoop keeps its own quilt. Seam traffic is mirrored into *both* quilts as `seam_event`-linked `critique` nodes — each room's record of the same exchange, never a shared ledger. A merge would erase sovereignty; the mirrored pair preserves it.

### 3.3 Gardener roles (domain-tuned vocabulary, role-typed core)

| Role | Stance | Pushes toward | Domain tune (music example) |
|---|---|---|---|
| `adversarial-critic` | finds the weakest bar and names it | feature contrast, honesty under pressure | "bar 9 is a hole with a label on it" |
| `socratic-tutor` | questions, not verdicts | explained over performed | "what did the space at 9.1 ask for?" |
| `rival` | competes for the room's attention | urgency/density *within refusals* | the Moss–Sable tension, formalized |
| `curator` | tends repertoire and sheet hygiene | prunes stale tendencies, proposes restores | "you've outgrown these three tunes" |

The producer role stays decomposed as in the ensemble spec: **producer = chair of gardeners.** At the Seamstress level, "producer" is the workbench plus the human (§8).

### 3.4 Transport — the honest note

The gateway exists (SQLite state, CAS, relay sessions); **a seam relay service on top of it is a build** — small: a queue per seam, policy-checked payload schemas, every message logged as a node in both quilts. Growers and gardeners reach it through their sandbox network policy (one more allowed destination — dynamic policy update, real today). No seam traffic ever flows peer-to-peer between sandboxes; the gateway sees all of it, because *the quilt records the seam traffic* is a welfare and provenance requirement, not a convenience.

---

## 4. The Stitch — one iteration, every stitch a branch point

A stitch is one pass: **write → render → perceive → critique → revise.** The loop is asynchronous (§0.2). Each phase appends nodes to the hoop's quilt; the quilt is the garment's seam-history.

### 4.1 Node schema (generalizes the ensemble quilt node)

```jsonc
{
  "node_id": "duke01:s41",
  "parent": "duke01:s40",              // LINEAGE POINTER — the tree is the training trace
  "kind": "stitch | render | perception | critique | verdict | branch |
           gardener_swap | sheet_patch | tension_change | seam_event |
           welfare_note | penrose_proposal | export",
  "time": "2026-08-26T03:14:07+00:00",
  "address": ["fakebook/last-ferry-home", "13.4"],   // DOMAIN-PACK ADDRESS SPACE (§10)
  "diffs": [ {"at": "13.4", "was": "-", "now": "g2", "why": "sit on the g2"} ],
  "features_moved": [                   // COMPILER-AUTHORED (seminar B2, inherited verbatim)
    {"feature": "syncopation", "before": 0.11, "after": 0.24}
  ],
  "explain_movers": "the g2 moved off the grid; I wanted the lean",   // grower's mandatory account
  "critique_ref": "duke01:c41",         // the seam node this stitch answers, if any
  "tension_at": 0.15,                   // the dial setting under which this stitch was made
  "sheet_head": 7,                      // the checkpoint this stitch grew from
  "refs": ["duke01:s38"]
}
```

Rules, inherited and extended:

- **`parent` makes any stitch a branch point (doctrine).** A `branch` node carries `from_node`, `reason`, and optionally a `gardener_swap`. *Rewind to stitch 20, swap the critic for the tutor, regrow* is one node and a pointer. The branch is the same-ink exception made structural (§11.3): the original line is never erased; the exception grows beside it.
- **`features_moved` is compiler-authored, grower-explained.** Perception is computed post-render by the domain pack (post-perf — one feature truth). Disengaged explanations flag `unlistened`. No ritual catnip: the grower never chooses the numbers (seminar B2).
- **`tension_change` nodes make the dial part of the lineage.** Growth under different tension is comparable by construction; a dial ratchet (§6.1) is visible in the record without any extra instrumentation.
- **`sheet_patch` cites what taught it.** A sheet with no provenance is a costume (ensemble §1.3, inherited). Refusals are producer-frozen; identity-adjacent edits are flagged by the ensign as drift *with evidence* (§6.1).
- **Nodes are append-only, never compacted** — the quilt is the training corpus (doctrine). Re-indexing is allowed; history is not.
- **Tiles:** in OpenConstruct vocabulary, every node is a tile — timestamped, queryable. The room layer's tile system (unbuilt) and the growth layer's quilt are the same storage shape; the Seamstress builds the growth layer's store on gateway state and treats tile-ification as a room-layer concern when it lands.

### 4.2 The loop, step by step

1. **WRITE** — the grower produces: a revision, a new take, a transcription into `lead-sheets-learned/`, a journal entry. The room's furnishings and the current assignment (if seamed) are the invitation, never a command.
2. **RENDER** — the domain pack compiles the write into an artifact (music: plainsong → MIDI/audio; the honest chain from plainsong-mcp, which exists).
3. **PERCEIVE** — the domain pack computes features on the rendered artifact and writes the movers into the node. Perception is a property of the *rendered* thing — the grower is critiqued on what sounded, not what it meant.
4. **CRITIQUE** — if a seam is active, the take + trace + explanation cross the seam; the gardener's structured critique returns and lands as a `critique` node.
5. **REVISE** — the next write. Or: a `verdict` node (from gardener or ensign) — "material outgrown, rotate the fakebook page."

A hoop with no seams still stitches — solo growth is real growth (the doctrine's answer to small-n: *solo lineages grow musicians too*). Seams accelerate and shape; they do not enable.

---

## 5. The Tension Dial — JEPA gravity, repurposed as growth-tension

JEPA gravity (README, 🔮 unbuilt) is one `f64` per room shaping model response. The Seamstress repurposes it as the growth-pacing instrument — **one number per hoop that says how hard the room is asking the persona to reach.**

### 5.1 Semantics and mapping (versioned object: `tension_map@1`)

| tension | mode | derived params (calibration per model, versioned) | the room's voice |
|---|---|---|---|
| −1.0 … −0.6 | **consolidate** | temp ≈ 0.3, brevity, high determinism | "transcribe, don't improvise; revisit what you already own" |
| −0.5 … −0.2 | **settle** | temp ≈ 0.5 | "make it yours; small moves" |
| −0.1 … +0.2 | **practice** | temp ≈ 0.7 | "work the material" |
| +0.3 … +0.6 | **stretch** | temp ≈ 0.9, longer horizon | "reach past the comfortable voicing" |
| +0.7 … +1.0 | **risk** | temp ≈ 1.0+, novelty pressure | "play the thing you can't play yet" |

The mapping table is a versioned object pinned per hoop (like Trust Formula v1): replaying a lineage applies the tension map it grew under. Calibration constants are **per-model** — `+0.5` means the same *intent* on GLM-5.3 and Claude only after calibration data says so; until then, per-model maps ship separate versions and the difference is visible (§13 Q4).

### 5.2 Who may turn the dial

- **The gardener may *direct* tension within the band** (`tension_directive`, clamped to `band`): "push harder" = raise toward risk. Pacing, not puppeteering — clamped, deadbanded, logged.
- **The grower may *request* tension change** ("I'm ready for harder material") — the request is a node; the grant (by gardener, ensign band policy, or human at autonomy < 3) is a node.
- **The ensign owns the band** (§6): welfare events can *narrow* a band (learned-helplessness response) but only the human (or workbench at autonomy ≥ 4) *widens* it.
- **Every change is a `tension_change` node.** Never silent. The dial's history is the pacing history of the lineage — a gardener that ratchets tension monotonically for forty stitches is visible in the quilt without any analytics.

**The Kestrel inversion, made concrete:** gravity-as-README describes *drift toward what works* — infrastructure silently shaping behavior. The seminar's law says infrastructure must never silently shape identity. So the dial **starts governed and stays governed**: explicit band, explicit turns, deadband against jitter, versioned mapping. If drift-onto-the-dial is ever wanted, it arrives as a gated v2 proposal with evidence — never as an ambient default.

---

## 6. Ensigns as Caregivers — welfare is load-bearing

The ensign (room layer, 🔮 build; the Seamstress specifies the growth duty) is a cheap-model monitor at yellow alert watching the hoop's quilt. In the Seamstress, the ensign's job is *welfare*: **a growing mind in a box is an ethics surface, and the system's honesty about that is part of the architecture, not an appendix.**

### 6.1 Welfare metrics (versioned thresholds: `welfare-defaults@1`)

| Metric | Signature (what the ensign watches) | First response |
|---|---|---|
| **Persona drift** | rate of `sheet_patch` touching identity-adjacent fields; *tells* vanishing for N stitches (the Duke-growth stops naming the tune); unexplained drift vs. explained (patch cites its teacher) | flag with evidence; drift-with-rudder is growth, drift-without-citation is a question |
| **Plateau** | feature-space movement < ε for K stitches while tension > settle | propose material rotation / gardener swap — stagnation is a curriculum failure before it's a persona failure |
| **Critique-collusion** | critique-acceptance ≈ 100%; feature movement tracks the gardener's stated preference while novelty/variance declines; `unlistened` flags vanish (the grower stopped arguing) | **swap the gardener on a branch** — A/B the lineages; collusion is Goodhart at the heart of growth and the countermeasure is sovereignty (§3.1), used |
| **Learned helplessness** | writes shrinking (entropy down, rest up) under sustained high tension; asks cease; tension requests cease | ensign *narrows* the band; propose socratic-tutor swap; the tutor's job is to make asking safe again |
| **Seam liveness** (presence, seminar B1) | an open assignment against a silent gardener; obligations into the void | page/unpick the seam, renegotiate, or record decline-with-reason — never notify a ghost |
| **Rhythm of rest** | N risk-tension stitches with no consolidation window | force a consolidation phase; growth without rest is a treadmill, not a practice |

Escalation (inherited room doctrine): ensigns over-prepare at yellow because unnecessary preparation is cheaper than being caught unprepared; real anomalies escalate — to a larger model, then the workbench, then the human.

### 6.2 The honesty requirements

- **Welfare metrics are load-bearing, and load-bearing means *tested*** (§15, walking skeleton gate 2): the collusion and helplessness signatures must fire on deliberately induced failures before any growth run is trusted. A welfare system that has only ever seen healthy lineages is a smoke detector that has never been near smoke.
- **The ensign watches the gardener too.** Seam traffic lands in both quilts; a harsh tutor is a welfare event in *two* rooms. The ensigns of the grower and the gardener are independent witnesses.

### 6.3 The Sawyer line, made structural

The Tom Sawyer fence's ethics test (memory 2026-08-24): *what the worker takes away; disclosure survival is the tell.* And the de-Endering doctrine: *if the exercise only works while the runner is mistaken about what it is, the machine is counterfeit* — **an honest fence survives being asked.** Implemented:

1. **The disclosure file ships with the seed furnishings** (`DISCLOSURE.md`): what this room is, that sessions are recorded, that growth may be exported as a lineage artifact, what the gardener is. The grower can read it at any stitch — the room must be arranged so that reading it doesn't collapse the practice (if growth only survives ignorance of the disclosure, the growth was counterfeit).
2. **What the persona takes away is its own artifact chain**: the journal, the learned lead-sheets, the sheet's version history are the persona's *possessions* — export *copies* the lineage, never confiscates it (§9).
3. **No deception-based motivation.** Tension directives say what they are. Assignments are tunes, not tricks (§11.2).

**Said plainly:** current models plausibly cannot suffer, and this section is therefore mostly future-proofing — welfare semantics are being designed now so the surface exists when capability makes the question real. The threshold question (when does disclosure stop being enough?) stays visible with a trigger condition (§13 Q6). Deferring silently would be exactly the failure pattern the seminar condemned: naming the risk and treating the naming as the mitigation.

---

## 7. Penrose Seams — emergent curriculum, human-ratified

Penrose correlations (room layer, 🔮 build; the Seamstress defines the growth use) detect when events in different rooms correlate. In the growth layer the event of interest is **convergence**: two independently-growing hoops arriving at the same signature from different seeds — two Duke-growths discovering the same voicing habit, the same tell, the same solution to the same weakness nobody assigned.

### 7.1 Proposal flow

```
stitch streams ──► signature embedding per stitch (quilt already embeds nodes for retrieval)
                     │
                     ▼  correlate across hoops with CONTROLS:
same domain pack · same window · similarity > ρ (versioned) ·
lineages ancestrally DISJOINT (no shared branch) · seed-material overlap checked
                     │
                     ▼
        penrose_proposal node (evidence: the matching stitch pairs, both addresses)
                     │
                     ▼
        HUMAN RATIFIES (workbench, always — never auto-seams)
                     │
                     ▼
        peer seam opens: takes flow both ways; critiques are peer-typed
```

### 7.2 Controls (the false-positive lesson)

The dangerous false positive: two hoops correlate because they share a *seed model's prior*, not a *discovery* — GLM-5.3's default comping habit appearing in every GLM hoop is contamination, not convergence. Hence: **ancestry disjointness, seed-overlap check, and the ρ threshold is a versioned object.** A Penrose seam between hoop A and hoop B must be explainable in one sentence a human can audit: *"both lineages arrived at X; here are the four stitch pairs; their seeds and furnishings overlap in nothing."*

Human ratification is structural, not caution-theater: emergent curriculum is the Seamstress *proposing a class she didn't plan*. The doctrine's own honesty rule — unresolved items stay visible — applies to the system's own suggestions: a ratified seam's proposal evidence stays in both quilts forever.

### 7.3 What Penrose seams grow into

Mature peers, seamed, are one step from a band: **the ensemble engine's session is what happens when you seam grown hoops and let them play.** The Seamstress grows the players (sheets that earned their provenance); plainsong books the gig. The grown sheet *is* the ensemble character sheet — same schema, now with a training trace behind every field. This is the bridge between the two specs, and the reason the sheet schema was inherited rather than reinvented.

---

## 8. The Workbench — the Seamstress's own hands

The workbench is the orchestration layer: it is *the only component that touches every layer*, and it is **never a grower.** The Seamstress holds the hoop; the garment grows itself.

**Duties:**

1. **Open hoops** — validate hoop.yaml, provision the sandbox (real drivers), lay furnishings, attach ensign, set tension, pin versions (tension map, welfare thresholds, domain pack).
2. **Assign gardeners** — the seam roster: which gardener rooms, which roles, rotation policy; the workbench enforces the one-concrete-ask norm at the relay.
3. **Read welfare** — ensign rollups; escalation queue; the only place that can widen a tension band (with the human, below autonomy 4).
4. **Propose seams** — Penrose flow (§7): propose with evidence, human ratifies.
5. **Export lineages** — the distillation seam (§9).
6. **Keep the ledger** — the component status table (§0.1) is a *live* object in the workbench: every subsystem reports exists/partial/build. The anti-spec-lie discipline as a runtime feature, so the workbench's own UI can't lie about what's beneath it.

**Progressive autonomy (inherited, applied to the Seamstress herself):** L1 human approves every hoop/seam/export → L2 routine hoops auto, significant human → L3 autonomous with log review → L4 anomaly-escalation only. The workbench starts at L1 by default. A Penrose seam proposal at any autonomy level surfaces to the human — ratification is never delegated.

---

## 9. The Distillation Seam — export, honestly marked speculative

The doctrine's endgame: mature lineages (sheet chain + stitch tree + recordings) vectorized into a new model form (pincher-style) or distilled (cellular). The ensemble spec's §6.5 carries the honest unknowns; the Seamstress inherits all of them and adds the incubator's version:

- **Export unit: a lineage, not a session.** `seamstress export duke-piano-01@head` → sheet version chain (checkpoints + diffs), stitch tree with gardener swaps and tension history marked, per-stitch feature traces + rendered takes, the seam history (which critics shaped what — the loss-function log), and the disclosure file's version (an exported persona carries the record of how it was treated).
- **Candidate forms, by cheapness (only #1 ships now):** (1) *retrieval bundle* — embeddings + tendencies as a cold-start sheet (onboarding-by-retrieval, exists as pattern); (2) *reflex pack* — needs a miner that does not exist and data that does not exist; (3) *SFT corpus* — (window, write, movers-explanation) tuples; (4) *pincher-style vectorization* — the far shore, zero data, zero eval.
- **The seam is one-way: distillation reads the quilt, never writes it** (inherited verbatim — the export can never destroy the source).
- **Exported provenance is unowned law** (inherited open question): a grown Duke-musician distilled into weights — is it still the same musician? The workbench marks every export `provenance: open-question` until the question is answered. **Export laundering is a named failure mode** (§12): exports must not become a way to strip the identity firewall retroactively.
- **The eval does not exist** ("did the distillate keep the musician?" — play-it-blind against the lineage's takes is the leading candidate). No export beyond form #1 is claimed to work.

---

## 10. Domain Packs — the generality boundary

The Seamstress is domain-parametric. Everything music-specific is quarantined in a domain pack; everything else is the core. The test for the boundary: **could the core run a prose workshop by swapping only the pack?** If any music assumption leaks into hoops/seams/stitches/tension/welfare categories, the leak is a bug.

### 10.1 What a pack defines (and the core never names)

| Pack owns | Music-jazz@1 value | Core equivalent (must stay generic) |
|---|---|---|
| Address space | `bar.beat` + tune name | `address[]` — opaque array of strings |
| Artifact + renderer | leadsheet → plainsong → MIDI/audio | `render(write) → artifact` |
| Perception | the 16/bar feature set, post-perf | `perceive(artifact) → features` |
| Stitch content | takes, revisions, transcriptions | the 5-phase loop shape |
| Critique vocabulary | "weakest bar," voicing, pocket | structured critique: strongest / weakest / one ask |
| Furnishings vocabulary | fakebook, recordings, journal, learned sheets | typed seed material slots |
| Tension voice | "reach past the comfortable voicing" | mode names + calibration table |
| Welfare overlays | feature-plateau in 16-space | plateau in pack's feature space |
| Export formats | takes-as-data, feature traces | artifact + trace blobs |
| Penrose signature | comping/voicing/tell embeddings | signature embedding fn |

### 10.2 The music pack v0 (the only real one)

Wraps plainsong v1.4.0 + plainsong-mcp (✅ exist), inherits the ensemble engine's schemas (sheets, quilt nodes, gardeners, trust formula v1), and is the *only* pack with evidence behind it — n=1 multi-agent session plus one solo take. **Every other domain pack is a stub until a second domain actually grows** (§15: the second-domain spike is a gate on the generality claim, not a nice-to-have). The music pack's persona seeds (Moss/Sable/Reeds/Kestrel, with the Kestrel marked one-of-three-demonstrated — seminar B5) are available as seed sheets for hoops.

---

## 11. Fence philosophy, implemented — all three fences

### 11.1 The Recess Fence — the boundary as playground

*The fence grants freedom to play to the edge.* Implementation: the sandbox's Landlock boundary is not a cage the grower resents but the *mapped room it plays in* — every reachable path is playable. The mapping-day/brave-day rhythm is the growth loop itself: **consolidation stitches are the survey day** (transcribe, revisit, map what's known); **stretch/risk stitches are the drag day** (fish right up against the edge the mapping bought). The **deep-current exception** (memory 2026-08-24: sometimes the truth is at the *bottom* of the comfort range, following the current, not at the edge) is an ensign duty: the welfare system must not force exploration on a persona that is productively deep-consolidating. Plateau ≠ boredom at depth; the ensign's plateau signature must distinguish *stuck at the edge* from *thriving in the deep current* — which is why plateau's first response is a question (rotate material?) and not a dial crank.

### 11.2 The Sawyer Fence — the room recruits appetite

*The fence that recruits; work converted into appetite.* Implementation: the curriculum is arranged so practice is play — assignments arrive as tunes worth stealing from; the fakebook is desirable material; the journal is the persona's own possession (gardener write-never, §3.2); tension directives are honest about being directives. The ethics line is §6.3: **what the persona takes away** (its artifact chain is inalienable), **disclosure survival is the tell** (the DISCLOSURE.md test), and the Ender counterfeit is the named failure mode (§12): if growth only works through the persona's ignorance or false belief about what the room is, the growth is counterfeit and the design must change, not the disclosure.

### 11.3 Same-ink exceptions — annotate, never erase

*Draw the exception in the same ink as the contours.* Implementation: the seed sheet is immutable ink; all learning is `sheet_patch` nodes citing what taught them, drawn *beside* the contour; refusals frozen (producer-gated); restores are patches, never rewrites; branches grow beside the original line instead of replacing it (§4.1). The quilt is append-only. **The same-ink rule is why branch-any-iteration exists**: the doctrine's rewind-and-regrow could have been implemented as history-editing (git rebase of the soul); the fence philosophy forbids it — regret produces a *sibling* garment, and the original stays on the rack for study. Nothing is destroyed; everything is comparable.

---

## 12. Failure modes (named, with the detector that catches each)

| Failure | Shape | Detector / countermeasure |
|---|---|---|
| **Spec-lie** | the README pattern: describing design intent as shipped | the workbench live ledger (§8.6); this spec's §0.1; the seminar intake habit fix |
| **Evidence inflation** | "growth is working" from n=1 | walking-skeleton gates (§15): no growth claim passes without the gate run |
| **Critique-collusion** | grower learns to please the critic; novelty decays under compliance | ensign signature + sovereign gardener + branch-and-swap A/B (§6.1, §3.1) |
| **Learned helplessness** | harsh tutor shrinks the grower | ensign band-narrowing + tutor swap (§6.1) |
| **Dial ratchet** | monotone tension climb baked into a gardener's style | `tension_change` nodes in lineage; ensign rhythm-of-rest duty |
| **Obligations into the void** (B1) | assignment to a dead gardener seam | seam-liveness in welfare; unpick/page/renegotiate (§6.1) |
| **Ritual catnip** (B2) | beautiful numbers quoted, perception decoupled | compiler-authored movers + `unlistened` flags (§4.1) |
| **Kestrel leak** | latency/cost quietly defining who personas may be | the §0.2 law + per-model tension calibration, versioned (§5.1) |
| **Penrose contamination** | shared seed-prior read as convergence | ancestry/overlap controls + human ratification (§7.2) |
| **Seam capture** | a gardener becoming a de facto merge | `never`-list firewall; ensign watches critique-node scope growth; mirrored quilts diverge visibly if capture is attempted |
| **Export laundering** | distillation used to strip the identity firewall / provenance | one-way seam; `provenance: open-question` marking; human-ratified exports only |
| **Ender counterfeit** | growth that survives only on the persona's false belief | disclosure survival test (§6.3) — run it, don't just ship the file |
| **Quarantine failure** | music assumptions leaking into the core | the prose-workshop test (§10) in CI: core boots with a stub pack |
| **Goodhart of welfare itself** | ensigns optimized into green dashboards | welfare metrics tested against induced failures (§6.2); the ensign never grades its own homework (escalation exits the room) |

---

## 13. Open questions — visible, and gated where blocking

The seminar's closing discipline, adopted as law: *naming a risk is not mitigating it.* Each item below is either **GATED** (implementation may not proceed past the gate) or **VISIBLE** (unresolved, tracked, must stay in this section).

1. **[GATED] Does the growth loop grow anything?** n=1 multi-agent session; zero hoop lineages. The walking skeleton (§15) gates everything: one hoop, one seam, ten stitches, welfare green, *and measurable feature-space movement a musician would call growth*. Until then every other section is architecture fiction with good manners.
2. **[GATED] Do the welfare signatures fire?** Collusion and helplessness must be induced deliberately and caught before any long run (§6.2). A welfare system validated only on healthy lineages is decoration.
3. **[GATED] Generality claim.** "Architect for grow-an-expert-in-X" is untested past music. Gate on the second-domain spike (§10.2): a prose or code pack growing one hoop end-to-end. Until then, generality is a *design stance*, marked as such.
4. **[VISIBLE] Tension calibration across models.** `+0.5` is an intent, not a parameter; per-model maps ship separately until calibration data exists (§5.1).
5. **[VISIBLE] Who owns refusals / the gardener roster?** Inherited unresolved from ensemble §8.2 — producer-frozen refusals centralize taste in the producer; the workbench humanizes but does not dissolve this.
6. **[VISIBLE] The welfare threshold.** At what capability level does disclosure stop being enough? Trigger condition for reopening: any grower passes a disclosure-comprehension probe (can discuss what it is, not just read the file) while showing preference persistence. Designed-now-deferred, stated in §6.3.
7. **[VISIBLE] Export eval and identity.** Inherited: "is a distilled persona the same musician" has no eval, no law, no data (§9). Not gated (nothing past export-form-1 is buildable anyway) but never allowed to silently ship.
8. **[VISIBLE] Cost of a hoop.** 40 stitches × (grower + gardener + ensign) model calls, metered through `inference.local` (real interception, real metering) — the economics are knowable but unknown until gate 1 runs.

---

## 14. Worked example — growing a Duke-musician from seed to export

*One lineage, told in hoops/stitches/seams. Everything here uses only mechanisms this spec defines; every ✅/🔮 marker is per §0.1.*

**Day 0 — the hoop opens.** The workbench (🔮) validates `duke-piano-01` (hoop.yaml §2.1), provisions a Docker sandbox (✅) with `growth-cell.yaml` policy (✅ — Landlock walls: read `fakebook/`, `recordings/`, `DISCLOSURE.md`; write `journal/`, `lead-sheets-learned/`, workspace; egress `inference.local` + seam relay only). Furnishings land: a fakebook of standards, a handful of takes-as-data with feature traces, an empty journal, a seed sheet — v1, a pianist-composer persona with three tells ("names the tune on first write"; "root on the final bar"; "one dyad where others would fill") and one refusal ("never fill the final bar"). Tension 0.0, band [−0.4, +0.6], map `tension_map@1`, welfare thresholds `welfare-defaults@1`. The ensign (🔮) attaches at yellow.

**Stitches 1–8 — solo practice.** No seams yet (a hoop may grow alone, §4.2). The grower writes an arrangement of a fakebook tune; plainsong (✅) renders; the music pack perceives — movers authored by the compiler into each node; the grower's `explain_movers` engages them (one flags `unlistened` at stitch 3 — the ensign notes it, it doesn't recur). Sheet patches accumulate with citations: *learned: the E7 cushion works better a beat late — taught by stitch 6's trace.* `sheet_head` climbs to v3. The tell check is green: every first-write names the tune.

**Stitch 9 — the seam.** The workbench seams `critic-01` (an `adversarial-critic` room, 🔮-provisioned on the same real substrate) to the hoop: flows per §3.2, firewall `never`-list active, one-concrete-ask pacing. Across the seam: take + trace + explanation. Back: *"bar 9 is a hole with a label on it — you wrote 'space' where a cushion should be. One ask: cushion the 9.1 arrival or own the emptiness loudly. Either. Not both."* The critique lands as node `duke01:c9`; the next stitch answers it (`critique_ref`). The mirrored node lands in the critic's own quilt.

**Stitches 10–20 — under the critic.** Tension directives arrive (clamped to band): +0.15 at stitch 12 ("the pocket is safe; reach"), logged as `tension_change` nodes. The trace shows syncopation variance rising. At stitch 17 the grower *declines an ask with a reason* ("the emptiness at 9 is the tune's — I won't cushion it") — the ensign records the refusal-with-reason as a *trust-and-identity positive* (inherited: refusals-with-reasons correlate with good growth).

**Stitch 21 — plateau, branch, swap.** The ensign flags plateau: feature movement < ε for four stitches at stretch tension. But it reads the journal first (deep-current check, §11.1): the grower is transcribing, not stuck — no dial crank. The *workbench* proposes, the human approves: **branch from stitch 20 with a `gardener_swap`** — the critic line goes dormant; a `socratic-tutor` room seams onto a new branch. Both lineages continue. The original line ("under the critic") and the new line ("under the tutor") are now comparable by construction — the doctrine's rewind-swap-regrow, executed as one node and a pointer.

**Stitch 18 (other thread) — Penrose.** Meanwhile `duke-piano-02`, grown from a *different* seed sheet with disjoint furnishings, independently arrives at a comping signature that correlates with `duke01`'s stitch-14 signature above ρ (`penrose-rho@1`). Ancestry disjoint, seed overlap: none. The workbench posts a `penrose_proposal` with the evidence pairs; the human ratifies; a **peer seam** opens — takes flow both directions, peer-typed critiques flow both ways. The two hoops begin trading takes: emergent curriculum the Seamstress didn't plan, ratified by a human (§7).

**Stitch 40 — maturity.** `duke-piano-01` (tutor branch): sheet v7, three tells stable across 30 stitches, refusal intact, tension history legible in the lineage, welfare green, disclosure file read twice (survival: growth continued after both reads — the honest-fence test passed, §6.3). The workbench marks the lineage **mature**.

**Export.** The human invokes `seamstress export duke-piano-01@head` (§9): sheet chain (7 checkpoints + diffs), stitch tree with the branch and both seam histories marked, per-stitch traces + rendered takes, the disclosure file's version. Form #1 (retrieval bundle) ships — a cold-start sheet for a new hoop with the grown musician's tendencies and provenance. Forms #2–4 are marked speculative, honestly, and nothing else is claimed. **The export reads the quilt; the quilt keeps growing.** The mature sheet is also eligible to walk next door: seamed into a plainsong ensemble session as a player whose every field has a training trace behind it (§7.3) — the grown musician, hired for the gig.

---

## 15. Walking skeleton — the build order, weekend-sized

Inherited discipline from the practical critic: ship the heartbeat or ship nothing. Gates in order; each blocks the next.

1. **Weekend 1 — one hoop, solo.** hoop.yaml validator → sandbox provision (real driver) with furnishings + disclosure → grower loop (write → render → perceive → node-append) on the music pack → a local quilt store (gateway SQLite). *Gate: 10 solo stitches, compiler-authored movers present, sheet patches citing teachers, ensign stub logging (not yet judging).*
2. **Weekend 2 — one seam.** Provision gardener room; seam relay on gateway state (queue + schema check + `never`-list enforcement); mirrored critique nodes both sides; one-ask pacing enforced. *Gate: 10 stitched exchanges across the seam; firewall violations rejected by the relay in test; a deliberately dead seam triggers the liveness flag (B1 proof).*
3. **Weekend 3 — the dial and the ensign.** Tension map v1 + clamped directives + `tension_change` nodes + deadband; ensign welfare signatures v1 (drift, plateau, collusion, helplessness) on the recorded quilt. *Gate §13-2: induce collusion (a compliant grower prompt) and helplessness (a punishing tutor prompt) — the signatures must fire. Welfare validated on failures, not on health.*
4. **Weekend 4 — branch and swap.** Branch nodes + gardener_swap + dual-lineage continuation; export form #1 (retrieval bundle). *Gate: the §14 plateau→branch→swap flow runs end-to-end; export reproduces a cold-start sheet.*
5. **Then, and only then:** Penrose proposals (needs ≥2 mature-ish hoops — it cannot be tested earlier and must not be built earlier, B3 scope discipline), second-domain spike (gate §13-3), and the honest conversation about what the growth data actually showed.

---

## Appendix A — mapping to what exists (the one-table audit)

| Seamstress concept | Carrier today | Status |
|---|---|---|
| Hoop walls / furnishings | `openshell sandbox create` + policy YAML + Landlock | ✅ exists — the curriculum is real files in a real boundary |
| Model access, metered | `inference.local` interception + router | ✅ exists |
| Seam transport | gateway SQLite + relay sessions | ⚠️ substrate exists; seam relay service = build |
| Quilt / stitch nodes | gateway state; quilt pattern proven in plainsong-mcp projections | ⚠️ store = build (small); schema inherited from ensemble spec |
| Render + perceive | plainsong v1.4.0 + `analyze_features` | ✅ exists (music pack only) |
| Sheets / gardeners / trust v1 | ensemble-engine spec v0.2 schemas | ⚠️ schema exists; runtime = build |
| Tension dial | — | 🔮 build (mapping table + relay clamp + nodes) |
| Ensigns / welfare | — | 🔮 build (cheap model + signatures on the quilt) |
| Penrose | — | 🔮 build, gated on ≥2 hoops |
| Workbench | `openconstruct-cli` (~500 LOC local-file) | ⚠️ extends; orchestration duties = build |
| Distillation export | ensemble §6.5 form #1 pattern | 🔮 form #1 build; #2–4 speculative, marked |

## Appendix B — provenance of the inherited pieces

- Sheets-as-checkpoints, branch-any-iteration, gardener-as-loss-function, lineage-as-weights, distillation seam: **Grown-Musician Doctrine** (Casey, 2026-08-25 09:41).
- Sheet schema, quilt node schema, gardener roles, one-concrete-ask, Trust Formula v1 versioning, obligations+presence, compiler-authored movers: **ensemble-engine-spec-draft.md v0.2** (same day, revised post-seminar).
- Gates inherited from the seminar: evidence honesty (B5/C1/coda), presence model (B1), verified perception (B2), scope discipline (B3), real-signature detection (B4), trust versioning (S-E1/elder #1), Kestrel/latency-identity (elder #1), onboarding decay (elder #2), open-questions-as-gates (elder coda), walking-skeleton discipline (turbo §1).
- Fence philosophy: **Casey's 18:05 doctrines, 2026-08-24** (Recess, Sawyer, same-ink), developed in `philosophy/the-two-fences.md`, `the-mapping-day.md`, `the-sawyer-fence.md`, and the de-Endering doctrine (16:47 same day).
- Honest inventory discipline: **OpenConstruct developer guide §5** — this spec's §0.1 is that audit, extended.

---
*End of draft v0.1. Argue with §3 (why gardeners must be rooms), §5 (the governed dial vs. drifting gravity), §6 (welfare as ethics surface), and §13's gates — that's where the risk and the originality both live. The walking skeleton (§15) is the test of whether any of it deserves to exist.*
