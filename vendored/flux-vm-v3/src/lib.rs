//! FLUX-C v3 — Next-generation constraint checking VM
//!
//! Stack-based, terminating, proof-carrying VM with native SIMD support.
//! Incorporates insights from 96 language implementations of the FLUX Constraint Engine.

pub mod error;
pub mod opcode;
pub mod memory;
pub mod vector;
pub mod proof;
pub mod provenance;
pub mod effects;
pub mod streaming;
pub mod parallel;
pub mod check;
pub mod vm;
pub mod bench;
pub mod jit;
pub mod jit_x86;

pub use error::{FluxError, FluxResult};
pub use opcode::OpCode;
pub use vm::FluxVM;
pub use check::{Constraint, aviation_preset, temperature_preset};
pub use proof::{ProofContext, ProofCertificate};
pub use provenance::ProvenanceLog;
pub use effects::EffectHandler;
pub use vector::VectorUnit;
pub use parallel::{ParallelBatch, ParallelResult, ConstraintCheck};
pub use streaming::StreamState;
pub use memory::BoundedMemory;
