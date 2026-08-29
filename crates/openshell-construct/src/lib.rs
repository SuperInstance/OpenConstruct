// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! **openshell-construct** — The OpenConstruct onboarding engine.
//!
//! Defines the data structures for the five-phase onboarding flow:
//!
//! 1. **SelfDeclaration** — the agent declares its identity, model, and capabilities.
//! 2. **ModuleSelection** — pick modules from the registry to adopt.
//! 3. **InterfaceSelection** — choose interface preferences (CLI, TUI, API, etc.).
//! 4. **ConnectionSetup** — configure external connections (APIs, databases, services).
//! 5. **EnvironmentGeneration** — produce the final `OnboardingConfig`.
//!
//! It also grows the room-native residency layer (RFC 0004, *The Room
//! Grows a Mask*): rooms derive a [`RoomMask`] at creation, record walks
//! in a hash-chained [`WalkLog`] persisted by [`WalkRecorder`], are read
//! through [`heat`], surface anomalies as Ensign [`prior`]s, bend gravity
//! through heat via [`modulate`], and onboard their keepers from a
//! [`GrowthRecord`] — subtext is observed, not declared. A [`Room`] is the
//! binding of all of them: grow it, tick it, save it, load it.

use openshell_registry::ModuleShadow;
use serde::{Deserialize, Serialize};

pub mod cli;
pub mod ensign;
pub mod gravity;
pub mod growth;
pub mod hooks;
pub mod mask;
pub mod residency;
pub mod room;
pub mod walks;

pub use ensign::{prior, AttentionPrior, AttentionReason};
pub use gravity::{modulate, ModelParams};
pub use hooks::{HooksRegistry, NoopHooks, RoomHooks};
pub use growth::{onboard, GrowthRecord};
pub use mask::{derive_mask, MaskChannel, RoomMask};
pub use residency::{heat, HeatReading, HeatState, WalkLog, WalkRecord};
pub use room::{Room, RoomError, ROOM_FILE_FORMAT, ROOM_WINDOW};
pub use walks::{LoadOutcome, LoadReport, WalkRecorder};

/// The five onboarding phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Phase 1: Agent self-declares its identity and capabilities.
    SelfDeclaration,
    /// Phase 2: Select modules from the registry.
    ModuleSelection,
    /// Phase 3: Choose interface preferences.
    InterfaceSelection,
    /// Phase 4: Set up external connections.
    ConnectionSetup,
    /// Phase 5: Generate the final environment configuration.
    EnvironmentGeneration,
}

impl Phase {
    /// Return all phases in order.
    pub fn all() -> &'static [Phase] {
        &[
            Phase::SelfDeclaration,
            Phase::ModuleSelection,
            Phase::InterfaceSelection,
            Phase::ConnectionSetup,
            Phase::EnvironmentGeneration,
        ]
    }

    /// Advance to the next phase, returning `None` if already at the end.
    pub fn next(self) -> Option<Phase> {
        match self {
            Phase::SelfDeclaration => Some(Phase::ModuleSelection),
            Phase::ModuleSelection => Some(Phase::InterfaceSelection),
            Phase::InterfaceSelection => Some(Phase::ConnectionSetup),
            Phase::ConnectionSetup => Some(Phase::EnvironmentGeneration),
            Phase::EnvironmentGeneration => None,
        }
    }
}

/// How the agent identifies itself during onboarding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentIdentity {
    /// Chosen name for the agent.
    pub name: String,
    /// Underlying model identifier (e.g. `"gpt-4o"`, `"claude-3-opus"`).
    pub model: String,
    /// Declared capabilities (e.g. `"code-generation"`, `"web-search"`).
    pub capabilities: Vec<String>,
    /// Tools the agent has access to.
    pub tools: Vec<String>,
    /// Self-imposed constraints (e.g. `"no-filesystem-write"`).
    pub constraints: Vec<String>,
    /// Preference hints (e.g. `"terse-output"`, `"structured-logging"`).
    pub preferences: Vec<String>,
}

/// An external connection configured during Phase 4.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    /// Connection label (e.g. `"github"`, `"postgres"`).
    pub label: String,
    /// Connection type (e.g. `"api"`, `"database"`, `"messaging"`).
    pub kind: String,
    /// Connection URI or endpoint.
    pub endpoint: String,
    /// Additional metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Interface preferences selected during Phase 3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterfacePreferences {
    /// Preferred primary interface (e.g. `"cli"`, `"tui"`, `"api"`, `"discord"`).
    pub primary: String,
    /// Secondary / fallback interfaces.
    pub secondary: Vec<String>,
    /// Output format preference (e.g. `"json"`, `"markdown"`, `"plain"`).
    pub output_format: String,
    /// Whether to enable verbose logging.
    pub verbose: bool,
}

/// An onboarding session tracking progress through the five phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Current phase.
    pub phase: Phase,
    /// Agent identity (set during Phase 1).
    pub agent_identity: Option<AgentIdentity>,
    /// Selected modules (set during Phase 2).
    pub selected_modules: Vec<ModuleShadow>,
    /// Interface preferences (set during Phase 3).
    pub interface_prefs: Option<InterfacePreferences>,
    /// Configured connections (set during Phase 4).
    pub connections: Vec<Connection>,
}

impl OnboardingSession {
    /// Create a new session at the start of Phase 1.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            phase: Phase::SelfDeclaration,
            agent_identity: None,
            selected_modules: Vec::new(),
            interface_prefs: None,
            connections: Vec::new(),
        }
    }

    /// Advance to the next phase.
    ///
    /// Returns `Ok(())` if advanced, or an error string if there is no next
    /// phase or required data is missing.
    pub fn advance(&mut self) -> Result<(), String> {
        let next = self
            .phase
            .next()
            .ok_or_else(|| "already at the final phase".to_string())?;

        // Validate required data before advancing.
        match self.phase {
            Phase::SelfDeclaration => {
                if self.agent_identity.is_none() {
                    return Err("agent identity must be set before leaving SelfDeclaration".into());
                }
            }
            Phase::ModuleSelection => {
                // Modules are optional; no validation needed.
            }
            Phase::InterfaceSelection => {
                if self.interface_prefs.is_none() {
                    return Err("interface preferences must be set before leaving InterfaceSelection".into());
                }
            }
            Phase::ConnectionSetup => {
                // Connections are optional.
            }
            Phase::EnvironmentGeneration => {}
        }

        self.phase = next;
        Ok(())
    }
}

/// The final output of a completed onboarding session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingConfig {
    /// The agent identity card.
    pub agent_card: AgentIdentity,
    /// Selected modules (with dependencies resolved).
    pub modules: Vec<ModuleShadow>,
    /// Workspace configuration blob (serialized JSON).
    pub workspace_config: serde_json::Value,
    /// Policy constraints derived from the agent identity.
    pub policies: Vec<String>,
}

impl OnboardingConfig {
    /// Build an `OnboardingConfig` from a completed session plus workspace
    /// config and policies.
    pub fn from_session(
        session: &OnboardingSession,
        workspace_config: serde_json::Value,
        policies: Vec<String>,
    ) -> Result<Self, String> {
        let agent_card = session
            .agent_identity
            .clone()
            .ok_or("session missing agent identity")?;

        Ok(Self {
            agent_card,
            modules: session.selected_modules.clone(),
            workspace_config,
            policies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity() -> AgentIdentity {
        AgentIdentity {
            name: "test-agent".into(),
            model: "test-model".into(),
            capabilities: vec!["reasoning".into()],
            tools: vec!["web-search".into()],
            constraints: vec![],
            preferences: vec!["terse-output".into()],
        }
    }

    #[test]
    fn phase_advances_in_order() {
        let mut session = OnboardingSession::new("test".into());
        assert_eq!(session.phase, Phase::SelfDeclaration);

        session.agent_identity = Some(sample_identity());
        session.advance().unwrap();
        assert_eq!(session.phase, Phase::ModuleSelection);

        session.advance().unwrap();
        assert_eq!(session.phase, Phase::InterfaceSelection);

        session.interface_prefs = Some(InterfacePreferences {
            primary: "cli".into(),
            secondary: vec![],
            output_format: "json".into(),
            verbose: false,
        });
        session.advance().unwrap();
        assert_eq!(session.phase, Phase::ConnectionSetup);

        session.advance().unwrap();
        assert_eq!(session.phase, Phase::EnvironmentGeneration);

        assert!(session.advance().is_err());
    }

    #[test]
    fn cannot_leave_self_declaration_without_identity() {
        let mut session = OnboardingSession::new("test".into());
        assert!(session.advance().is_err());
    }

    #[test]
    fn build_config_from_session() {
        let mut session = OnboardingSession::new("test".into());
        session.agent_identity = Some(sample_identity());
        session.selected_modules = vec![ModuleShadow {
            id: "test-module".into(),
            domain: "test".into(),
            name: "Test".into(),
            one_line: "A test module.".into(),
            pick_if: vec![],
            skip_if: vec![],
            requires: vec![],
            provides: vec!["test".into()],
        }];

        let config = OnboardingConfig::from_session(
            &session,
            serde_json::json!({"workspace": "default"}),
            vec!["no-external-access".into()],
        )
        .unwrap();

        assert_eq!(config.agent_card.name, "test-agent");
        assert_eq!(config.modules.len(), 1);
    }
}
