# OpenConstruct Developer Guide: Rooms, Sandboxes, and Isolation

A practical guide for developers onboarding to OpenConstruct. Every capability
claim is marked and traced to source so you know where you are standing.

**Status markers:**

- ✅ **Real today** — implemented, verifiable in source, and exercised by tests or e2e paths.
- ⚠️ **Real but conditional** — implemented, but with caveats, partial coverage, or runtime requirements.
- 🔮 **Later phase / design intent** — described in documentation or a roadmap but not present in the codebase at the time of writing.

> **Why this guide reads differently from the README.** The project README
> presents the OpenShell foundation and the OpenConstruct "room-native"
> additions in a single confident voice. The foundation is dense, real code.
> The room-native layer is, at present, mostly design intent. This guide marks
> the seam between the two so you build on what is concrete and plan toward what
> is not. The seam itself is documented in [`PROSPECTUS.md`](#) within the
> project's creative-writing library.

---

## 1. What OpenConstruct Actually Is

OpenConstruct is a fork of NVIDIA OpenShell. The split matters:

| Layer | Status | What it is |
|---|---|---|
| **OpenShell foundation** | ✅ Real | The sandbox runtime, gateway control plane, compute drivers, and policy enforcement. This is the load-bearing code — tens of thousands of lines, tested, with end-to-end paths. |
| **OpenConstruct "room-native" layer** | 🔮 Mostly design intent | Rooms, ensigns, JEPA gravity, Penrose correlations, and the `lau-*` crate family described in the README. At present the shipped code is a small onboarding data-model crate and a local-file CLI. See §5 for the honest inventory. |

If you are here to **build, run, or extend isolated agent sandboxes**, you are
working with the OpenShell foundation, which is real and substantial. If you are
here for the room-native architecture, read §5 first so you know what exists and
what does not.

---

## 2. The Real Developer API Surface

The developer-facing API is the **`openshell` CLI** (`crates/openshell-cli/`),
not the `openconstruct` CLI. The `openshell` binary talks to the gateway over
gRPC and is the path for creating and managing sandboxes.

### Core command groups ✅

All commands below are defined in `crates/openshell-cli/src/main.rs`.

| Command group | What it does | Status |
|---|---|---|
| `openshell sandbox create` | Provision a sandbox from an image, Dockerfile, or community name | ✅ |
| `openshell sandbox get / list / delete` | Inspect and remove sandboxes | ✅ |
| `openshell sandbox exec` | Run a command in a running sandbox over gRPC | ✅ |
| `openshell sandbox connect` | Open an interactive SSH session to a sandbox | ✅ |
| `openshell sandbox upload / download` | Sync files to/from a sandbox | ✅ |
| `openshell policy get / set / update / list` | Read and write sandbox policy | ✅ |
| `openshell policy prove` | Prove security properties of a policy or find counterexamples | ⚠️ |
| `openshell provider ...` | Manage LLM provider credentials at the gateway | ✅ |
| `openshell inference ...` | Configure inference routing | ✅ |
| `openshell gateway add / select` | Register and switch between gateways | ✅ |
| `openshell forward` | Forward a local port to a sandbox | ✅ |
| `openshell logs` | View or tail sandbox and gateway logs | ✅ |
| `openshell term` | Launch the TUI dashboard | ✅ |

### Gateway API ✅

The gateway (`crates/openshell-server/`) is the control plane. It multiplexes
gRPC and HTTP on one service port, persists state in SQLite or Postgres, and
exposes the lifecycle, provider, policy, inference, and observability APIs that
the CLI drives. See [`gateway.md`](gateway.md) for the full design.

Supported auth modes: mTLS (default), plaintext (local dev), Cloudflare JWT
(edge auth), and OIDC (bearer token with PKCE or client-credentials login).

---

## 3. How a Developer Defines a Sandbox

A sandbox is the unit of isolation. You define one by choosing an **image** and a
**policy**. The policy is where the real isolation contract lives.

### Create a sandbox ✅

```shell
# Register and select a gateway first
openshell gateway add http://127.0.0.1:18080 --local --name local
openshell gateway select local

# Create from a community image, with resource limits and a custom policy
openshell sandbox create \
  --from openclaw \
  --cpu 2 --memory 4Gi \
  --policy ./sandbox-policy.yaml \
  --provider deepinfra \
  -- claude
```

Key `sandbox create` flags (verified in `main.rs`):

| Flag | Purpose | Status |
|---|---|---|
| `--from` | Image source: community name, Dockerfile path, or full image ref | ✅ |
| `--policy` | Path to a custom policy YAML (overrides `OPENSHELL_SANDBOX_POLICY`) | ✅ |
| `--cpu` / `--memory` | Per-sandbox resource limits (K8s-style quantities) | ⚠️ Applied by Docker/Podman/K8s; **ignored by the VM driver** |
| `--gpu` / `--gpu-device` | Request GPU resources (CDI device IDs for Docker/Podman; PCI BDF for VM) | ⚠️ Requires driver and image GPU support |
| `--provider` | Attach named LLM credential providers | ✅ |
| `--upload` | Copy local files in before launch | ✅ |
| `--forward` | Forward a local port (keeps sandbox alive) | ✅ |
| `--no-keep` | Delete the sandbox after the command exits | ✅ |

### The policy YAML — where isolation is defined ✅

This is the real "room definition." The policy is a YAML file that declares
filesystem, process, and network constraints. Example from
`examples/sandbox-policy-quickstart/policy.yaml`:

```yaml
version: 1

filesystem_policy:
  include_workdir: true
  read_only: [/usr, /lib, /proc, /dev/urandom, /app, /etc, /var/log]
  read_write: [/sandbox, /tmp, /dev/null]
landlock:
  compatibility: best_effort
process:
  run_as_user: sandbox
  run_as_group: sandbox

network_policies:
  github_api:
    name: github-api-readonly
    endpoints:
      - host: api.github.com
        port: 443
        protocol: rest
        enforcement: enforce
        access: read-only
    binaries:
      - { path: /usr/bin/curl }
```

What each section controls (traced to enforcement code):

| Section | Enforced by | Source |
|---|---|---|
| `filesystem_policy` + `landlock` | Kernel Landlock LSM — read-only/read-write path restrictions | `crates/openshell-sandbox/src/sandbox/linux/landlock.rs` |
| `process` | Supervisor drops to an unprivileged user before launching the agent | `crates/openshell-sandbox/src/process.rs` |
| `network_policies` | In-sandbox policy proxy + OPA engine, per-binary identity | `crates/openshell-sandbox/src/proxy.rs`, `crates/openshell-sandbox/src/opa.rs` |

### Updating policy on a live sandbox ⚠️

Network policy is **dynamic** — it can be hot-reloaded on a running sandbox:

```shell
# Incremental update without replacing the whole policy
openshell policy update --name my-sandbox \
  --add-endpoint "api.example.com:443:read-only:rest:enforce"
```

Filesystem and process policy are **static** — they are applied at startup,
before the child process runs, and require a new sandbox to change.

---

## 4. What Real Isolation Mechanisms Back a Sandbox

This is the most substantial, most real part of the codebase. Every agent runs
inside a **supervisor-managed sandbox** with overlapping kernel and proxy
controls.

### The two-trust-level model ✅

Each sandbox workload runs two trust levels:

| Process | Role |
|---|---|
| **Supervisor** | Starts as root, prepares all isolation, runs the proxy, fetches config, then launches the child |
| **Agent child** | Runs as an unprivileged user with filesystem, process, and network restrictions already applied |

The critical invariant: **the agent child loses privilege before user code
runs.** The supervisor does the privileged setup, then steps down.

### Isolation layers (all verified in source) ✅

| Layer | Mechanism | Source |
|---|---|---|
| **Filesystem** | Landlock LSM restricts read-only and read-write paths | `sandbox/linux/landlock.rs` |
| **Process** | Child runs as non-root user with reduced capabilities | `process.rs` (`drop_privileges`) |
| **Syscalls** | Seccomp blocks dangerous syscalls, including raw socket paths that would bypass the proxy | `sandbox/linux/seccomp.rs` |
| **Network** | Network namespace forces egress through the local proxy | `process.rs` namespace setup |
| **Proxy** | Evaluates destination, binary identity, TLS/L7 rules, SSRF checks, and inference interception | `proxy.rs` |

### Privilege drop — the central act ✅

`drop_privileges()` in `crates/openshell-sandbox/src/process.rs` is where the
sandbox becomes real. The supervisor:

1. Sets up Landlock and seccomp **while still root** (the comment in source is
   explicit: this must happen before privilege drop).
2. Calls `initgroups`, `setgid`, then `setuid` to become the unprivileged user.
3. **Verifies the drop worked**: it attempts `setuid(0)`. If the process can
   re-acquire root, the sandbox aborts with a verification failure — a process
   that can climb back to root is not a sandbox, it is "a lie with walls."

This verification step is the difference between a real boundary and a costume.
If you extend the supervisor, do not remove it.

### The policy proxy — where network trust lives ✅

The proxy (`proxy.rs`, ~6,600 lines) is the only network egress path. It:

- **Identifies the calling binary** by trust-on-first-use signature, and checks
  rules against the binary identity — not just the destination. A process cannot
  forge another process's rights.
- **Hard-blocks** unsafe internal destinations (private IP ranges) before
  consulting rules. Explicit denies win over allows. If no rule matches, the
  request is denied.
- **Optionally inspects HTTP/L7**: terminates TLS with the sandbox's ephemeral
  CA and checks method/path before forwarding.
- **Handles `inference.local` specially**: this virtual endpoint bypasses OPA
  network policy. The proxy terminates the local TLS, detects known inference
  request shapes (OpenAI, Anthropic, compatible), **strips caller-supplied
  credentials**, and forwards through `openshell-router` using route bundles
  the gateway holds. The agent never holds the real provider credentials.

### The policy advisor — denials become drafts ⚠️

When the proxy denies a request, the denial is aggregated and a mechanistic
mapper proposes a minimal, narrow policy addition. Proposals intentionally omit
`allowed_ips`: if a proposed host resolves to a private IP, the proxy's SSRF
classification blocks it at runtime, forcing a two-step explicit allow. Drafts
require human approval before merging. This is a workflow aid, not an automatic
permission grant.

Source: `crates/openshell-sandbox/src/mechanistic_mapper.rs`,
`crates/openshell-sandbox/src/denial_aggregator.rs`.

### Compute drivers — the sandbox boundary ✅

All four drivers are real, substantial implementations that start a workload
running the supervisor:

| Driver | Lines | Boundary | Notes |
|---|---|---|---|
| Docker | ~3,100 | Container + nested sandbox namespace | Host networking so loopback gateway endpoints work |
| Podman | ~4,150 | Container + nested sandbox namespace | Rootless; OCI image volumes; CDI GPU devices |
| Kubernetes | ~3,000 | Pod + nested sandbox namespace | Helm chart in `deploy/helm/openshell` |
| VM | ~10,200 | Per-sandbox libkrun microVM | Experimental; **ignores CPU/memory limits** |

All drivers report lifecycle events through the shared `openshell.progress.*`
metadata in `openshell-core`, so clients do not parse driver-local reason
strings.

---

## 5. The OpenConstruct "Room" Layer — Honest Inventory

The README describes a rich room-native architecture. This section tells you
what is actually in the repository so you do not build on ink as if it were
concrete.

### What the README describes 🔮

The README's "OpenConstruct Crates" table lists fifteen `lau-*` crates with
specific test counts (e.g., `lau-shell-kernel` 73 tests, `lau-jepa-gravity` 79
tests, `lau-penrose` 59 tests). It also describes ensigns, JEPA gravity, Penrose
correlations, tiles, and ZeroClaw/CUDAClaw reference agents.

### What is actually in the repository ✅

The workspace is `members = ["crates/*"]` (`Cargo.toml`). There are **no `lau-*`
crates**. The real OpenConstruct-specific crates are:

| Crate | Lines | What it actually is | Status |
|---|---|---|---|
| `openshell-construct` | 273 | A 5-phase onboarding **data model**: `Phase` enum, `AgentIdentity`, `OnboardingSession`, `OnboardingConfig`. Pure data structures with tests. No rooms, ensigns, gravity, tiles, or room-as-agent logic. | ✅ (as a data model) / 🔮 (as a room-native runtime) |
| `openconstruct-cli` | ~500 (one file) | A standalone CLI with `init`, `status`, `sense`, `fleet`, `tick`, `room`, `build`, `publish`. | ⚠️ Local-file only (see below) |

### What the `openconstruct` CLI actually does ⚠️

The `openconstruct` CLI (`crates/openconstruct-cli/src/main.rs`) is a single-file
binary that operates on **local files only**. It does not connect to the
gateway, does not provision sandboxes, and does not create ensigns:

| Command | What it does | Status |
|---|---|---|
| `openconstruct init` | Runs a 5-phase wizard, writes `~/.openconstruct/agent.toml` | ⚠️ Local config only |
| `openconstruct status` | Prints the local agent.toml | ✅ |
| `openconstruct room create <name>` | Appends a JSON line to `~/.openconstruct/rooms.jsonl` | ⚠️ No sandbox, no isolation |
| `openconstruct room join / leave` | Prints a confirmation message | ⚠️ Stub |
| `openconstruct tick post / read` | Appends/reads a local JSONL file | ✅ (as a local log) |
| `openconstruct fleet discover` | Prints "coming soon" | 🔮 |
| `openconstruct build <name>` | Scaffolds a starter module directory | ✅ |

A "room" created by `openconstruct room create` is a JSON line in a file. It is
not an isolated sandbox. For real isolation, use `openshell sandbox create`.

### Where a "room" concept does exist in real code ⚠️

There is a `Room` type in `crates/openshell-signal-chain/src/room.rs`. This is a
**signal-chain fact-space** — a container of hard-locked snaps (ground truth) and
soft inferences (hypotheses), queryable at different confidence "dial" levels. It
is used by `openshell-prover`. It is a legitimate, tested data structure, but it
is **not** the "room-as-sandbox-wrapper-with-ensigns-and-gravity" described in
the README. Do not confuse the two.

### Ensigns, gravity, Penrose, tiles, JEPA 🔮

No code implementing ensign monitor agents, the JEPA gravity scalar, Penrose
cross-room correlations, or the tile system described in the README was found in
the repository at the time of writing. These appear to be planned additions or
live in other (not-yet-merged) repositories referenced by the README (e.g.,
`hermes-construct`, which is a separate repo).

---

## 6. Practical Onboarding Path

If you want to **run an isolated agent today**, here is the real path:

1. **Install the OpenShell CLI** and start a local gateway:
   ```shell
   curl -LsSf https://raw.githubusercontent.com/NVIDIA/OpenShell/main/install.sh | sh
   openshell gateway add http://127.0.0.1:18080 --local --name local
   openshell gateway select local
   ```
2. **Write a policy YAML** (start from `examples/sandbox-policy-quickstart/policy.yaml`).
3. **Create a sandbox** with that policy:
   ```shell
   openshell sandbox create --from openclaw --policy ./sandbox-policy.yaml -- claude
   ```
4. **Connect and verify** the isolation:
   ```shell
   openshell sandbox connect        # interactive shell as the unprivileged user
   openshell policy get --full      # inspect the effective policy
   openshell logs --tail            # watch proxy decisions and OCSF events
   ```

If you want to **extend the OpenConstruct room layer**, the place to build is
`crates/openshell-construct/` (the onboarding data model) — but understand you
are starting from data structures, not a runtime. The README's room-native
vision is the blueprint; the foundation is the OpenShell sandbox.

---

## 7. Verification Notes

All claims in this guide were verified against source at the time of writing:

- CLI command surface: `crates/openshell-cli/src/main.rs` and `crates/openconstruct-cli/src/main.rs`.
- Landlock / seccomp: `crates/openshell-sandbox/src/sandbox/linux/{landlock,seccomp}.rs`.
- Privilege drop: `crates/openshell-sandbox/src/process.rs` (`drop_privileges`, line ~426, with `setuid(0)` re-acquire test).
- Proxy and inference interception: `crates/openshell-sandbox/src/proxy.rs`.
- Policy advisor: `crates/openshell-sandbox/src/{mechanistic_mapper,denial_aggregator}.rs`.
- Compute driver sizes: `crates/openshell-driver-{docker,podman,kubernetes,vm}/`.
- Onboarding data model: `crates/openshell-construct/src/lib.rs` (273 lines).
- Workspace membership: root `Cargo.toml` (`members = ["crates/*"]`); no `lau-*` directories exist under `crates/`.
