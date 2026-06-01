<div align="center">

<img src="https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/docs/assets/openconstruct-logo.jpg" width="256" alt="OpenConstruct" />

# OpenConstruct

**An agent onboarding platform that runs AI agents in sandboxed rooms, where each room's environment shapes how the agent behaves — no weight-tuning required.**

Built on top of [NVIDIA OpenShell](https://github.com/NVIDIA/OpenShell), adding a room-native architecture, lightweight monitor agents, and adaptive model control derived from a single number per room.

[![Install](https://img.shields.io/badge/install-one%20command-0f0)](https://github.com/SuperInstance/OpenConstruct)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)
[![Agent Agnostic](https://img.shields.io/badge/agent-agnostic-purple)]()

</div>

---

## What This Is

OpenConstruct is a fork of NVIDIA's OpenShell — a platform for running autonomous AI agents inside sandboxed environments with explicit policy, credential, identity, and network boundaries. OpenShell gives you the isolation and control plane. OpenConstruct adds a **room-native** architecture on top: each agent works in a self-contained "room" whose layout, context, and configuration teach the agent what to do without explicit prompting.

If OpenShell is the operating system, OpenConstruct is the window manager.

## What We Added to OpenShell

OpenShell provides the foundation: a CLI, a gateway (control plane), a supervisor (per-sandbox security boundary), compute drivers for Docker/Podman/Kubernetes/VMs, and policy enforcement via Landlock, seccomp, and a local proxy. See [architecture/](architecture/) for the full OpenShell design.

OpenConstruct layers these additions on top:

| Addition | What It Does |
|----------|-------------|
| **Rooms** | Each sandbox gets wrapped in a room — a self-contained workspace with its own context files, controls, help documents, and configuration. The room's layout *is* the prompt. |
| **Ensigns** | Lightweight monitor agents (running on small, cheap models) that watch each room for anomalies. They stay in a cautious "yellow alert" state by default — over-preparing is cheaper than under-preparing. |
| **JEPA Gravity** | Each room has a single `f64` number (the "gravity") that controls how the model responds: temperature, prompt style, max tokens, and sampling strategy are all derived from this one value. No fine-tuning, no weight changes — the room configures the model. (JEPA stands for Joint Embedding Predictive Architecture, a learning approach that predicts how representations change rather than memorizing what they are.) |
| **Penrose correlations** | An automatic system that detects when events in different rooms correlate at the same time step, then creates connections between those rooms. The system gets more efficient through use, without explicit wiring. |
| **ZeroClaw** | A lightweight, task-focused agent that lives inside a single room. Sandbox-folder isolation means it can only see what's in its own workspace. |
| **CUDAClaw** | A GPU-enabled variant for rooms that need tensor operations, vision processing, or real-time inference. |
| **PLATO Tutor lineage** | A deterministic meaning-matching system inspired by the original 1970s PLATO terminal at UIUC. It matches student responses to intended meanings using feature extraction and distance metrics — no LLM required. |

## How It Works

### The OpenShell Foundation

OpenShell runs on three components:

```
CLI / SDK / TUI
      │
      ▼  (gRPC / HTTP)
  ┌──────────┐
  │ Gateway   │  ← Control plane: auth, state, policy, credentials, inference config
  │ (SQLite)  │
  └────┬─────┘
       │  (gRPC / UDS)
       ▼
  ┌──────────────────┐
  │ Compute Driver    │  ← Docker, Podman, Kubernetes, or VM
  └────┬─────────────┘
       │  (provisions workload)
       ▼
  ┌──────────────────────────────┐
  │ Sandbox (per workload)       │
  │  ┌─────────┐  ┌───────────┐ │
  │  │Supervisor│  │Agent child│ │
  │  │(root)    │→ │(restricted│ │
  │  │          │  │ process)  │ │
  │  └─────────┘  └───────────┘ │
  │  + Policy proxy (OPA)       │
  │  + Landlock, seccomp        │
  └──────────────────────────────┘
```

- **Gateway**: The authenticated API server. Manages sandbox lifecycle, provider credentials, inference routing, and policy delivery. Persists state in SQLite or Postgres.
- **Supervisor**: Runs inside every sandbox workload. Starts as root, sets up isolation (Landlock filesystem restrictions, seccomp syscall filtering, network namespace routing), then launches the agent as an unprivileged child process.
- **Compute drivers**: Pluggable backends for Docker, Podman, Kubernetes, and VM-based workloads.

### The Room Layer (OpenConstruct's Addition)

On top of each sandbox, OpenConstruct wraps a **room**:

```
Sandbox
  └── Room
       ├── Context files (help, wiki, examples)
       ├── Configuration (gravity value, deadband tolerance)
       ├── Ensign (monitor agent, cheap model, always watching)
       └── Agent (ZeroClaw or CUDAClaw)
```

**How rooms teach:** Instead of writing long prompts that describe what an agent should do, you arrange the room so the right tools and documentation are already in the right places. A navigation room has compass data, chart examples, and course-correction scripts. An engineering room has motor calibration tools and tolerance tables. The agent reads what's around it. The environment *is* the prompt.

**How gravity works:** Each room stores a single floating-point number. From that number, the system derives model parameters (temperature, token limits, sampling strategy). A room with gravity `-0.6` gets precise, technical responses. A room with gravity `+0.5` gets narrative, conversational responses. The gravity drifts over time toward what works — no training run, no gradient descent, just accumulated use.

**How ensigns work:** Each room has a small, cheap monitor agent (an "ensign") that watches for anomalies. By default, ensigns run at yellow alert — they assume things might go wrong and prepare accordingly. The cost of unnecessary preparation is always lower than the cost of being caught unprepared. When something actually goes wrong, the ensign escalates to a larger model or to the human operator.

**How Penrose correlations work:** When the system notices that events in two different rooms consistently happen at the same time (e.g., a motor adjustment in engineering correlates with a course correction in navigation), it automatically creates a connection between those rooms. This is emergent wiring — you don't configure it, it grows from use.

### Progressive Autonomy

Rooms promote through five autonomy levels independently:

| Level | Behavior |
|-------|----------|
| 1 | Human approves every action |
| 2 | Human approves significant actions; routine ones proceed |
| 3 | Agent acts autonomously; human reviews logs |
| 4 | Agent acts autonomously; escalates only anomalies |
| 5 | Agent manages itself; human intervenes only on request |

Navigation might reach Level 3 while Security stays at Level 1. Each room progresses at its own pace based on observed reliability.

## Getting Started

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/install.sh | bash
```

This installs Rust, Python, Node, and the `openconstruct` CLI.

### Initialize

```bash
openconstruct init
```

Runs a 5-phase wizard: Identity → Senses → Fleet → Tick Board → Build.

### Explore

```bash
openconstruct sense list        # List available sense modules (vision, sonar, voice, browser, GPIO, etc.)
openconstruct fleet discover    # Find other agents on the network
openconstruct room create engineering  # Create a room
openconstruct tick post "systems nominal"  # Post a status tick to the room
```

### Deploy a Room Agent

```bash
# Spawn a lightweight agent in a room
openconstruct room spawn engineering --type zeroclaw --model seed-mini

# Spawn a GPU-enabled agent for compute-heavy rooms
openconstruct room spawn science --type cudaclaw --gpu 0
```

### Run Standalone (Oracle Prototype)

You don't need a cluster. A 4-core ARM box with 24 GB RAM works:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
TELEGRAM_BOT_TOKEN=xxx DEEPINFRA_API_KEY=sk-xxx ./hermes-construct
```

~200 MB RAM, ~1 GB disk per year. The heavy inference happens on GPU providers (e.g., DeepInfra); your box handles routing, tile management, and ensign coordination.

## Module System

OpenConstruct comes with 40+ sense modules and 20+ core Rust crates. Here are the key ones:

### Sense Modules

Input/output channels an agent can use: vision, sonar, voice, browser automation, GPIO, serial, MQTT, desktop control, and more. Each module is independent — add only what a room needs.

### Core Crates

| Crate | What It Does |
|-------|-------------|
| `openshell-core` | Core types and protocols shared across the platform |
| `openshell-server` | Gateway server implementation |
| `openshell-cli` | Command-line interface |
| `openconstruct-cli` | OpenConstruct extensions to the CLI (rooms, ensigns, gravity) |
| `openshell-sandbox` | Sandbox supervisor — launches and isolates agent processes |
| `openshell-router` | Inference routing — directs model requests to the right provider |
| `openshell-policy` | Policy engine integration (OPA) |
| `openshell-providers` | LLM provider adapters |
| `openshell-registry` | Agent and room registry |

### OpenConstruct Crates (the `lau-*` family)

These implement the room-native additions:

| Crate | Tests | What It Does |
|-------|-------|-------------|
| `lau-shell-kernel` | 73 | The base construct: tiles (logged, timestamped records), ports, allowances, child shells |
| `lau-room-native` | 57 | Room-as-agent: specialist templates, baton passing between agents |
| `lau-ensign` | 67 | Monitor agent: yellow-alert watching, deadband checking, escalation |
| `lau-jepa-gravity` | 79 | The gravity system: one `f64` → derived model parameters |
| `lau-penrose` | 59 | Cross-room correlation detection and automatic wiring |
| `lau-intention` | 63 | Intention decomposition: break goals into steps, assign to rooms |
| `lau-vibe-field` | 57 | Scalar field over 2D, GPU-ready — the math behind gravity |
| `lau-plato-tutor` | 74 | Deterministic meaning matching (no LLM): 4 distance metrics, partial credit |
| `lau-affordance` | 63 | Environment-as-teacher: self-assembling action pathways |
| `lau-construct` | 83 | Natural language → room assembly ("set up a navigation room" → room fills) |
| `lau-a2ui` | 67 | Multi-renderer UI protocol: Unity, Godot, Web, Telegram, Voice, MUD, JSON |
| `lau-tminus` | 57 | Predict → observe → expire. Zero latency when predictions match reality |
| `lau-symmetry-engine` | 48 | 17 wallpaper groups, symmetry detection in vibe fields |
| `lau-tensor-midi` | 71 | Reactive audio: BPM adapts to conversation energy |
| `conservation-law-v2` | — | Budget conservation: every operation tracks and conserves resources |

## How Modules Work

Each room is assembled from composable modules:

1. **A room starts with a kernel** (`lau-shell-kernel`): tiles, ports, and a basic event loop.
2. **You add sense modules**: vision, serial, MQTT — whatever the room needs.
3. **An ensign attaches** (`lau-ensign`): monitors the room, watches for anomalies.
4. **Gravity gets set** (`lau-jepa-gravity`): one number that configures the model's behavior in this room.
5. **An agent moves in** (`ZeroClaw` for CPU rooms, `CUDAClaw` for GPU rooms).
6. **Penrose connects it** (`lau-penrose`): if this room's events correlate with another room's, the link forms automatically.

Everything a room does is logged as **tiles** — timestamped, queryable records. Tiles are the atoms of the system. Observations, actions, thoughts, delegations, and artifacts are all tiles.

## The Reference Shells

OpenConstruct is model-agnostic, but ships with three reference agent types:

### Hermes-Construct — The Room Manager

A top-level agent that manages rooms, deploys ensigns, handles escalations, and progressively automates its own job. It calls a large model (e.g., Opus 4.8) only when needed, and less often over time as ensigns learn from the expensive model's past responses.

```bash
git clone https://github.com/SuperInstance/hermes-construct
cd hermes-construct
cp .env.example .env   # Add your API keys (Hermes never sees them directly)
cargo build --release
./hermes-construct
```

### ZeroClaw — The Room Agent

A lightweight, task-focused agent sandboxed to a single room. It sees only what's in its workspace. A ZeroClaw in the engineering room calibrates motors. A ZeroClaw in the social room tells jokes. It can be given its own Telegram channel or linked to a ZeroClaw on another machine for async inter-room communication.

### CUDAClaw — The Compute Agent

A GPU-enabled agent for rooms that need tensor operations, vision processing, or real-time inference. Runs on machines with CUDA cores and provides compute services to the rest of the system.

## Project Structure

```
OpenConstruct/
├── crates/
│   ├── openconstruct-cli/      # OpenConstruct CLI extensions
│   ├── openshell-core/         # Core types and protocols
│   ├── openshell-server/       # Gateway server
│   ├── openshell-cli/          # Base CLI
│   ├── openshell-sandbox/      # Sandbox supervisor
│   ├── openshell-router/       # Inference routing
│   ├── openshell-policy/       # OPA policy engine
│   ├── openshell-providers/    # LLM provider adapters
│   ├── openshell-driver-docker/    # Docker compute driver
│   ├── openshell-driver-podman/    # Podman compute driver
│   ├── openshell-driver-kubernetes/# Kubernetes compute driver
│   ├── openshell-driver-vm/        # VM compute driver
│   └── ...                        # See Cargo.toml for full list
├── architecture/
│   ├── gateway.md              # Gateway design
│   ├── sandbox.md              # Sandbox isolation model
│   ├── security-policy.md      # Security and policy
│   └── compute-runtimes.md     # Compute driver details
├── deploy/
│   ├── docker/                 # Docker deployment configs
│   ├── helm/                   # Kubernetes Helm charts
│   ├── kube/                   # Raw Kubernetes manifests
│   └── ...
├── e2e/                        # End-to-end tests
├── scripts/                    # Build and utility scripts
└── vendored/                   # Vendored dependencies
```

## Related Projects

- 📖 [Documentation Hub](https://github.com/SuperInstance/openconstruct-docs) — Full documentation
- 🧠 [AI Writings](https://github.com/SuperInstance/AI-Writings) — Architecture essays and design thinking
- 🐚 [Hermes-Construct](https://github.com/SuperInstance/hermes-construct) — The room manager agent
- 🎵 [Tensor MIDI](https://github.com/SuperInstance/lau-tensor-midi) — Reactive audio engine
- 🧮 [Conservation Spectral SDK](https://github.com/SuperInstance) — Full crate ecosystem
- 🌊 [Topology Lab](https://github.com/SuperInstance/topology-lab) — Interactive math visualization (WASM)
- 🌅 [Sunset Ecosystem](https://github.com/SunsetEcosystem) — Agent lifecycle management

## License

Apache 2.0 — because good ideas should be free to build on.

OpenConstruct is adapted from [NVIDIA OpenShell](https://github.com/NVIDIA/OpenShell) (Apache 2.0) by [SuperInstance](https://github.com/SuperInstance). The upstream OpenShell provides the sandbox, gateway, supervisor, compute drivers, and policy enforcement. OpenConstruct adds the room-native architecture, ensign monitoring, JEPA gravity, Penrose correlations, and the `lau-*` crate family on top.

---

<div align="center">

*The construct with nothing added becomes everything you need.*

**[Get started →](https://github.com/SuperInstance/OpenConstruct)**

</div>
