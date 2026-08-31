// SPDX-FileCopyrightText: Copyright (c) 2025-2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! **openshell-registry** — Module registry for the `OpenConstruct` onboarding engine.
//!
//! Provides a [`ModuleRegistry`] containing [`ModuleShadow`] descriptors that
//! describe capabilities an agent can adopt during onboarding. The initial
//! registry ships with a curated set of `SuperInstance` modules and supports
//! domain filtering plus simple dependency resolution.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A lightweight descriptor for a module that can be selected during onboarding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleShadow {
    /// Unique identifier (crate-style, e.g. `"spectral-graph-core"`).
    pub id: String,
    /// Domain category (e.g. `"mathematics"`, `"policy"`, `"infrastructure"`).
    pub domain: String,
    /// Human-readable module name.
    pub name: String,
    /// One-line description of what the module provides.
    pub one_line: String,
    /// Heuristic: when to suggest picking this module.
    pub pick_if: Vec<String>,
    /// Heuristic: when to suggest skipping this module.
    pub skip_if: Vec<String>,
    /// IDs of modules that must also be selected (declared dependencies).
    pub requires: Vec<String>,
    /// Capabilities this module provides (used for dependency resolution).
    pub provides: Vec<String>,
}

/// The module registry: a collection of [`ModuleShadow`] entries with lookup helpers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRegistry {
    modules: Vec<ModuleShadow>,
    /// Index: module id → position in `modules`.
    #[serde(skip)]
    index: HashMap<String, usize>,
}

impl ModuleRegistry {
    /// Create a new registry from the given module list.
    pub fn new(modules: Vec<ModuleShadow>) -> Self {
        let index = modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.clone(), i))
            .collect();
        Self { modules, index }
    }

    /// Return the default `SuperInstance` module registry.
    pub fn default_registry() -> Self {
        Self::new(superinstance_modules())
    }

    /// List all modules.
    pub fn list(&self) -> &[ModuleShadow] {
        &self.modules
    }

    /// Filter modules by domain.
    pub fn by_domain(&self, domain: &str) -> Vec<&ModuleShadow> {
        self.modules.iter().filter(|m| m.domain == domain).collect()
    }

    /// Look up a module by id.
    pub fn get(&self, id: &str) -> Option<&ModuleShadow> {
        self.index.get(id).map(|&i| &self.modules[i])
    }

    /// Resolve the full set of modules needed to satisfy the given selection,
    /// including transitive dependencies.
    ///
    /// Returns a topologically ordered list or an error if a dependency is
    /// missing or a cycle is detected.
    pub fn resolve(&self, selected: &[String]) -> Result<Vec<&ModuleShadow>, RegistryError> {
        // Reverse edges: dependents → dependency direction
        // Actually, let's just do a simple topological sort where deps come first.
        fn visit<'a>(
            id: &str,
            registry: &'a ModuleRegistry,
            needed: &HashSet<String>,
            visited: &mut HashSet<String>,
            visiting: &mut HashSet<String>,
            result: &mut Vec<&'a ModuleShadow>,
        ) -> Result<(), RegistryError> {
            if visited.contains(id) {
                return Ok(());
            }
            if visiting.contains(id) {
                return Err(RegistryError::CyclicDependency { id: id.to_string() });
            }
            visiting.insert(id.to_string());
            let module = registry.get(id).unwrap();
            for dep in &module.requires {
                if needed.contains(dep) {
                    visit(dep, registry, needed, visited, visiting, result)?;
                }
            }
            visiting.remove(id);
            visited.insert(id.to_string());
            result.push(module);
            Ok(())
        }

        let mut needed = HashSet::new();
        let mut stack: Vec<String> = selected.to_vec();
        while let Some(id) = stack.pop() {
            if needed.contains(&id) {
                continue;
            }
            let module = self
                .get(&id)
                .ok_or_else(|| RegistryError::MissingModule { id: id.clone() })?;
            needed.insert(id);
            for dep in &module.requires {
                if !needed.contains(dep) {
                    stack.push(dep.clone());
                }
            }
        }

        // Topological sort (Kahn's algorithm)
        let mut in_degree: HashMap<&String, usize> = HashMap::new();
        for id in &needed {
            in_degree.insert(id, 0);
        }
        for id in &needed {
            let module = self.get(id).unwrap();
            for dep in &module.requires {
                if needed.contains(dep) {
                    *in_degree.entry(dep).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<&ModuleShadow> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();

        let mut visiting: HashSet<String> = HashSet::new();
        for id in &needed {
            visit(id, self, &needed, &mut visited, &mut visiting, &mut result)?;
        }

        Ok(result)
    }
}

/// Errors produced by the registry.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// A referenced module does not exist in the registry.
    #[error("module not found: {id}")]
    MissingModule { id: String },
    /// A dependency cycle was detected.
    #[error("cyclic dependency involving: {id}")]
    CyclicDependency { id: String },
}

/// Return the curated list of `SuperInstance` modules shipped with `OpenConstruct`.
pub fn superinstance_modules() -> Vec<ModuleShadow> {
    vec![
        ModuleShadow {
            id: "spectral-graph-core".into(),
            domain: "mathematics".into(),
            name: "Spectral Graph Core".into(),
            one_line: "Spectral analysis and graph Laplacian computations.".into(),
            pick_if: vec!["graph-analysis".into(), "spectral-methods".into()],
            skip_if: vec!["no-graph-work".into()],
            requires: vec![],
            provides: vec!["spectral-graph".into(), "graph-laplacian".into()],
        },
        ModuleShadow {
            id: "conservation-regime".into(),
            domain: "policy".into(),
            name: "Conservation Regime".into(),
            one_line: "Conservation law enforcement and regime management.".into(),
            pick_if: vec!["policy-enforcement".into(), "conservation".into()],
            skip_if: vec![],
            requires: vec![],
            provides: vec!["conservation-laws".into(), "regime-policy".into()],
        },
        ModuleShadow {
            id: "sheaf-cohomology".into(),
            domain: "mathematics".into(),
            name: "Sheaf Cohomology".into(),
            one_line: "Sheaf-theoretic cohomology computations for topological data.".into(),
            pick_if: vec!["topology".into(), "cohomology".into()],
            skip_if: vec!["no-abstract-math".into()],
            requires: vec!["spectral-graph-core".into()],
            provides: vec!["sheaf-cohomology".into()],
        },
        ModuleShadow {
            id: "symplectic-geometry".into(),
            domain: "mathematics".into(),
            name: "Symplectic Geometry".into(),
            one_line: "Symplectic structures and Hamiltonian mechanics primitives.".into(),
            pick_if: vec!["geometry".into(), "hamiltonian".into()],
            skip_if: vec![],
            requires: vec![],
            provides: vec!["symplectic".into(), "hamiltonian".into()],
        },
        ModuleShadow {
            id: "plato-room".into(),
            domain: "infrastructure".into(),
            name: "Plato Room".into(),
            one_line: "Persistent reasoning room for multi-turn deliberation.".into(),
            pick_if: vec!["reasoning".into(), "deliberation".into()],
            skip_if: vec!["stateless-only".into()],
            requires: vec![],
            provides: vec!["reasoning-room".into()],
        },
        ModuleShadow {
            id: "plato-loader".into(),
            domain: "infrastructure".into(),
            name: "Plato Loader".into(),
            one_line: "Loader and configuration manager for Plato Rooms.".into(),
            pick_if: vec!["reasoning".into()],
            skip_if: vec![],
            requires: vec!["plato-room".into()],
            provides: vec!["room-loader".into()],
        },
        ModuleShadow {
            id: "conservation-protocol".into(),
            domain: "policy".into(),
            name: "Conservation Protocol".into(),
            one_line: "Protocol-level conservation enforcement across modules.".into(),
            pick_if: vec!["conservation".into(), "protocol".into()],
            skip_if: vec![],
            requires: vec!["conservation-regime".into()],
            provides: vec!["conservation-protocol".into()],
        },
        ModuleShadow {
            id: "spectral-deadband".into(),
            domain: "mathematics".into(),
            name: "Spectral Deadband".into(),
            one_line: "Deadband filtering using spectral methods for signal processing.".into(),
            pick_if: vec!["signal-processing".into(), "deadband".into()],
            skip_if: vec![],
            requires: vec!["spectral-graph-core".into()],
            provides: vec!["spectral-deadband".into()],
        },
        ModuleShadow {
            id: "tropical-algebra".into(),
            domain: "mathematics".into(),
            name: "Tropical Algebra".into(),
            one_line: "Tropical semiring algebra for optimization and scheduling.".into(),
            pick_if: vec!["optimization".into(), "tropical".into()],
            skip_if: vec![],
            requires: vec![],
            provides: vec!["tropical-algebra".into()],
        },
        ModuleShadow {
            id: "deadband-rs".into(),
            domain: "infrastructure".into(),
            name: "Deadband RS".into(),
            one_line: "Rust-native deadband filtering library for signal chains.".into(),
            pick_if: vec!["signal-processing".into(), "filtering".into()],
            skip_if: vec![],
            requires: vec![],
            provides: vec!["deadband-filter".into()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_ten_modules() {
        let reg = ModuleRegistry::default_registry();
        assert_eq!(reg.list().len(), 10);
    }

    #[test]
    fn domain_filter_works() {
        let reg = ModuleRegistry::default_registry();
        let math = reg.by_domain("mathematics");
        assert!(math.len() >= 4);
    }

    #[test]
    fn resolve_with_dependencies() {
        let reg = ModuleRegistry::default_registry();
        let resolved = reg
            .resolve(&["sheaf-cohomology".into()])
            .expect("resolution should succeed");
        let ids: Vec<&str> = resolved.iter().map(|m| m.id.as_str()).collect();
        // Should include spectral-graph-core (dep) before sheaf-cohomology
        assert!(ids.contains(&"spectral-graph-core"));
        assert!(ids.contains(&"sheaf-cohomology"));
        assert!(
            ids.iter()
                .position(|&x| x == "spectral-graph-core")
                .unwrap()
                < ids.iter().position(|&x| x == "sheaf-cohomology").unwrap()
        );
    }

    #[test]
    fn resolve_missing_module_errors() {
        let reg = ModuleRegistry::default_registry();
        let result = reg.resolve(&["nonexistent".into()]);
        assert!(result.is_err());
    }
}
