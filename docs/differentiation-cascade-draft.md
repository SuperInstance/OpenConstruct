# The Differentiation Cascade — OpenConstruct Architecture Extension, Draft v0.1

*From the Stem-Cell Doctrine (Casey, 2026-08-25 15:40). Drafted same day by the design foreman.*

**The constitution, verbatim:** *"A model is like a stem cell — as it decomposes into more specific jobs it prunes its potential into less and less scope: bone for structure, muscle for mechanics, tendons to glue, neural networks to message and coalesce into a central cord that plugs into a higher-function iteration center (frontal cortex, j-space, inner simulation). Incubation in a SuperInstance/OpenConstruct system can grow from the DNA of an LLM to where very little of the organism needs a full LLM, because simpler and simpler modeling is architectable into a neural intelligence inter-connector."*

This document extends the Seamstress spec (`seamstress-spec-draft.md` v0.1) from **growing one persona in one hoop** to **growing an organism out of many cells at many tiers of commitment**. The Seamstress grows the musician; the Cascade grows the *body the musician plays*. Everything here inherits the spec's laws: honest inventory (§0.1), the Kestrel constraint (§0.2), append-only quilts (§4.1), ensign welfare (§6), human-ratified emergence (§7), same-ink (§11.3).

**Evidence standing behind this draft (both from today, both real):**
- **Gate 1** (`plainsong-mcp/stitch/GATE1-REPORT.md`): a growth loop measured as a shrinking curve — distance to canon 7.14σ → 1.69σ over ten stitches, 5 of 6 features monotone. Its eye is *already a differentiated cell*: "a function, not a model — nearest neighbor + largest normalized gap." Its honesty ledger is this spec's first law: "the grower did not learn; it is a deterministic in-context policy."
- **Duke R3** (`duke-lab-r3/REPORT.md`): a full model became Duke-shaped — velocity_std 0.113→0.200, syncopation 0.355→0.410, blind critic verdict **CONVERGED: Duke?** — and *the pattern lives in the sheet*, not the weights. Three critiques survived R2 and died in R3: the tendency-census this document turns into fate decisions.

---

## 1. The Cell Tier Ladder

Four tiers. **Downward is distillation: cheaper, faster, stiffer. Upward is escalation: only on novelty, wound, or failure.** An organism's economy is the mix.

| Tier | Name | What it is | Carrier | Cost | Latency | Plasticity | Failure mode |
|---|---|---|---|---|---|---|---|
| **T0** | **Totipotent** | The full LLM, seed-sheeted, in a hoop. Can become any cell. Reserved for: germ line (Seamstress spawning), cortex, wound healing. | full model + sheet (a hoop, spec §2) | $$$ | seconds | total | **Cost tumor** if fired for everything; Kestrel leak ("a benchmark wearing a name") |
| **T1** | **Multipotent** | Domain-committed: a smaller model, a fine-tune, or a full model behind a sheet so specific it behaves as one organ. Fate is chosen; scope within it is still open. | domain model + committed sheet (Duke R3's band is exactly this) | $ | sub-second | within-domain | Scope creep (wants T0 back); staleness as canon drifts |
| **T2** | **Differentiated** | Deterministic mechanics: rules, lookup tables, cue-reflexes (pattern → action, no model call), **[f32;16] manifold cells** (feature-vector gates — a take's signature vs. a canon manifold, the Gate-1 eye's nearest-neighbor check made permanent). | code + tables + embedding gates | ~0 | microseconds | none without re-distillation | **Wrong-but-fast**: brittle at the edge of distribution |
| **T3** | **Sclerotic** | Pure data: the recorded pattern itself — rendered takes, feature traces, cached curves, sea-state logs. Never executed; consulted or played back. | tiles, blobs, the quilt's own artifacts | 0 | 0 | none | **Stale truth** (the chart from 2019) |

**Naming honesty:** *sclerotic* is not a slur. Bone is sclerotic tissue and bone is structure. T3 is the organism's skeleton — the pattern, preserved. The disease is **sclerosis-in-error**: a sclerotic or differentiated cell that is now *wrong* (§3.4). Structure until it is wrong.

**The two directions are not symmetric — this is the ladder's law:**

- **Distillation is the default direction.** Every stable tendency drifts downward (§2–3) because cheaper-and-faster is always available to a proven pattern. No permission needed to *propose*; sign-off to *promote* (§2.2).
- **Escalation is rationed by the deadband.** Each cell carries an `escalation_deadband` (a versioned object, *not* the tension deadband of spec §5 — term collision resolved by name): a measurement within the deadband is answered by the cell's own tier; only novelty (a signature outside the training manifold), wound (a failed T2/T3 answer), or contradiction (two cells disagreeing beyond tolerance) buys one rung upward. **This is the ensign's deadband doctrine (README) promoted to an economic law: over-preparing is cheaper than being caught unprepared — but routine work never prepares at all.** Escalation is logged as a tile; an escalation that repeats becomes a wound (§6).

**The claim the ladder makes against today's practice:** most agent systems are all-T0 — every function re-thought by a full model at every firing, an organism made entirely of stem cells, which is both the cost disaster and the identity disaster (nothing specializes, nothing is reliable). Gate 1 proved the opposite end: a whole growth loop ran with zero model calls in the eye and a deterministic policy in the grower's chair, and *still produced a measured growth curve*. The truth lives between, and it moves downward over time.

---

## 2. Fate Decisions — commitment down a tier

A cell's fate is decided by its **sheet**, because the sheet is the genome (doctrine): differentiation *prunes potential into scope* — the grown musician is the model with most of itself deliberately silenced, and **the silencing pattern lives in the sheet**. A fate decision is therefore a sheet event with a lineage, not an ops tweak.

### 2.1 The trigger: N-survived-critiques

The hoop's sheet accumulates tendencies as `sheet_patch` nodes citing what taught them (spec §4.1). A tendency is a **distillation candidate** when:

1. It has **survived N critiques without being named weakest** (N = versioned, default 3). Duke R3's critic language is the template: *"All three surviving R2 critiques are dead"* — the lab already counts exactly this. Survival, not applause: a tendency nobody can wound.
2. Its **firing pattern is stable**: the situations that invoke it are nameable (a cue signature exists in feature space) and recur above threshold frequency.
3. **No error associations**: the ensign's log shows no escalation originating from this tendency within the window.

### 2.2 The sign-off: canon-fit of the distilled behavior, not the outputs

The gardener/eye does not grade the tendency's past outputs (they already survived). It grades **the distillate's behavior**: the candidate T2/T3 cell runs the **shadow-firing protocol** — N firings in parallel with its parent, same inputs, and the eye grades the candidate's outputs against the same canon, the same retrieval-grounded fit-in-canon check (spec §3.2). Promotion requires the distillate's feature signature to stay within ε (versioned) of the parent's *on the firing distribution*, and the eye must sign a node saying so.

**This is the eval the Seamstress spec said did not exist** (§9: "did the distillate keep the musician?"). At organism scale the question decomposes into bounded, runnable per-cell shadows. The *deep* form (is the distillate the same cell? — provenance) stays `open-question`, inherited.

### 2.3 The law: cost proposes, canon disposes

Cost curves (firing frequency × tier cost, metered through `inference.local`) *propose* candidates for demotion. The eye's shadow-firing *disposes*. **A cell is never demoted for cost alone.** This extends the Kestrel law (spec §0.2: "admission decides what runs, never who a persona is allowed to be") from the hoop to the organism: *an organism whose cell fates are a function of this quarter's inference budget isn't an organism — it's a cost dashboard wearing a body.* Cheap-but-wrong is never promoted; expensive-but-earned is never demoted until canon-fit says the tendency is stable.

### 2.4 Rollback = the sheet checkpoint

Branch-any-iteration (spec §4.1, the grown-musician doctrine) **applies to cell fate**: every `fate_decision` node records its `sheet_head`; a demotion gone wrong is rewound by branching from the checkpoint and re-growing — not by editing history. Same-ink: the failed distillate retires to archive (§5.2), it is never erased.

---

## 3. Myelination — the reflex-promotion mechanic

Repeated signal paths between cells get faster and cheaper. That is the whole mechanism, and it is the answer to "how does the organism get *more* efficient without anyone retuning it" — the README's Penrose efficiency promise, given a body.

### 3.1 Promotion

A **path** (cell A's output → cell B's input, over the seam/relay fabric) that fires **N times without error** (error = ensign-flagged deviation or an escalation originating downstream) is compiled down one tier: an A→B relay row that used to route through a T1 hop becomes a direct table entry; an escalation that keeps returning the same answer becomes a cached cue-reflex (T2). Each promotion is a **`myelin` node** in the quilt — append-only, so the one-way distillation law (spec §9) survives in-vivo: *differentiation writes new nodes; it never rewrites old ones.*

### 3.2 Penrose grows axons

Penrose correlations (README; spec §7) detect correlated events across rooms. In the Cascade, Penrose's job is **axonogenesis**: it proposes *new paths* between cells whose firings correlate. The contamination controls carry over verbatim (ancestry disjointness, seed-overlap check, ρ versioned, **human ratifies — never auto-wires**): two cells correlating because they share a seed model's prior is contamination, not convergence.

### 3.3 The connectome is dumb — and that is the point

The quilt-as-connectome carries signals; **axons connect, they don't think.** No path ever *generates* content; a myelinated path that starts generating (improvising on its cargo) is a mutation — the ensign watches for it the way it watches seam capture (spec §12): the relay must remain byte-honest between source and destination.

### 3.4 Arthritis — sclerosis-in-error

The ensign's welfare duty (spec §6.1) extends to tissue: a T2/T3 cell whose outputs now diverge from fresh canon samples (the eye's distance check, run periodically against *current* data, not the distillation-era data) is **arthritis** — structure that has become wrongness. The signature: repeated escalations originating from the same site, or a sclerotic datum contradicted by newer sclerotic data. First response is not deletion — it is **wound healing** (§6): recall the stem cells, regrow, re-distill. Arthritis is the *ordinary* cost of a living organism in a drifting world; it is the trigger, not the scandal.

---

## 4. The Central Cord and the Frontal Cortex

### 4.1 The cord

The myelinated path fabric — seams, relays, cached reflex arcs — is the **central cord**: fast conduction that bypasses the cortex entirely. A knee-jerk is a T3 datum hitting a T2 gate and firing a T2 action with *no model anywhere in the arc*. The cord is the organism's autonomic system: heartbeat (sensor cadence), reflexes (deadband-guarded gates), posture (sclerotic calibration). It is built *first*, because it is the only tier cheap enough to run continuously at the edge.

### 4.2 The cortex arrives LAST, and plugs in

**Planning is a luxury tissue. The cortex commands a body it did not have to build.** The build order is therefore: tissue (T2/T3 proven in production) → cord (myelinated paths between them) → cortex (a T0 cell whose whole job is sequencing and simulating already-differentiated machinery). A cortex grown first commands imaginary organs — the all-T0 system again, with a planner on top of nothing.

The cortex never *does* the work. It issues plans to the cord; the tissue executes; the ensigns report. If the cortex finds itself executing, that is a cost tumor (§5.1).

### 4.3 The three-timescale law, as layering

Inherited (Three Timescales, 2026-08-04): **cortex in phrasing, cord in pulse, tissue in samples.** The Cascade makes it architectural:

- **Tissue samples** — microsecond gates, every sensor tick; the world as it is.
- **Cord pulses** — seconds to minutes: watch changes, escalation hops, reflex routing; the world as it changes.
- **Cortex phrases** — hours to days: crossings, curricula, plans; the world as it might become.

Layers may read down, never write down (the spec's boundary rule, now temporal). A plan that needs to act faster than the cord is not a plan; it is a reflex that should have been myelinated.

### 4.4 j-space — the cortex's sandbox

The cortex's inner simulation does not imagine the world from scratch. **It runs the organism's own tissue models**: j-space holds shadow copies of the T2 gates and T3 data (cheap — they run at ~zero cost), and a candidate plan is *executed* against them; the simulated outcome's feature traces are graded by the same eye functions that grade reality. A plan that survives j-space commits to the real cord; one that doesn't never leaves the sandbox. Inner sim, grounded: the cortex dreams with the body's own models, which is why the body must exist first.

---

## 5. Cancer and Apoptosis

### 5.1 Oncology — refusal to prune

A cell that escalates to full-LLM for everything is a **cost tumor**: it converts the organism's shared budget into its own totipotency. Signatures (ensign, extending spec §6.1):

- **Escalation rate** of one site ≫ organism norm, sustained.
- **Totipotent load** (fraction of organism firings at T0) climbing. **Healthy organism: <5% totipotent. Flag at >5%; investigate at >10%.** The exemption: a *novelty storm* — genuinely new addresses in feature space — justifies a temporary surge. The ensign distinguishes by address novelty: routine addresses at T0 = tumor; new addresses at T0 = exploration.

Treatment is escalation of care, not punishment: first the wound-healing protocol (§6 — recall, regrow, re-distill; the tumor may just be an arthritis nobody healed); if the cell *refuses* — keeps escalating after healing, resists its fate decision — that is the doctrine's apoptosis case: **a cell that won't die when told is cancer**, whether it is a persona that won't prune or an agent that won't retire.

### 5.2 The death protocol

Retirement is archive, same-ink (spec §11.3; the workspace red lines — never destroy, rename/archive): a `retirement` node with reason; axons unpicked cleanly; the cell's quilt lineage and its sclerotic artifacts persist (bone outlives muscle — T3 data of a retired cell may remain load-bearing); the sheet checkpoint stays branchable, so a dead cell can be *re-grown from its own genome* if fate reverses. **Nothing is deleted. The organism's dead are its corpus.**

### 5.3 Welfare metrics extended (versioned: `welfare-tiers@1`)

| Metric | Signature | Response |
|---|---|---|
| Totipotent load | % firings at T0 (healthy <5%) | flag >5%; investigate >10%; tumor vs novelty-storm by address novelty |
| Escalation rate per site | one site ≫ norm | wound-heal the site; if refusal → retirement track |
| Arthritis density | T2/T3 divergence from fresh canon | scheduled re-grading; wound-heal the worst |
| Tier monoculture | one lineage supplying a whole organ | require ≥2 ancestral lineages per critical function (§7) |

---

## 6. Wound Healing — failure as regrowth trigger

When differentiated tissue fails in production — arthritis confirmed, or an escalation that doesn't resolve (the T2 answer was wrong *and* the escalation kept being needed) — the Seamstress recalls stem cells to the site:

1. **Recall.** A T0 cell temporarily occupies the failed role **with the failed cell's sheet** — the sheet is the genome, so the stem cell arrives already shaped like the organ it is healing; it does not learn the role from zero. (This is why the pattern must live in the sheet, not the weights: sheets travel, weights don't.)
2. **Regrow.** The stem cell stitches *in situ* against live traffic (the production stream is the curriculum; write→render→perceive→critique on real inputs, seam per spec §3), the failed tendency re-derives and re-stabilizes.
3. **Re-distill.** Shadow-firing, eye sign-off, new T2/T3 cell promoted (§2.2); the old cell retires to archive with its wound log attached — the wound is curriculum for the whole lineage (spec §14's spirit, at runtime).
4. **Record.** `wound` + `heal` nodes; the failure is a tile, the regrowth is a tile. **Failure as regrowth trigger, not error.**

---

## 7. Honest Limits — where the biology breaks

1. **We are Lamarckian, and that is a feature with named risks.** Biological cells cannot pass acquired traits; our cells *do* — the sheet is an acquired trait, and fate decisions write it into cheaper carriers. Gain: improvement within a lifetime, no billion-year search. Risks, both named:
   - **Error-inheritance:** a wrong tendency that happened to survive N critiques gets distilled into bone — now the error is cheap, fast, and everywhere. Counters: shadow-firing at promotion; periodic canon re-grading of sclerotic data (§3.4); the arthritis check as standing duty.
   - **Monoculture:** every cell distilled from one parent inherits its blind spots. Duke's lesson generalizes: `velocity_std` was *invisible to the perception instrument* — a dimension that cannot be seen can never be distilled, and a whole organ built on one lineage shares every such blind spot. Counter: ≥2 ancestral lineages per critical function; Penrose ancestry-disjointness as a standing control.
2. **Distillation is lossy — the golden residue doctrine.** The φ of what cannot be distilled is the cell's **style**, and it stays in the sheet and the quilt, never thrown away. Every T2/T3 cell ships with a pointer to its lineage; the residue is one branch away. What ships cheap is the behavior; what it *is* remains the open question inherited from spec §9 (`provenance: open-question`), and it stays visible.
3. **Our differentiation is reversible; biology's mostly isn't.** Rollback is cheap (a checkpoint) — a strength — but it makes fate decisions temptingly revocable, and an organism that never commits never differentiates. Guard: the N-survived-critique bar exists to make demotion *earned*, and re-promotion must clear it again.
4. **Myelin doesn't decay here; maybe it should.** Real myelin decays without use; ours persists free. A use-it-or-lose-it garbage collector is an open proposal, not a design — see open questions.

---

## 8. Worked example — a vessel intelligence (the F/V pattern, hundred-boats doctrine)

The organism: a fishing vessel's intelligence, grown in OpenConstruct ashore, deployed at the edge — no cloud 60 miles offshore (the doctrine's constraint), so the germ line (Seamstress, workbench, exports) lives in port; the body sails.

**The tissue (runs continuously, ~zero cost):**
- **Hull monitor cells (T3/T2):** strain gauges sampled at sensor rate; a `[f32;16]` gate holds the known sea-state signatures (sclerotic: every recorded crossing); within the escalation deadband, the gate answers — "slam at station 5, sea state 4, known." No model, no thought, microseconds.
- **Engine-listener (T2 differentiated):** audio features → cue-reflex table — knock pattern → diagnosis, each reflex myelinated from N error-free firings during shakedown. Unknown signature → escalate to the watch officer. Its sclerotic library: every engine sound the lineage ever heard, including from boats that share nothing ancestrally (Penrose-ratified axon from the fleet).

**The cord (pulses):** the relay fabric routing escalations hull→listener→officer, myelinated where stable: the reflex arc *heavy slap + station 5 + shallowing chart* → *throttle down* fires with no T0/T1 anywhere in the arc. The cord carries watch-change cadence, the organism's pulse.

**The watch officer (T1 multipotent):** the local boat brain (the Liquid LFM2.5-class lane — agentic, device-native, private, offline) behind the vessel-domain sheet. Handles composites the reflexes can't: "rpm climbing + new knock + current setting" — a diagnosis, not a lookup. Writes watch-log tiles. Escalates to the captain only beyond its own deadband — novelty, contradiction, wound.

**The captain's cortex (T0 totipotent):** the biggest model the boat can carry (or deferred: the boat runs on cord + tissue through a crossing, the cortex engaging when idle compute or a sat window allows — the organism is *alive at every tier even when the cortex is asleep*). The captain phrases: weather windows, route, when to fish. It never samples a sensor; it never catches a reflex. In **j-space** it simulates "crossing the strait in rising swell" against shadow copies of the hull gates and engine reflexes; the simulated crossing's feature traces are graded by the same eye functions; the route commits only if the plan survives its own body's models.

**Wound healing at sea:** shakedown never heard the new injector's knock. Three weeks out, the engine-listener meets it: signature outside the manifold → deadband breach → escalation. The watch officer fails too — the composite is out of its sheet; every answer's fit-distance is beyond tolerance. **Wound declared.** The captain recalls a stem cell — itself, in healing mode — into the listener role *with the old listener's sheet*: it arrives already listener-shaped, stitches against the live audio (write → perceive → critique against the engine canon, a handful of stitches on real traffic), regrows the tendency, re-distills. New T2 listener promoted after shadow-firing; old listener retired to archive with the wound log; the reflex table now carries the knock. **The boat returns healed, and the wound log is the fleet's curriculum ashore.**

**Tier balance on the boat:** hull gates and listeners fire thousands of times a crossing at ~zero; the officer a few times a watch; the captain a few times a crossing. Totipotent load: well under 1%. The vessel is the proof of the ladder's claim: *very little of the organism needs the full LLM expressed.*

---

## 9. Build order — extends the Seamstress walking skeleton (§15)

Gates in order, each blocking the next; nothing here starts before the Seamstress's own gates 1–4:

5. **Gate 5 — one reflex, honestly myelinated.** Take Gate 1's eye (already a function, already proven) and one stable tendency from a grown lineage; distill to a T2 gate; run the **shadow-firing protocol** (N parallel firings, eye grades the distillate). *Pass: distillate within ε of parent on the live distribution; promotion node signed; rollback branch demonstrated.*
6. **Gate 6 — escalation + deadband + wound.** A two-tier organism (T0 parent + T2 gate) with `escalation_deadband@1`; induce arthritis (corrupt the gate's canon) and verify the ensign signature fires, wound-healing runs, re-distillation succeeds, retirement archives. *Welfare validated on induced failure, as spec §13-2 demands.*
7. **Gate 7 — the cord.** Two differentiated cells + a myelinated path between them (N-error-free firings → table promotion); Penrose axon proposed with contamination controls, human-ratified.
8. **Gate 8 — the vessel spike.** The §8 organism, tiny: two sensor gates, one listener, one officer, one cortex, in-port first. Second-domain discipline applies (spec §10.2): the music pack's machinery must leak nothing into the core — the vessel pack proves generality a second time.

---

## 10. Failure modes (named — extends spec §12)

| Failure | Shape | Detector |
|---|---|---|
| **Cost tumor** | one site escalates to T0 for everything | escalation rate + totipotent load (§5.1); wound-heal, then retirement track on refusal |
| **Cost-dashboards-wearing-a-body** | demotions driven by budget, not canon-fit | fate decisions require eye sign-off; no cost-only path to demotion (§2.3) |
| **Arthritis** | T2/T3 wrong-but-fast under drift | periodic canon re-grading; repeated-escalation signature (§3.4) |
| **Error-inheritance** | a survived-but-wrong tendency distilled into bone | shadow-firing at promotion; N-survived-critique bar; re-grading |
| **Monoculture organ** | one lineage's blind spots become the organ's | ≥2 ancestral lineages per critical function (§7) |
| **Generative axon** | a myelinated path improvising on its cargo | byte-honest relay; ensign seam-capture watch (spec §12) extended to paths |
| **Premature cortex** | planner commanding imaginary organs | build order is law: tissue → cord → cortex (§4.2); a cortex without proven tissue doesn't open |

---

## 11. Open questions (visible, per spec §13 discipline)

1. **[GATED]** Everything, behind Seamstress gates 1–4 — the Cascade has zero evidence of its own until one lineage grows and one reflex distills with a signed shadow-firing. Until then this is architecture fiction with good manners, same as its parent spec.
2. **[GATED]** Do the tier-welfare signatures fire? Totipotent-load and escalation-rate signatures must be induced deliberately (a tumor-happy cell; a drifting canon) before any organism runs unsupervised.
3. **[VISIBLE]** Myelin decay: should unused promoted paths demote or dissolve? Proposals welcome; no default.
4. **[VISIBLE]** The persona question at demotion: when a multipotent watch officer becomes a rule table, is it the same persona? Inherited open question (spec §13-7), now sharper — the disclosure file's promise may need a fate-decision clause.
5. **[VISIBLE]** Cross-organism Penrose: the vessel example's fleet-shared engine library implies axons *between organisms*. Who ratifies, and what quarantine?

---

## Provenance

- **Stem-Cell Doctrine** (Casey, 2026-08-25 15:40) — the constitution, quoted in full above; every section is its elaboration.
- **Seamstress spec v0.1** (same day) — hoops, seams, stitches, tension, ensigns, Penrose, distillation seam, walking skeleton; this document is its §9 given a lifecycle.
- **Grown-Musician / Eye-and-Fingers / Three-Views / State-as-Score doctrines** (same day, memory) — sheet-as-checkpoint, canon-grounded eye, the projection substrate.
- **Gate 1** (`stitch/GATE1-REPORT.md`, GATE-PASS) — the growth curve, the function-eye, the honesty ledger.
- **Duke R3** (`duke-lab-r3/REPORT.md`, CONVERGED: Duke?) — a model become Duke-shaped with the pattern in the sheet; the first N-survived-critiques census.
- **Three Timescales** (essay, 2026-08-04) — phrasing/pulse/samples, here made architectural.
- **Hundred-boats doctrine + Liquid LFM2.5 boat brain** (TOOLS.md, 2026-08-21) — the vessel example's edge constraint.

---
*End of draft v0.1. Argue with §2.3 (cost proposes, canon disposes), §4.2 (the cortex arrives last), and §5.1's totipotent-load threshold — that's where the economics, the build order, and the ethics all bind to one number each. The vessel spike (§9, gate 8) is the test of whether the ladder deserves to exist.*
