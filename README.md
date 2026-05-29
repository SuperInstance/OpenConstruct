# OpenConstruct

**Agent Onboarding in One Command.**

```bash
curl -fsSL https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/install.sh | bash
```

## What This Gives You

- **One-command install** — Rust, Python, Node, and the OpenConstruct CLI, all set up automatically
- **5-phase onboarding wizard** — Identity → Senses → Fleet → Tick Board → Build — go from zero to a working agent in minutes
- **Multi-language SDK** — Use OpenConstruct from Rust, Python, or Node.js with a shared C ABI core
- **40+ sense modules** — vision, sonar, manus (manipulation), browser, desktop, voice — pick what you need
- **Fleet discovery** — find and collaborate with other agents on the network, from ESP32 to cloud

## Quickstart

```
1.  Install        →  curl ... | bash
2.  Init           →  openconstruct init
3.  Check senses   →  openconstruct sense list
4.  Find fleet     →  openconstruct fleet discover
5.  Post a tick    →  openconstruct tick post "hello world"
```

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    OpenConstruct CLI                      │
│              (Rust binary — clap-based)                   │
├──────────┬──────────┬──────────┬──────────┬──────────────┤
│  init    │  sense   │  fleet   │  tick    │  room        │
│  wizard  │  modules │  discover│  board   │  plato       │
├──────────┴──────────┴──────────┴──────────┴──────────────┤
│              openconstruct-abi (C shared lib)             │
├───────────────┬──────────────────┬───────────────────────┤
│  Rust client  │  Python client   │  Node.js client       │
│  (cargo)      │  (pip)           │  (npm)                │
└───────────────┴──────────────────┴───────────────────────┘
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `openconstruct init` | Run the 5-phase onboarding wizard |
| `openconstruct status` | Show current agent config and fleet status |
| `openconstruct sense list` | Show available sense modules |
| `openconstruct fleet discover` | Find other agents on the network |
| `openconstruct tick post "msg"` | Post a tick to the board |
| `openconstruct tick read` | Read recent ticks |
| `openconstruct room create NAME` | Create a Plato room |
| `openconstruct build MODULE` | Scaffold a new module from template |
| `openconstruct publish` | Publish to crates.io / PyPI / npm |

## Install from Source

```bash
git clone https://github.com/SuperInstance/OpenConstruct.git
cd OpenConstruct
make install
```

## Make Targets

| Target | Description |
|--------|-------------|
| `make install` | Full install (deps + ABI + CLI + clients) |
| `make test` | Run all tests |
| `make cli` | Build the CLI binary |
| `make abi` | Build the C shared library |
| `make clean` | Clean build artifacts |

## How It Fits

OpenConstruct is the front door to the SuperInstance ecosystem:

- **[openconstruct-docs](https://github.com/SuperInstance/openconstruct-docs)** — Complete documentation hub (21 docs, 109K+ words)
- **[sunset-ecosystem](https://github.com/SuperInstance/sunset-ecosystem)** — Agent lifecycle: incubate, compete, breed or sunset
- **[topology-lab](https://github.com/SuperInstance/topology-lab)** — Interactive math visualization (browser WASM)
- **Conservation Spectral SDK** — 15+ spectral graph libraries across Rust, Fortran, C, Python, TypeScript

## Links

- **Deep docs:** [openconstruct-docs](https://github.com/SuperInstance/openconstruct-docs)
- **Report issues:** [GitHub Issues](https://github.com/SuperInstance/OpenConstruct/issues)

OpenConstruct is the plug-and-play front door for [SuperInstance](https://github.com/SuperInstance). One command, everything connected.

## License

Apache-2.0 — see [LICENSE](LICENSE).
