# flux-vm-v3

A stack-based virtual machine for constraint checking. 60 opcodes, deterministic execution, SHA-256 proof certificates.

---

## The Idea

Compile constraint definitions to bytecode, then either interpret or JIT-compile to native code. The bytecode is designed so that:
- Every program terminates (max 4096 cycles per constraint)
- The same bytecode always produces the same result (deterministic)
- A SHA-256 hash chain from source to result creates a tamper-evident audit trail

## Pipeline

```
GUARD source          ← Write constraints in a readable DSL
    ↓
FLUX-C bytecode       ← 60 opcodes, stack-based
    ↓
JIT / interpreter     ← Execute or compile to native
    ↓
Error mask (u8)       ← 0 = pass, nonzero = violations
    ↓
Proof certificate     ← SHA-256 hash chain, Merkle proofs
```

## Quick Start

```bash
git clone https://github.com/SuperInstance/flux-vm-v3
cd flux-vm-v3
cargo build --release
cargo test
```

### Run a preset

```bash
# 10 industry presets built in
flux-check run --preset automotive_can --value 3000
flux-check run --preset aviation_adsb --value 45000
flux-check run --preset nuclear_reactor --value 350

# Benchmark
flux-check bench --preset automotive_can --iterations 1000000
# → 179M checks/sec
```

### Use as a library

```rust
use flux_vm::{VM, Bytecode};

let mut vm = VM::new();
vm.load_bytecode(bytecode);
let result = vm.execute();
println!("Error mask: {:08b}", result.error_mask());
```

## The Opcodes

The 60 opcodes were chosen by studying 96 language implementations. Each opcode exists because a specific language revealed a need:

| Category | Opcodes | Insight From |
|----------|---------|-------------|
| Stack manipulation | `PUSH`, `POP`, `DUP`, `SWAP` | Forth / Factor |
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV` | Universal |
| Comparison | `LT`, `GT`, `EQ`, `LTE`, `GTE` | IEEE 754 semantics |
| Bounds | `CHECK_LO`, `CHECK_HI`, `CHECK_RANGE` | GD&T tolerance stacks |
| Bitwise | `AND`, `OR`, `XOR`, `NOT` | Error mask operations |
| Control | `JMP`, `JMP_IF`, `HALT` | No backward jumps |
| Effects | `EMIT`, `STREAM`, `PARALLEL` | Koka / Eff |
| Proof | `HASH`, `MERKLE`, `CERTIFY` | Audit trail |
| NaN trap | `NAN_CHECK` | The bug that started this |

### Why No Backward Jumps

Backward jumps enable loops. Loops can run forever. A constraint checker that runs forever is wrong by definition. Every FLUX-C program has a bounded execution length — the compiler enforces this.

### Why Stack-Based

Register-based VMs are faster to execute but harder to verify. A stack machine has one canonical state at any point — the stack contents. This makes:
- Termination proofs simpler (bounded stack depth)
- Content-addressing natural (same instruction sequence = same hash)
- Proof certificates smaller (no register allocation to track)

## The JIT

`jit.rs` and `jit_x86.rs` compile bytecode to native x86_64:

1. Extract constraint bounds from bytecode
2. Generate tight comparison loop: `ucomisd` → bitmask construction
3. NaN trap at the top of the loop
4. Result: 179M checks/sec on Zen 5

Currently x86_64 only. ARM JIT is the next target.

## Industry Presets

| Preset | What It Checks |
|--------|---------------|
| `automotive_can` | CAN bus sensor ranges (RPM, temp, voltage) |
| `aviation_adsb` | ADS-B altitude, speed, heading bounds |
| `medical_fhir` | Patient vitals within physiological ranges |
| `financial_fix` | FIX protocol price/quantity bounds |
| `energy_scada` | SCADA power grid sensor thresholds |
| `iot_mqtt` | IoT sensor ranges (temp, humidity, pressure) |
| `maritime_nmea` | NMEA GPS, heading, depth bounds |
| `nuclear_reactor` | Core temperature, pressure, neutron flux |
| `railway_ertms` | ERTMS speed, distance, signal bounds |
| `robotics` | Joint angles, motor current, proximity |

```bash
flux-check list-presets
```

## Test Results

```
running 29 tests
test result: ok. 29 passed; 0 failed; 0 ignored
```

## What to Read Next

| If you want to... | Go to... |
|---|---|
| Write constraints in the DSL | [guardc-v3](https://github.com/SuperInstance/guardc-v3) |
| See the 96-language research | [constraint-theory-ecosystem](https://github.com/SuperInstance/constraint-theory-ecosystem) |
| Use the standalone Rust fracture library | [flux-fracture](https://github.com/SuperInstance/flux-fracture) |
| Use the standalone C fracture header | [flux-fracture-c](https://github.com/SuperInstance/flux-fracture-c) |
| Read about what old languages teach | [OLD-LANGUAGE-ARCHITECTURE.md](https://github.com/SuperInstance/constraint-theory-ecosystem/blob/main/docs/OLD-LANGUAGE-ARCHITECTURE.md) |

## License

MIT
