// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Error types for the signal chain.
//!
//! This module provides a unified error enum for all operations that can fail
//! within the signal chain, including dial validation, room lookups, and
//! cycle detection.

use thiserror::Error;

/// Errors that can occur during signal chain operations.
///
/// Each variant provides context about what went wrong, making it easy to
/// handle failures gracefully in fleet operations.
///
/// # Examples
///
/// ```rust
/// use openshell_signal_chain::{Dial, SignalChainError};
///
/// // Attempting to create a dial with an out-of-range value
/// let result = Dial::try_new(-0.5);
/// assert!(matches!(result.unwrap_err(), SignalChainError::InvalidDial(_)));
/// ```
#[derive(Error, Debug)]
pub enum SignalChainError {
    /// Dial value was outside the valid [0.0, 1.0] range.
    ///
    /// Contains the invalid value that was provided.
    #[error("Invalid dial value: {0}. Must be between 0.0 and 1.0")]
    InvalidDial(f64),

    /// Room name was empty.
    #[error("Room name cannot be empty")]
    EmptyName,

    /// A requested room does not exist in the chain.
    ///
    /// Contains the room name that was not found.
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    /// Attempted to create a room that already exists.
    ///
    /// Contains the name of the duplicate room.
    #[error("Room already exists: {0}")]
    RoomExists(String),

    /// A cascade operation detected a cycle in the room hierarchy.
    ///
    /// Contains a description of the cycle path.
    #[error("Cascade cycle detected: {0}")]
    CascadeCycle(String),
}
