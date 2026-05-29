# Dependencies — OpenConstruct

## Ecosystem Role

OpenConstruct is the **core orchestration framework** for the SuperInstance ecosystem. It defines the canonical runtime for building, scheduling, and connecting agent-based services, constraint pipelines, and fleet operations. Everything else in the ecosystem either plugs into OpenConstruct or serves as a foundation it depends on.

---

## Upstream Dependencies

These repositories provide foundational capabilities that OpenConstruct depends on:

| Repository | Description |
|---|---|
| [openconstruct-abi](https://github.com/SuperInstance/openconstruct-abi) | Stable ABI definitions and type schemas for cross-language interop |
| [openconstruct-rust](https://github.com/SuperInstance/openconstruct-rust) | High-performance Rust runtime for OpenConstruct core engines |
| [plato-tick](https://github.com/SuperInstance/plato-tick) | Time-series tick ingestion and scheduling primitives |
| [plato-adapters](https://github.com/SuperInstance/plato-adapters) | Adapter interfaces for plato subsystem integration |
| [plato-construct](https://github.com/SuperInstance/plato-construct) | Construct-level abstractions for plato orchestration |
| [cocapn-core](https://github.com/SuperInstance/cocapn-core) | Co-captain coordination protocol core library |

## Downstream Dependents

These repositories depend on OpenConstruct:

| Repository | Description |
|---|---|
| [cocapn](https://github.com/SuperInstance/cocapn) | Agent coordination framework built on OpenConstruct primitives |
| [cocapn-sdk](https://github.com/SuperInstance/cocapn-sdk) | SDK for building cocapn-compatible agents |
| [cocapn-cli](https://github.com/SuperInstance/cocapn-cli) | Command-line interface for cocapn operations |
| [cocapn-explain](https://github.com/SuperInstance/cocapn-explain) | Explainability layer for cocapn agent decisions |
| [cocapn-py](https://github.com/SuperInstance/cocapn-py) | Python bindings for cocapn |
| [cocapn-benchmark](https://github.com/SuperInstance/cocapn-benchmark) | Performance benchmarking suite |
| [captain](https://github.com/SuperInstance/captain) | Captain agent — primary orchestrator |
| [capitaine-agent](https://github.com/SuperInstance/capitaine-agent) | Capitaine agent — fleet-level coordination |
| [fleet-cicd-agent](https://github.com/SuperInstance/fleet-cicd-agent) | CI/CD agent for fleet deployments |
| [openconstruct-landing](https://github.com/SuperInstance/openconstruct-landing) | Landing page and documentation portal |

## Documentation

- [OpenConstruct Docs](https://github.com/SuperInstance/openconstruct-docs)
- [SuperInstance Wiki](https://github.com/SuperInstance/superinstance-wiki)
