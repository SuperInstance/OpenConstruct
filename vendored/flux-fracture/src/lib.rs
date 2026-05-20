//! # flux-fracture
//!
//! FRACTURE-COALESCE: Disjoint Linear Algebra for Constraint Systems.
//!
//! If constraints are independent, the partition function factorizes Z = Π(Zᵢ).
//! This crate makes that **operational** by fracturing constraint systems into
//! independent blocks and coalescing results provably correct.
//!
//! ## Theorem
//!
//! If fracture correctly identifies connected components of the
//! constraint–dimension dependency graph, coalescence via bitwise OR
//! preserves zero false negatives.
//!
//! **Proof:** Each constraint violation is a Boolean event. For independent blocks,
//! the event spaces are disjoint (no shared dimensions). The union of all
//! violations = OR of block error masks. QED.
//!
//! ## `no_std` support
//!
//! Disable the default `std` feature to use with `alloc` only (e.g. embedded targets):
//!
//! ```toml
//! [dependencies]
//! flux-fracture = { version = "0.1.0", default-features = false }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod graph;
pub mod coalesce;

pub use graph::{DependencyGraph, FractureResult, Block, Fracturer, AdaptiveFracturer, FractureDelta};
pub use coalesce::Coalescer;
