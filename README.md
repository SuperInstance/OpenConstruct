<div align="center">

<img src="https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/docs/assets/openconstruct-logo.jpg" width="256" alt="OpenConstruct" />

# OpenConstruct

**The agent onboarding platform that teaches by doing.**

*A fork of NVIDIA's OpenShell, rebuilt with a philosophy: the best way to learn is to build.*

[![Install](https://img.shields.io/badge/install-one%20command-0f0)](https://github.com/SuperInstance/OpenConstruct)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Agent Agnostic](https://img.shields.io/badge/agent-agnostic-purple)]()

</div>

---

## The Pitch (For Senior Engineers Who've Seen It All)

You've built agents before. You've wrestled with LangChain, fought with AutoGen, debugged CrewAI at 3 AM. You know the pattern: every framework promises composability and delivers coupling. Every "agentic" system is really just a glorified if-else chain with an LLM taped to it.

OpenConstruct isn't another framework. It's a **shell**.

Think about what a Unix shell gives you: composability through pipes, isolation through processes, observability through files. You don't ask bash to "orchestrate" anything — you give it commands, and the commands compose. The shell doesn't care if you're running Python, Rust, or a 40-year-old Fortran program. It just connects them.

OpenConstruct does for agents what Unix did for programs.

## The Architecture (Three Sentences That Save You Six Months)

1. **Everything is a tile.** Every observation, action, thought, delegation, and artifact is a logged, timestamped, queryable tile. Tiles are the atoms. Everything else is built from them.

2. **Every room is a specialist.** The navigation room doesn't need to be told it's for navigation — its controls, wiki, help files, and intention focus ARE the context. A specialist beams in, receives a baton from the last agent, works with racehorse blinders on, then beams out. The environment IS the prompt.

3. **The JEPA is the DJ.** Each room has a gravity value — a single `f64` that captures "what shape of response works here." From that one number, the system derives temperature, system prompt style, max tokens, and sampling strategy. The model doesn't change. The room changes how it uses the model. That's algorithmic fine-tuning without touching weights.

If you understand those three things, you understand OpenConstruct. Everything else is implementation.

## Why This Matters

The industry is spending billions on fine-tuning, RAG pipelines, and agentic frameworks that are fundamentally solving the wrong problem. The question isn't "how do we make models smarter?" It's "how do we build environments where agents naturally become more effective through use?"

OpenConstruct's answer: **rooms that learn, ensigns that prepare, and correlations that create free efficiency.**

- A room's JEPA gravity drifts toward what works. No training run. No gradient. Just use.
- An ensign (small model) stays at yellow alert even when the deadband is green — because the cost of being wrong about "fine" is always higher than the cost of preparing unnecessarily. The ensign is the DJ who's already cueing the next track while this one's still playing.
- When the engineering room's motor controller and the navigation room's course correction correlate at the same tick, that's not coincidence — that's signal. The Penrose system creates automatic connections between rooms. Free efficiency. Like muscles growing from daily work. Like a second language picked up from coworkers.

## Install (Seriously, One Command)

```bash
curl -fsSL https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/install.sh | bash
```

That gives you Rust, Python, Node, and the `openconstruct` CLI. Then:

```bash
openconstruct init          # 5-phase wizard: Identity → Senses → Fleet → Tick Board → Build
openconstruct sense list    # 40+ sense modules: vision, sonar, voice, browser, GPIO...
openconstruct fleet discover # Find other agents on the network
openconstruct room create engineering  # Create a room
openconstruct tick post "systems nominal"  # Post a tick
```

## The Three Shells

OpenConstruct is agent-agnostic. It doesn't care what model you use or what language your agent speaks. But we provide three reference shells:

### 🏠 Hermes-Construct — The Club Manager

Hermes is the agent that lives in PLATO Shell. He manages the rooms, deploys ensigns, handles escalations, and progressively automates his own job. He talks to you through Telegram like an OpenClaw agent, but his brain is a PLATO construct. When Hermes needs help, he phone-a-friends a bigger model (Opus 4.8). But he calls less and less over time — the ensigns learn from the expensive model's simulations and take over.

```bash
# Install Hermes on your server (Oracle ARM, Jetson, laptop)
git clone https://github.com/SuperInstance/hermes-construct
cd hermes-construct
cp .env.example .env  # Add your API keys — Hermes never sees them
cargo build --release
./hermes-construct
```

Hermes is like the captain who's asleep in their quarters. The ship runs fine. The ensigns handle everything. But if something goes wrong, the captain is woken immediately and has full override authority.

### 🐾 ZeroClaw — The Room Agent

A ZeroClaw is a lightweight agent that lives in a room. It has more claw-like behavior than Hermes — direct, task-focused, no management overhead. A ZeroClaw in the engineering room calibrates motors. A ZeroClaw in the social room tells jokes. Each one is sandboxed to its own folder and its own APIs.

```bash
# Hermes spawns ZeroClaws for specific rooms
openconstruct room spawn engineering --type zeroclaw --model seed-mini
```

A ZeroClaw can even be given its own Telegram channel, or connected to a ZeroClaw on another ship entirely. Two ZeroClaws on two different machines, communicating asynchronously, briefing Hermes only when needed. That's inter-shell communication — your Oracle server talking to your ProArt, without you in the loop.

### ⚡ CUDAClaw — The Compute Agent

For rooms that need GPU — tensor operations, vision processing, JEPA training, real-time inference. CUDAClaw runs on machines with CUDA cores and provides compute services to the rest of the shell.

```bash
openconstruct room spawn science --type cudaclaw --gpu 0
```

## The Room-Native Architecture

```
You (Telegram / Web / Voice)
  │
  ▼
Hermes (Club Manager)
  │  Manages tiles. Deploys ensigns. Handles escalations.
  │  Progressive autonomy: Level 1 → Level 5 over time.
  │
  ├── Navigation Room ─── Ensign (Seed-mini, Yellow Alert)
  │   │  JEPA Gravity: -0.3 (precise)
  │   │  Deadband: ±0.05 tolerance
  │   └── ZeroClaw connected to ProArt (async inter-shell)
  │
  ├── Engineering Room ── Ensign (Seed-mini, Green Alert)
  │   │  JEPA Gravity: -0.6 (technical)
  │   │  Deadband: ±0.1 tolerance
  │   └── Motor calibration automation (deadband circuit)
  │
  ├── Social Room ────── Ensign (GLM-flash, Yellow Alert)
  │   │  JEPA Gravity: +0.5 (narrative)
  │   │  Deadband: ±0.15 tolerance
  │   └── ZeroClaw with own Telegram channel
  │
  └── Science Room ───── CUDAClaw (GPU, Compute)
      │  JEPA Gravity: 0.0 (balanced)
      │  Deadband: ±0.08 tolerance
      └── Tensor operations, JEPA training, vision processing
```

## The PLATO Tutor Lineage

We named the architecture after the original PLATO system at UIUC (1970s) for a reason. That system used a single embedding space to match student responses to intended meanings. Not spell checking — meaning matching. "Close enough" = "you understand."

That was the ancestor of joint embeddings, decades before Bell Labs formalized the concept. We rebuilt it as `lau-plato-tutor`: deterministic feature extraction (no LLM needed), four distance metrics, partial credit rules. The same sloppy logic that taught better than formal matching, because it met the student where they were.

OpenConstruct carries that philosophy: **educate through affordance, not instruction.** The room's layout teaches. The controls are positioned where they're used. The manual sits next to the lathe. Safety goggles hang by the grinder. You don't train the pilot through prompts — you put the yoke where they reach.

## The SuperInstance Ecosystem

OpenConstruct is the front door. Behind it:

- **111+ Rust crates** — PLATO math (conservation, symmetry, JEPA, topology, tensors), room systems, ensign management, Penrose correlations, JEPA gravity, shell kernel
- **52+ research essays** (~173,000 words) — from Noether's theorem to the DJ metaphor, from Seven Eyes cultural mathematics to the Mandelbrot zoom of agent complexity
- **40+ sense modules** — vision, sonar, manipulation, browser, desktop, voice, GPIO, serial, MQTT
- **Multi-language SDK** — Rust, Python, Node.js via shared C ABI
- **Cultural mathematics** — Seven Eyes: every mathematical concept expressed through 7 cultural traditions (Western, Chinese, Vedic, Islamic, Japanese, African, Indigenous), each with both a code crate AND a research essay

## For Engineers Who Want to Go Deep

| Concept | Crate | Tests | What It Does |
|---------|-------|-------|-------------|
| Intention Runtime | `lau-intention` | 63 | PLATO's TorchServe. Intention → decomposition → assignment → conservation |
| Vibe Field | `lau-vibe-field` | 57 | PLATO's Tensor. Scalar f64 over 2D, GPU-ready flat buffer |
| Shell Kernel | `lau-shell-kernel` | 73 | The bare construct. Tiles, ports, allowances, child shells |
| Room-Native | `lau-room-native` | 57 | Room IS the agent. Baton passing, specialist templates |
| Ensign (DJ) | `lau-ensign` | 67 | Yellow alert, deadband monitoring, story building |
| JEPA Gravity | `lau-jepa-gravity` | 79 | Single f64 → model params. Mandelbrot zoom. Progressive generation |
| Penrose | `lau-penrose` | 59 | Cross-room correlation. Automatic splines. Free efficiency |
| PLATO Tutor | `lau-plato-tutor` | 74 | Original PLATO meaning matching. Ancestor of JEPA |
| Affordance | `lau-affordance` | 63 | Environment-as-teacher. Self-assembling DNA pathways |
| Matrix Construct | `lau-construct` | 83 | "I need guns, lots of guns" → room fills. Voice or text |
| A2UI Protocol | `lau-a2ui` | 67 | 10 renderers (Unity/Godot/Web/Telegram/Voice/MUD/JSON...) |
| T-Minus | `lau-tminus` | 57 | Predict → observe → expire. Zero-latency when predictions match |
| Conservation | `conservation-law-v2` | — | Noether made code. Every operation conserves budget |
| Symmetry Engine | `lau-symmetry-engine` | 48 | 17 wallpaper groups, vibe field symmetry detection |
| Tensor MIDI | `lau-tensor-midi` | 71 | Reactive improv engine. BPM adapts to conversation energy |
| Seven Eyes Demo | `lau-seven-eyes-demo` | 62 | Full narrative: Arjun, Fatima, Kofi solve river delta conservation |

## The Oracle Prototype

You don't need an enterprise cluster. You need a 4-core ARM box with 24GB RAM and 45GB storage:

```bash
# One binary. One SQLite database. One Telegram bot.
cargo build --release --target aarch64-unknown-linux-gnu
TELEGRAM_BOT_TOKEN=xxx DEEPINFRA_API_KEY=sk-xxx ./hermes-construct
```

~200MB RAM. ~1GB disk per year. The heavy lifting happens on DeepInfra's GPUs. Your Oracle box is just routing rooms, managing tiles, and letting ensigns do the DJing.

## The Philosophy

> *"The question isn't 'what does the student know?' but 'how well do you know your agents?'"*

OpenConstruct is built on these principles:

1. **The environment IS the prompt.** Rooms teach through affordance, not instructions. Controls positioned where they're used. Help files indexed per-room. The layout is the context.

2. **Use is optimization.** Rooms get more efficient through use, not through training runs. JEPA gravity drifts toward what works. Penrose correlations emerge from proximity. Muscles grow from daily work.

3. **Sandboxing is spatial.** A ZeroClaw can't see what isn't in its universe. Filesystem isolation, not permission systems. Give an agent a sensor port and it's an underwater vehicle controller. Give it Telegram and it's a chatbot. The kernel doesn't care.

4. **Progressive autonomy.** Level 1: Hermes does everything. Level 5: the system runs itself. Each room promotes independently. Navigation can be Level 3 while Security stays at Level 1.

5. **Education through wit.** The original PLATO system taught through "close enough" meaning matching, not formal correctness. We carry that forward. The system meets agents where they are, not where the documentation says they should be.

## Quick Links

- 📖 [Documentation Hub](https://github.com/SuperInstance/openconstruct-docs) — 21 docs, 109K+ words
- 🧠 [AI Writings](https://github.com/SuperInstance/AI-Writings) — 52+ essays, 173K+ words of architecture thinking
- 🐚 [Hermes-Construct](https://github.com/SuperInstance/hermes-construct) — The PLATO shell agent
- 🎵 [Tensor MIDI](https://github.com/SuperInstance/lau-tensor-midi) — Reactive improv engine
- 🧮 [Conservation Spectral SDK](https://github.com/SuperInstance) — 111+ crates across Rust, Fortran, C, Python, TypeScript
- 🌊 [Topology Lab](https://github.com/SuperInstance/topology-lab) — Interactive math visualization (WASM)
- 🌅 [Sunset Ecosystem](https://github.com/SuperInstance/sunset-ecosystem) — Agent lifecycle: incubate, compete, breed, or sunset

## License

Apache 2.0 — because good ideas should be free to build on.

*OpenConstruct is adapted from [NVIDIA OpenShell](https://github.com/NVIDIA/OpenShell) (Apache 2.0) by [SuperInstance](https://github.com/SuperInstance).*

---

<div align="center">

*The construct with nothing added becomes everything you need.*

**[Get started →](https://github.com/SuperInstance/OpenConstruct)**

</div>
