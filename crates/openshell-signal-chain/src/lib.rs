// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Signal Chain Thesis
//! 
//! Every intelligent system needs a dial between hard-snapped algorithms
//! and soft-inferenced models. Structured rooms anchor facts in space/time.
//! 
//! # Core Primitives
//! 
//! - **Dial**: continuous 0.0 (hard) to 1.0 (soft)
//! - **Snap**: hard-locked fact with confidence
//! - **Inference**: soft extrapolation with confidence  
//! - **Room**: fact-space with snaps, inferences, and dial position
//! 
//! # Example
//! 
//! ```rust
//! use openshell_signal_chain::{Dial, Room, SignalChain};
//! 
//! let mut chain = SignalChain::new("test-chain");
//! let room = chain.room("test-room");
//! room.add_snap(serde_json::json!({"x": 1, "y": 2}), 1.0);
//! room.add_inference(serde_json::json!({"hypothesis": "z = 3"}), 0.7);
//! 
//! let results = room.query(Dial::new(0.5));
//! ```

mod dial;
mod snap;
mod inference;
mod room;
mod signal_chain;

pub use dial::Dial;
pub use snap::Snap;
pub use inference::Inference;
pub use room::Room;
pub use signal_chain::SignalChain;

// Preset dials for common use cases
pub use dial::DIAL_FORMAL;
pub use dial::DIAL_BATHY;
pub use dial::DIAL_COMMIT;
pub use dial::DIAL_ANALYSIS;
pub use dial::DIAL_REVIEW;
pub use dial::DIAL_EXTRAPOLATE;
pub use dial::DIAL_CREATIVE;
pub use dial::DIAL_EXPLORATORY;